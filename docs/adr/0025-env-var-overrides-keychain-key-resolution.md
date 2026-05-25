# ADR-0025: Env-var override pattern for `ANTHROPIC_API_KEY` (env-first, keychain-fallback)

**Status:** Accepted
**Date:** 2026-05-24
**Accepted:** 2026-05-24 (M08.5.5 Stage A.fix impl commit per CLAUDE.md §11)
**Deciders:** @kknipe2k
**Tags:** persistence, ci, dev-experience, security

## Context

`crates/runtime-main/src/key_store.rs::read_api_key` (pre-M08.5.5)
reads the Anthropic API key from the OS keychain only. The M08.5 IRL
re-verify on the Windows build machine 2026-05-23 surfaced the
practical impact: `tests/e2e-tauri/smoke.e2e.ts` test #2 hardcodes a
placeholder API key (`sk-ant-test-1234567890123456`) and saves it to
the OS keychain via the SetupPanel UI; tests 3-6 then make real
Anthropic API calls + read the keychain entry → auth fails with the
placeholder. Setting `ANTHROPIC_API_KEY` locally has no effect
because keychain takes precedence.

Without a per-process override, the local-test-with-real-key workflow
is broken: a developer cannot run the full e2e suite locally without
first manually re-entering their real key into the SetupPanel UI,
then re-running, then manually clearing it from the keychain (or
overwriting it with the placeholder on the next test run). This
friction makes the local harness less useful than CI for the
key-required tests.

CI uses `ANTHROPIC_TEST_KEY` GitHub secret directly (CI's
`e2e-tauri-driver` job sets it as `ANTHROPIC_API_KEY` env var). The
runtime IS already env-aware for CI; the local path is what's missing.

## Decision

Adopt the **env-first, keychain-fallback** pattern in
`read_api_key`: check `std::env::var("ANTHROPIC_API_KEY")` first; if
set + non-empty, return it; otherwise fall through to the OS keychain
read.

Concretely (full diff in
`docs/build-prompts/M08.5.5-mcp-resilience.md` § A.3.2):

```rust
pub fn read_api_key() -> Result<SecretString, KeyStoreError> {
    if let Ok(env_key) = std::env::var("ANTHROPIC_API_KEY") {
        if !env_key.is_empty() {
            return Ok(SecretString::from(env_key));
        }
    }
    let entry = Entry::new(SERVICE, USER)?;
    match entry.get_password() {
        Ok(s) => Ok(SecretString::from(s)),
        Err(keyring::Error::NoEntry) => Err(KeyStoreError::NotFound),
        Err(e) => Err(e.into()),
    }
}
```

The env var name `ANTHROPIC_API_KEY` matches the upstream Anthropic
SDK convention (the standard variable Anthropic's docs name for
client-library API key reads), so local devs and CI both use the same
name across the entire toolchain.

## Consequences

### Positive

- Local tests with real keys work without keychain pollution (run
  `$env:ANTHROPIC_API_KEY = "sk-ant-..." ; npm run test:e2e:tauri` in
  PowerShell, or use the M08.5.5-introduced `.env.local` loader).
- CI behavior unchanged (CI sets the env var; the read still picks up
  the env var first).
- Matches the upstream Anthropic SDK convention; no surprise for
  developers familiar with the SDK.
- Trivially reversible (delete the env var, keychain takes over).

### Negative

- Two key-resolution paths to maintain (env var + keychain). The
  precedence rule is documented in the docstring and tested.
- A misconfigured env var (e.g., a typo in the key) silently overrides
  a working keychain entry. The runtime's existing Anthropic-401
  error surface (`ProviderError::Auth`) catches this on first API
  call; not a silent failure.

### Neutral / future implications

- The `.env.local` loader added in M08.5.5 Stage A.fix to
  `wdio.conf.ts` provides a convenient local-only path that doesn't
  leak the key into PowerShell history. Both `.env.local` and manual
  `$env:ANTHROPIC_API_KEY = ...` paths work; the env-var precedence
  rule is the same.
- If a future milestone adds more secret-managed credentials (e.g.,
  per-MCP-server auth tokens), the same env-first pattern is the
  forward path (`MCP_<SERVER>_AUTH_TOKEN` env var → keyring entry).
  This ADR establishes the precedent.

## Alternatives Considered

### Alternative A: Keychain-only (status quo)

**Rejected because:** the local-test-with-real-key friction is real
and ongoing; without the override, every local test run requires
manual SetupPanel re-entry. The friction makes the local harness
materially less useful than CI, defeating the M08.5.5 Stage A.fix
investment in harness hardening.

### Alternative B: Env-var-only (no keychain)

**Rejected because:** the SetupPanel-based onboarding UX (M07 + M08
Settings panel) writes to keychain. Removing keychain support would
break the documented user onboarding flow; the runtime would have no
way to persist a key between launches without an env var.

### Alternative C: User-configurable precedence (env-first vs keychain-first toggle)

**Rejected because:** runtime configurability for credential
resolution is over-engineering for v0.1. Env-first is the
industry-standard convention (12-factor app config, all major SDKs)
and the simpler default.

## Related

- Spec sections: §13 (Privacy & telemetry — keychain integration);
  §14 (First-run UX — SetupPanel key entry).
- Prior ADRs: none (first key-resolution-precedence ADR).
- External: 12-factor app config (<https://12factor.net/config>);
  Anthropic SDK env var convention
  (<https://docs.anthropic.com/en/api/getting-started>).

## Notes

Status flips `Proposed → Accepted` in the M08.5.5 Stage A.fix impl
commit per CLAUDE.md §11. Adoption verification: the three new tests
in `key_store.rs` (env-overrides-keychain, empty-env-falls-back,
unset-env-falls-back) pin the precedence; the M08.5.5 D.fix re-verify
includes a manual `$env:ANTHROPIC_API_KEY = "sk-ant-real-key" ; npm
run test:e2e:tauri` confirming tests 3-6 now pass with the env-var
key, NOT the keychain placeholder.
