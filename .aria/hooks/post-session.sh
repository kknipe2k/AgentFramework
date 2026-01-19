#!/bin/bash
#
# ARIA Post-Session Hook
#
# Runs after each session to:
# 1. Trigger offline learning
# 2. Update policy for next session
# 3. Generate learning report
#
# Usage: Called automatically or manually via:
#   bash .aria/hooks/post-session.sh
#

set -e

ARIA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEARNER="$ARIA_ROOT/lib/offline-learner.py"
LEARNED_DIR="$ARIA_ROOT/learned"

echo "========================================"
echo "ARIA Post-Session Learning"
echo "========================================"
echo ""

# Check if we have the learner
if [[ ! -f "$LEARNER" ]]; then
    echo "Warning: Offline learner not found"
    exit 0
fi

# Check if we have data to learn from
SIGNALS_FILE="$ARIA_ROOT/state/signals.jsonl"
DECISIONS_FILE="$ARIA_ROOT/state/decisions.jsonl"
MODEL_LEARNING="$ARIA_ROOT/logs/model_learning.json"

has_data=false

if [[ -f "$SIGNALS_FILE" ]] && [[ -s "$SIGNALS_FILE" ]]; then
    signal_count=$(wc -l < "$SIGNALS_FILE")
    echo "Found $signal_count signals"
    has_data=true
fi

if [[ -f "$DECISIONS_FILE" ]] && [[ -s "$DECISIONS_FILE" ]]; then
    decision_count=$(wc -l < "$DECISIONS_FILE")
    echo "Found $decision_count decisions"
    has_data=true
fi

if [[ -f "$MODEL_LEARNING" ]]; then
    outcome_count=$(python3 -c "
import json
with open('$MODEL_LEARNING') as f:
    data = json.load(f)
print(len(data.get('recent_outcomes', [])))
" 2>/dev/null || echo "0")
    if [[ "$outcome_count" -gt 0 ]]; then
        echo "Found $outcome_count model outcomes"
        has_data=true
    fi
fi

if [[ "$has_data" != "true" ]]; then
    echo "No new data to learn from"
    exit 0
fi

echo ""
echo "Running offline learning..."
echo ""

# Run the learner
python3 "$LEARNER" learn

echo ""
echo "========================================"
echo "Learning Complete"
echo "========================================"

# Show updated stats
echo ""
python3 "$LEARNER" stats
