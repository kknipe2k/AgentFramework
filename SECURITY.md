# Security Policy

## Reporting a Vulnerability

**Do not file a public GitHub issue** for security vulnerabilities. Public disclosure before a fix is available puts users at risk.

Instead, use one of these private channels:

- **GitHub Security Advisories** (preferred): https://github.com/kknipe2k/AgentFramework/security/advisories/new
- **Private email** to the maintainer contact: *(replace with real email at v0.1 publication; placeholder until then)*

Please include:
- Affected version(s) (commit SHA or release tag)
- Description of the issue
- Steps to reproduce (or proof-of-concept code if safe to share)
- Impact assessment as you see it
- Any mitigating factors you're aware of
- Whether you'd like public credit (and how you'd like to be credited)

## Response Timeline

We aim for the following SLOs once a report is received:

| Severity (CVSS v3.1) | Acknowledgment | Initial assessment | Fix target |
|---|---|---|---|
| Critical (9.0–10.0)  | within 24 hours | within 72 hours | within 14 days |
| High (7.0–8.9)       | within 48 hours | within 7 days   | within 30 days |
| Medium (4.0–6.9)     | within 7 days   | within 14 days  | within 60 days |
| Low (0.1–3.9)        | within 14 days  | within 30 days  | next minor release |

These are targets, not contractual guarantees. Single-maintainer projects miss SLOs sometimes; we'll communicate when that happens.

## Disclosure Policy

- **Embargo:** 90 days from initial report by default. Extends if the fix is genuinely complex; shortens if active exploitation is observed.
- **Coordinated disclosure:** the reporter and maintainers agree on a publication date once a fix is staged.
- **CVE:** requested via GitHub Security Advisories for any vulnerability that warrants one (typically Medium and above; some Lows depending on impact).
- **Credit:** the reporter is credited in the changelog and the published advisory unless they prefer anonymity.
- **Active exploitation:** if exploitation is observed in the wild, the embargo is dropped — we publish what we know immediately along with mitigation guidance.

## Scope

In scope:
- The runtime binary (`runtime-main`, `runtime-drone`, `runtime-sandbox`) and its dependencies as shipped in a release artifact.
- The Tauri webview integration (renderer ↔ main IPC, allowlist enforcement, capability checks).
- The schemas in `schemas/` (validation bypasses, schema-injection attacks).
- The Anthropic API client and SSE parsing in `runtime-main`.
- The capability enforcement layers (§8.security L1–L5 in the runtime spec).
- The OS keychain integration (secrets handling).
- The drone snapshot, recovery, and IPC implementations.
- The example frameworks in `examples/` if a vulnerability there allows escape from declared capabilities.

Out of scope:
- Vulnerabilities in user-installed third-party MCP servers (report those upstream).
- Vulnerabilities in user-generated tools/skills/agents (these are user-managed; we provide capability enforcement, not the underlying logic).
- Vulnerabilities in Anthropic's API (report to Anthropic).
- Operator-tier installations where the user has explicitly chosen to disable safety gates (documented as user-accepted risk; see runtime spec §13 and §14).
- Issues that require physical access to an unlocked, logged-in user device.
- Self-inflicted issues from sharing API keys or installing artifacts from untrusted sources after explicit warnings.

## Threat Model

The full threat model lives in `docs/SECURITY.md` (separate from this disclosure policy). Summary:

We defend against:
- **Malicious model output** — prompt-injection-driven trojan artifacts. Mitigated via §8.security L2 capability enforcement, L3 sandboxed validation, L4 tier review.
- **Compromised registry** — upstream serves a poisoned artifact. Mitigated via hash-locked installs (`skills.lock`), L3 sandbox runs on every install regardless of source, L5 provenance.
- **User error** — installing a known-bad artifact. Mitigated via mandatory capability disclosure (L1), tier-gated review (L4 Novice tier requires manual review), known-bad pattern deny-list.

We do NOT defend against:
- Operator-tier user knowingly installing a known-bad artifact.
- Skills attempting prompt injection on the next agent (mitigation is process-wide, not generator-specific).
- Attacks on the runtime binary itself outside the categories above (signed-release tampering is the OS's concern; we sign releases).

## What's Safe to Share

When reporting, please err on the side of less sharing initially. We may follow up for:
- Exact reproduction steps once a private channel is established.
- Sanitized logs (the runtime's `skills.audit.jsonl` and `signals.jsonl` may contain prompts; redact them before sending).
- A minimal proof-of-concept artifact if the vulnerability is in artifact handling.

Do not include:
- Your own Anthropic API key (or anyone else's).
- Production secrets or production user data.
- Personal data of any individual unless it is your own and you've explicitly chosen to share.

## After a Fix Lands

- The vulnerability is described in the changelog under "Security" with severity, affected versions, and remediation version.
- A GitHub Security Advisory is published.
- A CVE number (if assigned) is referenced from both.
- The reporter is notified before public disclosure.
- A blog or social post may follow if the issue is significant enough to warrant active user awareness; this is not used for promotion of the project.

## Maintainer contact

*(To be filled in at v0.1 publication. Until then, security reports go through GitHub Security Advisories on this repository.)*
