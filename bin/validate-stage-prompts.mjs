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
const ID_PATTERN = /^M\d{2}(\.\d+)?\.[A-Z]\d?(\.fix)?$/;

const KNOWN_ROOTS = new Set(['work_stage_prompt', 'closeout_stage_prompt']);

const COMMON_REQUIRED = [
  'context',
  'read_first',
  'scope_locks',
  'gates',
  'retrospective_requirements',
  'commit_protocol',
  'commit_message',
  'approval_surface',
];

const REQUIRED_BY_ROOT = {
  work_stage_prompt: [...COMMON_REQUIRED, 'deliverable'],
  closeout_stage_prompt: [
    ...COMMON_REQUIRED,
    'cumulative_reads',
    'deliverables',
    'gap_analysis_requirements',
    'append_only_verification',
    'three_artifact_review',
  ],
};

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

function validateBlock(block) {
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
        `(e.g. M01.A, M04.A2, M03.5.A, M04.B.fix)`,
    );
  }

  const idLabel = idAttr ? `id="${idAttr}"` : '(no id)';
  for (const required of REQUIRED_BY_ROOT[tag]) {
    if (!hasTag(block.xml, required)) {
      errors.push(`<${tag} ${idLabel}> missing required <${required}>`);
    }
  }

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
    const blocks = extractBlocks(content);
    totalBlocks += blocks.length;

    for (const block of blocks) {
      const { errors, skip } = validateBlock(block);
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
  console.error(
    `\nFAIL: ${totalErrors} error(s) across ${filesWithIssues.size} file(s).`,
  );
  process.exit(1);
}

main();
