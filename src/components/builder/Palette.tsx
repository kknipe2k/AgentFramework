import { useEffect, useMemo, useState } from 'react';
import { listInstalledArtifacts, type InstalledArtifact } from '../../lib/ipc';
import type { BuilderNodeKind } from '../../lib/builderStore';

// M08.C — the Builder Palette (spec Phase 9): five tabs (Tools / Skills
// / Agents / HITL / Hooks), a per-tab name filter, every item a native
// drag source carrying the application/x-builder-node payload D1's drop
// handler reads. Tools/Skills/Agents list built-ins + whatever
// list_installed_artifacts returns (Stage B's skills.lock reader); HITL
// lists the §6a trigger types; Hooks lists the §4a firing points. M09
// later adds the "Generate…" buttons — M08 shows installed / built-in
// artifacts only.

const PALETTE_TABS = ['tools', 'skills', 'agents', 'hitl', 'hooks'] as const;
type PaletteTab = (typeof PALETTE_TABS)[number];

/** Runtime built-in tools — always available as drag sources. */
const BUILTIN_TOOLS = ['Read', 'Write', 'Bash'] as const;

/** The §6a HITL trigger types (the framework.json `hitl_policy` keys). */
const HITL_TRIGGERS = [
  'on_gap',
  'on_risky_tool',
  'on_dont_touch_edit',
  'on_failure_threshold',
  'on_capability_violation',
  'on_budget_threshold',
  'on_plan_approval',
  'per_task',
  'per_epic',
] as const;

/** The §4a hook firing points (the framework.json `hooks` keys). */
const HOOK_POINTS = [
  'pre_task',
  'post_task',
  'pre_file_edit',
  'post_file_edit',
  'pre_commit',
  'pre_agent_spawn',
  'session_end',
] as const;

/** Per-tab noun for the filter placeholder + empty state. */
const TAB_NOUN: Record<PaletteTab, string> = {
  tools: 'tools',
  skills: 'skills',
  agents: 'agents',
  hitl: 'HITL triggers',
  hooks: 'hook points',
};

interface PaletteItem {
  /** The builder-node kind D1's addNode instantiates. */
  kind: BuilderNodeKind;
  /** The artifact ref / trigger name — addNode's second arg + the drag
   *  payload; also the item's data-testid suffix. */
  ref: string;
  label: string;
}

/** Build the (unfiltered) item list for `tab` — built-ins + installed. */
function paletteItemsForTab(tab: PaletteTab, installed: InstalledArtifact[]): PaletteItem[] {
  switch (tab) {
    case 'tools':
      return [
        ...BUILTIN_TOOLS.map((t) => ({ kind: 'tool' as const, ref: t, label: t })),
        ...installed
          .filter((a) => a.kind === 'tool')
          .map((a) => ({ kind: 'tool' as const, ref: a.key, label: a.key })),
      ];
    case 'skills':
      return installed
        .filter((a) => a.kind === 'skill')
        .map((a) => ({ kind: 'skill' as const, ref: a.key, label: a.key }));
    case 'agents':
      return installed
        .filter((a) => a.kind === 'agent')
        .map((a) => ({ kind: 'agent' as const, ref: a.key, label: a.key }));
    case 'hitl':
      return HITL_TRIGGERS.map((t) => ({ kind: 'hitl' as const, ref: t, label: t }));
    case 'hooks':
      return HOOK_POINTS.map((h) => ({ kind: 'hook' as const, ref: h, label: h }));
  }
}

export function Palette(): JSX.Element {
  const [tab, setTab] = useState<PaletteTab>('tools');
  const [filter, setFilter] = useState('');
  const [installed, setInstalled] = useState<InstalledArtifact[]>([]);

  useEffect(() => {
    // Installed artifacts feed the Tools/Skills/Agents tabs — the same
    // skills.lock read the ImportPanel uses on mount (M07-IRL #6). An
    // absent lock resolves to []; a backend error logs and the list
    // stays empty (built-ins still render).
    void listInstalledArtifacts()
      .then(setInstalled)
      .catch((e) => console.error('list_installed_artifacts error:', e));
  }, []);

  const items = useMemo(() => paletteItemsForTab(tab, installed), [tab, installed]);
  const needle = filter.trim().toLowerCase();
  const shown =
    needle.length === 0 ? items : items.filter((it) => it.label.toLowerCase().includes(needle));

  return (
    <div className="builder-palette" data-testid="builder-palette">
      <nav className="builder-palette__tabs" role="tablist" aria-label="Palette categories">
        {PALETTE_TABS.map((t) => (
          <button
            key={t}
            type="button"
            role="tab"
            aria-selected={t === tab}
            className={`builder-palette__tab${t === tab ? ' builder-palette__tab--active' : ''}`}
            data-testid={`palette-tab-${t}`}
            onClick={() => setTab(t)}
          >
            {t}
          </button>
        ))}
      </nav>
      <input
        className="builder-palette__filter"
        data-testid="palette-filter"
        placeholder={`Filter ${TAB_NOUN[tab]}…`}
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
      />
      {shown.length === 0 ? (
        <p className="builder-palette__empty" data-testid="palette-empty">
          No {TAB_NOUN[tab]}.
        </p>
      ) : (
        <ul className="builder-palette__list">
          {shown.map((it) => (
            <li
              key={`${it.kind}:${it.ref}`}
              className="builder-palette__item"
              data-testid={`palette-item-${it.ref}`}
              draggable
              onDragStart={(e) => {
                // The C->D1 contract — D1's onDrop reads this MIME type
                // + JSON payload via screenToFlowPosition.
                e.dataTransfer.setData(
                  'application/x-builder-node',
                  JSON.stringify({ kind: it.kind, ref: it.ref }),
                );
                e.dataTransfer.effectAllowed = 'copy';
              }}
            >
              {it.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
