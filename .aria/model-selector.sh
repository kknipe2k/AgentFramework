#!/bin/bash
# ARIA Model Selector & Token Tracker
# Intelligently selects model (opus/sonnet/haiku) based on:
# 1. Task complexity (heuristic baseline)
# 2. Learned success rates (adapts over time)
# 3. Budget constraints
# 4. Failure escalation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# Check dependencies
aria_check_deps jq python3 bc || exit 1

ARIA_DIR="$SCRIPT_DIR"
STATE_DIR="$ARIA_DIR/state"
RALPH_DIR="$ARIA_DIR/ralph"
PRD_FILE="$RALPH_DIR/prd.json"
LOGS_DIR="$ARIA_DIR/logs"
USAGE_FILE="$LOGS_DIR/token_usage.json"
LEARNING_FILE="$LOGS_DIR/model_learning.json"

# Colors from common.sh
RED="$ARIA_RED"
GREEN="$ARIA_GREEN"
YELLOW="$ARIA_YELLOW"
BLUE="$ARIA_BLUE"
MAGENTA="$ARIA_MAGENTA"
NC="$ARIA_NC"

mkdir -p "$LOGS_DIR" "$STATE_DIR"

# ============================================
# MODEL CONFIGURATION
# ============================================

# Model costs (per 1M tokens, approximate)
declare -A INPUT_COSTS=(
    ["opus"]=15.00
    ["sonnet"]=3.00
    ["haiku"]=0.25
)

declare -A OUTPUT_COSTS=(
    ["opus"]=75.00
    ["sonnet"]=15.00
    ["haiku"]=1.25
)

# Model capabilities (1-10 scale)
declare -A MODEL_CAPABILITY=(
    ["opus"]=10
    ["sonnet"]=7
    ["haiku"]=4
)

# Default budget (in dollars)
DEFAULT_BUDGET=${ARIA_MODEL_BUDGET:-10.00}

# ============================================
# USAGE TRACKING
# ============================================

init_usage() {
    if [[ ! -f "$USAGE_FILE" ]]; then
        cat > "$USAGE_FILE" << EOF
{
    "session_start": "$(date -Iseconds)",
    "budget": $DEFAULT_BUDGET,
    "total_input_tokens": 0,
    "total_output_tokens": 0,
    "total_cost": 0.0,
    "by_model": {
        "opus": {"input": 0, "output": 0, "cost": 0.0, "calls": 0},
        "sonnet": {"input": 0, "output": 0, "cost": 0.0, "calls": 0},
        "haiku": {"input": 0, "output": 0, "cost": 0.0, "calls": 0}
    },
    "history": []
}
EOF
    fi
}

# Record token usage
record_usage() {
    local model="$1"
    local input_tokens="$2"
    local output_tokens="$3"
    local task_id="${4:-unknown}"

    init_usage

    # Calculate cost
    local input_cost=$(echo "scale=6; $input_tokens * ${INPUT_COSTS[$model]} / 1000000" | bc)
    local output_cost=$(echo "scale=6; $output_tokens * ${OUTPUT_COSTS[$model]} / 1000000" | bc)
    local total_cost=$(echo "scale=6; $input_cost + $output_cost" | bc)

    # Update usage file with Python (more reliable JSON handling)
    python3 << EOF
import json
from datetime import datetime

with open('$USAGE_FILE', 'r') as f:
    data = json.load(f)

# Update totals
data['total_input_tokens'] += $input_tokens
data['total_output_tokens'] += $output_tokens
data['total_cost'] = round(data['total_cost'] + $total_cost, 6)

# Update by model
data['by_model']['$model']['input'] += $input_tokens
data['by_model']['$model']['output'] += $output_tokens
data['by_model']['$model']['cost'] = round(data['by_model']['$model']['cost'] + $total_cost, 6)
data['by_model']['$model']['calls'] += 1

# Add to history
data['history'].append({
    'timestamp': datetime.now().isoformat(),
    'model': '$model',
    'input_tokens': $input_tokens,
    'output_tokens': $output_tokens,
    'cost': round($total_cost, 6),
    'task_id': '$task_id'
})

# Keep only last 100 history entries
data['history'] = data['history'][-100:]

with open('$USAGE_FILE', 'w') as f:
    json.dump(data, f, indent=2)
EOF

    echo "$total_cost"
}

# Get current usage stats
get_usage() {
    init_usage
    cat "$USAGE_FILE"
}

# Get remaining budget
get_remaining_budget() {
    init_usage
    python3 << EOF
import json
with open('$USAGE_FILE', 'r') as f:
    data = json.load(f)
remaining = data['budget'] - data['total_cost']
print(f"{remaining:.4f}")
EOF
}

# Check if over budget
is_over_budget() {
    local remaining=$(get_remaining_budget)
    local result=$(echo "$remaining <= 0" | bc)
    [[ "$result" == "1" ]]
}

# ============================================
# LEARNING SYSTEM
# ============================================

# Initialize learning data
init_learning() {
    if [[ ! -f "$LEARNING_FILE" ]]; then
        cat > "$LEARNING_FILE" << 'EOF'
{
    "version": 1,
    "created_at": "",
    "updated_at": "",
    "task_types": {
        "feature": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "bugfix": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "refactoring": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "testing": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "documentation": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "setup": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "general": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}}
    },
    "complexity_levels": {
        "low": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "medium": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}},
        "high": {"opus": {"success": 0, "fail": 0}, "sonnet": {"success": 0, "fail": 0}, "haiku": {"success": 0, "fail": 0}}
    },
    "recent_outcomes": []
}
EOF
        # Update timestamps
        python3 << EOF
import json
from datetime import datetime
with open('$LEARNING_FILE', 'r') as f:
    data = json.load(f)
data['created_at'] = datetime.now().isoformat()
data['updated_at'] = datetime.now().isoformat()
with open('$LEARNING_FILE', 'w') as f:
    json.dump(data, f, indent=2)
EOF
    fi
}

# Record outcome of a model selection (called after iteration completes)
record_outcome() {
    local model="$1"
    local task_type="$2"
    local complexity="$3"
    local outcome="$4"  # "success" or "fail"
    local story_id="${5:-unknown}"

    init_learning

    # Determine complexity level
    local complexity_level="medium"
    if [[ $complexity -le 3 ]]; then
        complexity_level="low"
    elif [[ $complexity -ge 7 ]]; then
        complexity_level="high"
    fi

    python3 << EOF
import json
from datetime import datetime

with open('$LEARNING_FILE', 'r') as f:
    data = json.load(f)

# Update task type stats
task_type = '$task_type'
if task_type not in data['task_types']:
    data['task_types'][task_type] = {
        "opus": {"success": 0, "fail": 0},
        "sonnet": {"success": 0, "fail": 0},
        "haiku": {"success": 0, "fail": 0}
    }

data['task_types'][task_type]['$model']['$outcome'] += 1

# Update complexity level stats
data['complexity_levels']['$complexity_level']['$model']['$outcome'] += 1

# Add to recent outcomes (keep last 50)
data['recent_outcomes'].append({
    'timestamp': datetime.now().isoformat(),
    'model': '$model',
    'task_type': task_type,
    'complexity': $complexity,
    'complexity_level': '$complexity_level',
    'outcome': '$outcome',
    'story_id': '$story_id'
})
data['recent_outcomes'] = data['recent_outcomes'][-50:]

data['updated_at'] = datetime.now().isoformat()

with open('$LEARNING_FILE', 'w') as f:
    json.dump(data, f, indent=2)
EOF

    echo "Recorded: $model on $task_type ($complexity_level) = $outcome"
}

# Get success rate for a model on a task type
get_success_rate() {
    local model="$1"
    local task_type="$2"

    init_learning

    python3 << EOF
import json

with open('$LEARNING_FILE', 'r') as f:
    data = json.load(f)

task_type = '$task_type'
model = '$model'

if task_type in data['task_types'] and model in data['task_types'][task_type]:
    stats = data['task_types'][task_type][model]
    total = stats['success'] + stats['fail']
    if total >= 3:  # Need at least 3 samples
        rate = stats['success'] / total
        print(f"{rate:.2f}")
    else:
        print("-1")  # Not enough data
else:
    print("-1")  # No data
EOF
}

# Get best model for task type based on learned data
get_learned_model() {
    local task_type="$1"
    local min_cost="${2:-false}"  # If true, prefer cheaper models when success rates are similar

    init_learning

    python3 << EOF
import json

with open('$LEARNING_FILE', 'r') as f:
    data = json.load(f)

task_type = '$task_type'
min_cost = '$min_cost' == 'true'

# Get success rates for each model
models = ['haiku', 'sonnet', 'opus']  # Order by cost (cheapest first)
rates = {}
has_data = False

for model in models:
    if task_type in data['task_types'] and model in data['task_types'][task_type]:
        stats = data['task_types'][task_type][model]
        total = stats['success'] + stats['fail']
        if total >= 3:
            rates[model] = stats['success'] / total
            has_data = True
        else:
            rates[model] = -1
    else:
        rates[model] = -1

if not has_data:
    print("none")  # No learned data, use heuristics
else:
    # Find best model
    # If min_cost, prefer cheaper model if success rate is within 10%
    best_model = None
    best_rate = -1

    for model in models:
        if rates[model] >= 0:
            if best_model is None:
                best_model = model
                best_rate = rates[model]
            elif min_cost:
                # Prefer cheaper model if rate is within 10%
                if rates[model] >= best_rate - 0.1:
                    best_model = model
                    best_rate = rates[model]
            else:
                # Just pick highest rate
                if rates[model] > best_rate:
                    best_model = model
                    best_rate = rates[model]

    if best_model and best_rate >= 0.5:  # Only recommend if >50% success
        print(best_model)
    else:
        print("none")  # Success rate too low, use heuristics
EOF
}

# Get learning stats summary
get_learning_stats() {
    init_learning

    python3 << EOF
import json

with open('$LEARNING_FILE', 'r') as f:
    data = json.load(f)

print("Task Type Success Rates:")
print("-" * 60)

for task_type, models in sorted(data['task_types'].items()):
    rates = []
    for model in ['haiku', 'sonnet', 'opus']:
        stats = models[model]
        total = stats['success'] + stats['fail']
        if total > 0:
            rate = stats['success'] / total * 100
            rates.append(f"{model}: {rate:.0f}% ({total})")
        else:
            rates.append(f"{model}: -")
    print(f"  {task_type:15} {' | '.join(rates)}")

print("")
print("Recent outcomes:", len(data['recent_outcomes']))

# Show recent success/fail
recent_success = sum(1 for o in data['recent_outcomes'][-10:] if o['outcome'] == 'success')
recent_fail = sum(1 for o in data['recent_outcomes'][-10:] if o['outcome'] == 'fail')
print(f"Last 10: {recent_success} success, {recent_fail} fail")
EOF
}

# ============================================
# TASK COMPLEXITY ANALYSIS
# ============================================

# Analyze task complexity from description
analyze_complexity() {
    local task_description="$1"
    local files_changed="${2:-0}"
    local story_priority="${3:-5}"

    local complexity=5  # Default medium

    # Keywords that suggest high complexity
    local high_keywords="refactor|architect|redesign|migrate|security|performance|optimize|complex|multi-file|database|schema"
    if echo "$task_description" | grep -qiE "$high_keywords"; then
        complexity=$((complexity + 3))
    fi

    # Keywords that suggest low complexity
    local low_keywords="typo|comment|rename|format|style|simple|minor|small|docs|readme"
    if echo "$task_description" | grep -qiE "$low_keywords"; then
        complexity=$((complexity - 3))
    fi

    # Adjust by files changed
    if [[ $files_changed -gt 5 ]]; then
        complexity=$((complexity + 2))
    elif [[ $files_changed -le 1 ]]; then
        complexity=$((complexity - 1))
    fi

    # Adjust by priority (1 = highest priority = more important = use better model)
    if [[ $story_priority -le 2 ]]; then
        complexity=$((complexity + 2))
    elif [[ $story_priority -ge 4 ]]; then
        complexity=$((complexity - 1))
    fi

    # Clamp to 1-10
    [[ $complexity -lt 1 ]] && complexity=1
    [[ $complexity -gt 10 ]] && complexity=10

    echo "$complexity"
}

# Get task type from PRD story
get_task_type() {
    local story_id="$1"

    if [[ ! -f "$PRD_FILE" ]]; then
        echo "unknown"
        return
    fi

    local title=$(jq -r ".userStories[] | select(.id == \"$story_id\") | .title" "$PRD_FILE" 2>/dev/null)

    # Categorize by keywords in title
    if echo "$title" | grep -qiE "test|spec|coverage"; then
        echo "testing"
    elif echo "$title" | grep -qiE "doc|readme|comment"; then
        echo "documentation"
    elif echo "$title" | grep -qiE "fix|bug|error|issue"; then
        echo "bugfix"
    elif echo "$title" | grep -qiE "refactor|clean|simplify"; then
        echo "refactoring"
    elif echo "$title" | grep -qiE "feature|add|implement|create"; then
        echo "feature"
    elif echo "$title" | grep -qiE "setup|config|init"; then
        echo "setup"
    else
        echo "general"
    fi
}

# ============================================
# MODEL SELECTION
# ============================================

# Select model based on task
# Priority: 1. Forced model 2. Learned data 3. Heuristics 4. Failure escalation
select_model() {
    local task_description="${1:-}"
    local story_id="${2:-}"
    local force_model="${3:-}"
    local consecutive_failures="${4:-0}"

    # If model is forced, use it
    if [[ -n "$force_model" ]]; then
        echo "$force_model"
        return
    fi

    # Check budget
    local remaining=$(get_remaining_budget)
    local budget_pct=$(echo "scale=2; $remaining / $DEFAULT_BUDGET * 100" | bc 2>/dev/null || echo "100")

    # Get task complexity
    local complexity=5
    if [[ -n "$task_description" ]]; then
        complexity=$(analyze_complexity "$task_description")
    fi

    # Get task type
    local task_type="general"
    if [[ -n "$story_id" ]]; then
        task_type=$(get_task_type "$story_id")
    fi

    local selected_model="sonnet"  # Default
    local selection_reason="default"

    # ==========================================
    # STEP 1: Check learned data first
    # ==========================================
    local prefer_cheap="false"
    (( $(echo "$budget_pct < 50" | bc -l) )) && prefer_cheap="true"

    local learned_model=$(get_learned_model "$task_type" "$prefer_cheap")

    if [[ "$learned_model" != "none" ]]; then
        selected_model="$learned_model"
        selection_reason="learned:$task_type"
    else
        # ==========================================
        # STEP 2: Fall back to heuristics
        # ==========================================
        selection_reason="heuristic"

        # 2a. Budget-based baseline
        if (( $(echo "$budget_pct < 20" | bc -l) )); then
            selected_model="haiku"
            selection_reason="heuristic:low_budget"
        elif (( $(echo "$budget_pct < 50" | bc -l) )); then
            selected_model="sonnet"
        fi

        # 2b. Complexity-based adjustment
        if [[ $complexity -ge 8 ]]; then
            selected_model="opus"
            selection_reason="heuristic:high_complexity"
        elif [[ $complexity -ge 5 ]]; then
            selected_model="sonnet"
        elif [[ $complexity -le 3 ]]; then
            selected_model="haiku"
            selection_reason="heuristic:low_complexity"
        fi

        # 2c. Task type adjustment (heuristic rules)
        case "$task_type" in
            "documentation")
                [[ "$selected_model" == "opus" ]] && selected_model="sonnet"
                ;;
            "testing")
                [[ "$selected_model" == "haiku" ]] && selected_model="sonnet"
                ;;
            "refactoring"|"feature")
                [[ $complexity -ge 6 ]] && selected_model="opus"
                ;;
            "bugfix")
                [[ "$selected_model" == "haiku" ]] && selected_model="sonnet"
                ;;
        esac
    fi

    # ==========================================
    # STEP 3: Failure escalation (overrides learned)
    # ==========================================
    if [[ $consecutive_failures -ge 3 ]]; then
        # If failing repeatedly, escalate model
        case "$selected_model" in
            "haiku") selected_model="sonnet" ;;
            "sonnet") selected_model="opus" ;;
        esac
    fi

    # 5. Final budget check - never use opus if nearly broke
    if (( $(echo "$budget_pct < 10" | bc -l) )); then
        selected_model="haiku"
    fi

    echo "$selected_model"
}

# Get model flag for Claude CLI
get_model_flag() {
    local model="$1"

    case "$model" in
        "opus")   echo "--model claude-opus-4-5-20251101" ;;
        "sonnet") echo "--model claude-sonnet-4-20250514" ;;
        "haiku")  echo "--model claude-haiku-3-5-20241022" ;;
        *)        echo "" ;;  # Use default
    esac
}

# ============================================
# REPORTING
# ============================================

show_status() {
    init_usage

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              ARIA Model & Token Status                     ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    local data=$(cat "$USAGE_FILE")

    local total_cost=$(echo "$data" | jq -r '.total_cost')
    local budget=$(echo "$data" | jq -r '.budget')
    local remaining=$(get_remaining_budget)
    local pct=$(echo "scale=1; $remaining / $budget * 100" | bc 2>/dev/null || echo "100")

    echo -e "${BLUE}Budget:${NC}"
    echo "  Total:     \$$budget"
    echo "  Spent:     \$$total_cost"
    echo "  Remaining: \$$remaining ($pct%)"
    echo ""

    # Progress bar
    local bar_width=40
    local filled=$(echo "scale=0; $bar_width * (100 - $pct) / 100" | bc 2>/dev/null || echo "0")
    local empty=$((bar_width - filled))

    printf "  ["
    printf "%${filled}s" | tr ' ' '#'
    printf "%${empty}s" | tr ' ' '-'
    printf "] $pct%% remaining\n"
    echo ""

    echo -e "${BLUE}Usage by Model:${NC}"
    for model in opus sonnet haiku; do
        local calls=$(echo "$data" | jq -r ".by_model.$model.calls")
        local cost=$(echo "$data" | jq -r ".by_model.$model.cost")
        local input=$(echo "$data" | jq -r ".by_model.$model.input")
        local output=$(echo "$data" | jq -r ".by_model.$model.output")

        if [[ "$calls" != "0" ]]; then
            printf "  %-8s %3d calls | %8d in | %8d out | \$%.4f\n" "$model:" "$calls" "$input" "$output" "$cost"
        fi
    done
    echo ""

    # Recommendations
    if (( $(echo "$pct < 20" | bc -l) )); then
        echo -e "${RED}⚠ Budget low - system will prefer haiku${NC}"
    elif (( $(echo "$pct < 50" | bc -l) )); then
        echo -e "${YELLOW}Budget moderate - avoiding opus for simple tasks${NC}"
    else
        echo -e "${GREEN}Budget healthy - optimal model selection enabled${NC}"
    fi
    echo ""
}

show_recommendation() {
    local task_description="$1"
    local story_id="${2:-}"

    local model=$(select_model "$task_description" "$story_id")
    local complexity=$(analyze_complexity "$task_description")
    local task_type=$(get_task_type "$story_id")
    local remaining=$(get_remaining_budget)

    echo ""
    echo -e "${BLUE}Model Recommendation:${NC}"
    echo ""
    echo "  Task: $task_description"
    echo "  Type: $task_type"
    echo "  Complexity: $complexity/10"
    echo "  Budget remaining: \$$remaining"
    echo ""
    echo -e "  ${GREEN}Recommended model: $model${NC}"
    echo "  Flag: $(get_model_flag $model)"
    echo ""
}

# ============================================
# BUDGET MANAGEMENT
# ============================================

set_budget() {
    local new_budget="$1"

    init_usage

    python3 << EOF
import json
with open('$USAGE_FILE', 'r') as f:
    data = json.load(f)
data['budget'] = float($new_budget)
with open('$USAGE_FILE', 'w') as f:
    json.dump(data, f, indent=2)
EOF

    echo -e "${GREEN}Budget set to \$$new_budget${NC}"
}

reset_usage() {
    rm -f "$USAGE_FILE"
    init_usage
    echo -e "${GREEN}Usage tracking reset${NC}"
}

# ============================================
# CLI
# ============================================

main() {
    local command="${1:-help}"
    shift || true

    case "$command" in
        "select")
            select_model "$@"
            ;;
        "recommend")
            show_recommendation "$@"
            ;;
        "flag")
            local model=$(select_model "$@")
            get_model_flag "$model"
            ;;
        "record")
            # record <model> <input_tokens> <output_tokens> [task_id]
            record_usage "$@"
            ;;
        "status"|"usage")
            show_status
            ;;
        "remaining")
            get_remaining_budget
            ;;
        "budget")
            if [[ -n "$1" ]]; then
                set_budget "$1"
            else
                echo "Current budget: \$$(python3 -c "import json; print(json.load(open('$USAGE_FILE'))['budget'])" 2>/dev/null || echo "$DEFAULT_BUDGET")"
            fi
            ;;
        "reset")
            reset_usage
            ;;
        "complexity")
            analyze_complexity "$@"
            ;;
        "outcome")
            # record_outcome <model> <task_type> <complexity> <outcome> [story_id]
            record_outcome "$@"
            ;;
        "stats"|"learn")
            get_learning_stats
            ;;
        "learn-reset")
            rm -f "$LEARNING_FILE"
            init_learning
            echo -e "${GREEN}Learning data reset${NC}"
            ;;
        "help"|*)
            echo "ARIA Model Selector & Token Tracker"
            echo ""
            echo "Usage: $0 <command> [args]"
            echo ""
            echo "Model Selection:"
            echo "  select <task> [story_id] [force_model] [failures]"
            echo "                          - Select best model for task"
            echo "  recommend <task>        - Show recommendation with reasoning"
            echo "  flag <task>             - Get Claude CLI model flag"
            echo "  complexity <task>       - Analyze task complexity (1-10)"
            echo ""
            echo "Token Tracking:"
            echo "  record <model> <in> <out> [task_id]"
            echo "                          - Record token usage"
            echo "  status                  - Show usage status"
            echo "  remaining               - Show remaining budget"
            echo ""
            echo "Budget:"
            echo "  budget [amount]         - Get/set budget"
            echo "  reset                   - Reset usage tracking"
            echo ""
            echo "Learning:"
            echo "  outcome <model> <task_type> <complexity> <outcome> [story_id]"
            echo "                          - Record success/fail outcome"
            echo "  stats                   - Show learning statistics"
            echo "  learn-reset             - Reset learning data"
            echo ""
            echo "Model Selection Logic:"
            echo "  - Complexity 1-3:  haiku (simple tasks)"
            echo "  - Complexity 4-7:  sonnet (moderate tasks)"
            echo "  - Complexity 8-10: opus (complex tasks)"
            echo "  - Budget <20%:    force haiku"
            echo "  - Budget <50%:    avoid opus"
            echo "  - 3+ failures:    escalate model"
            echo "  - Learned data takes priority over heuristics"
            echo ""
            echo "Environment:"
            echo "  ARIA_MODEL_BUDGET      - Budget in dollars (default: 10.00)"
            ;;
    esac
}

main "$@"
