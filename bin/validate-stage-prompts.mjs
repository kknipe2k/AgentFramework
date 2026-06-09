#!/usr/bin/env node
/**
 * Stage-prompt schema validator.
 *
 * Extracts fenced ```xml blocks from `docs/build-prompts/M*-*.md`, identifies
 * each block's root element by regex, and asserts protocol compliance per
 * `STAGE-PROMPT-PROTOCOL.md`:
 *
 *   - exactly one root element per block (regex-matched at the block head)
 *   - root must be `<work_stage_prompt>` or `<closeout_stage_prompt>` (other
 *     roots are treated as documentation example snippets and silently skipped)
 *   - root carries an `id` attribute matching `M\d{2}(\.\d+)?\.<X>[<d>][.fix]`
 *     (e.g. `M01.A`, `M04.A2`, `M03.5.A`, `M04.B.fix`)
 *   - required common tags present (`<context>`, `<read_first>`, `<scope_locks>`,
 *     `<gates>`, `<retrospective_requirements>`, `<commit_protocol>`,
 *     `<commit_message>`, `<approval_surface>`)
 *   - work-stage requires `<deliverable>`
 *   - closeout requires `<cumulative_reads>`, `<deliverables>`,
 *     `<gap_analysis_requirements>`, `<append_only_verification>`,
 *     `<three_artifact_review>`
 *
 * Regex-based tag detection rather than strict XML parse: protocol-conforming
 * prompts can carry prose inside `<context>` etc. that includes unescaped
 * `<other_tag>` references (documentation pattern). Strict XML would force
 * `&lt;` escaping throughout phase docs, which the protocol doesn't require.
 *
 * Exits 0 on all-clean, non-zero with per-block error list on failure.
 * Phase docs with zero stage-prompt blocks (M01, M02 — predate protocol) are
 * silently skipped.
 *
 * Lands per the "build the validator before adding more schema" rule. Until a
 * validator ships, the protocol is decorative; this is the foundation for
 * adding `<verifier_stage_prompt>` (Stage V) as a real-enforced schema variant.
 */

import { readFileSync, readdirSync, existsSync, statSync } from 'node:fs';
import { join } from 'node:path';

const PHASE_DOC_DIR = 'docs/build-prompts';
// Allow zero or more `.<minor>` components to support X.5.5 nested fix
// cycles (the M08.5.5 MCP-resilience cycle on top of M08.5 — first repo
// case 2026-05-23). Pattern was `(\.\d+)?` (zero-or-one); now `(\.\d+)*`
// (zero-or-more). Backward-compatible: M01.A, M03.5.A, M04.B.fix all
// still match.
const ID_PATTERN = /^M\d{2}(\.\d+)*\.[A-Z]\d?(\.fix)?$/;

const KNOWN_ROOTS = new Set([
  'work_stage_prompt',
  'closeout_stage_prompt',
  'verifier_stage_prompt',
]);

const COMMON_REQUIRED = [
  'context',
  'read_first',
  'gates',
  'retrospective_requirements',
  'commit_protocol',
  'commit_message',
  'approval_surface',
];

const REQUIRED_BY_ROOT = {
  work_stage_prompt: [...COMMON_REQUIRED, 'scope_locks', 'deliverable'],
  closeout_stage_prompt: [
    ...COMMON_REQUIRED,
    'scope_locks',
    'cumulative_reads',
    'deliverables',
    'gap_analysis_requirements',
    'append_only_verification',
    'three_artifact_review',
  ],
  // Verifier (v1.5+, see STAGE-PROMPT-PROTOCOL.md §14): replaces
  // `<deliverable>` with `<scope_to_verify>` + the four pass declarations.
  // Omits `<scope_locks>` (V's role is verification, not constraint).
  verifier_stage_prompt: [
    ...COMMON_REQUIRED,
    'scope_to_verify',
    'verification_passes',
    'findings_format',
    'merge_gate',
  ],
};

// Lean-validator pass-through set (STAGE-PROMPT-PROTOCOL.md §11). Optional
// slots introduced v1.3 → v1.8 are NOT allowlisted here: the regex parser is
// permissive for non-root tags, so they pass through structurally without
// enforcement. The v1.8 additions recognized as pass-through (no body
// cross-check until v1.9+ per the §15 v1.8 changelog §G maintainer decision):
//   <construction_reachability_check>  (children: <wire>)
//   <wire_signature_audit>             (children: <wrapper>)
//   <wire_trace_vs_adr_reconcile>      (children: <trace>)
//   <phase_doc_inventory_audit shape=> (optional attr on type="store_slot")
// This comment is the v1.8 "registration" — documentation, not logic.
// Version-gated enforcement: <simplify_pass> (v1.6, closeout) + the v1.11
// <tdd_discipline strict> → two-commit <execution_steps> cross-check (below).

// Required tags that apply only when the phase doc's `**Protocol version:**`
// banner is at or above the listed version. M05 and earlier closeouts predate
// `<simplify_pass>`; v1.6 makes it required for M06+ closeouts only. See
// STAGE-PROMPT-PROTOCOL.md §15 v1.6 changelog item #18.
const VERSION_GATED_REQUIRED = {
  closeout_stage_prompt: [{ tag: 'simplify_pass', minVersion: 106 }], // 106 = v1.6 (encoded)
};

// v1.11: the v1.7 `<tdd_discipline strict="true">` → two-commit
// `<execution_steps>` coupling, PROMOTED from authoring-discipline (lean
// pass-through) to an enforced cross-check — the promotion the v1.7 changelog
// item #4 deferred ("to v1.8+ once 2+ milestones show clean signal"); M06–M08.8
// supplied the signal, the M08.9.B red-surface miss the trigger. Gated at v1.11
// (encoded 111) so M08.9 (v1.8) + all prior docs are grandfathered. When a work
// stage declares strict TDD, its `<execution_steps>` MUST carry the explicit
// two-commit step sequence — the collapsed single-surface form is what let
// M08.9.B skip the red-phase surface.
const STRICT_TDD_MIN_VERSION = 111;
const STRICT_TDD_REQUIRED_STEPS = [
  'red_phase_commit',
  'surface_for_red_approval',
  'green_phase_commit',
  'surface_for_final_approval',
];

// Parses the phase doc's `**Protocol version:** vX.Y` banner into a numeric
// value (e.g. 1.6) for comparison against `VERSION_GATED_REQUIRED.minVersion`.
// Returns null if no banner is present — caller treats null as "unconstrained"
// (no version-gated requirements apply).
function detectProtocolVersion(content) {
  const match = content.match(/\*\*Protocol version:\*\*\s*v(\d+)\.(\d+)/);
  if (!match) return null;
  const major = Number.parseInt(match[1], 10);
  const minor = Number.parseInt(match[2], 10);
  // Encode as major*100 + minor so two-digit minors order correctly
  // (v1.10 → 110, v1.11 → 111). The earlier `major + minor / 10` collapsed
  // v1.10 → 2.0 / v1.11 → 2.1, breaking any v1.10+ version gate.
  return major * 100 + minor;
}

// Render an encoded version (major*100 + minor) back to `vM.m` for messages.
function formatVersion(encoded) {
  return `v${Math.floor(encoded / 100)}.${encoded % 100}`;
}

function extractBlocks(content) {
  const blocks = [];
  const lines = content.split('\n');
  let inBlock = false;
  let current = [];
  let startLine = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line === '```xml' && !inBlock) {
      inBlock = true;
      current = [];
      startLine = i + 1;
      continue;
    }
    if (line === '```' && inBlock) {
      blocks.push({ xml: current.join('\n'), startLine });
      inBlock = false;
      continue;
    }
    if (inBlock) current.push(line);
  }
  return blocks;
}

function detectRoot(xml) {
  const match = xml.match(/^\s*<([A-Za-z_][\w-]*)([^>]*?)\/?>/);
  if (!match) return { tag: null, idAttr: null };
  const tag = match[1];
  const attrs = match[2] || '';
  const idMatch = attrs.match(/\bid\s*=\s*["']([^"']+)["']/);
  return { tag, idAttr: idMatch ? idMatch[1] : null };
}

function hasTag(xml, tag) {
  const re = new RegExp(`<${tag}(\\s+[^>]*?)?(\\s*/>|>)`, 'i');
  return re.test(xml);
}

// Bias-guard rule for the v1.5 verifier schema (STAGE-PROMPT-PROTOCOL.md §14):
// V's <read_first> must NOT load prior retros, milestone summaries, or
// gap-analysis entries. Structural enforcement of the "fresh-context" mandate.
const VERIFIER_FORBIDDEN_READ_FIRST_PATTERNS = [
  { regex: /retrospectives\/M\d/i, label: 'per-stage retrospective' },
  { regex: /-summary\.md/i, label: 'milestone summary' },
  { regex: /gap-analysis\.md/i, label: 'gap-analysis ledger' },
];

function checkVerifierBiasGuard(xml, idLabel) {
  const errors = [];
  const readFirstMatch = xml.match(/<read_first>([\s\S]*?)<\/read_first>/i);
  if (!readFirstMatch) return errors; // already flagged by missing required tag
  const readFirstContent = readFirstMatch[1];
  for (const { regex, label } of VERIFIER_FORBIDDEN_READ_FIRST_PATTERNS) {
    if (regex.test(readFirstContent)) {
      errors.push(
        `<verifier_stage_prompt ${idLabel}> bias guard: <read_first> references ${label} ` +
          `(forbidden per STAGE-PROMPT-PROTOCOL.md §14 — verifier must run with fresh context)`,
      );
    }
  }
  return errors;
}

// v1.11 cross-check (STAGE-PROMPT-PROTOCOL.md §7/§11/§13): a work stage that
// declares `<tdd_discipline strict="true">` MUST carry the explicit two-commit
// `<execution_steps>` sequence. Gated at v1.11 (STRICT_TDD_MIN_VERSION) so all
// prior phase docs are grandfathered.
function checkStrictTddTwoCommit(xml, tag, idLabel, protocolVersion) {
  if (tag !== 'work_stage_prompt') return [];
  if (protocolVersion === null || protocolVersion < STRICT_TDD_MIN_VERSION) return [];
  if (!/<tdd_discipline\b[^>]*\bstrict\s*=\s*["']true["']/i.test(xml)) return [];
  const stepsMatch = xml.match(/<execution_steps>([\s\S]*?)<\/execution_steps>/i);
  const stepsBody = stepsMatch ? stepsMatch[1] : '';
  const missing = STRICT_TDD_REQUIRED_STEPS.filter((step) => !stepsBody.includes(step));
  if (missing.length === 0) return [];
  return [
    `<work_stage_prompt ${idLabel}> declares <tdd_discipline strict="true"> but its ` +
      `<execution_steps> omits the two-commit step(s): ${missing.join(', ')}. Strict TDD ` +
      `requires the explicit red-gate sequence (red_phase_commit → surface_for_red_approval → ` +
      `green_phase_commit → surface_for_final_approval) — the collapsed single-surface form is ` +
      `what let M08.9.B skip the red-phase surface (STAGE-PROMPT-PROTOCOL.md §7/§13, enforced v1.11+).`,
  ];
}

function validateBlock(block, protocolVersion) {
  const errors = [];
  const { tag, idAttr } = detectRoot(block.xml);

  if (!tag) {
    errors.push('no root element detected at block head');
    return { errors, skip: false };
  }

  if (!KNOWN_ROOTS.has(tag)) {
    return { errors: [], skip: true };
  }

  if (!idAttr) {
    errors.push(`<${tag}> missing required \`id\` attribute`);
  } else if (!ID_PATTERN.test(idAttr)) {
    errors.push(
      `<${tag} id="${idAttr}"> id does not match M[NN][.<minor>].<X>[<d>][.fix] ` +
        `(e.g. M01.A, M04.A2, M03.5.A, M04.B.fix, M05.V)`,
    );
  }

  const idLabel = idAttr ? `id="${idAttr}"` : '(no id)';
  for (const required of REQUIRED_BY_ROOT[tag]) {
    if (!hasTag(block.xml, required)) {
      errors.push(`<${tag} ${idLabel}> missing required <${required}>`);
    }
  }

  const versionGated = VERSION_GATED_REQUIRED[tag] || [];
  for (const { tag: requiredTag, minVersion } of versionGated) {
    if (protocolVersion === null || protocolVersion < minVersion) continue;
    if (!hasTag(block.xml, requiredTag)) {
      errors.push(
        `<${tag} ${idLabel}> missing required <${requiredTag}> ` +
          `(phase doc Protocol version ${formatVersion(protocolVersion)} ≥ ${formatVersion(minVersion)})`,
      );
    }
  }

  if (tag === 'verifier_stage_prompt') {
    errors.push(...checkVerifierBiasGuard(block.xml, idLabel));
  }

  errors.push(...checkStrictTddTwoCommit(block.xml, tag, idLabel, protocolVersion));

  return { errors, skip: false };
}

function main() {
  if (!existsSync(PHASE_DOC_DIR) || !statSync(PHASE_DOC_DIR).isDirectory()) {
    console.error(`error: ${PHASE_DOC_DIR} not found`);
    process.exit(2);
  }

  const phaseDocs = readdirSync(PHASE_DOC_DIR)
    .filter((f) => /^M\d.*\.md$/.test(f))
    .sort();

  let totalBlocks = 0;
  let stagePromptBlocks = 0;
  let totalErrors = 0;
  const filesWithIssues = new Set();

  for (const filename of phaseDocs) {
    const path = join(PHASE_DOC_DIR, filename);
    const content = readFileSync(path, 'utf8');
    const protocolVersion = detectProtocolVersion(content);
    const blocks = extractBlocks(content);
    totalBlocks += blocks.length;

    for (const block of blocks) {
      const { errors, skip } = validateBlock(block, protocolVersion);
      if (skip) continue;
      stagePromptBlocks++;

      if (errors.length > 0) {
        filesWithIssues.add(path);
        console.error(`\n${path}:${block.startLine}`);
        for (const err of errors) console.error(`  - ${err}`);
        totalErrors += errors.length;
      }
    }
  }

  console.log(
    `\nstage-prompt validator — scanned ${stagePromptBlocks} stage-prompt block(s) ` +
      `(${totalBlocks} xml block(s) total) across ${phaseDocs.length} phase doc(s).`,
  );

  if (totalErrors === 0) {
    console.log('PASS: all stage-prompt blocks conform to STAGE-PROMPT-PROTOCOL.md.');
    process.exit(0);
  }
  console.error(`\nFAIL: ${totalErrors} error(s) across ${filesWithIssues.size} file(s).`);
  process.exit(1);
}

main();
