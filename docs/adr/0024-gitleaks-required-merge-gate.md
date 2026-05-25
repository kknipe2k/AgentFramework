# ADR-0024: Gitleaks secret-scan as required merge gate + lefthook pre-commit hook

**Status:** Accepted
**Date:** 2026-05-24
**Accepted:** 2026-05-24 (M08.5.5 Stage A.fix impl commit per CLAUDE.md §11)
**Deciders:** @kknipe2k
**Tags:** security, ci, dev-experience

## Context

The orchestration session ran `gitleaks v8.21.2` over the full git
history of the repo on 2026-05-23 — 324 commits, all branches, 0
leaks confirmed (`docs/m08.5-irl-re-verify-handoff.md` § "Pending"
table). The repo is currently clean of committed secrets and the
`.gitignore:37-39` rules (`.env`, `.env.*`, `!.env.example`) prevent
the most common accidental-commit path. But there is **no continuous
gate** preventing a future accidental commit of an `ANTHROPIC_API_KEY=
sk-ant-real-key` line, an SSH private key, an AWS credential, or any
other secret pattern.

The 2026 industry-standard approach (per Google Stitch DESIGN.md
adoption metrics, GitHub Advanced Security baseline, Anthropic's own
public commits): every repo has a continuous secret-scan gate on
both the pre-commit local hook and the CI required-status-check path.

## Decision

Adopt **gitleaks** as the project's secret-scanning gate, integrated
at two layers:

1. **Local pre-commit hook** via `lefthook.yml` —
   `gitleaks git --staged --no-banner` blocks the commit when any
   secret pattern matches in the staged diff. Local devs install
   gitleaks via the OS package manager (apt / winget / brew /
   cargo install). The build-machine setup guide
   (`docs/build-machine-tauri-driver-setup.md`) adds Phase 0.5
   covering the install.

2. **CI required-status-check** via `.github/workflows/ci.yml` — the
   `gitleaks` job uses `gitleaks/gitleaks-action@v2` with
   `fetch-depth: 0` (full history scan) and `GITHUB_TOKEN`
   (PR-commenting). Required for merge per the `main` branch
   protection rule. Gated by `detect-cargo.outputs.code_changed` so
   docs-only changes skip the gate (per the PR #98 docs-only-skip
   pattern).

The default gitleaks rule set (`gitleaks.toml` embedded in the
binary) covers Anthropic, OpenAI, AWS, GCP, GitHub PAT, GitHub
fine-grained PAT, generic API key patterns, and ~150 provider-
specific patterns. No custom config needed at adoption; add a
`.gitleaksignore` override only when a false-positive needs
allowlisting.

## Consequences

### Positive

- Continuous gate prevents the secret-commit-class bug class entirely.
- Local pre-commit hook gives developers immediate feedback (failures
  don't wait for CI).
- CI gate is the authoritative enforcement (a developer who skips the
  hook still cannot merge).
- Zero-config adoption (gitleaks ships its rule set; no toml authoring
  needed).

### Negative

- False positives possible (gitleaks' broad rule set sometimes flags
  test fixtures). Mitigation: `.gitleaksignore` for known-safe
  patterns OR inline `gitleaks:allow` comments per the upstream docs.
- Adds a build-machine prerequisite (gitleaks must be on PATH for the
  pre-commit hook to fire). Mitigation: setup guide documents the
  install, and the CI gate is the authoritative check even if local
  hook is bypassed.

### Neutral / future implications

- A `GITLEAKS_LICENSE` is only required for organization accounts
  (free license available at gitleaks.io). The personal
  `kknipe2k/AgentFramework` repo does not need one. If the repo
  transfers to an org in the future, register for the free license.
- A future ADR could expand the secret-scan ecosystem (trufflehog,
  detect-secrets, custom patterns). For v0.1, gitleaks alone is the
  baseline.

## Alternatives Considered

### Alternative A: Pre-commit hook only (no CI gate)

**Rejected because:** local hooks can be bypassed (`git commit
--no-verify`). The authoritative enforcement must be CI-required.

### Alternative B: CI gate only (no pre-commit)

**Rejected because:** developers get feedback only AFTER pushing,
which is too late for a secret already on a feature branch (rewrite
history pain). The pre-commit hook catches before the secret enters
git's object store.

### Alternative C: trufflehog instead of gitleaks

**Rejected because:** trufflehog and gitleaks have comparable rule
sets; gitleaks has lower latency on the pre-commit path (faster scan
of staged diff vs trufflehog's broader scan modes). gitleaks is the
de-facto industry standard in 2026 (referenced as the baseline in
the GitHub Advanced Security docs).

## Related

- Spec sections: §13 (Privacy & telemetry) — no telemetry is the
  rule; preventing accidental secret commits enforces it structurally.
- Prior ADRs: none (first secret-scan ADR).
- External: <https://github.com/gitleaks/gitleaks>;
  <https://github.com/gitleaks/gitleaks-action>; CVE-2024-24576
  (BatBadBut — unrelated but reinforces the importance of treating
  secrets and command-injection vectors with the same rigor).

## Notes

Status flips `Proposed → Accepted` in the M08.5.5 Stage A.fix impl
commit per CLAUDE.md §11. Adoption verification: a deliberate test
branch commits a secret-pattern fixture (NOT pushed), confirms
gitleaks fails both locally + in a `gh workflow run` dispatch.
