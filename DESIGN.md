---
# DESIGN.md — design tokens (W3C DTCG-style). RULES LAYER + VISUAL LAYER.
# The visual layer below was authored by Claude Design over the M08.6 rules
# seed, per CLAUDE.md §4 rule 11. Token values are the committed "Light
# Instrument" system; mockups live in the Claude Design workbench export
# (Agent Runtime Workbench.html + Design Tokens.html). Light, cool-neutral
# surfaces · crisp dark text · one vivid electric-blue accent · IBM Plex Sans
# (UI) + IBM Plex Mono (code/JSON/IDs/token counts). All text pairings meet
# WCAG AA, including the five node-kind colors on the light canvas (rule 9).
status: visual-layer-authored
color:
  # ---- surfaces (cool blue-gray) ----
  app-bg:        { $value: "#e9eef6", $type: color }
  canvas-bg:     { $value: "#f1f4fa", $type: color }
  surface-0:     { $value: "#ffffff", $type: color }
  surface-1:     { $value: "#f6f8fc", $type: color }
  surface-2:     { $value: "#eef2f8", $type: color }
  surface-3:     { $value: "#e6ebf3", $type: color }
  # ---- lines ----
  border-subtle: { $value: "#e7ecf3", $type: color }
  border:        { $value: "#d6deea", $type: color }
  border-strong: { $value: "#b9c4d6", $type: color }
  # ---- text (AA on white) ----
  text-0:        { $value: "#14202f", $type: color }   # primary  ~15:1
  text-1:        { $value: "#3d4759", $type: color }   # secondary ~9:1
  text-2:        { $value: "#687486", $type: color }   # tertiary ~4.7:1
  text-3:        { $value: "#98a3b4", $type: color }   # non-essential only
  # ---- accent (electric blue) ----
  accent:          { $value: "#2563eb", $type: color }
  accent-hover:    { $value: "#1d4ed8", $type: color }
  accent-active:   { $value: "#1e40af", $type: color }
  accent-on-light: { $value: "#1d4ed8", $type: color } # AA text on white 5.9:1
  accent-bg:       { $value: "#eef4ff", $type: color }
  accent-bg-strong:{ $value: "#dbe7ff", $type: color }
  accent-border:   { $value: "#bcd2fb", $type: color }
  # ---- semantic status ----
  ok:    { $value: "#16a34a", $type: color }
  ok-text:   { $value: "#15803d", $type: color }
  ok-bg:     { $value: "#e7f7ed", $type: color }
  warn:  { $value: "#d97706", $type: color }
  warn-text: { $value: "#b45309", $type: color }
  warn-bg:   { $value: "#fdf2dc", $type: color }
  error: { $value: "#dc2626", $type: color }
  error-text:{ $value: "#b91c1c", $type: color }
  error-bg:  { $value: "#fdeaea", $type: color }
  info:  { $value: "#2563eb", $type: color }
  # ---- five node kinds (color / AA text on white / tint) ----
  kind-agent: { $value: "#2563eb", $type: color }
  kind-agent-text: { $value: "#1e40af", $type: color }   # 8.72:1
  kind-agent-bg:   { $value: "#eef4ff", $type: color }
  kind-tool:  { $value: "#0d9488", $type: color }
  kind-tool-text:  { $value: "#0f766e", $type: color }   # 5.47:1
  kind-tool-bg:    { $value: "#e6faf6", $type: color }
  kind-skill: { $value: "#7c3aed", $type: color }
  kind-skill-text: { $value: "#6d28d9", $type: color }   # 7.10:1
  kind-skill-bg:   { $value: "#f4f0ff", $type: color }
  kind-hook:  { $value: "#db2777", $type: color }
  kind-hook-text:  { $value: "#be185d", $type: color }   # 6.04:1
  kind-hook-bg:    { $value: "#fdeef6", $type: color }
  kind-hitl:  { $value: "#d97706", $type: color }
  kind-hitl-text:  { $value: "#b45309", $type: color }   # 5.02:1
  kind-hitl-bg:    { $value: "#fdf2dc", $type: color }
  # ---- runtime-only kinds carried from the baseline ----
  kind-gap:   { $value: "#ea580c", $type: color }
  kind-gap-text: { $value: "#c2410c", $type: color }
  kind-mcp:   { $value: "#475569", $type: color }
  # ---- runtime status (orthogonal to kind) ----
  st-idle:     { $value: "#98a3b4", $type: color }
  st-active:   { $value: "#2563eb", $type: color }
  st-complete: { $value: "#16a34a", $type: color }
  st-error:    { $value: "#dc2626", $type: color }
  st-gap:      { $value: "#ea580c", $type: color }
  st-blocked:  { $value: "#d97706", $type: color }
typography:
  font-sans: { $value: "'IBM Plex Sans', system-ui, -apple-system, 'Segoe UI', sans-serif", $type: fontFamily }
  font-mono: { $value: "'IBM Plex Mono', 'SF Mono', 'Cascadia Mono', Consolas, monospace", $type: fontFamily }
  display: { $value: { fontSize: "28px", lineHeight: "34px", fontWeight: 600, letterSpacing: "-0.01em" }, $type: typography }
  title:   { $value: { fontSize: "20px", lineHeight: "27px", fontWeight: 600 }, $type: typography }
  h2:      { $value: { fontSize: "16px", lineHeight: "22px", fontWeight: 600 }, $type: typography }
  body:    { $value: { fontSize: "14px", lineHeight: "20px", fontWeight: 400 }, $type: typography }
  small:   { $value: { fontSize: "12.5px", lineHeight: "17px", fontWeight: 400 }, $type: typography }
  label:   { $value: { fontSize: "11px", lineHeight: "14px", fontWeight: 600, letterSpacing: "0.07em", textTransform: "uppercase" }, $type: typography }
  mono:    { $value: { fontSize: "12.5px", lineHeight: "18px", fontWeight: 400 }, $type: typography }
  micro:   { $value: { fontSize: "11px", lineHeight: "15px", fontWeight: 400 }, $type: typography }
spacing:
  s1:  { $value: "4px",  $type: dimension }
  s2:  { $value: "8px",  $type: dimension }
  s3:  { $value: "12px", $type: dimension }
  s4:  { $value: "16px", $type: dimension }
  s5:  { $value: "20px", $type: dimension }
  s6:  { $value: "24px", $type: dimension }
  s8:  { $value: "32px", $type: dimension }
  s10: { $value: "40px", $type: dimension }
  s12: { $value: "48px", $type: dimension }
  radius-xs:   { $value: "3px",   $type: dimension }
  radius-sm:   { $value: "5px",   $type: dimension }
  radius-md:   { $value: "8px",   $type: dimension }
  radius-lg:   { $value: "12px",  $type: dimension }
  radius-pill: { $value: "999px", $type: dimension }
elevation:
  e0: { $value: "none", $type: shadow }
  e1: { $value: "0 1px 2px rgba(16,32,46,.06), 0 1px 1px rgba(16,32,46,.04)", $type: shadow }
  e2: { $value: "0 4px 14px rgba(16,32,46,.09), 0 1px 3px rgba(16,32,46,.06)", $type: shadow }
  e3: { $value: "0 14px 38px rgba(16,32,46,.16), 0 4px 10px rgba(16,32,46,.08)", $type: shadow }
  focus-ring: { $value: "0 0 0 3px rgba(37,99,235,.32)", $type: shadow }
---

# DESIGN.md — Agent Runtime design system

> **What this is.** Both layers of the design system. The **rules layer**
> (interaction principles + component-behavior rules, derived from the
> M08.6 IRL findings) is the contract. The **visual layer** (tokens above
> + the Colors / Typography / Layout / Elevation / Components sections
> below) was authored by **Claude Design** over that contract and every
> spec satisfies it. Tokens ship as `src/styles` custom properties; the
> Builder + runtime renderer consume them.
>
> **Reference mockups.** `Agent Runtime Workbench.html` (interactive — the
> live-graph execution view, Builder, Tester, Settings) and
> `Design Tokens.html` (the full token reference incl. the node-kind AA
> proof). Both are committed in `docs/design/workbench-mockup/`.
>
> **How it evolves.** **Stage D** design reviews at each UI-bearing
> milestone closeout check the shipped UI against this file and update it.

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
3. **Progressive disclosure for dense surfaces.** Panels not in active
   use collapse / hide. No always-on horizontal bars consuming permanent
   space. (IRL #18 Settings panel.)
4. **Differentiate by type, visibly.** Distinct node/edge/artifact kinds
   are visually distinct (color + weight), with sensible defaults and,
   where useful, user control. (IRL #26 uniform edge styling.)
5. **Recoverability is a right.** Every constructive action has an inverse
   reachable from the UI — delete, clear, undo. No surface may trap the
   user into restarting the app to recover. (IRL #11/#12/#13.)
6. **No silent failure; no silent discard.** An error surfaces where the
   user is looking, in plain language; in-progress edits are not silently
   discarded on a state change without a warn/cancel path. (IRL #5/#15
   opaque errors, #16 JSON-edit discard.)
7. **Plain language over machine output.** User-facing errors translate
   the internal cause; raw serde/typify/enum strings never reach the user.
8. **Labels tell the truth.** A control's label matches what it does in
   the current state. No "Promote to Promoted" when already Promoted.
   (IRL #20.)
9. **Contrast meets accessibility.** Text on any surface meets WCAG AA contrast;
   no relying on background-only styling that leaves text near-invisible.
   (IRL #3 import-panel contrast.)

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

- **Modals** render above all chrome (portal + sufficient z-index), every
  button responds, content scrolls within a bounded height, and the label
  set is complete + untruncated. (IRL #23/#24 MCP modal.)
- **Panels** support progressive disclosure (collapse/expand); dense
  config panels default collapsed.
- **Badges** (validation, capability, source) carry a visible label or
  icon + meet contrast; a state change updates them live where the
  underlying state is live.
- **Toasts** confirm actions, carry plain-language text, auto-dismiss,
  and are non-blocking.

---

## Colors

The palette is **light and cool-neutral** with **one vivid accent**
(electric blue). It runs on three planes:

- **Surfaces** — `app-bg` (workspace backdrop) → `canvas-bg` (the dotted
  graph field) → `surface-0` (cards, panels, nodes) → `surface-1/2/3`
  (hover, wells, tracks). Hairline `border` / `border-strong` lines do the
  structural work; elevation is reserved for things that truly float.
- **Text** — `text-0`→`text-3`. `text-0`–`text-2` are AA-legible body
  text (≥4.5:1 on white); `text-3` is for non-essential hints only.
- **Accent** — `accent` for primary actions, selection, the live
  "executing" state, and the playhead. `accent-on-light` (#1d4ed8) is the
  AA-safe text/icon shade on white; never paint accent text with the raw
  `accent`. One accent only — no secondary brand hue.

**Semantic status** (`ok` / `warn` / `error` / `info`) each ship a fill,
an AA `*-text` shade, and a soft `*-bg` tint for badges, toasts, and the
error surface.

### Node kinds — two-axis model (rule 4)

The five framework node kinds are distinguished on a **kind axis** and a
separate **status axis** so the two signals never collide:

| Kind | color (bar + glyph) | text on white | ratio | tint |
|------|---------------------|---------------|-------|------|
| Agent | `#2563eb` | `#1e40af` | 8.72:1 | `#eef4ff` |
| Tool  | `#0d9488` | `#0f766e` | 5.47:1 | `#e6faf6` |
| Skill | `#7c3aed` | `#6d28d9` | 7.10:1 | `#f4f0ff` |
| Hook  | `#db2777` | `#be185d` | 6.04:1 | `#fdeef6` |
| HITL  | `#d97706` | `#b45309` | 5.02:1 | `#fdf2dc` |

- **Kind** is carried by a 4px left accent bar + a monospace glyph chip
  (`A` / `T` / `S` / `H` / `‖`) + the label color. All five pass AA on
  both white and their own tint (proof: `Design Tokens.html`).
- **Status** is carried by the node ring/glow + a status dot, using the
  `st-*` family: idle (gray), active (accent, animated ring + flowing
  edge), complete (green), error (red), gap (orange, pulsing), blocked
  (amber). So a *running Tool* reads as Tool (teal bar) **and** running
  (blue pulse) at once.
- Runtime-only kinds Gap (`#ea580c`) and MCP (`#475569`) carry forward
  from the dark baseline.

### Edges (rule 4 / IRL #26)

Edges differentiate by kind: **agent** solid neutral 2px · **tool** teal
1.75px (animated dashed flow while a call is in flight) · **skill** violet
1.75px dashed (loaded into context — no flow animation) · **hook** pink
1.5px · **HITL** amber 2.5px. A colored port dot terminates each edge at
the child; it fills in once that node has run.

## Typography

Two families: **IBM Plex Sans** for all UI text and **IBM Plex Mono** for
the instrument register — code, JSON, IDs, tool/server names, token counts
and dollar amounts. The mono is doing semantic work, not decoration:
anything machine-generated or copy-exact is set in mono.

Scale (see `typography` tokens): `display 28/34` · `title 20/27` ·
`h2 16/22` · `body 14/20` · `small 12.5/17` · `label 11/14` (uppercase,
0.07em tracking, `text-2`) · `mono 12.5/18` · `micro 11/15`. Tabular
numerals (`font-variant-numeric: tabular-nums`) on all counts, budgets and
durations so columns and live-updating values don't jitter.

## Layout & spacing

- **4px grid** (`s1`…`s12`). Component padding and gaps snap to it; prefer
  flex/grid `gap` over per-element margins.
- **Shell** — a 52px top chrome (brand · tab nav · tier + key state), then
  a three-pane workspace: a 232px left rail (palette / task+results), a
  fluid center (canvas + transport), and a 360px right rail (Inspector /
  Output). The canvas is a dotted field (`canvas-bg`, 22px radial grid).
- **Density** is tokenized (`comfortable` / `regular` / `compact`) via
  `--pad-panel` and `--row-h`, exposed as a user setting.

## Elevation & shapes

- **Radii** are restrained: `xs 3` (chips/badges) · `sm 5` (buttons/inputs)
  · `md 8` (cards/nodes) · `lg 12` (panels/modals) · `pill 999`.
- **Elevation** is subtle and purposeful: `e1` for resting cards/nodes,
  `e2` for popovers and the run banner, `e3` for modals; structure
  otherwise comes from hairline borders, not shadow. `focus-ring` is the
  3px accent halo on every focusable control.

## Components (visual specs + mockups)

All specs satisfy the component-behavior rules above. Live mockups:
`Agent Runtime Workbench.html`.

- **Canvas + five node kinds** — node card: `surface-0`, `md` radius, `e1`,
  4px kind bar, glyph chip, name (`body`/600), mono sub-line, status dot.
  Tool nodes reveal inline `in` / `out` / duration while active/complete.
  Selected = accent ring. (Builder + Tester tabs.)
- **Inspector (right rail)** — node identity header (kind glyph + status
  badge), a `kv` grid (mono values), and kind-specific sections (tool
  last-invocation I/O; agent capabilities + "narrowed on spawn" note).
- **Live-graph execution view** *(centerpiece)* — budget bar + dotted
  canvas + run banner + transport (play/scrub/speed, with HITL/incident/
  gap ticks on the timeline). Nodes light up as events arrive; edges flow
  on active tool calls; the **agent reply text streams into the Output
  rail keyed to the selected node** (fixes the #1 "reply trapped in debug
  log" gap). A tool gap flips the node to `gap` + a banner suspends the
  session; a capability violation flips a node to `error` with a blocked
  badge + a toast. (Tester tab — press play.)
- **Tester results** — metric cards (Result / Verify / Tokens / Spend,
  mono tabular values) + a capability-gaps list.
- **Badges** — pill, label-or-icon, AA tint families: validation
  (`ok`/`warn`/`error`), capability/tier, and mono `source` chips
  (`builtin` / `mcp` / `generated`). Update live with state.
- **Toasts** — bottom-right stack, left status border, icon + title +
  plain-language message, auto-dismiss (~4.2s), dismissible, non-blocking.
  Fires on every save/apply/run/suspension (rule 1).
- **Modals** — `overlay` at z-index 300 over a blurred scrim, `lg` radius,
  `e3`, bounded `max-height: 86vh` with internal scroll, Esc / scrim-click
  / × to close, complete + untruncated button labels (fixes IRL #24).
- **Validation / error surface** — per-node card: plain-English cause +
  `→` actionable fix + a "Show raw error" disclosure holding the literal
  serde/typify string. Validate/Save give toast feedback; an invalid node
  blocks Save with a reason (rules 1/6/7; IRL #5/#15/#17).
- **Panels** — disclosure header (rotating chevron) + bounded body; dense
  config panels default collapsed (rule 3).

## How this file evolves

- **Seeded** (M08.6): the rules layer, from the IRL findings.
- **Visual layer** (this commit): the committed "Light Instrument" token
  system + component specs/mockups, authored by Claude Design.
- **Ongoing**: **Stage D** design reviews at each UI-bearing milestone
  closeout reconcile shipped UI with this file (the analog of retros
  updating CLAUDE.md).
