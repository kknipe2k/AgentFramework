# ADR-0018: Import-fetch SSRF egress hardening

**Status:** Accepted
**Date:** 2026-05-20
**Deciders:** @kknipe2k (maintainer)
**Tags:** import, capability, security, ssrf, network, m07.5

## Context

The import pipeline fetches a **user-supplied URL server-side** (spec
Phase 7 §2152-2211; the M07.E Builder Import panel ships in v0.1, so the
path is live). On `main` @ `ddf2a69` that fetch is unhardened:

- `crates/runtime-main/src/import/fetch.rs:33-43` — `HttpFetcher::get`
  is `reqwest::Client::new().get(url).send()` with default config:
  **it auto-follows up to 10 HTTP redirects**, has no scheme
  restriction, no request timeout, and reads the response body
  unbounded (`resp.bytes()`).
- `import/mod.rs:339` — `host_of` is a hand-rolled URL host extractor.
  Parser confusion between a custom parser and the HTTP client's real
  parser is a documented SSRF-bypass class; it also does not normalize
  IPv4 encoding tricks (`http://0x7f000001/`, `http://2130706433/`).
- `src-tauri/src/commands.rs:1054-1077` — `EnforcerGate::check` (the
  L1 `NetworkGate` impl) constructs a fresh `CapabilityEnforcer`,
  grants itself the exact declaration it then checks, and returns the
  always-`Ok` result — a tautology (simplify finding CQ-M07-1), behind
  a doc comment that claims "default-deny + domain scope … no
  phone-home."

**The threat.** A server-side fetch of a user-supplied URL is the
canonical Server-Side Request Forgery (SSRF) surface. The desktop
threat model is concrete: a socially-engineered URL paste ("import this
skill from `<url>`") turns the runtime into a confused deputy that
issues requests from the user's network position — reaching
`localhost` services, the LAN (router admin, NAS, printers), and any
internal/VPN-reachable host. **DNS rebinding** defeats a naive
validate-then-fetch check (the resolve-check-fetch pattern is broken by
design — the host re-resolves to an internal address between the check
and the connect). **Redirect-based bypass** defeats an
original-host-only check (`https://raw.githubusercontent.com/…` → HTTP
302 → `http://192.168.1.1/…`, which `reqwest` follows unchecked).

This is squarely a supply-chain / import concern — the same surface
ADR-0014's threat model governs (sandbox-before-trust, hash-lock). An
unguarded fetch is the gap in that posture, and `EnforcerGate`'s doc
comment already *claims* the protection, so this is also an honesty
gap.

Constraints: the import feature is "paste any GitHub-raw (or other
public) URL" — a strict per-host allowlist would break it, so the
control must be "allow any *public* host, reject every *internal* one,"
i.e. an egress IP-classification blocklist, not a host allowlist. The
project is dependency-conservative (Hard Rule 6 — `cargo deny` must
pass; no new core dependency without justification). `std::net`'s
single-call `IpAddr::is_global()` is **still unstable** (Rust issue
#27709), so a stable-Rust classifier must compose the individual
predicates and hand-roll the ranges `std` omits.

Without a decision, M07.5 would ship the simple "honest no-op" gate
(CQ-M07-1 was filed with "OR correct the comment" as an option) — which
removes the dishonest code but leaves the actual SSRF hole open in a
shipped v0.1 feature.

## Decision

**We adopt a real SSRF-hardened import-fetch egress gate, replacing the
tautological `EnforcerGate` / `NetworkGate` with a pure, exhaustively
tested `import::egress` module and a hardened `HttpFetcher`.**

The egress control has six parts:

1. **Scheme allowlist — `https` only.** No `http`, no `file`, no other
   scheme. Enforced after parsing.
2. **Proper URL parsing.** Parse with `reqwest::Url` (the WHATWG URL
   parser, already a transitive dependency — `reqwest` re-exports
   `url`). It canonicalizes IPv4 encoding tricks to a normal form,
   defeating the `0x7f000001` / `2130706433` bypass class. The
   hand-rolled `host_of` is retired.
3. **IP classification — reject every non-public range.** Resolve the
   host and classify **every** resolved address; reject loopback,
   RFC-1918 private, link-local (`169.254/16` and `fe80::/10`), CGNAT
   shared (`100.64/10`), IPv6 ULA (`fc00::/7`), unspecified, multicast,
   broadcast, documentation/TEST-NET, and **IPv4-mapped IPv6**
   (`::ffff:a.b.c.d` — unwrapped before classification). A single
   rejected address fails the whole fetch.
4. **DNS pinning — defeat DNS rebinding.** Resolve the host once,
   classify the resulting address(es), and pin the HTTP connection to
   the validated address via `reqwest`'s `ClientBuilder::resolve`, so
   the client does not re-resolve and connect somewhere else.
5. **Redirect re-validation.** The hardened `HttpFetcher` disables
   `reqwest`'s automatic redirect following (`redirect::Policy::none()`);
   `fetch_with` drives a bounded manual redirect loop that re-runs the
   full egress validation (scheme + resolve + classify + re-pin) on
   each `Location` before following it.
6. **Resource bounds.** A connect timeout, a request timeout, and a
   response-body-size cap (a streamed read with a running byte counter,
   robust against a lying `Content-Length`).

The decision logic is a **pure, fully-tested seam**:
`import::egress::classify_ip` (given an `IpAddr`, allowed?) and
`import::egress::check_url` (scheme + parse) are pure functions covered
to the runtime-main ≥95 gate; DNS resolution is behind a new `Resolver`
seam trait (the established `Fetcher` / `Sandbox` / `Clock`
injected-seam archetype) so tests inject "this host resolves to
`127.0.0.1`" and assert rejection. Only the irreducible socket calls —
the real `reqwest` GET and the real DNS lookup — remain in the
`import/fetch.rs` OS-call coverage holdout.

**No new dependency** is required: `reqwest::Url` (re-exported),
`tokio::net::lookup_host` (DNS), and `std::net` IP predicates are all
already available. A vetted focused crate (e.g. `http-acl`) MAY be
adopted for the IP-range classification if it clears `cargo deny`; the
default is the hand-rolled classifier with an exhaustive test matrix
(the dependency-conservative path — the logic is bounded and the test
matrix is the real assurance).

## Consequences

### Positive

- **The import path has a real SSRF defense** — confused-deputy LAN /
  `localhost` pivots, DNS rebinding, and redirect-based bypass are all
  closed. The `EnforcerGate` doc comment's "no phone-home / scoped
  egress" claim becomes true.
- **The whole control is TDD-able.** The "honest no-op" had no red
  test; the egress classifier has an exhaustive one — reject
  `http://`, `https://127.0.0.1`, `https://[::1]`, `https://10.0.0.1`,
  `https://169.254.169.254`, IPv4-mapped, `https://0x7f000001`, and a
  redirect to a private address; cap an over-size body.
- **It completes ADR-0014's import threat model.** Sandbox-before-trust
  + hash-lock-on-install + a real fetch egress gate is a coherent
  supply-chain posture; the cryptographic-provenance layer (v1.0) still
  attaches at the `share_provenance` seam.
- **No new dependency** — `cargo deny` is unaffected; Hard Rule 6 is a
  non-event.

### Negative

- **DNS pinning fixes the connection to one resolved address.** A host
  behind a CDN with many addresses is fetched via the single address
  the validation resolved — correct and intended (it is the
  rebinding defense), but it means the fetch does not benefit from the
  client's own address-failover. Acceptable: an import is a one-shot
  GET, not a long-lived connection.
- **The fetch path is larger and more complex** — a new `egress`
  module, a `Resolver` seam, a manual redirect loop. This is the
  deliberate trade the maintainer chose ("the right pattern, not the
  simple one"); the complexity is contained in one module with a pure,
  tested core.
- **The `NetworkGate` trait and `EnforcerGate` struct are removed.**
  `import_artifact_with`'s signature changes (the `gate` parameter
  becomes a `resolver`). A localized breaking change inside
  `runtime-main` + its one `src-tauri` call site, both landed in the
  same M07.5 stage.

### Neutral / future implications

- **The `egress` module is the v1.0 attach point for a user-configured
  import-domain allowlist.** v0.1 allows any public host; a future
  release can layer an explicit allowlist on the same seam.
- **The `Resolver` seam generalizes.** Any future runtime feature that
  must resolve a host for a security decision reuses it.

## Alternatives Considered

### Alternative A: "Honest no-op" gate + corrected comment

Make `EnforcerGate::check` an explicit structural pass-through and
correct the doc comment to say v0.1 does no per-host check (the
CQ-M07-1 "OR correct the comment" option; the original M07.5 phase-doc
draft).

**Rejected because:** it is the *simple* choice, not the right one. It
removes the dishonest code but leaves the SSRF hole open in a shipped
v0.1 feature, and it is not even TDD-able (no behavior to red-test).
The maintainer explicitly directed "choose the right pattern, do not go
for simple."

### Alternative B: Strict per-host domain allowlist

Allow only an enumerated set of hosts (e.g. `raw.githubusercontent.com`,
`gist.githubusercontent.com`).

**Rejected because:** the import feature's contract is "paste a
GitHub-raw **or other** public URL" (MVP §M7) — a tight allowlist
breaks it. The correct granularity is "allow any *public* host, reject
every *internal* range" — an egress IP-classification blocklist. (A
*user-configured* allowlist layered on top is a reasonable v1.0
addition — the `egress` module is built to host it.)

### Alternative C: Adopt an off-the-shelf SSRF-safe HTTP crate

Depend on a crate such as `agent-fetch` (a sandboxed SSRF-protecting
HTTP client) or `http-acl` (an IP/host/port ACL).

**Rejected as the default** (offered as a build-time option): a new
core dependency must clear `cargo deny` and a maintenance/maturity
review (Hard Rule 6). The IP-classification logic is bounded (~60 lines)
and its assurance comes from an exhaustive test matrix, not from the
crate's reputation; hand-rolling keeps the dependency surface minimal.
If the build evaluates a crate and it clears `cargo deny` cleanly, it
may be adopted for the IP-range checks specifically — the decision is
recorded here so either path is ADR-covered.

## Related

- **ADR-0017** — *Import validate-vs-commit lifecycle split* — the
  sibling M07.5 decision; ADR-0017 changes *when* the install commits,
  ADR-0018 hardens *how* the artifact bytes are fetched.
- **ADR-0014** — *skills.lock integrity* — the import threat model
  (sandbox-before-trust, hash-lock) this egress gate completes.
- **ADR-0010** — *MCP dispatch dependency inversion* — the
  injected-seam archetype the new `Resolver` seam follows.
- **Spec sections:** §8.security L1 (the network capability layer this
  gate is); Hard Rule 4 (no phone-home — only the user URL is hit);
  MVP §M7 (the import feature).
- **M07 closeout:** simplify finding CQ-M07-1 (`docs/gap-analysis.md`
  M07 Fix backlog) — the tautological `EnforcerGate` this ADR replaces.
- **Phase doc:** `docs/build-prompts/M07.5-tier-gate-fix.md` Stage
  B.fix — the fix-cycle stage that implements this ADR.
- **External:** OWASP SSRF Prevention guidance; DNS-rebinding-vs-SSRF
  research (the resolve-check-fetch pattern is "broken by design");
  `reqwest` `ClientBuilder::resolve` (DNS pinning) +
  `redirect::Policy::none()`.

## Notes

This ADR is filed `Proposed`. M07.5 Stage B.fix flips it to `Accepted`
in the impl commit that lands the egress hardening (the M06.5.A.fix /
ADR-0012 precedent — the stage that implements an ADR flips it).

`IpAddr::is_global()` (Rust issue #27709) would collapse the classifier
to one call; it is unstable, and the project's MSRV-inclusive CI
forbids relying on it. The hand-rolled classifier composes the stable
`std::net` predicates (`is_loopback`, `is_private`, `is_link_local`,
`is_multicast`, `is_broadcast`, `is_unspecified`, `is_documentation`,
`Ipv6Addr::to_ipv4_mapped`) and hand-codes the ranges `std` omits on
stable (CGNAT `100.64/10`, IPv6 ULA `fc00::/7`, IPv6 link-local
`fe80::/10`). The exhaustive test matrix — every category, both
families, the IPv4-mapped and IPv4-encoding bypasses — is the assurance.
