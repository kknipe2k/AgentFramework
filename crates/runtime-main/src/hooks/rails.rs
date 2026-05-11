//! Rails primitive — `JSONLogic`-evaluated policy checks. Spec §4a.
//!
//! ## Operator allowlist
//!
//! Hard rails block; soft rails warn. Each rail is `{ id, fires_on,
//! check, message }` per spec §4a + framework JSON. The `check` is a
//! `JSONLogic` expression evaluated against a `facts` object the runtime
//! collects (event payload, agent context, attempted action). The
//! operator allowlist is fixed at v0.1 per `docs/gotchas.md` #18:
//!
//! ```text
//! var, ==, !=, <, <=, >, >=, and, or, not, in, +, -, *, /
//! ```
//!
//! Operators outside this set return [`RailError::UnsupportedOperator`]
//! — extending the allowlist requires a deliberate spec edit + ADR per
//! `CLAUDE.md` §11.
//!
//! ## Coverage scope
//!
//! Stage D ships the evaluator + allowlist + rail dispatch surface.
//! The integration site (capability enforcer / plan loop) is M05+ — the
//! evaluator is callable today by any caller that supplies a facts
//! object.

use serde_json::{json, Value};
use thiserror::Error;

/// Errors raised during `JSONLogic` evaluation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RailError {
    /// An operator outside the v0.1 allowlist appeared in the expression.
    #[error("unsupported JSONLogic operator: {0} (allowlist: var, ==, !=, <, <=, >, >=, and, or, not, in, +, -, *, /)")]
    UnsupportedOperator(String),
    /// The expression structure is malformed.
    #[error("malformed JSONLogic expression: {0}")]
    Malformed(String),
    /// `var` referenced a path not present in the facts object.
    #[error("missing variable: {0}")]
    MissingVar(String),
}

/// Allowlisted `JSONLogic` operators per gotcha #18.
const ALLOWED_OPERATORS: &[&str] = &[
    "var", "==", "!=", "<", "<=", ">", ">=", "and", "or", "not", "in", "+", "-", "*", "/",
];

/// Evaluate a `JSONLogic` expression against the given facts.
///
/// The expression is `Value` — typically an object with a single
/// operator key (e.g., `{"==": [{"var": "tier"}, "novice"]}`). Literals
/// (numbers, strings, booleans, arrays) pass through as-is.
///
/// # Errors
///
/// - [`RailError::UnsupportedOperator`] if the expression names an
///   operator outside the allowlist.
/// - [`RailError::Malformed`] if the structure is invalid.
/// - [`RailError::MissingVar`] if a `var` reference is unresolved.
///
/// # Panics
///
/// Cannot panic in practice: the only `expect` checks `map.len() == 1`
/// just after a `len() == 1` guard — narrowed by the match arm.
pub fn evaluate(expr: &Value, facts: &Value) -> Result<Value, RailError> {
    match expr {
        Value::Object(map) if map.len() == 1 => {
            let (op, args) = map.iter().next().expect("len == 1");
            if !ALLOWED_OPERATORS.contains(&op.as_str()) {
                return Err(RailError::UnsupportedOperator(op.clone()));
            }
            apply_op(op, args, facts)
        }
        // Literal pass-through.
        _ => Ok(expr.clone()),
    }
}

fn apply_op(op: &str, args: &Value, facts: &Value) -> Result<Value, RailError> {
    match op {
        "var" => apply_var(args, facts),
        "==" => apply_binary(args, facts, |a, b| Ok(Value::Bool(a == b))),
        "!=" => apply_binary(args, facts, |a, b| Ok(Value::Bool(a != b))),
        "<" => apply_compare(args, facts, |a, b| a < b),
        "<=" => apply_compare(args, facts, |a, b| a <= b),
        ">" => apply_compare(args, facts, |a, b| a > b),
        ">=" => apply_compare(args, facts, |a, b| a >= b),
        "and" => apply_logical(args, facts, true),
        "or" => apply_logical(args, facts, false),
        "not" => apply_not(args, facts),
        "in" => apply_in(args, facts),
        "+" => apply_arith(args, facts, |a, b| a + b, 0.0),
        "-" => apply_subtract(args, facts),
        "*" => apply_arith(args, facts, |a, b| a * b, 1.0),
        "/" => apply_divide(args, facts),
        // Unreachable — the caller already checked the allowlist.
        _ => Err(RailError::UnsupportedOperator(op.to_string())),
    }
}

fn apply_var(args: &Value, facts: &Value) -> Result<Value, RailError> {
    let path = match args {
        Value::String(s) => s.clone(),
        Value::Array(a) if a.len() == 1 => a[0]
            .as_str()
            .ok_or_else(|| RailError::Malformed("var arg must be a string".into()))?
            .to_string(),
        Value::Array(a) if a.is_empty() => return Ok(facts.clone()),
        other => {
            return Err(RailError::Malformed(format!(
                "var arg must be string or [string]: got {other}"
            )))
        }
    };
    let mut current = facts;
    for segment in path.split('.') {
        current = match current.get(segment) {
            Some(v) => v,
            None => return Err(RailError::MissingVar(path)),
        };
    }
    Ok(current.clone())
}

fn apply_binary<F>(args: &Value, facts: &Value, f: F) -> Result<Value, RailError>
where
    F: Fn(&Value, &Value) -> Result<Value, RailError>,
{
    let pair = args
        .as_array()
        .ok_or_else(|| RailError::Malformed("binary op requires a 2-element array".into()))?;
    if pair.len() != 2 {
        return Err(RailError::Malformed(
            "binary op requires exactly 2 args".into(),
        ));
    }
    let a = evaluate(&pair[0], facts)?;
    let b = evaluate(&pair[1], facts)?;
    f(&a, &b)
}

fn apply_compare<F>(args: &Value, facts: &Value, cmp: F) -> Result<Value, RailError>
where
    F: Fn(f64, f64) -> bool,
{
    apply_binary(args, facts, |a, b| {
        let lhs = num(a)?;
        let rhs = num(b)?;
        Ok(Value::Bool(cmp(lhs, rhs)))
    })
}

fn apply_logical(args: &Value, facts: &Value, all: bool) -> Result<Value, RailError> {
    let arr = args
        .as_array()
        .ok_or_else(|| RailError::Malformed("logical op requires an array".into()))?;
    let mut last = Value::Bool(all);
    for v in arr {
        let evaluated = evaluate(v, facts)?;
        let truthy = is_truthy(&evaluated);
        if all {
            if !truthy {
                return Ok(evaluated);
            }
        } else if truthy {
            return Ok(evaluated);
        }
        last = evaluated;
    }
    Ok(last)
}

fn apply_not(args: &Value, facts: &Value) -> Result<Value, RailError> {
    let inner = match args {
        Value::Array(a) if a.len() == 1 => evaluate(&a[0], facts)?,
        // `{"not": expr}` shorthand: arg is the expression itself.
        other => evaluate(other, facts)?,
    };
    Ok(Value::Bool(!is_truthy(&inner)))
}

fn apply_in(args: &Value, facts: &Value) -> Result<Value, RailError> {
    let arr = args
        .as_array()
        .ok_or_else(|| RailError::Malformed("`in` requires a 2-element array".into()))?;
    if arr.len() != 2 {
        return Err(RailError::Malformed("`in` requires exactly 2 args".into()));
    }
    let needle = evaluate(&arr[0], facts)?;
    let haystack = evaluate(&arr[1], facts)?;
    let found = match (&haystack, &needle) {
        (Value::Array(items), _) => items.contains(&needle),
        (Value::String(s), Value::String(n)) => s.contains(n.as_str()),
        _ => {
            return Err(RailError::Malformed(
                "`in` haystack must be array or string".into(),
            ))
        }
    };
    Ok(Value::Bool(found))
}

fn apply_arith<F>(args: &Value, facts: &Value, f: F, identity: f64) -> Result<Value, RailError>
where
    F: Fn(f64, f64) -> f64,
{
    let arr = args
        .as_array()
        .ok_or_else(|| RailError::Malformed("arith op requires an array".into()))?;
    let mut acc = identity;
    let mut started = false;
    for v in arr {
        let n = num(&evaluate(v, facts)?)?;
        if started {
            acc = f(acc, n);
        } else {
            acc = n;
            started = true;
        }
    }
    Ok(json!(acc))
}

fn apply_subtract(args: &Value, facts: &Value) -> Result<Value, RailError> {
    let arr = args
        .as_array()
        .ok_or_else(|| RailError::Malformed("`-` requires an array".into()))?;
    if arr.is_empty() {
        return Err(RailError::Malformed("`-` requires at least 1 arg".into()));
    }
    let first = num(&evaluate(&arr[0], facts)?)?;
    if arr.len() == 1 {
        // Unary negation per `JSONLogic`.
        return Ok(json!(-first));
    }
    let mut acc = first;
    for v in &arr[1..] {
        acc -= num(&evaluate(v, facts)?)?;
    }
    Ok(json!(acc))
}

fn apply_divide(args: &Value, facts: &Value) -> Result<Value, RailError> {
    let arr = args
        .as_array()
        .ok_or_else(|| RailError::Malformed("`/` requires an array".into()))?;
    if arr.len() != 2 {
        return Err(RailError::Malformed("`/` requires exactly 2 args".into()));
    }
    let lhs = num(&evaluate(&arr[0], facts)?)?;
    let rhs = num(&evaluate(&arr[1], facts)?)?;
    if rhs == 0.0 {
        return Err(RailError::Malformed("division by zero".into()));
    }
    Ok(json!(lhs / rhs))
}

fn num(v: &Value) -> Result<f64, RailError> {
    v.as_f64()
        .ok_or_else(|| RailError::Malformed(format!("expected number, got {v}")))
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Outcome of evaluating a single rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RailOutcome {
    /// Rail check evaluated truthy → rail triggers. Hard rails block;
    /// soft rails warn.
    Triggered,
    /// Rail check evaluated falsy → rail does not trigger.
    Quiet,
}

/// Evaluate a single rail's `check` against the facts. Boolean truthy =
/// triggered. The caller decides whether to block (hard) or warn (soft)
/// based on the rail's policy.
///
/// # Errors
///
/// See [`RailError`].
pub fn evaluate_rail(check: &Value, facts: &Value) -> Result<RailOutcome, RailError> {
    let result = evaluate(check, facts)?;
    if is_truthy(&result) {
        Ok(RailOutcome::Triggered)
    } else {
        Ok(RailOutcome::Quiet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn no_facts() -> Value {
        json!({})
    }

    #[test]
    fn equality_operator() {
        let expr = json!({"==": [1, 1]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
        let expr = json!({"==": [1, 2]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(false)));
    }

    #[test]
    fn inequality_operator() {
        let expr = json!({"!=": ["a", "b"]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
    }

    #[test]
    fn ordering_operators() {
        assert_eq!(
            evaluate(&json!({"<": [1, 2]}), &no_facts()),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            evaluate(&json!({">=": [3, 3]}), &no_facts()),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            evaluate(&json!({">": [2, 3]}), &no_facts()),
            Ok(Value::Bool(false))
        );
        assert_eq!(
            evaluate(&json!({"<=": [3, 2]}), &no_facts()),
            Ok(Value::Bool(false))
        );
    }

    #[test]
    fn var_resolves_dotted_paths() {
        let facts = json!({"agent": {"tier": "novice"}});
        let expr = json!({"var": "agent.tier"});
        assert_eq!(evaluate(&expr, &facts), Ok(Value::String("novice".into())));
    }

    #[test]
    fn var_missing_returns_error() {
        let facts = json!({});
        let expr = json!({"var": "agent.tier"});
        assert_eq!(
            evaluate(&expr, &facts),
            Err(RailError::MissingVar("agent.tier".into()))
        );
    }

    #[test]
    fn var_array_arg_form_supported() {
        let facts = json!({"k": "v"});
        let expr = json!({"var": ["k"]});
        assert_eq!(evaluate(&expr, &facts), Ok(Value::String("v".into())));
    }

    #[test]
    fn var_empty_array_returns_facts() {
        let facts = json!({"k": "v"});
        let expr = json!({"var": []});
        assert_eq!(evaluate(&expr, &facts), Ok(facts.clone()));
    }

    #[test]
    fn logical_and() {
        let expr = json!({"and": [true, true, true]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
        let expr = json!({"and": [true, false, true]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(false)));
    }

    #[test]
    fn logical_or() {
        let expr = json!({"or": [false, false, true]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
        let expr = json!({"or": [false, false, false]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(false)));
    }

    #[test]
    fn logical_not() {
        let expr = json!({"not": [true]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(false)));
        let expr = json!({"not": [false]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
    }

    #[test]
    fn in_operator_array() {
        let expr = json!({"in": ["b", ["a", "b", "c"]]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
        let expr = json!({"in": ["x", ["a", "b", "c"]]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(false)));
    }

    #[test]
    fn in_operator_substring() {
        let expr = json!({"in": ["lo", "hello"]});
        assert_eq!(evaluate(&expr, &no_facts()), Ok(Value::Bool(true)));
    }

    #[test]
    fn arithmetic_operators() {
        assert_eq!(
            evaluate(&json!({"+": [1, 2, 3]}), &no_facts()),
            Ok(json!(6.0))
        );
        assert_eq!(
            evaluate(&json!({"-": [10, 3, 2]}), &no_facts()),
            Ok(json!(5.0))
        );
        assert_eq!(
            evaluate(&json!({"*": [2, 3, 4]}), &no_facts()),
            Ok(json!(24.0))
        );
        assert_eq!(
            evaluate(&json!({"/": [10, 4]}), &no_facts()),
            Ok(json!(2.5))
        );
        assert_eq!(evaluate(&json!({"-": [5]}), &no_facts()), Ok(json!(-5.0)));
    }

    #[test]
    fn divide_by_zero_returns_malformed() {
        let expr = json!({"/": [1, 0]});
        assert!(matches!(
            evaluate(&expr, &no_facts()),
            Err(RailError::Malformed(_))
        ));
    }

    #[test]
    fn unknown_operator_rejected() {
        let expr = json!({"some_random_op": [1, 2]});
        assert_eq!(
            evaluate(&expr, &no_facts()),
            Err(RailError::UnsupportedOperator("some_random_op".into()))
        );
    }

    #[test]
    fn malformed_binary_op_rejected() {
        let expr = json!({"==": [1]}); // wrong arity
        assert!(matches!(
            evaluate(&expr, &no_facts()),
            Err(RailError::Malformed(_))
        ));
    }

    #[test]
    fn nested_expression_with_var() {
        // Spec §4a-style rule: `agent.tier == "novice" and agent.requested_shell == true`
        let facts = json!({"agent": {"tier": "novice", "requested_shell": true}});
        let expr = json!({
            "and": [
                {"==": [{"var": "agent.tier"}, "novice"]},
                {"==": [{"var": "agent.requested_shell"}, true]}
            ]
        });
        assert_eq!(evaluate(&expr, &facts), Ok(Value::Bool(true)));
    }

    #[test]
    fn evaluate_rail_returns_triggered_for_truthy_check() {
        let check = json!({"==": [1, 1]});
        assert_eq!(
            evaluate_rail(&check, &no_facts()),
            Ok(RailOutcome::Triggered)
        );
    }

    #[test]
    fn evaluate_rail_returns_quiet_for_falsy_check() {
        let check = json!({"==": [1, 2]});
        assert_eq!(evaluate_rail(&check, &no_facts()), Ok(RailOutcome::Quiet));
    }

    #[test]
    fn literal_value_passes_through_evaluate() {
        // Lock the contract that a bare literal isn't treated as an op.
        assert_eq!(evaluate(&json!(42), &no_facts()), Ok(json!(42)));
        assert_eq!(
            evaluate(&Value::String("hi".into()), &no_facts()),
            Ok(Value::String("hi".into()))
        );
        assert_eq!(evaluate(&Value::Null, &no_facts()), Ok(Value::Null));
    }

    #[test]
    fn truthy_table_locks_each_value_kind() {
        // Direct unit tests on `is_truthy` aren't ideal (private helper);
        // exercise via `evaluate_rail` with a literal check value.
        assert_eq!(
            evaluate_rail(&Value::Bool(true), &no_facts()),
            Ok(RailOutcome::Triggered)
        );
        assert_eq!(
            evaluate_rail(&Value::Null, &no_facts()),
            Ok(RailOutcome::Quiet)
        );
        assert_eq!(
            evaluate_rail(&json!(0), &no_facts()),
            Ok(RailOutcome::Quiet)
        );
        assert_eq!(
            evaluate_rail(&json!(1), &no_facts()),
            Ok(RailOutcome::Triggered)
        );
        assert_eq!(
            evaluate_rail(&Value::String(String::new()), &no_facts()),
            Ok(RailOutcome::Quiet)
        );
        assert_eq!(
            evaluate_rail(&Value::String("hi".into()), &no_facts()),
            Ok(RailOutcome::Triggered)
        );
        assert_eq!(
            evaluate_rail(&json!([]), &no_facts()),
            Ok(RailOutcome::Quiet)
        );
        assert_eq!(
            evaluate_rail(&json!([1]), &no_facts()),
            Ok(RailOutcome::Triggered)
        );
        assert_eq!(
            evaluate_rail(&json!({}), &no_facts()),
            Ok(RailOutcome::Quiet)
        );
    }
}
