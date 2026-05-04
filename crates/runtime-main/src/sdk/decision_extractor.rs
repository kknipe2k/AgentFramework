//! Heuristic decision extraction from streamed text.
//!
//! M02 ships the simplest version: detect `Decision:`/`Rationale:` markers
//! anywhere in the text (intervening blank lines OK; the *last* matching
//! pair wins so multi-decision blocks pick the most recent). M04 verify+rails
//! replaces this with a structured emitter injected by the prompt template,
//! eliminating the heuristic.
//!
//! Pure function; full property-test coverage of malformed inputs.

/// Structured decision extracted from a streamed text block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRecord {
    /// What was decided.
    pub decision: String,
    /// Why it was decided.
    pub rationale: String,
    /// Tool used to act on the decision, if any.
    pub tool_used: Option<String>,
}

/// Extract a decision record from a text block.
///
/// Heuristic: scans line-by-line for `Decision:`, `Rationale:`, and
/// `Tool used:` markers (case-sensitive, leading whitespace tolerated).
/// Returns `None` if either `Decision:` or `Rationale:` is missing.
///
/// When multiple `Decision:`/`Rationale:` lines appear, the *last* values
/// win — matches the conversational pattern of a model revising its
/// decision mid-stream.
///
/// # Examples
///
/// ```
/// use runtime_main::sdk::extract_decision;
/// let text = "Decision: pick haiku\nRationale: cost-sensitive task\n";
/// let d = extract_decision(text).unwrap();
/// assert_eq!(d.decision,  "pick haiku");
/// assert_eq!(d.rationale, "cost-sensitive task");
/// ```
#[must_use]
pub fn extract_decision(text: &str) -> Option<DecisionRecord> {
    let mut decision: Option<String> = None;
    let mut rationale: Option<String> = None;
    let mut tool_used: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Decision:") {
            decision = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Rationale:") {
            rationale = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Tool used:") {
            tool_used = Some(rest.trim().to_string());
        }
    }
    match (decision, rationale) {
        (Some(d), Some(r)) => Some(DecisionRecord {
            decision: d,
            rationale: r,
            tool_used,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn extracts_decision_and_rationale() {
        let t = "Decision: A\nRationale: B\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision, "A");
        assert_eq!(d.rationale, "B");
        assert!(d.tool_used.is_none());
    }

    #[test]
    fn extracts_tool_used_when_present() {
        let t = "Decision: ship\nRationale: green CI\nTool used: cargo test\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.tool_used.as_deref(), Some("cargo test"));
    }

    #[test]
    fn returns_none_when_decision_missing() {
        assert!(extract_decision("Rationale: only").is_none());
    }

    #[test]
    fn returns_none_when_rationale_missing() {
        assert!(extract_decision("Decision: only").is_none());
    }

    #[test]
    fn returns_none_for_empty_input() {
        assert!(extract_decision("").is_none());
    }

    #[test]
    fn handles_intervening_blank_lines() {
        let t = "Decision: A\n\n\nRationale: B\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision, "A");
        assert_eq!(d.rationale, "B");
    }

    #[test]
    fn handles_leading_whitespace() {
        let t = "   Decision: A   \n   Rationale: B   \n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision, "A");
        assert_eq!(d.rationale, "B");
    }

    #[test]
    fn last_decision_wins_when_multiple() {
        let t = "Decision: first\nRationale: r1\nDecision: second\nRationale: r2\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision, "second");
        assert_eq!(d.rationale, "r2");
    }

    #[test]
    fn ignores_unrelated_lines() {
        let t = "intro paragraph\nDecision: A\ninterstitial\nRationale: B\noutro\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision, "A");
        assert_eq!(d.rationale, "B");
    }

    proptest! {
        #[test]
        fn never_panics_on_arbitrary_input(s in "\\PC{0,1000}") {
            let _ = extract_decision(&s);
        }
    }
}
