---
# DESIGN.md — design tokens (W3C DTCG-style). RULES LAYER SEEDED; VISUAL LAYER PENDING.
# This frontmatter is a PLACEHOLDER. The token values below are NOT a
# designed palette — they are stubs so the structure exists. The visual
# layer (real colors, type scale, spacing, component mockups) is authored
# by Claude Design (claude.ai/design) and committed over this seed.
# Per CLAUDE.md §4 rule 11: this file is the design RULES, not a complete
# design system. Do not claim a coherent visual system exists until the
# Claude Design pass lands.
status: rules-seeded-visual-pending
color: {}
typography: {}
spacing: {}
elevation: {}
---

# DESIGN.md — Agent Runtime design system

> **What this is (honest scope).** The **rules layer** of the design
> system — interaction principles + do's/don'ts derived from the M08.6
> IRL findings (`docs/M08.6-irl-findings.md`). It is **not** the visual
> layer. Colors, typography scale, spacing, component mockups are
> authored by **Claude Design** (the third brain — a browser product)
> and committed over the placeholder frontmatter above.
>
> **Lanes (per the three-brain model).** Orchestration seeds the *rules*
> (this file's prose — quality rules, like CLAUDE.md). Claude Design owns
> the *visual* (tokens + components + mockups). The Builder + runtime
> renderer build *to* this file. It evolves through **Stage D** design
> reviews at each UI-bearing milestone closeout.
>
> **When it's load-bearing.** Not for M8.7a (backend, no UI). First real
> need: M8.7b/c (the live-graph execution view) and the M8.6.7 Builder
> track. Run the Claude Design pass before the Builder UI work.

---

## Interaction principles (rules — from the IRL gaps)

These are the system-level rules every UI surface obeys. Each traces to a
finding the IRL walk surfaced.

1. **Every action gives feedback.** A click that does work produces a
   visible result — a toast, a status change, a state badge. Silent
   success is a defect. (IRL #1 key-save, #14 validate, #21 budget-save —
   three instances of the same no-feedback class.)
2. **State is visible, not assumed.** If a value is loaded/active (an API
   key from the keychain, a saved budget, the current tier), the UI shows
   it as loaded/active — not an empty field the user must guess at.
   (IRL #1, #19 tier-desync.)
3. **Progressive disclosure for dense surfaces.** Panels that are not in
   active use collapse / hide. No always-on horizontal bars consuming
   permanent space. (IRL #18 Settings panel.)
4. **Differentiate by type, visibly.** Distinct node/edge/artifact kinds
   are visually distinct (color + weight), with sensible defaults and, where
   useful, user control. (IRL #26 uniform edge styling.)
5. **Recoverability is a right.** Every constructive action has an
   inverse reachable from the UI — delete, clear, undo. No surface may
   trap the user into restarting the app to recover. (IRL #11/#12/#13.)
6. **No silent failure; no silent discard.** An error surfaces where the
   user is looking, in plain language; in-progress edits are not silently
   discarded on a state change without a warn/cancel path. (IRL #5/#15
   opaque errors, #16 JSON-edit discard.)
7. **Plain language over machine output.** User-facing errors translate
   the internal cause; raw serde/typify/enum strings never reach the
   user. (IRL #5/#15.)
8. **Labels tell the truth.** A control's label matches what it does in
   the current state. No "Promote to Promoted" when already Promoted.
   (IRL #20.)
9. **Contrast meets accessibility.** Text on any surface meets WCAG AA
   contrast; no relying on background-only styling that leaves text
   near-invisible. (IRL #3 import-panel contrast.)

## Do's and don'ts (concrete)

**Do**
- Confirm every save/apply with a transient, dismissible toast.
- Show loaded/active state inline (a green "active" affordance on a
  populated key field; the real current tier).
- Give canvas elements delete + the canvas a clear + the editor undo/redo.
- Color- and weight-differentiate Agent / Tool / Skill / Hook / HITL
  edges and nodes.
- Translate validation failures to per-node, plain-English messages.
- Collapse non-active panels behind a disclosure control.

**Don't**
- Don't let a click do work with zero visible feedback.
- Don't render an empty input for a value that is actually set.
- Don't surface raw `(root): data did not match any variant of untagged
  enum …` to a user.
- Don't discard in-progress edits on a tab switch without warn + cancel.
- Don't ship an always-visible large chrome bar with no collapse.
- Don't trap a user with no recovery path (no delete / no clear / no undo).

## Component-behavior rules (design-system level)

- **Modals** render above all chrome (portal + sufficient z-index),
  every button responds, content scrolls within a bounded height, and
  the label set is complete + untruncated. (IRL #23/#24 MCP modal.)
- **Panels** support progressive disclosure (collapse/expand); dense
  config panels default collapsed.
- **Badges** (validation, capability, source) carry a visible label or
  icon + meet contrast; a state change updates them live where the
  underlying state is live.
- **Toasts** confirm actions, carry plain-language text, auto-dismiss,
  and are non-blocking.

---

## Visual layer — TO BE AUTHORED BY CLAUDE DESIGN (placeholders)

> These sections are intentionally empty stubs. Claude Design reads the
> codebase + these rules and authors the visual system here. Until then,
> there is **no** committed palette/type scale (rule 11 — not claiming one
> exists).

### Colors
_Pending Claude Design. (Dark canvas baseline exists in `src/styles.css`;
not yet a tokenized, documented palette.)_

### Typography
_Pending Claude Design._

### Layout & spacing
_Pending Claude Design._

### Elevation & shapes
_Pending Claude Design._

### Components (visual specs + mockups)
_Pending Claude Design. The component-**behavior** rules above are the
contract the visual specs must satisfy._

---

## How this file evolves

- **Seeded** (this commit): the rules layer, from the IRL findings.
- **Visual layer**: the maintainer runs Claude Design (browser) with the
  orchestrator-authored brief; the output is committed over the
  placeholders.
- **Ongoing**: **Stage D** design reviews at each UI-bearing milestone
  closeout check the shipped UI against this file and update it as the
  system grows (the analog of retros updating CLAUDE.md).
