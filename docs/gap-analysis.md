# Gap Analysis — Agent Runtime (Living Document)

> **APPEND-ONLY.** This file is the project's running ledger of code↔spec gaps,
> contradictions, ambiguities, open questions, and prioritized fix backlog.
>
> Per `CLAUDE.md` §20 Gap Analysis Protocol, **no prior entry may be edited,
> reordered, or deleted.** New milestones append a section at the bottom only.
> If a prior finding is later resolved, do NOT modify the original entry — add
> a status line to the current milestone's "Carry-forward" section that
> references the prior entry by milestone tag and states the resolution.
>
> Authored by Claude during the **Phase Closeout — Gap Analysis** stage of
> each parent milestone (Stage E in M01; the final stage of M02–M11). User
> reviews alongside the milestone PR. Approval gates the merge.
>
> CI enforces append-only via a diff check (added in M01 Stage D's CI workflow).

---

## How to use this document

- **What it is:** the cumulative quality + spec audit across all milestones to
  date. Every milestone's entry reviews the *whole* codebase and *whole* spec,
  not just what shipped this milestone.
- **What it isn't:** per-stage retrospective (those live in
  `docs/build-prompts/retrospectives/`). Retrospectives evaluate the build
  *process* — did the prompt-driven workflow work? This file evaluates the
  build *product* — does the code match the spec, what did the spec get
  wrong, what's the prioritized fix backlog?
- **When updated:** at the very end of every parent milestone, after Stage D
  (or final stage) commits, before the milestone PR is opened. The Phase
  Closeout CLI prompt instructs Claude to append the new entry per the
  template below.
- **Review:** user reviews the new entry alongside the milestone PR diff and
  the parent-milestone summary. Approval is required before the PR opens.
- **Where the carry-forward goes:** if M02 finds that an M01 fix-backlog item
  is now resolved (or was always wrong), M02's "Carry-forward" section says
  so. M01's entry is **never edited.** This preserves the audit trail.

---

## Entry template

Every milestone entry has six sections. If a section has nothing to report,
write **"None observed."** — do not omit the section.

```markdown
## M[NN] — <Title> (<YYYY-MM-DD>, commit `<sha>`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M[NN].A through M[NN].<X>
> Reviewed against: agent-runtime-spec.md, schemas/*.v1.json, prior gap
> analysis entries (if any).

### Codebase deep dive

<200–500 words. Cumulative review of the code shipped to date — not just
this milestone. What's solid, what's notable, what surprised. Reference
specific files/modules. If something is structurally weak in a way that
will compound, name it here.>

### Adherence to spec

<For each area touched by this milestone, classify with file:line citations.>

- ✅ **<area>** — matches spec at `<spec section / file:line>` — cite code at
  `<crate/file.rs:line>`
- ⚠️ **<area>** — deviates from spec at `<spec section>` — code at
  `<crate/file.rs:line>` — reason: <one line>
- ❌ **<area>** — contradicts spec at `<spec section>` — code at
  `<crate/file.rs:line>` — resolution: <plan, with milestone tag>

### Spec review (forward-looking)

<Cumulative scan of the spec. Items here may surface from this milestone's
work or from re-reading prior sections with fresh eyes.>

- **Missing items:**
  - `<spec section>` — <what's missing> — surfaces in <future milestone>
- **Contradictions:**
  - `<spec section A>` vs. `<spec section B>` — <description> — recommend:
    <fix in next docs(spec) PR>
- **Ambiguity:**
  - `<spec section>` — <ambiguity> — Claude resolved this milestone by
    <choice>; should the spec lock the choice?
- **Open questions:**
  - <question that the spec doesn't answer; relevant to which milestone>
- **Recommended spec changes:**
  - `<file:section>` — <change> — rationale: <one line>

### Fix backlog

<Code AND spec fixes. Severity levels are non-elastic — if everything is
"important," the prioritization is meaningless.>

- 🔴 **Critical** (must fix before next milestone starts):
  - `<area>` — <fix> — owner: <code | spec> — at `<file:line>`
- 🟡 **Important** (should fix this release cycle, may queue for a dedicated
  prep session before a later milestone):
  - `<area>` — <fix> — owner: <code | spec>
- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - `<area>` — <fix>

### Carry-forward from prior milestones

<For every unresolved fix-backlog item from any prior milestone entry,
state current status. Do NOT modify the prior entry.>

- **M[prior NN] critical item "<name>"** — <status: resolved at
  `<file:line>` / still open / deferred to <milestone> with rationale>

<If this is M01, write "N/A — first milestone.">

### Sign-off

**Claude:** I have generated this gap analysis after the final stage of
M[NN]. This is my honest assessment of the cumulative code-vs-spec state.
User review pending. The PR remains undrafted until this entry is approved.

**Surfaced at:** <YYYY-MM-DD HH:MM TZ>
```

---

## Milestone entries

<!-- ============================================================ -->
<!-- New entries are appended below this line. Earliest first.    -->
<!-- DO NOT edit, reorder, or delete any entry below.             -->
<!-- ============================================================ -->

*No entries yet. M01 will be the first.*
