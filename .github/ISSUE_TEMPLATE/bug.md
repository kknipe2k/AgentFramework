---
name: Bug report
about: Report a defect in the runtime, schemas, examples, or documentation.
title: '[bug] '
labels: bug, needs-triage
assignees: ''
---

<!--
Thanks for filing. Please complete the sections below. Issues missing
required sections may be closed pending more info.

For SECURITY issues: do not file here. Use GitHub Security Advisories
or follow the flow in SECURITY.md.
-->

## Summary

<!-- One sentence: what's wrong. -->

## Component

- [ ] Specification (`agent-runtime-spec.md`)
- [ ] Schemas (`schemas/`)
- [ ] Example framework (`examples/aria/` or `examples/ralph/`)
- [ ] Runtime code (Rust crates) — once code lands
- [ ] Frontend (React/TS) — once code lands
- [ ] CI / build / tooling
- [ ] Documentation
- [ ] Other (specify)

## What did you do?

<!-- Steps to reproduce. Be specific enough that someone else can reproduce
     locally. Minimal example preferred. -->

1.
2.
3.

## What did you expect to happen?

## What actually happened?

<!-- Include error messages verbatim. Stack traces in code blocks. -->

```
<paste output here>
```

## Environment

- **OS:** <!-- Windows 10 / 11 / Server, version -->
- **Runtime version:** <!-- v0.1.x once releases exist; or commit SHA -->
- **Browser/webview:** <!-- if relevant -->
- **Framework loaded:** <!-- examples/aria, examples/ralph, custom, or n/a -->
- **Active mode:** <!-- LITE/STANDARD/FULL/FULL+ — once mode router lands; n/a in v0.1 -->
- **Tier:** <!-- Novice / Promoted — once tier system lands; n/a otherwise -->

## Logs

<!--
Sanitize before sharing. Per SECURITY.md: redact prompts, tool inputs,
API keys, personal data.

Logs typically live at:
  Windows: %APPDATA%\agent-runtime\logs\
  macOS:   ~/Library/Application Support/agent-runtime/logs/
  Linux:   ~/.local/share/agent-runtime/logs/
-->

```
<paste sanitized log excerpt here>
```

## Reproduction artifact (optional)

<!-- If the bug is in framework JSON or artifact loading, attach a minimal
     reproducing framework.json or skill.md/tool.md/agent.md. -->

## Severity (your assessment)

- [ ] Blocking — runtime unusable
- [ ] High — significant feature broken or wrong output
- [ ] Medium — feature works but with caveats / wrong UI
- [ ] Low — cosmetic, typo, edge case

## Additional context

<!-- Anything else relevant. Links to related issues, recent changes that
     might have introduced the bug, etc. -->
