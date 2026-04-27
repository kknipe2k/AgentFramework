# Contributing

Thanks for considering a contribution. This project is in early development — your patience is appreciated as the foundations land.

## State of the project

We're working toward **v0.1.0 Windows Preview**, a desktop runtime for agentic AI workflows. See `docs/MVP-v0.1.md` for the build checklist and `agent-runtime-spec.md` (especially §0d Release Scope Matrix) for what's in scope and what isn't.

Until v0.1 ships, the codebase contains:
- **`agent-runtime-spec.md`** — the authoritative spec for what we're building.
- **`schemas/`** — JSON Schemas (source of truth for type generation).
- **`examples/aria/`** and **`examples/ralph/`** — reference framework artifacts proving the spec is sufficient.
- **`docs/`** — supporting documentation (MVP checklist, ADRs, this contributing guide).
- **`.aria/`** — the existing shell-based ARIA framework. **This is reference material**, not the product. It moves to `archive/aria-shell/` once v0.1 of the runtime ships.

## Before you contribute

1. **Read the spec.** Specifically §0 (positioning), §0a (capability matrix), §0b (Tool/Skill/Agent terminology), §0d (release scope), §12 (engineering charter).
2. **Read this document fully.**
3. **Open an issue or proposal** before significant work. Drive-by PRs without prior discussion may be closed even if the code is good — scope-control is critical for an OSS project at this stage.

## What we accept right now

| Type | Acceptance |
|---|---|
| Bug reports against the spec (typos, contradictions, missing info) | ✅ welcome |
| Bug reports against `examples/aria/` or `examples/ralph/` | ✅ welcome |
| Schema validation issues | ✅ welcome |
| Documentation improvements (README, docs, ADRs) | ✅ welcome |
| New ADR proposals (`docs/adr/`) | ✅ welcome — file as `docs/adr/XXXX-title.md` with status: `Proposed` |
| Code contributions (Rust, TS) | ⏳ deferred until M1 lands; once Cargo workspace exists, see "Code contributions" below |
| New skills/tools/agents in `examples/` | ⏳ deferred until v0.1 generators land; hand-authored artifacts at Operator-tier feel for now |
| Feature proposals beyond §0d v0.1 scope | ⏳ filed and acknowledged, queued for v1.0+ |
| Breaking changes to schemas | 🛑 require ADR + maintainer approval |

## Code contributions (once code lands)

### Setup

Once the Cargo workspace lands (M1), getting started is:

```bash
# Prerequisites (Windows-first; macOS/Linux supported in v1.0)
# - Rust stable (per rust-toolchain.toml — pinned)
# - Node.js 20+
# - Tauri prerequisites: https://tauri.app/start/prerequisites/
# - Git

git clone https://github.com/kknipe2k/AgentFramework.git
cd AgentFramework
cargo build --workspace
npm install

# Run the dev loop (renderer + main + drone, all hot-reloading)
cargo tauri dev

# Run tests
cargo test --workspace
npm run test
```

### Quality gates (per §12 Engineering Charter)

Before opening a PR, verify locally:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
npm run lint
npm run test
npm audit
```

CI mirrors these gates. PRs that fail any of them won't be reviewed until they pass.

Coverage thresholds: **80% line on all new code; 100% on safety primitives** (drone, capability enforcer, plan state machine, snapshot/recovery). Drops in coverage block merge.

### Code review

- 2 maintainer approvals required for PRs touching core (drone, capability, providers, schemas).
- 1 maintainer approval for documentation, ADRs, examples.
- CODEOWNERS (`.github/CODEOWNERS`) auto-assigns reviewers based on path.
- Squash-merge only. Linear history. No force pushes to `main`.

### Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): summary in imperative present tense

optional body explaining what and why (not how — code shows how).

optional footer:
  Co-authored-by: ...
  Refs: #123
```

Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`. Scopes follow crate names (`runtime-drone`, `runtime-main`, `runtime-core`, etc.) or document categories (`spec`, `examples`, `schemas`).

### Sign-off (DCO)

Every commit must be signed off:

```bash
git commit -s -m "feat(drone): add heartbeat backoff"
```

The `-s` adds `Signed-off-by: Your Name <your-email>` to the commit, which is your assertion of the [Developer Certificate of Origin](https://developercertificate.org/). We use DCO instead of a CLA to keep contribution friction low while maintaining IP hygiene.

### Architecture Decision Records

Per §12, an ADR is required for any change that:
- Adds, modifies, or removes a §0a Capability Matrix primitive.
- Changes the schemas (`schemas/*.v*.json`).
- Adds a new `LLMProvider` implementation.
- Changes capability enforcement behavior (any §8.security L1–L5 layer).
- Changes the IPC protocol between main, drone, or sandbox.
- Adopts a new core dependency (anything that becomes a runtime dependency, not dev-only).

Use `docs/adr/0000-template.md` as the starting point. ADRs are immutable once merged — superseded ADRs link to their successor.

## Reporting bugs

Use the bug template in `.github/ISSUE_TEMPLATE/`. Include:
- What you did
- What you expected
- What happened
- Reproduction steps (minimal, ideally a failing test or framework JSON snippet)
- Your environment (OS, version, runtime version)
- Logs (sanitized — see `SECURITY.md` for what NOT to share)

## Proposing features

Use the feature proposal template. Address:
- Problem statement (one paragraph)
- Proposed solution (specific enough to argue against)
- Why now? Why not v1.0+?
- What would change in §0a or §0d if accepted?
- Alternatives considered

If accepted in principle, the next step is an ADR. Significant features without an ADR won't be merged.

## Style

- **Rust:** rustfmt + clippy::pedantic. No manual style debates; the linter wins.
- **TypeScript:** prettier + eslint with shared config. Same.
- **Markdown:** prefer ATX headings (`#`), reference-style links sparingly. Run `markdownlint` if available.
- **Comments:** "no comments by default" per project guidelines. Explain *why* when non-obvious; the *what* should be clear from code. No comments restating identifier names. No comments referencing tickets ("added for #123") — that lives in commit messages.
- **Documentation:** every `pub` Rust API and exported TS API needs a doc comment with at least one example. CI enforces this.

## Communication

- **GitHub Issues** — bug reports, feature proposals, questions about the spec.
- **GitHub Discussions** — broader design conversation, RFC-style debate (enabled at v0.1 launch).
- **Pull Requests** — code, documentation, ADRs, examples.

We don't have a chat server (Slack/Discord/Matrix) yet. May add one once there's a community to support; meanwhile, async-first via GitHub.

## Getting your first PR merged

1. Pick a `good first issue` (or open a new issue describing what you want to fix).
2. Comment on the issue saying you're working on it (avoids parallel work).
3. Fork → branch off `main` → develop → push → open PR.
4. Reference the issue in the PR description.
5. Address review feedback; re-request review when ready.
6. Once approved + CI green, a maintainer squash-merges.

Quick wins for first-timers:
- Spec typos and contradictions.
- ADRs for decisions that have been made informally.
- Schema improvements (more precise patterns, additional examples).
- Documentation gaps.

## What we won't accept

- Code that doesn't compile or pass CI gates.
- PRs without DCO sign-off.
- Scope changes to §0a or §0d without an accompanying ADR.
- Adding a third-party dependency without `cargo deny` review (license compatibility, supply-chain hygiene).
- Telemetry, analytics, or "phone home" code (per spec §13). This is a hard line.
- Code that uses AI assistance without disclosing it. Disclose if you used AI tools to write the code; we don't reject AI-assisted contributions — only undisclosed ones.

## Code of Conduct

This project follows the [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). See `CODE_OF_CONDUCT.md` for the project-specific reporting flow.

## License

By contributing, you agree your contribution is licensed under Apache 2.0 (the project's license). Your DCO sign-off is your assertion of this.

## Questions?

Open a GitHub Discussion (once enabled) or an issue tagged `question`. We'll answer when we can; this is a small project and response times vary.

Thank you.
