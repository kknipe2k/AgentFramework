#!/bin/bash
#
# ARIA Meta Reasoning Engine with Offline RL Integration
#
# Provides abstract meta-reasoning primitives that use learned policies
# to make better decisions over time.
#
# Usage:
#   source .aria/lib/meta-reasoning.sh
#   meta_select_model "feature" 6 "api"
#   meta_select_strategy "bugfix" 1 false
#   meta_map_solution_space "Implement retry logic"
#

ARIA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEARNED_DIR="$ARIA_ROOT/learned"
POLICY_FILE="$LEARNED_DIR/policy.json"
LEARNER_SCRIPT="$ARIA_ROOT/lib/offline-learner.py"

# Ensure learned directory exists
mkdir -p "$LEARNED_DIR"

# =============================================================================
# POLICY LOADING
# =============================================================================

_load_policy() {
    if [[ -f "$POLICY_FILE" ]]; then
        cat "$POLICY_FILE"
    else
        echo '{"model_selection":{},"strategy_selection":{},"calibration":{},"recommendations":[]}'
    fi
}

_get_policy_recommendation() {
    local section="$1"
    local context_key="$2"
    local policy
    policy=$(_load_policy)
    echo "$policy" | python3 -c "
import json, sys
policy = json.load(sys.stdin)
section = '$section'
key = '$context_key'
if section in policy and key in policy[section]:
    rec = policy[section][key]
    print(f\"{rec['recommended']}|{rec['confidence']}|{rec['samples']}\")
else:
    print('')
" 2>/dev/null || echo ""
}

# =============================================================================
# META REASONING PRIMITIVES
# =============================================================================

# Map solution space: enumerate approaches with probabilities
# Usage: meta_map_solution_space "problem description"
# Output: JSON array of approaches
meta_map_solution_space() {
    local problem="$1"
    local task_type="${2:-general}"

    # Default approaches with base probabilities
    local approaches=(
        "direct_implementation|0.6|Implement directly without refactoring"
        "tdd_approach|0.7|Write tests first, then implement"
        "refactor_first|0.65|Clean up related code, then implement"
        "minimal_spike|0.75|Build minimal prototype to validate approach"
        "decompose_further|0.5|Break into smaller sub-tasks"
    )

    # Check learned policy for adjustments
    local policy
    policy=$(_load_policy)

    # Output as JSON
    echo "["
    local first=true
    for approach in "${approaches[@]}"; do
        IFS='|' read -r name prob desc <<< "$approach"

        # Check if we have learned data for this approach
        local learned_prob
        learned_prob=$(echo "$policy" | python3 -c "
import json, sys
policy = json.load(sys.stdin)
# Look for any context that matches this strategy
for ctx, rec in policy.get('strategy_selection', {}).items():
    if rec.get('recommended') == '$name':
        print(rec['confidence'])
        break
" 2>/dev/null)

        if [[ -n "$learned_prob" ]]; then
            prob="$learned_prob"
        fi

        if [[ "$first" != "true" ]]; then
            echo ","
        fi
        first=false

        echo "  {\"name\": \"$name\", \"probability\": $prob, \"description\": \"$desc\"}"
    done
    echo "]"
}

# Select model using Thompson Sampling from learned priors
# Usage: meta_select_model "task_type" complexity "code_area"
# Output: model|confidence|rationale
meta_select_model() {
    local task_type="$1"
    local complexity="$2"
    local code_area="${3:-unknown}"

    # Determine complexity bucket
    local complexity_bucket
    if [[ "$complexity" -le 3 ]]; then
        complexity_bucket="low"
    elif [[ "$complexity" -le 7 ]]; then
        complexity_bucket="medium"
    else
        complexity_bucket="high"
    fi

    local context_key="${task_type}|${complexity_bucket}|${code_area}"

    # Check learned policy
    local recommendation
    recommendation=$(_get_policy_recommendation "model_selection" "$context_key")

    if [[ -n "$recommendation" ]]; then
        IFS='|' read -r model confidence samples <<< "$recommendation"
        echo "$model|$confidence|Learned from $samples past observations for $context_key"
        return 0
    fi

    # Fall back to heuristics if no learned data
    local model rationale
    if [[ "$complexity" -le 3 ]]; then
        model="haiku"
        rationale="Low complexity task, using efficient model (no learned data for $context_key)"
    elif [[ "$complexity" -le 7 ]]; then
        model="sonnet"
        rationale="Medium complexity, balanced choice (no learned data for $context_key)"
    else
        model="opus"
        rationale="High complexity, using most capable model (no learned data for $context_key)"
    fi

    echo "$model|0.5|$rationale"
}

# Select strategy using learned priors
# Usage: meta_select_strategy "task_type" iteration has_failures
# Output: strategy|confidence|rationale
meta_select_strategy() {
    local task_type="$1"
    local iteration="${2:-1}"
    local has_failures="${3:-false}"

    local iter_bucket
    if [[ "$iteration" -eq 1 ]]; then
        iter_bucket="first"
    else
        iter_bucket="retry"
    fi

    local fail_str
    if [[ "$has_failures" == "true" ]]; then
        fail_str="has_failures"
    else
        fail_str="no_failures"
    fi

    local context_key="${task_type}|${iter_bucket}|${fail_str}"

    # Check learned policy
    local recommendation
    recommendation=$(_get_policy_recommendation "strategy_selection" "$context_key")

    if [[ -n "$recommendation" ]]; then
        IFS='|' read -r strategy confidence samples <<< "$recommendation"
        echo "$strategy|$confidence|Learned from $samples observations"
        return 0
    fi

    # Fall back to heuristics
    local strategy rationale
    if [[ "$has_failures" == "true" ]]; then
        strategy="tdd_approach"
        rationale="After failures, TDD provides better feedback loop (no learned data)"
    elif [[ "$iteration" -gt 1 ]]; then
        strategy="minimal_spike"
        rationale="On retry, validate approach with spike first (no learned data)"
    else
        strategy="direct_implementation"
        rationale="First attempt, try direct approach (no learned data)"
    fi

    echo "$strategy|0.5|$rationale"
}

# Calibrate confidence based on learned adjustments
# Usage: meta_calibrate_confidence 0.8 "architecture"
# Output: adjusted_confidence
meta_calibrate_confidence() {
    local stated_confidence="$1"
    local decision_type="${2:-implementation}"

    local policy
    policy=$(_load_policy)

    local adjustment
    adjustment=$(echo "$policy" | python3 -c "
import json, sys
policy = json.load(sys.stdin)
adj = policy.get('calibration', {}).get('$decision_type', 0.0)
print(adj)
" 2>/dev/null)

    if [[ -z "$adjustment" ]]; then
        adjustment="0.0"
    fi

    # Calculate adjusted confidence
    python3 -c "
conf = $stated_confidence + $adjustment
print(max(0.0, min(1.0, conf)))
"
}

# Check for dead-end signals
# Usage: meta_check_dead_end "file_path"
# Output: true|false|reason
meta_check_dead_end() {
    local file_path="$1"
    local signals_file="$ARIA_ROOT/state/signals.jsonl"

    if [[ ! -f "$signals_file" ]]; then
        echo "false|No signal data"
        return 0
    fi

    # Count recent edits to same file
    local edit_count
    edit_count=$(grep -c "\"file_path\":\"$file_path\"" "$signals_file" 2>/dev/null || echo "0")

    if [[ "$edit_count" -gt 3 ]]; then
        echo "true|File edited $edit_count times - possible dead end"
        return 0
    fi

    # Check for flip-flopping (edit, undo, edit pattern)
    # This is a simplified check
    local recent_actions
    recent_actions=$(tail -20 "$signals_file" 2>/dev/null | grep "Edit" | wc -l)

    if [[ "$recent_actions" -gt 5 ]]; then
        echo "true|High edit frequency - consider different approach"
        return 0
    fi

    echo "false|No dead-end signals detected"
}

# Record outcome for learning
# Usage: meta_record_outcome "model" "task_type" complexity "success|failure" "story_id"
meta_record_outcome() {
    local model="$1"
    local task_type="$2"
    local complexity="$3"
    local outcome="$4"
    local story_id="${5:-unknown}"

    # Append to model learning log
    local learning_file="$ARIA_ROOT/logs/model_learning.json"

    # Use Python to update JSON safely
    python3 -c "
import json
from datetime import datetime
from pathlib import Path

learning_file = Path('$learning_file')

# Load or initialize
if learning_file.exists():
    with open(learning_file) as f:
        data = json.load(f)
else:
    data = {
        'version': 1,
        'created_at': datetime.now().isoformat(),
        'task_types': {},
        'complexity_levels': {},
        'recent_outcomes': []
    }

# Update task_types
task_type = '$task_type'
model = '$model'
outcome = '$outcome'

if task_type not in data['task_types']:
    data['task_types'][task_type] = {}
if model not in data['task_types'][task_type]:
    data['task_types'][task_type][model] = {'success': 0, 'fail': 0}

if outcome == 'success':
    data['task_types'][task_type][model]['success'] += 1
else:
    data['task_types'][task_type][model]['fail'] += 1

# Update complexity_levels
complexity = int('$complexity')
if complexity <= 3:
    level = 'low'
elif complexity <= 7:
    level = 'medium'
else:
    level = 'high'

if level not in data['complexity_levels']:
    data['complexity_levels'][level] = {}
if model not in data['complexity_levels'][level]:
    data['complexity_levels'][level][model] = {'success': 0, 'fail': 0}

if outcome == 'success':
    data['complexity_levels'][level][model]['success'] += 1
else:
    data['complexity_levels'][level][model]['fail'] += 1

# Add to recent_outcomes
data['recent_outcomes'].append({
    'timestamp': datetime.now().isoformat(),
    'model': model,
    'task_type': task_type,
    'complexity': complexity,
    'complexity_level': level,
    'outcome': outcome,
    'story_id': '$story_id'
})

# Keep only last 100 recent outcomes
data['recent_outcomes'] = data['recent_outcomes'][-100:]
data['updated_at'] = datetime.now().isoformat()

# Save
learning_file.parent.mkdir(parents=True, exist_ok=True)
with open(learning_file, 'w') as f:
    json.dump(data, f, indent=2)

print('Outcome recorded')
"
}

# Trigger offline learning (run between sessions)
meta_learn() {
    if [[ -f "$LEARNER_SCRIPT" ]]; then
        python3 "$LEARNER_SCRIPT" learn
    else
        echo "Warning: Offline learner not found at $LEARNER_SCRIPT"
    fi
}

# Show learning statistics
meta_stats() {
    if [[ -f "$LEARNER_SCRIPT" ]]; then
        python3 "$LEARNER_SCRIPT" stats
    else
        echo "Warning: Offline learner not found at $LEARNER_SCRIPT"
    fi
}

# Get learned recommendations for display
meta_get_recommendations() {
    local policy
    policy=$(_load_policy)

    echo "$policy" | python3 -c "
import json, sys
policy = json.load(sys.stdin)
recs = policy.get('recommendations', [])
if recs:
    for rec in recs:
        print(f'  - {rec}')
else:
    print('  No recommendations yet (need more training data)')
"
}

# =============================================================================
# META REASONING ENGINE - COMPOSITE FUNCTION
# =============================================================================

# Full meta-reasoning cycle for a task
# Usage: meta_reason "task_description" "task_type" complexity
# Output: Structured reasoning output
meta_reason() {
    local task_description="$1"
    local task_type="${2:-general}"
    local complexity="${3:-5}"

    echo "============================================="
    echo "META REASONING ENGINE"
    echo "============================================="
    echo ""
    echo "TASK: $task_description"
    echo "TYPE: $task_type | COMPLEXITY: $complexity"
    echo ""

    # 1. MAP SOLUTION SPACE
    echo "1. SOLUTION SPACE"
    echo "-----------------"
    meta_map_solution_space "$task_description" "$task_type"
    echo ""

    # 2. SELECT PATH
    echo "2. CHOSEN PATH"
    echo "--------------"
    local model_result strategy_result
    model_result=$(meta_select_model "$task_type" "$complexity")
    strategy_result=$(meta_select_strategy "$task_type" 1 false)

    IFS='|' read -r model model_conf model_rationale <<< "$model_result"
    IFS='|' read -r strategy strat_conf strat_rationale <<< "$strategy_result"

    echo "Model: $model (confidence: $model_conf)"
    echo "  Rationale: $model_rationale"
    echo ""
    echo "Strategy: $strategy (confidence: $strat_conf)"
    echo "  Rationale: $strat_rationale"
    echo ""

    # 3. CHECKPOINTS
    echo "3. EXECUTION CHECKPOINTS"
    echo "------------------------"
    echo "  [ ] After reading code: Does approach still make sense?"
    echo "  [ ] After first edit: Is this solving the right problem?"
    echo "  [ ] Before committing: Does this match acceptance criteria?"
    echo ""

    # 4. DEAD-END SIGNALS TO WATCH
    echo "4. DEAD-END SIGNALS"
    echo "-------------------"
    echo "  - Same file edited 3+ times"
    echo "  - Test flip-flopping (pass→fail→pass)"
    echo "  - Increasing complexity instead of decreasing"
    echo ""

    # 5. LEARNED RECOMMENDATIONS
    echo "5. LEARNED RECOMMENDATIONS"
    echo "--------------------------"
    meta_get_recommendations
    echo ""

    echo "============================================="
}

# Export functions for use in other scripts
export -f meta_map_solution_space
export -f meta_select_model
export -f meta_select_strategy
export -f meta_calibrate_confidence
export -f meta_check_dead_end
export -f meta_record_outcome
export -f meta_learn
export -f meta_stats
export -f meta_reason
