//! Structured emitter — spec §2 + §3a delimited-block parser.
//!
//! Replaces M02's `decision_extractor` heuristic (line-by-line `Decision:`
//! / `Rationale:` matching) with a deterministic delimiter-scoped parser.
//! The orchestrator agent's prompt template (framework-JSON territory;
//! out of scope for Stage B) instructs the model to emit decisions /
//! plan-creations inside `<<DECISION>>...<<END>>` and
//! `<<PLAN>>...<<END>>` blocks. This module parses those blocks.
//!
//! Closes M02 🟡 carry-forward "decision-extractor false positives": with
//! delimiter-scoped parsing, `Decision:` text inside markdown code blocks
//! / quoted content cannot trigger a false emit unless it's wrapped in
//! the delimiters.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.
//!
//! # Block syntax
//!
//! ```text
//! <<DECISION>>
//!   Decision: <text>
//!   Rationale: <text>
//!   Tool used: <text>     (optional)
//! <<END>>
//!
//! <<PLAN>>
//!   Plan ID: <uuid>
//!   Title: <text>
//!   Approval required: <true|false>
//!   Tasks:
//!     - <task1 title>
//!     - <task2 title>
//! <<END>>
//! ```
//!
//! Key fields are line-prefixed inside the block. Unknown lines outside
//! recognized prefixes are ignored within a block. Text **outside** any
//! block is ignored entirely (no false-positive risk).

use thiserror::Error;

/// One typed output produced by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitterOutput {
    /// A decision record (mirrors M02's `DecisionRecord` shape so
    /// downstream `event_pipeline` translation stays unchanged).
    Decision {
        /// What was decided.
        decision: String,
        /// Why it was decided.
        rationale: String,
        /// Tool used to act on the decision, if any.
        tool_used: Option<String>,
    },
    /// A plan-creation event (consumed by `plan_loop` to emit
    /// `plan_created` + start the FSM). Tasks are surfaced as titles
    /// only; the SDK assigns task IDs.
    PlanCreation {
        /// Caller-supplied plan UUID, or empty when the model didn't
        /// emit one (the SDK then assigns).
        plan_id: String,
        /// Plan title.
        title: String,
        /// Whether the plan needs HITL approval.
        approval_required: bool,
        /// Task titles in execution order.
        task_titles: Vec<String>,
    },
}

/// Parse errors.
///
/// Per CLAUDE.md §9, the `decision`/`rationale` fields being malformed
/// inside an otherwise-well-formed block is a hard error; silently
/// dropping malformed blocks would re-introduce the M02 false-positive
/// risk.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum EmitterError {
    /// Block opened with `<<DECISION>>` or `<<PLAN>>` but never closed.
    #[error("unterminated emitter block: {tag}")]
    Unterminated {
        /// The tag whose `<<END>>` was missing.
        tag: String,
    },
    /// Block opened inside another open block. Nesting is not supported
    /// because deterministic single-block scoping is the whole point.
    #[error("nested emitter block detected: {inner} inside {outer}")]
    NestedBlock {
        /// The outer block tag.
        outer: String,
        /// The inner (rejected) block tag.
        inner: String,
    },
    /// Decision block missing `Decision:` or `Rationale:`.
    #[error("decision block missing required field: {0}")]
    DecisionMissingField(&'static str),
    /// Plan block missing `Title:`.
    #[error("plan block missing required field: {0}")]
    PlanMissingField(&'static str),
}

const TAG_DECISION_OPEN: &str = "<<DECISION>>";
const TAG_PLAN_OPEN: &str = "<<PLAN>>";
const TAG_END: &str = "<<END>>";

/// Parse `text` into a vector of typed emitter outputs.
///
/// Text outside of recognized blocks is ignored — including any
/// `Decision:` lines that happen to appear in markdown code blocks or
/// quoted content. Inside a block, unknown lines are also ignored
/// (forward-compatible: future fields don't break the parser).
///
/// # Errors
///
/// - [`EmitterError::Unterminated`] when an opening tag has no matching
///   `<<END>>`.
/// - [`EmitterError::NestedBlock`] when an opening tag appears inside
///   another open block.
/// - [`EmitterError::DecisionMissingField`] / [`EmitterError::PlanMissingField`]
///   when a well-formed block is missing required fields.
///
/// # Examples
///
/// ```
/// use runtime_main::sdk::{parse_structured, EmitterOutput};
/// let text = "<<DECISION>>\nDecision: pick haiku\nRationale: cost\n<<END>>\n";
/// let outputs = parse_structured(text).unwrap();
/// assert!(matches!(outputs[0], EmitterOutput::Decision { .. }));
/// ```
pub fn parse_structured(text: &str) -> Result<Vec<EmitterOutput>, EmitterError> {
    let mut outputs = Vec::new();
    let mut current: Option<BlockBuilder> = None;

    for raw in text.lines() {
        let trimmed = raw.trim();
        if trimmed == TAG_DECISION_OPEN {
            if let Some(b) = &current {
                return Err(EmitterError::NestedBlock {
                    outer: b.tag().to_string(),
                    inner: TAG_DECISION_OPEN.into(),
                });
            }
            current = Some(BlockBuilder::Decision(DecisionBuilder::default()));
            continue;
        }
        if trimmed == TAG_PLAN_OPEN {
            if let Some(b) = &current {
                return Err(EmitterError::NestedBlock {
                    outer: b.tag().to_string(),
                    inner: TAG_PLAN_OPEN.into(),
                });
            }
            current = Some(BlockBuilder::Plan(PlanBuilder::default()));
            continue;
        }
        if trimmed == TAG_END {
            let b = current.take().ok_or_else(|| EmitterError::Unterminated {
                tag: TAG_END.into(),
            })?;
            outputs.push(b.finalize()?);
            continue;
        }
        if let Some(b) = current.as_mut() {
            b.feed(trimmed);
        }
        // Lines outside any open block are ignored — the false-positive
        // elimination contract.
    }

    if let Some(b) = current {
        return Err(EmitterError::Unterminated {
            tag: b.tag().to_string(),
        });
    }
    Ok(outputs)
}

enum BlockBuilder {
    Decision(DecisionBuilder),
    Plan(PlanBuilder),
}

impl BlockBuilder {
    const fn tag(&self) -> &'static str {
        match self {
            Self::Decision(_) => TAG_DECISION_OPEN,
            Self::Plan(_) => TAG_PLAN_OPEN,
        }
    }

    fn feed(&mut self, line: &str) {
        match self {
            Self::Decision(b) => b.feed(line),
            Self::Plan(b) => b.feed(line),
        }
    }

    fn finalize(self) -> Result<EmitterOutput, EmitterError> {
        match self {
            Self::Decision(b) => b.finalize(),
            Self::Plan(b) => b.finalize(),
        }
    }
}

#[derive(Default)]
struct DecisionBuilder {
    decision: Option<String>,
    rationale: Option<String>,
    tool_used: Option<String>,
}

impl DecisionBuilder {
    fn feed(&mut self, line: &str) {
        if let Some(rest) = line.strip_prefix("Decision:") {
            self.decision = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Rationale:") {
            self.rationale = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Tool used:") {
            self.tool_used = Some(rest.trim().to_string());
        }
    }

    fn finalize(self) -> Result<EmitterOutput, EmitterError> {
        let decision = self
            .decision
            .ok_or(EmitterError::DecisionMissingField("Decision"))?;
        let rationale = self
            .rationale
            .ok_or(EmitterError::DecisionMissingField("Rationale"))?;
        Ok(EmitterOutput::Decision {
            decision,
            rationale,
            tool_used: self.tool_used,
        })
    }
}

#[derive(Default)]
struct PlanBuilder {
    plan_id: Option<String>,
    title: Option<String>,
    approval_required: Option<bool>,
    task_titles: Vec<String>,
    in_tasks_section: bool,
}

impl PlanBuilder {
    fn feed(&mut self, line: &str) {
        if let Some(rest) = line.strip_prefix("Plan ID:") {
            self.plan_id = Some(rest.trim().to_string());
            self.in_tasks_section = false;
        } else if let Some(rest) = line.strip_prefix("Title:") {
            self.title = Some(rest.trim().to_string());
            self.in_tasks_section = false;
        } else if let Some(rest) = line.strip_prefix("Approval required:") {
            let v = rest.trim().to_lowercase();
            self.approval_required = Some(v == "true" || v == "yes");
            self.in_tasks_section = false;
        } else if line.starts_with("Tasks:") {
            self.in_tasks_section = true;
        } else if self.in_tasks_section {
            if let Some(rest) = line.strip_prefix('-') {
                let title = rest.trim();
                if !title.is_empty() {
                    self.task_titles.push(title.to_string());
                }
            }
        }
    }

    fn finalize(self) -> Result<EmitterOutput, EmitterError> {
        let title = self.title.ok_or(EmitterError::PlanMissingField("Title"))?;
        Ok(EmitterOutput::PlanCreation {
            plan_id: self.plan_id.unwrap_or_default(),
            title,
            approval_required: self.approval_required.unwrap_or(true),
            task_titles: self.task_titles,
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::match_wildcard_for_single_variants,
    reason = "test panics on unexpected variant; using `_` keeps test bodies short"
)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty_vec() {
        assert!(parse_structured("").unwrap().is_empty());
    }

    #[test]
    fn text_outside_blocks_is_ignored() {
        // Per CLAUDE.md §9 — `Decision:` in code blocks must NOT trigger
        // a false positive (closes M02 🟡 carry-forward).
        let text = "Some prose here.\n\
                    ```rust\n\
                    // Decision: this is just a comment\n\
                    fn main() { /* Rationale: testing */ }\n\
                    ```\n\
                    More prose.\n";
        assert!(parse_structured(text).unwrap().is_empty());
    }

    #[test]
    fn single_decision_block_parses() {
        let text =
            "<<DECISION>>\nDecision: pick haiku\nRationale: cost-sensitive\nTool used: estimate_cost\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        assert_eq!(outputs.len(), 1);
        match &outputs[0] {
            EmitterOutput::Decision {
                decision,
                rationale,
                tool_used,
            } => {
                assert_eq!(decision, "pick haiku");
                assert_eq!(rationale, "cost-sensitive");
                assert_eq!(tool_used.as_deref(), Some("estimate_cost"));
            }
            other => panic!("expected Decision, got {other:?}"),
        }
    }

    #[test]
    fn decision_block_without_tool_used() {
        let text = "<<DECISION>>\nDecision: A\nRationale: B\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        match &outputs[0] {
            EmitterOutput::Decision { tool_used, .. } => assert!(tool_used.is_none()),
            other => panic!("expected Decision, got {other:?}"),
        }
    }

    #[test]
    fn multiple_decision_blocks_parse_in_order() {
        let text = "<<DECISION>>\nDecision: first\nRationale: r1\n<<END>>\n\
                    <<DECISION>>\nDecision: second\nRationale: r2\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        assert_eq!(outputs.len(), 2);
        match (&outputs[0], &outputs[1]) {
            (
                EmitterOutput::Decision { decision: d1, .. },
                EmitterOutput::Decision { decision: d2, .. },
            ) => {
                assert_eq!(d1, "first");
                assert_eq!(d2, "second");
            }
            _ => panic!("expected two Decision outputs"),
        }
    }

    #[test]
    fn plan_block_with_tasks() {
        let text = "<<PLAN>>\n\
                    Plan ID: 11111111-2222-3333-4444-555555555555\n\
                    Title: Migrate auth\n\
                    Approval required: true\n\
                    Tasks:\n\
                      - Audit current auth flow\n\
                      - Draft new flow\n\
                      - Implement\n\
                    <<END>>\n";
        let outputs = parse_structured(text).unwrap();
        match &outputs[0] {
            EmitterOutput::PlanCreation {
                plan_id,
                title,
                approval_required,
                task_titles,
            } => {
                assert_eq!(plan_id, "11111111-2222-3333-4444-555555555555");
                assert_eq!(title, "Migrate auth");
                assert!(*approval_required);
                assert_eq!(task_titles.len(), 3);
                assert_eq!(task_titles[0], "Audit current auth flow");
            }
            other => panic!("expected PlanCreation, got {other:?}"),
        }
    }

    #[test]
    fn plan_block_approval_required_false() {
        let text = "<<PLAN>>\nTitle: T\nApproval required: false\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        match &outputs[0] {
            EmitterOutput::PlanCreation {
                approval_required, ..
            } => assert!(!*approval_required),
            other => panic!("expected PlanCreation, got {other:?}"),
        }
    }

    #[test]
    fn plan_block_default_approval_required_true_when_omitted() {
        let text = "<<PLAN>>\nTitle: T\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        match &outputs[0] {
            EmitterOutput::PlanCreation {
                approval_required,
                plan_id,
                task_titles,
                ..
            } => {
                assert!(*approval_required);
                assert!(plan_id.is_empty());
                assert!(task_titles.is_empty());
            }
            other => panic!("expected PlanCreation, got {other:?}"),
        }
    }

    #[test]
    fn mixed_decision_and_plan_blocks() {
        let text = "<<PLAN>>\nTitle: T\n<<END>>\n\
                    <<DECISION>>\nDecision: d\nRationale: r\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        assert_eq!(outputs.len(), 2);
        assert!(matches!(outputs[0], EmitterOutput::PlanCreation { .. }));
        assert!(matches!(outputs[1], EmitterOutput::Decision { .. }));
    }

    #[test]
    fn unterminated_decision_block_errors() {
        let text = "<<DECISION>>\nDecision: A\nRationale: B\n";
        let err = parse_structured(text).unwrap_err();
        match err {
            EmitterError::Unterminated { tag } => assert_eq!(tag, TAG_DECISION_OPEN),
            other => panic!("expected Unterminated, got {other:?}"),
        }
    }

    #[test]
    fn unterminated_plan_block_errors() {
        let text = "<<PLAN>>\nTitle: T\n";
        let err = parse_structured(text).unwrap_err();
        assert!(matches!(err, EmitterError::Unterminated { .. }));
    }

    #[test]
    fn nested_decision_inside_plan_errors() {
        let text = "<<PLAN>>\nTitle: T\n<<DECISION>>\nDecision: x\n<<END>>\n";
        let err = parse_structured(text).unwrap_err();
        match err {
            EmitterError::NestedBlock { outer, inner } => {
                assert_eq!(outer, TAG_PLAN_OPEN);
                assert_eq!(inner, TAG_DECISION_OPEN);
            }
            other => panic!("expected NestedBlock, got {other:?}"),
        }
    }

    #[test]
    fn nested_plan_inside_decision_errors() {
        let text =
            "<<DECISION>>\nDecision: x\nRationale: y\n<<PLAN>>\nTitle: T\n<<END>>\n<<END>>\n";
        let err = parse_structured(text).unwrap_err();
        match err {
            EmitterError::NestedBlock { outer, inner } => {
                assert_eq!(outer, TAG_DECISION_OPEN);
                assert_eq!(inner, TAG_PLAN_OPEN);
            }
            other => panic!("expected NestedBlock, got {other:?}"),
        }
    }

    #[test]
    fn decision_block_missing_decision_field_errors() {
        let text = "<<DECISION>>\nRationale: only\n<<END>>\n";
        let err = parse_structured(text).unwrap_err();
        assert!(matches!(
            err,
            EmitterError::DecisionMissingField("Decision")
        ));
    }

    #[test]
    fn decision_block_missing_rationale_field_errors() {
        let text = "<<DECISION>>\nDecision: only\n<<END>>\n";
        let err = parse_structured(text).unwrap_err();
        assert!(matches!(
            err,
            EmitterError::DecisionMissingField("Rationale")
        ));
    }

    #[test]
    fn plan_block_missing_title_errors() {
        let text = "<<PLAN>>\nPlan ID: x\n<<END>>\n";
        let err = parse_structured(text).unwrap_err();
        assert!(matches!(err, EmitterError::PlanMissingField("Title")));
    }

    #[test]
    fn end_outside_block_errors() {
        let text = "<<END>>\n";
        let err = parse_structured(text).unwrap_err();
        assert!(matches!(err, EmitterError::Unterminated { tag } if tag == TAG_END));
    }

    #[test]
    fn unknown_lines_inside_block_are_ignored() {
        let text = "<<DECISION>>\n\
                    Decision: A\n\
                    Free-form prose that doesn't match any prefix\n\
                    Rationale: B\n\
                    <<END>>\n";
        let outputs = parse_structured(text).unwrap();
        assert_eq!(outputs.len(), 1);
    }

    #[test]
    fn empty_task_dash_lines_dropped() {
        let text = "<<PLAN>>\nTitle: T\nTasks:\n  -\n  - real task\n<<END>>\n";
        let outputs = parse_structured(text).unwrap();
        match &outputs[0] {
            EmitterOutput::PlanCreation { task_titles, .. } => {
                assert_eq!(task_titles, &["real task"]);
            }
            other => panic!("expected PlanCreation, got {other:?}"),
        }
    }

    #[test]
    fn errors_format_with_useful_text() {
        let e = EmitterError::Unterminated {
            tag: "<<X>>".into(),
        };
        assert!(e.to_string().contains("unterminated"));
        let e = EmitterError::NestedBlock {
            outer: "<<A>>".into(),
            inner: "<<B>>".into(),
        };
        assert!(e.to_string().contains("nested"));
        let e = EmitterError::DecisionMissingField("Decision");
        assert!(e.to_string().contains("Decision"));
        let e = EmitterError::PlanMissingField("Title");
        assert!(e.to_string().contains("Title"));
    }
}
