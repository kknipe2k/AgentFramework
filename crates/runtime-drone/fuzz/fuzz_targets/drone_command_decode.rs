#![no_main]
use libfuzzer_sys::fuzz_target;
use runtime_core::DroneCommand;

// Fuzz the IPC frame decoder with arbitrary bytes.
//
// Invariants:
//   1. Must not panic on any input.
//   2. If the input deserializes to a DroneCommand, the variant must be
//      one of the spec §1d variants — serde + tagged enum enforces this.
//   3. Untrusted bytes through this path must not bypass validation.
//
// Run for 30s in CI on PRs; 1 hour nightly on main.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Mimics the LinesCodec path: each newline-delimited frame is parsed.
        for line in s.lines() {
            let _: Result<DroneCommand, _> = serde_json::from_str(line);
        }
    }
});
