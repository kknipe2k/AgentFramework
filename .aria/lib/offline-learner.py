#!/usr/bin/env python3
"""
ARIA Offline Reinforcement Learning System

Uses traceability data to improve decision-making over time.
Based on Thompson Sampling with Beta priors for contextual bandits.

Run after each session: python .aria/lib/offline-learner.py learn
Run before session: python .aria/lib/offline-learner.py export-policy
"""

import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict
from collections import defaultdict
import math
import random

# Paths
ARIA_ROOT = Path(__file__).parent.parent
STATE_DIR = ARIA_ROOT / "state"
LOGS_DIR = ARIA_ROOT / "logs"
LEARNED_DIR = ARIA_ROOT / "learned"
PRIORS_DIR = LEARNED_DIR / "priors"
HISTORY_DIR = LEARNED_DIR / "history"

# Ensure directories exist
for d in [LEARNED_DIR, PRIORS_DIR, HISTORY_DIR]:
    d.mkdir(parents=True, exist_ok=True)


@dataclass
class Episode:
    """A single (state, action, reward) tuple"""
    timestamp: str
    decision_point: str  # model_selection | strategy_selection | confidence_calibration
    state: Dict[str, Any]
    action: str
    reward: float
    metadata: Dict[str, Any]


@dataclass
class BetaPrior:
    """Beta distribution prior for Thompson Sampling"""
    alpha: float = 1.0  # successes + 1
    beta: float = 1.0   # failures + 1

    def sample(self) -> float:
        """Sample from Beta distribution"""
        return random.betavariate(self.alpha, self.beta)

    def mean(self) -> float:
        """Expected value"""
        return self.alpha / (self.alpha + self.beta)

    def update(self, reward: float):
        """Update prior with observed reward (0-1 scale)"""
        # Convert reward to success/failure
        if reward > 0.5:
            self.alpha += reward
        else:
            self.beta += (1 - reward)

    def confidence_interval(self, percentile: float = 0.95) -> Tuple[float, float]:
        """Approximate confidence interval"""
        n = self.alpha + self.beta - 2
        if n < 2:
            return (0.0, 1.0)
        p = self.mean()
        z = 1.96 if percentile == 0.95 else 2.576
        margin = z * math.sqrt(p * (1 - p) / n)
        return (max(0, p - margin), min(1, p + margin))


class ContextKey:
    """Generate context keys for prior lookup"""

    @staticmethod
    def model_selection(task_type: str, complexity_bucket: str, code_area: str) -> str:
        """Key for model selection context"""
        return f"{task_type}|{complexity_bucket}|{code_area}"

    @staticmethod
    def strategy_selection(task_type: str, iteration: int, has_failures: bool) -> str:
        """Key for strategy selection context"""
        iter_bucket = "first" if iteration == 1 else "retry"
        fail_str = "has_failures" if has_failures else "no_failures"
        return f"{task_type}|{iter_bucket}|{fail_str}"

    @staticmethod
    def complexity_bucket(complexity: int) -> str:
        """Convert complexity 1-10 to bucket"""
        if complexity <= 3:
            return "low"
        elif complexity <= 7:
            return "medium"
        else:
            return "high"


class EpisodeExtractor:
    """Extract episodes from traceability data"""

    def __init__(self):
        self.signals = self._load_jsonl(STATE_DIR / "signals.jsonl")
        self.decisions = self._load_jsonl(STATE_DIR / "decisions.jsonl")
        self.model_learning = self._load_json(LOGS_DIR / "model_learning.json")
        self.progress = self._load_json(STATE_DIR / "progress.json")

    def _load_jsonl(self, path: Path) -> List[Dict]:
        """Load JSONL file"""
        if not path.exists():
            return []
        entries = []
        with open(path) as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        entries.append(json.loads(line))
                    except json.JSONDecodeError:
                        continue
        return entries

    def _load_json(self, path: Path) -> Dict:
        """Load JSON file"""
        if not path.exists():
            return {}
        with open(path) as f:
            return json.load(f)

    def extract_model_selection_episodes(self) -> List[Episode]:
        """Extract model selection episodes from model_learning.json"""
        episodes = []

        recent = self.model_learning.get("recent_outcomes", [])
        for outcome in recent:
            # State
            state = {
                "task_type": outcome.get("task_type", "general"),
                "complexity": outcome.get("complexity", 5),
                "complexity_bucket": ContextKey.complexity_bucket(outcome.get("complexity", 5)),
                "code_area": outcome.get("code_area", "unknown"),
            }

            # Action
            action = outcome.get("model", "sonnet")

            # Reward calculation
            success = outcome.get("outcome") == "success"
            base_reward = 1.0 if success else -0.5

            # Cost penalty (normalized)
            # Approximate costs: haiku=0.01, sonnet=0.03, opus=0.15
            cost_map = {"haiku": 0.01, "sonnet": 0.03, "opus": 0.15}
            cost = cost_map.get(action, 0.03)
            avg_cost = 0.03
            cost_penalty = -0.1 * (cost / avg_cost)

            reward = base_reward + cost_penalty
            # Normalize to 0-1 range
            reward = (reward + 0.6) / 1.7  # Maps [-0.6, 1.1] to [0, 1]
            reward = max(0, min(1, reward))

            episodes.append(Episode(
                timestamp=outcome.get("timestamp", datetime.now().isoformat()),
                decision_point="model_selection",
                state=state,
                action=action,
                reward=reward,
                metadata={"story_id": outcome.get("story_id")}
            ))

        return episodes

    def extract_decision_episodes(self) -> List[Episode]:
        """Extract confidence calibration episodes from decisions.jsonl"""
        episodes = []

        for decision in self.decisions:
            confidence = decision.get("confidence", 0.5)
            verified = decision.get("verified")

            if verified is None:
                continue  # Skip unverified decisions

            # State
            state = {
                "stated_confidence": confidence,
                "decision_type": self._infer_decision_type(decision.get("action", "")),
                "context_complexity": self._infer_complexity(decision.get("context", "")),
            }

            # Action: what confidence adjustment would have been correct?
            actual_outcome = 1.0 if verified else 0.0
            error = abs(confidence - actual_outcome)

            if error < 0.2:
                action = "accept_confidence"
            elif confidence > actual_outcome:
                action = "adjust_down"
            else:
                action = "adjust_up"

            # Reward: Brier score
            reward = 1 - (confidence - actual_outcome) ** 2

            episodes.append(Episode(
                timestamp=decision.get("timestamp", datetime.now().isoformat()),
                decision_point="confidence_calibration",
                state=state,
                action=action,
                reward=reward,
                metadata={"original_action": decision.get("action")}
            ))

        return episodes

    def _infer_decision_type(self, action: str) -> str:
        """Infer decision type from action description"""
        action_lower = action.lower()
        if any(word in action_lower for word in ["architect", "design", "structure", "pattern"]):
            return "architecture"
        elif any(word in action_lower for word in ["test", "mock", "assert", "spec"]):
            return "testing"
        else:
            return "implementation"

    def _infer_complexity(self, context: str) -> str:
        """Infer complexity from context"""
        if len(context) < 50:
            return "simple"
        elif len(context) < 200:
            return "medium"
        else:
            return "complex"

    def extract_all(self) -> List[Episode]:
        """Extract all episodes"""
        episodes = []
        episodes.extend(self.extract_model_selection_episodes())
        episodes.extend(self.extract_decision_episodes())
        return episodes


class PolicyStore:
    """Store and retrieve learned policies"""

    def __init__(self):
        self.model_priors: Dict[str, Dict[str, BetaPrior]] = defaultdict(
            lambda: defaultdict(BetaPrior)
        )
        self.strategy_priors: Dict[str, Dict[str, BetaPrior]] = defaultdict(
            lambda: defaultdict(BetaPrior)
        )
        self.calibration_adjustments: Dict[str, float] = {}
        self._load()

    def _load(self):
        """Load priors from disk"""
        # Model selection priors
        model_path = PRIORS_DIR / "model-selection.json"
        if model_path.exists():
            with open(model_path) as f:
                data = json.load(f)
                for context, actions in data.items():
                    for action, prior in actions.items():
                        self.model_priors[context][action] = BetaPrior(
                            alpha=prior["alpha"],
                            beta=prior["beta"]
                        )

        # Strategy selection priors
        strategy_path = PRIORS_DIR / "strategy-selection.json"
        if strategy_path.exists():
            with open(strategy_path) as f:
                data = json.load(f)
                for context, actions in data.items():
                    for action, prior in actions.items():
                        self.strategy_priors[context][action] = BetaPrior(
                            alpha=prior["alpha"],
                            beta=prior["beta"]
                        )

        # Calibration adjustments
        calib_path = PRIORS_DIR / "confidence-calibration.json"
        if calib_path.exists():
            with open(calib_path) as f:
                self.calibration_adjustments = json.load(f)

    def save(self):
        """Save priors to disk"""
        # Model selection
        model_data = {}
        for context, actions in self.model_priors.items():
            model_data[context] = {
                action: {"alpha": prior.alpha, "beta": prior.beta}
                for action, prior in actions.items()
            }
        with open(PRIORS_DIR / "model-selection.json", "w") as f:
            json.dump(model_data, f, indent=2)

        # Strategy selection
        strategy_data = {}
        for context, actions in self.strategy_priors.items():
            strategy_data[context] = {
                action: {"alpha": prior.alpha, "beta": prior.beta}
                for action, prior in actions.items()
            }
        with open(PRIORS_DIR / "strategy-selection.json", "w") as f:
            json.dump(strategy_data, f, indent=2)

        # Calibration
        with open(PRIORS_DIR / "confidence-calibration.json", "w") as f:
            json.dump(self.calibration_adjustments, f, indent=2)

    def update_from_episode(self, episode: Episode):
        """Update priors from a single episode"""
        if episode.decision_point == "model_selection":
            context_key = ContextKey.model_selection(
                episode.state["task_type"],
                episode.state["complexity_bucket"],
                episode.state.get("code_area", "unknown")
            )
            self.model_priors[context_key][episode.action].update(episode.reward)

        elif episode.decision_point == "strategy_selection":
            context_key = ContextKey.strategy_selection(
                episode.state["task_type"],
                episode.state.get("iteration", 1),
                episode.state.get("has_failures", False)
            )
            self.strategy_priors[context_key][episode.action].update(episode.reward)

        elif episode.decision_point == "confidence_calibration":
            # Track calibration errors by decision type
            decision_type = episode.state["decision_type"]
            if decision_type not in self.calibration_adjustments:
                self.calibration_adjustments[decision_type] = 0.0

            # Exponential moving average of adjustment needed
            stated = episode.state["stated_confidence"]
            actual = episode.reward  # Brier score approximates actual outcome
            adjustment = actual - stated
            alpha = 0.1  # Learning rate
            self.calibration_adjustments[decision_type] = (
                (1 - alpha) * self.calibration_adjustments[decision_type] +
                alpha * adjustment
            )

    def select_model(self, task_type: str, complexity: int, code_area: str = "unknown") -> Tuple[str, float, str]:
        """
        Select best model using Thompson Sampling.
        Returns: (model, confidence, rationale)
        """
        context_key = ContextKey.model_selection(
            task_type,
            ContextKey.complexity_bucket(complexity),
            code_area
        )

        models = ["haiku", "sonnet", "opus"]
        priors = self.model_priors[context_key]

        # Thompson Sampling: sample from each prior
        samples = {model: priors[model].sample() for model in models}

        # Select best
        best_model = max(samples, key=samples.get)
        confidence = priors[best_model].mean()

        # Generate rationale
        ci_low, ci_high = priors[best_model].confidence_interval()
        n_samples = priors[best_model].alpha + priors[best_model].beta - 2

        rationale = f"Based on {int(n_samples)} past observations for {context_key}: "
        rationale += f"{best_model} has {confidence:.0%} expected success rate "
        rationale += f"(95% CI: {ci_low:.0%}-{ci_high:.0%})"

        return best_model, confidence, rationale

    def select_strategy(self, task_type: str, iteration: int, has_failures: bool) -> Tuple[str, float, str]:
        """
        Select best strategy using Thompson Sampling.
        Returns: (strategy, confidence, rationale)
        """
        context_key = ContextKey.strategy_selection(task_type, iteration, has_failures)

        strategies = [
            "direct_implementation",
            "tdd_approach",
            "refactor_first",
            "minimal_spike",
            "decompose_further"
        ]
        priors = self.strategy_priors[context_key]

        # Thompson Sampling
        samples = {strategy: priors[strategy].sample() for strategy in strategies}

        # Select best
        best_strategy = max(samples, key=samples.get)
        confidence = priors[best_strategy].mean()

        # Generate rationale
        n_samples = priors[best_strategy].alpha + priors[best_strategy].beta - 2

        rationale = f"For {context_key}, {best_strategy} has "
        rationale += f"{confidence:.0%} success rate from {int(n_samples)} observations"

        return best_strategy, confidence, rationale

    def calibrate_confidence(self, stated_confidence: float, decision_type: str) -> float:
        """Adjust confidence based on learned calibration"""
        adjustment = self.calibration_adjustments.get(decision_type, 0.0)
        calibrated = stated_confidence + adjustment
        return max(0.0, min(1.0, calibrated))

    def export_policy(self) -> Dict:
        """Export policy for use in session"""
        policy = {
            "version": 1,
            "updated_at": datetime.now().isoformat(),
            "model_selection": {},
            "strategy_selection": {},
            "calibration": self.calibration_adjustments,
            "recommendations": []
        }

        # Export model selection recommendations
        for context, actions in self.model_priors.items():
            best_model = max(actions.keys(), key=lambda m: actions[m].mean())
            confidence = actions[best_model].mean()
            n_samples = actions[best_model].alpha + actions[best_model].beta - 2

            if n_samples >= 3:  # Only include if we have enough data
                policy["model_selection"][context] = {
                    "recommended": best_model,
                    "confidence": round(confidence, 2),
                    "samples": int(n_samples)
                }

        # Export strategy selection recommendations
        for context, actions in self.strategy_priors.items():
            best_strategy = max(actions.keys(), key=lambda s: actions[s].mean())
            confidence = actions[best_strategy].mean()
            n_samples = actions[best_strategy].alpha + actions[best_strategy].beta - 2

            if n_samples >= 3:
                policy["strategy_selection"][context] = {
                    "recommended": best_strategy,
                    "confidence": round(confidence, 2),
                    "samples": int(n_samples)
                }

        # Generate human-readable recommendations
        policy["recommendations"] = self._generate_recommendations()

        return policy

    def _generate_recommendations(self) -> List[str]:
        """Generate human-readable recommendations from learned data"""
        recommendations = []

        # Model recommendations
        for context, actions in self.model_priors.items():
            for model, prior in actions.items():
                n = prior.alpha + prior.beta - 2
                if n >= 5:  # Enough data
                    success_rate = prior.mean()
                    if success_rate >= 0.8:
                        recommendations.append(
                            f"Use {model} for {context} (learned: {success_rate:.0%} success over {int(n)} tasks)"
                        )
                    elif success_rate <= 0.3:
                        recommendations.append(
                            f"Avoid {model} for {context} (learned: only {success_rate:.0%} success over {int(n)} tasks)"
                        )

        # Calibration recommendations
        for decision_type, adjustment in self.calibration_adjustments.items():
            if abs(adjustment) >= 0.1:
                direction = "overconfident" if adjustment < 0 else "underconfident"
                recommendations.append(
                    f"Agents are {direction} on {decision_type} decisions (adjust by {adjustment:+.0%})"
                )

        return recommendations


def learn_from_session():
    """Main learning function - run after each session"""
    print("ARIA Offline Learning: Extracting episodes...")

    # Extract episodes
    extractor = EpisodeExtractor()
    episodes = extractor.extract_all()
    print(f"  Found {len(episodes)} episodes")

    if not episodes:
        print("  No new episodes to learn from")
        return

    # Save episodes to history
    history_file = HISTORY_DIR / "episodes.jsonl"
    with open(history_file, "a") as f:
        for ep in episodes:
            f.write(json.dumps(asdict(ep)) + "\n")
    print(f"  Saved episodes to {history_file}")

    # Load and update policy
    policy_store = PolicyStore()

    for episode in episodes:
        policy_store.update_from_episode(episode)

    policy_store.save()
    print("  Updated priors saved")

    # Export policy for next session
    policy = policy_store.export_policy()
    with open(LEARNED_DIR / "policy.json", "w") as f:
        json.dump(policy, f, indent=2)
    print(f"  Exported policy to {LEARNED_DIR / 'policy.json'}")

    # Print recommendations
    if policy["recommendations"]:
        print("\n  Learned Recommendations:")
        for rec in policy["recommendations"]:
            print(f"    - {rec}")

    # Print stats
    print(f"\n  Policy Stats:")
    print(f"    Model selection contexts: {len(policy['model_selection'])}")
    print(f"    Strategy selection contexts: {len(policy['strategy_selection'])}")
    print(f"    Calibration adjustments: {len(policy['calibration'])}")


def export_policy():
    """Export current policy for use in session"""
    policy_store = PolicyStore()
    policy = policy_store.export_policy()

    with open(LEARNED_DIR / "policy.json", "w") as f:
        json.dump(policy, f, indent=2)

    print(json.dumps(policy, indent=2))


def query_recommendation(task_type: str, complexity: int, code_area: str = "unknown"):
    """Query the learned policy for a recommendation"""
    policy_store = PolicyStore()

    model, model_conf, model_rationale = policy_store.select_model(task_type, complexity, code_area)
    strategy, strat_conf, strat_rationale = policy_store.select_strategy(task_type, 1, False)

    print(f"Task: {task_type} (complexity {complexity}, area: {code_area})")
    print(f"\nModel Selection:")
    print(f"  Recommended: {model} ({model_conf:.0%} confidence)")
    print(f"  Rationale: {model_rationale}")
    print(f"\nStrategy Selection:")
    print(f"  Recommended: {strategy} ({strat_conf:.0%} confidence)")
    print(f"  Rationale: {strat_rationale}")


def show_stats():
    """Show learning statistics"""
    # Count episodes
    history_file = HISTORY_DIR / "episodes.jsonl"
    episode_count = 0
    if history_file.exists():
        with open(history_file) as f:
            episode_count = sum(1 for _ in f)

    # Load policy
    policy_file = LEARNED_DIR / "policy.json"
    if policy_file.exists():
        with open(policy_file) as f:
            policy = json.load(f)
    else:
        policy = {}

    print("ARIA Offline Learning Statistics")
    print("=" * 40)
    print(f"Total episodes collected: {episode_count}")
    print(f"Model selection contexts: {len(policy.get('model_selection', {}))}")
    print(f"Strategy selection contexts: {len(policy.get('strategy_selection', {}))}")
    print(f"Last updated: {policy.get('updated_at', 'Never')}")

    if policy.get("recommendations"):
        print(f"\nLearned Recommendations ({len(policy['recommendations'])}):")
        for rec in policy["recommendations"]:
            print(f"  - {rec}")


def main():
    if len(sys.argv) < 2:
        print("Usage: offline-learner.py <command> [args]")
        print("Commands:")
        print("  learn              - Learn from current session data")
        print("  export-policy      - Export policy for session use")
        print("  query <type> <complexity> [area] - Query recommendation")
        print("  stats              - Show learning statistics")
        return

    command = sys.argv[1]

    if command == "learn":
        learn_from_session()
    elif command == "export-policy":
        export_policy()
    elif command == "query" and len(sys.argv) >= 4:
        task_type = sys.argv[2]
        complexity = int(sys.argv[3])
        code_area = sys.argv[4] if len(sys.argv) > 4 else "unknown"
        query_recommendation(task_type, complexity, code_area)
    elif command == "stats":
        show_stats()
    else:
        print(f"Unknown command: {command}")


if __name__ == "__main__":
    main()
