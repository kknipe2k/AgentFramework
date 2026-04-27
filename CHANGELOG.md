# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive product specification (`agent-runtime-spec.md`) for a Tauri-based desktop runtime for agentic AI workflows. 21 phase + section headings covering: project positioning, capability matrix, three-concept model (Tool/Skill/Agent), dev loop, release scope matrix, drone, recovery, multi-session, IPC, event pipeline, budget, signals/VDR, LLMProvider abstraction, live graph, plan/task primitive, mode/sizing, gap detection, verify/rails, MCP manager, framework loader, HITL policy, registry, generators with 5-layer security, builder canvas, persistence, secrets vault, reconciliation/degraded modes, engineering charter, privacy/telemetry, first-run UX.
- JSON Schema source-of-truth files in `schemas/` (Draft 2020-12): `common.v1.json`, `skill.v1.json`, `tool.v1.json`, `agent.v1.json`, `framework.v1.json`. All 19 example artifacts validate.
- `examples/aria/` reference framework (19 files, 1947 lines) reconstructing every row of the capability matrix.
- `examples/ralph/` sibling framework (4 files, 367 lines) demonstrating the `loop_policy: continuous` variant; reuses aria/ tools and skills via `source: external`.
- `docs/MVP-v0.1.md` build checklist (11 milestones across ~6 months elapsed; novice-and-experienced two-path success criterion).
- Engineering Charter in spec §12 codifying CI gates, coverage thresholds, dependency hygiene, doc tests, ADR requirements, release signing, security disclosure flow, license + DCO.
- Privacy & Telemetry policy in spec §13: zero telemetry by default, no analytics, no crash reporter; user data export and delete-all-local in Settings.
- First-Run UX state machine in spec §14.
- ADR template and initial ADRs covering ARIA-as-archetype, Tauri-over-Electron, and Engineering Charter adoption.
- LICENSE (Apache 2.0).
- NOTICE file with notable third-party dependency attributions.
- CODE_OF_CONDUCT.md (Contributor Covenant 2.1 by reference).
- SECURITY.md (private disclosure flow + response SLOs + scope + threat model summary).
- CONTRIBUTING.md (state of project, what's accepted now, code-contribution setup, quality gates, DCO, ADR requirements).

### Status

Pre-implementation. The runtime binary does not yet exist. This repository contains:
- The specification we're building toward
- JSON schemas validating the example artifacts
- Two reference frameworks (`aria/` and `ralph/`) that the runtime will eventually load
- The existing `.aria/` shell-based ARIA framework (reference material; moves to `archive/aria-shell/` once v0.1 of the runtime ships)

First milestone (M1: Foundation — Cargo workspace + drone + CI) is the next deliverable.

---

## Versioning

- **0.x** — pre-stable. Schemas may change; APIs are not guaranteed compatible across 0.x versions.
- **1.0+** — semver strict. Breaking changes to framework JSON schema, AgentEvent union, or any `pub` Rust API require a major bump.

## Release artifacts

Once releases begin (v0.1.0 Windows Preview), each release will include:
- Signed Windows installer (`.msi`) at v0.1; macOS `.dmg` and Linux AppImage from v1.0.
- SBOM in CycloneDX format.
- Source tarball.
- SLSA Level 3 provenance attestations from v1.0.
