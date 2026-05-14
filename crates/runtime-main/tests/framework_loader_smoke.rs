//! Integration test for `framework_loader::load_and_validate` against the
//! reference framework `examples/aria/framework.json`. Spec §4b Layer 1
//! end-to-end smoke + the gotcha #69 multi-call invariant (sequential
//! loads must succeed independently, not consume shared state).

use runtime_core::event::AgentEvent;
use runtime_main::framework_loader::{load_and_validate, Emitter};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
struct CollectingEmitter {
    events: Mutex<Vec<AgentEvent>>,
}

#[async_trait::async_trait]
impl Emitter for CollectingEmitter {
    async fn emit(&self, event: AgentEvent) {
        self.events.lock().expect("no poisoning").push(event);
    }
}

fn aria_framework_path() -> PathBuf {
    // examples/aria/framework.json lives at workspace root relative to
    // CARGO_MANIFEST_DIR (= crates/runtime-main/).
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .join("examples")
        .join("aria")
        .join("framework.json")
}

#[tokio::test]
async fn valid_aria_framework_loads_with_zero_gaps() {
    // The reference framework is the archetype proof for §0a Capability
    // Matrix; every primitive reference must resolve. If this test fails,
    // either the framework drifted or the loader is over-flagging.
    let path = aria_framework_path();
    assert!(
        path.exists(),
        "examples/aria/framework.json missing at {}",
        path.display(),
    );

    let emitter = CollectingEmitter::default();
    // The aria framework declares agents in `Object { id, path }` form (not
    // inline), so the loader has nothing to walk at Layer 1 (per-agent
    // allowed_*/spawns lives in agents/*.md which the loader doesn't open
    // until M07 registry-import). Expectation: zero gaps.
    let result = load_and_validate(&path, &emitter).await;
    assert!(
        result.is_ok(),
        "valid aria framework failed to load: {result:?}",
    );
    assert!(
        emitter.events.lock().unwrap().is_empty(),
        "valid framework emitted unexpected gap events",
    );
}

#[tokio::test]
async fn two_consecutive_loads_succeed() {
    // Gotcha #69: per-method multi-call invariant. The loader's
    // `read_to_string` + `from_str` chain holds no shared state, but the
    // contract that two sequential loads BOTH succeed is the load-bearing
    // assertion — protects against any future refactor that adds caching
    // / locking that would surface a single-use bug.
    let path = aria_framework_path();
    let emitter = CollectingEmitter::default();
    let first = load_and_validate(&path, &emitter).await;
    let second = load_and_validate(&path, &emitter).await;
    assert!(first.is_ok(), "first load failed: {first:?}");
    assert!(second.is_ok(), "second load failed: {second:?}");
    assert!(
        emitter.events.lock().unwrap().is_empty(),
        "valid framework emitted unexpected gap events across consecutive loads",
    );
}
