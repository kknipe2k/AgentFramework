//! Runtime runtime-main placeholder.
//!
//! Real implementation lands in subsequent stages of M01 and later milestones.

pub mod providers;

/// Returns the string `"ok"`. Placeholder for Stage A; real exports come later.
///
/// # Examples
///
/// ```
/// assert_eq!(runtime_main::placeholder(), "ok");
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
