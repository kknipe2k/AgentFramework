//! Pins the IPC frame-cap constant's agreed value (TD-053).
//!
//! Mutation-killer rationale (M09.5.C mutation gate): cargo-mutants
//! tests the mutated package only, and the behavioral at-cap/oversize
//! kills live in runtime-drone / runtime-sandbox / runtime-main — so a
//! `4 * 1024 * 1024 → 4 + 1024 * 1024` mutation survived runtime-core's
//! own suite. For a security constant the value IS the contract; pin it
//! here, in the crate the mutant lives in.

#[test]
fn max_ipc_frame_bytes_is_exactly_4_mib() {
    assert_eq!(runtime_core::MAX_IPC_FRAME_BYTES, 4_194_304);
}
