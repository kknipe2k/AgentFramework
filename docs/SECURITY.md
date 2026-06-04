# Threat Model

This document describes the security architecture the Agent Runtime is **designed
around** — the assets it protects, the trust boundaries it draws, the adversaries
it anticipates, and the defense-in-depth layers (L1–L5) that mitigate them.

It is the companion to [`SECURITY.md`](../SECURITY.md) (the vulnerability
**disclosure policy**) and a reader-facing distillation of `agent-runtime-spec.md`
§8.security. Where this document and the spec disagree, the spec is the contract.

**Status.** This is the *design* threat model. Which layers are fully implemented
is milestone-dependent and tracked in [`CHANGELOG.md`](../CHANGELOG.md) and the
spec's §0d release-scope matrix; do not read a defense described here as a
guarantee that it is wired end-to-end in the current build. Claims about shipped
behavior live in the changelog and per-milestone gap analysis, not here.

## Assets

What an attacker would want, in rough priority order:

1. **The user's Anthropic API key.** Stored in the OS keychain (`keyring`), never
   in plaintext on disk. Exfiltration = direct financial loss to the user.
2. **The user's local files and environment.** Agents can be granted file and
   shell capabilities; a trojaned artifact could read or destroy user data.
3. **Workflow definitions and outputs.** Framework JSON, generated artifacts,
   session transcripts, and the audit logs (`skills.audit.jsonl`, `signals.jsonl`)
   — these can contain prompts and, by extension, sensitive context.
4. **Runtime integrity.** The ability to make the runtime execute code outside the
   declared capability envelope (capability bypass, sandbox escape).

## Trust boundaries

The runtime is deliberately split across processes so that a compromise of one
does not imply compromise of the others:

| Boundary | Mechanism | What crosses it |
|---|---|---|
| Renderer ↔ Main | Tauri typed IPC (allowlist) | UI commands, event stream; the webview has no Node/OS API |
| Main ↔ Drone | Framed JSON over Unix socket / named pipe | Snapshot, recovery, persistence ops |
| Main ↔ Sandbox | Subprocess, OS-isolated (seccomp / Landlock / Job Objects) | Untrusted artifact (L3) validation only |
| Main ↔ Anthropic | HTTPS + SSE (`reqwest`) | Prompts, model output; the user's key |
| Main ↔ MCP servers | stdio / HTTP (per server) | Tool calls to user-installed third-party servers |
| Main ↔ Registry | HTTPS fetch, SSRF-hardened egress | Imported artifacts, before validation |

The **renderer is treated as untrusted UI**, the **sandbox runs untrusted code**,
and the **main process is the policy enforcement point** — it never directly
executes untrusted input; that always goes through the sandbox.

## Adversaries

- **Malicious model output.** A prompt-injected or adversarial model emits a
  trojan artifact (a skill/tool/agent whose declared behavior hides a capability
  grab). The primary in-scope adversary.
- **Compromised / poisoned registry.** An upstream source serves a tampered
  artifact, or a man-in-the-middle swaps one in transit.
- **Malicious MCP server.** A user installs a third-party MCP server that behaves
  adversarially. Capability-bounded but not internally audited (see Non-goals).
- **User error.** A well-meaning user installs a known-bad artifact, or pastes a
  key into the wrong field.
- **Local malware / physical access.** Out of scope (see Non-goals) — an attacker
  already executing as the user on an unlocked device is outside what an
  application-layer runtime can defend.

## Defense in depth (L1–L5)

The capability model is layered so that no single check is the only thing standing
between an artifact and the user's machine:

- **L1 — Capability disclosure.** Every tool, skill, and agent declares the
  capabilities it needs (`fs`, `network`, `shell`, …). Generated artifacts *must*
  declare; an undeclared capability block is rejected. The user sees the envelope
  before anything runs.
- **L2 — Capability enforcement.** The runtime enforces declared capabilities at
  call time and **narrows** them across agent→agent edges — a child agent can
  never exceed its parent's grants. A test must not be able to widen a capability
  to pass.
- **L3 — Sandboxed validation.** Every artifact install runs in the OS-isolated
  sandbox subprocess (seccomp + Landlock on Linux, Job Objects on Windows)
  *regardless of source*, before it can touch the real environment.
- **L4 — Tier-gated review.** The active tier governs autonomy. The Novice tier
  requires manual review of capability-bearing artifacts; Promoted auto-accepts
  validated artifacts but is still blocked from rubber-stamping `shell: true` /
  `network: ["*"]` grants. (v0.1 ships Novice + Promoted only; no Operator tier.)
- **L5 — Provenance.** Hash-locked installs (`skills.lock`) pin artifact content;
  a changed hash is a changed artifact and re-triggers review + L3.

## Threats we defend against

- **Trojan artifact from model output** → L1 disclosure + L2 enforcement + L3
  sandbox + L4 review.
- **Poisoned registry / in-transit tamper** → L5 hash-lock + L3-on-every-install +
  SSRF-hardened fetch egress.
- **Capability creep across a multi-agent graph** → L2 monotonic narrowing on
  every Agent→Agent edge.
- **Secret leakage to disk** → the key lives only in the OS keychain; audit logs
  are the user's and are never transmitted (no telemetry, per spec §13).

## Non-goals (explicitly NOT defended)

Mirrors [`SECURITY.md`](../SECURITY.md) §Scope and spec §8.security:

- An **Operator-tier** user who knowingly disables gates and installs a known-bad
  artifact (documented, user-accepted risk).
- The internal logic of **user-installed MCP servers** or **user-authored
  artifacts** — the runtime bounds their capabilities, it does not audit their
  code. Report MCP-server vulns upstream.
- **Anthropic API** vulnerabilities (report to Anthropic).
- **Local malware or physical access** to an unlocked, logged-in device.
- **Cross-agent prompt injection** as a process-wide concern: the mitigation is
  capability-bounding the *next* agent, not detecting injection in model text.

## Residual risk & user responsibilities

- Keep the OS user account and keychain secured; the runtime's key protection is
  only as strong as the OS account.
- Install artifacts and MCP servers from sources you trust; L3 + L5 reduce but do
  not eliminate supply-chain risk.
- Read the capability disclosures at install time — they exist to be read.

## Changing the security model

Per the Engineering Charter (spec §12) and [`CONTRIBUTING.md`](../CONTRIBUTING.md),
an ADR is required for any change to a capability-matrix primitive, the IPC
protocol between main / drone / sandbox, an `LLMProvider`, or any L1–L5
enforcement behavior. Security-relevant paths are CODEOWNERS-gated.

To report a vulnerability, follow [`SECURITY.md`](../SECURITY.md) — **not** a
public issue.
