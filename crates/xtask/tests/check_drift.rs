//! xtask drift-detection integration tests.
//!
//! These tests share mutable filesystem state (the generated/*.rs files) and
//! must run serially. The `serial_test` approach uses a file-system lock to
//! prevent parallel execution without requiring external crates.
//!
//! Requires the workspace to be in a state where committed types match generated
//! (i.e., this test runs from a clean checkout after Stage B's implementation).

use std::process::Command;

/// Run all drift tests sequentially in a single #[test] to avoid parallel
/// filesystem interference. Each section is a logical test case.
#[test]
fn drift_detection_tests() {
    // === Case 1: --check passes when in sync ===
    {
        // First, ensure files are in sync by regenerating.
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["regenerate-types"])
            .output()
            .expect("run xtask regenerate-types");
        assert!(
            output.status.success(),
            "regenerate-types should succeed. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["regenerate-types", "--check"])
            .output()
            .expect("run xtask --check");
        assert!(
            output.status.success(),
            "drift check should pass on a clean checkout. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // === Case 2: regenerate-types writes files with correct headers ===
    {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["regenerate-types"])
            .output()
            .expect("run xtask regenerate-types");
        assert!(output.status.success(), "regenerate-types should succeed");

        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let common_rs = workspace_root.join("crates/runtime-core/src/generated/common.rs");
        let text = std::fs::read_to_string(&common_rs).expect("read generated common.rs");
        assert!(
            text.contains("AUTO-GENERATED FILE"),
            "generated file should have auto-gen header"
        );
        assert!(
            text.contains("typify"),
            "generated file should reference typify in header"
        );
    }

    // === Case 3: --check detects drift ===
    {
        use std::fs;
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let target = workspace_root.join("crates/runtime-core/src/generated/common.rs");
        let original = fs::read_to_string(&target).expect("read original");

        // Mutate: append a comment.
        fs::write(&target, format!("{original}\n// drift-test\n")).expect("write mutation");

        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["regenerate-types", "--check"])
            .output()
            .expect("run xtask --check");

        // Restore BEFORE asserting (so a panicking assertion doesn't leave the file dirty).
        fs::write(&target, &original).expect("restore");

        assert!(
            !output.status.success(),
            "drift check should detect the mutation. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
