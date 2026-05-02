//! Runtime xtask placeholder.
//!
//! Real implementation lands in subsequent stages of M01 and later milestones.

/// Returns the string `"ok"`. Placeholder for Stage A; real exports come later.
///
/// # Examples
///
/// ```
/// assert_eq!(xtask::placeholder(), "ok");
/// ```
#[must_use]
pub const fn placeholder() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_returns_ok() {
        assert_eq!(placeholder(), "ok");
    }
}
