import { useEffect, useMemo, useState } from 'react';
import { nextAgentRef, useBuilderStore, type BuilderNodeKind } from '../../lib/builderStore';
import {
  listInstalledArtifacts,
  mcpListServers,
  mcpListServerTools,
  type InstalledArtifact,
} from '../../lib/ipc';
import type { Framework } from '../../types/framework';

// M08.C — the Builder Palette (spec Phase 9): five tabs (Tools / Skills
// / Agents / HITL / Hooks), a per-tab name filter, every item a native
// drag source carrying the application/x-builder-node payload D1's drop
// handler reads. Tools/Skills/Agents list built-ins + whatever
// list_installed_artifacts returns (Stage B's skills.lock reader); HITL
// lists the §6a trigger types; Hooks lists the §4a firing points.
//
// M08.6.E — adds a THIRD item source: the currently-loaded framework
// (builderStore.framework). A loaded framework's resolved agents /
// tools / skills (Stage B's loader inlines `{id,path}` agents; Stage D
// applies the layout) become drag-source Palette items in the matching
// tab, de-duplicated by (kind, ref) against the built-ins + installed
// sets. Closes the IRL "defined agents not shareable" observation: a
// framework's own artifacts are first-class reusable canvas nodes per
// ADR-0022. Each item carries `data-source` so the user can tell where
// it came from; the drag payload remains the uniform
// `application/x-builder-node` contract.

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

/**
 * Where a Palette item comes from. Drives the `data-source` attribute
 * so the user can see at a glance whether an item is a runtime built-in
 * (always available), an installed artifact from `skills.lock`, a
 * resolved artifact in the open framework (M08.6.E), or a tool exposed by
 * a connected MCP server (M09.C). HITL trigger types + hook firing points
 * are runtime primitives — also `'builtin'`.
 */
type PaletteItemSource = 'builtin' | 'installed' | 'framework' | 'mcp';

interface PaletteItem {
  /** The builder-node kind D1's addNode instantiates. */
  kind: BuilderNodeKind;
  /** The artifact ref / trigger name — addNode's second arg + the drag
   *  payload; also the item's data-testid suffix. */
  ref: string;
  label: string;
  /** Visible-and-testable origin marker (M08.6.E). */
  source: PaletteItemSource;
}

/** Pull the loaded framework's resolved artifacts as Palette items for
 *  one of the artifact tabs. Agents key on `id`; tools / skills key on
 *  `name`. Stage B's loader inlines `{id,path}` agents — `entry.id`
 *  works for both shapes (the {id,path} ref form carries `id` too). */
function frameworkItemsForKind(
  framework: Framework,
  kind: 'agent' | 'tool' | 'skill',
): PaletteItem[] {
  switch (kind) {
    case 'agent':
      return framework.agents.map((entry) => ({
        kind: 'agent' as const,
        ref: entry.id,
        label: entry.id,
        source: 'framework' as const,
      }));
    case 'tool':
      return framework.tools.map((t) => ({
        kind: 'tool' as const,
        ref: t.name,
        label: t.name,
        source: 'framework' as const,
      }));
    case 'skill':
      return framework.skills.map((s) => ({
        kind: 'skill' as const,
        ref: s.name,
        label: s.name,
        source: 'framework' as const,
      }));
  }
}

/** De-duplicate `items` by (kind, ref), keeping the first occurrence.
 *  Callers order the union so the surviving entry's source is the
 *  intended winner — for the artifact tabs the precedence is
 *  built-ins → installed → framework. */
function dedupeByKindRef(items: PaletteItem[]): PaletteItem[] {
  const seen = new Set<string>();
  const out: PaletteItem[] = [];
  for (const item of items) {
    const key = `${item.kind}:${item.ref}`;
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    out.push(item);
  }
  return out;
}

/** Build the (unfiltered) item list for `tab` — built-ins + installed
 *  + the loaded framework's resolved artifacts (M08.6.E) + a connected
 *  MCP server's tools (M09.C), de-duplicated by (kind, ref). */
function paletteItemsForTab(
  tab: PaletteTab,
  installed: InstalledArtifact[],
  framework: Framework,
  mcpTools: PaletteItem[],
): PaletteItem[] {
  switch (tab) {
    case 'tools':
      return dedupeByKindRef([
        ...BUILTIN_TOOLS.map((t) => ({
          kind: 'tool' as const,
          ref: t,
          label: t,
          source: 'builtin' as const,
        })),
        ...installed
          .filter((a) => a.kind === 'tool')
          .map((a) => ({
            kind: 'tool' as const,
            ref: a.key,
            label: a.key,
            source: 'installed' as const,
          })),
        ...frameworkItemsForKind(framework, 'tool'),
        // M09.C — a connected MCP server's tools, keyed by the canonical
        // `<server>__<tool>` ref the §5a resolver accepts. Last in the
        // union so a built-in / installed / framework tool of the same
        // ref wins the dedupe (precedence preserved).
        ...mcpTools,
      ]);
    case 'skills':
      return dedupeByKindRef([
        ...installed
          .filter((a) => a.kind === 'skill')
          .map((a) => ({
            kind: 'skill' as const,
            ref: a.key,
            label: a.key,
            source: 'installed' as const,
          })),
        ...frameworkItemsForKind(framework, 'skill'),
      ]);
    case 'agents':
      // M09.A — the blank-create affordance. A fresh project's Agents tab
      // was empty (installed + framework only); prepend a "+ New agent"
      // item carrying a fresh `agent-N` ref through the uniform drag
      // contract. The existing addNode drop path mints `builderAgent`;
      // the Palette re-derives `nextAgentRef` each render so repeated
      // creates advance the id. (The "+ New" affordance for tools/skills
      // widens in a later ADR-0032 slice.)
      return dedupeByKindRef([
        {
          kind: 'agent' as const,
          ref: nextAgentRef(framework),
          label: '+ New agent',
          source: 'builtin' as const,
        },
        ...installed
          .filter((a) => a.kind === 'agent')
          .map((a) => ({
            kind: 'agent' as const,
            ref: a.key,
            label: a.key,
            source: 'installed' as const,
          })),
        ...frameworkItemsForKind(framework, 'agent'),
      ]);
    case 'hitl':
      return HITL_TRIGGERS.map((t) => ({
        kind: 'hitl' as const,
        ref: t,
        label: t,
        source: 'builtin' as const,
      }));
    case 'hooks':
      return HOOK_POINTS.map((h) => ({
        kind: 'hook' as const,
        ref: h,
        label: h,
        source: 'builtin' as const,
      }));
  }
}

export function Palette(): JSX.Element {
  const [tab, setTab] = useState<PaletteTab>('tools');
  const [filter, setFilter] = useState('');
  const [installed, setInstalled] = useState<InstalledArtifact[]>([]);
  // M09.C — a connected MCP server's tools, fetched once on mount as
  // ready-made `source:'mcp'` Palette items so the Tools tab can surface
  // them alongside the other three sources.
  const [mcpTools, setMcpTools] = useState<PaletteItem[]>([]);
  // The Palette draws from the loaded framework as a THIRD source
  // (M08.6.E). `s.framework` is a referentially stable object until
  // applyLoadedFramework / replaceFramework / addNode / connectEdge /
  // updateNode swaps it via set(), so a bare selector returns the same
  // reference across renders — useShallow is for derived arrays per
  // gotcha #75, not single-object selectors.
  const framework = useBuilderStore((s) => s.framework);

  useEffect(() => {
    // Installed artifacts feed the Tools/Skills/Agents tabs — the same
    // skills.lock read the ImportPanel uses on mount (M07-IRL #6). An
    // absent lock resolves to []; a backend error logs and the list
    // stays empty (built-ins still render).
    void listInstalledArtifacts()
      .then(setInstalled)
      .catch((e) => console.error('list_installed_artifacts error:', e));
  }, []);

  useEffect(() => {
    // M09.C — fetch each registered MCP server's tools and project them
    // into `source:'mcp'` Palette items (labelled `<server> · <tool>`,
    // ref `<server>__<tool>`). Per-server failures are skipped so one
    // offline server doesn't blank the rest; any failure (no McpClient,
    // registry error) logs and leaves the MCP source empty — built-ins
    // still render (the listInstalledArtifacts resilience precedent).
    void mcpListServers()
      .then(async (servers) => {
        const perServer = await Promise.all(
          servers.map(async (server) => {
            try {
              const tools = await mcpListServerTools(server.name);
              return tools.map((t) => ({
                kind: 'tool' as const,
                ref: `${server.name}__${t.name}`,
                label: `${server.name} · ${t.name}`,
                source: 'mcp' as const,
              }));
            } catch (e) {
              console.error(`mcp_list_server_tools error for ${server.name}:`, e);
              return [];
            }
          }),
        );
        setMcpTools(perServer.flat());
      })
      .catch((e) => console.error('mcp_list_servers error:', e));
  }, []);

  const items = useMemo(
    () => paletteItemsForTab(tab, installed, framework, mcpTools),
    [tab, installed, framework, mcpTools],
  );
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
              className={`builder-palette__item builder-palette__item--${it.source}`}
              data-testid={`palette-item-${it.ref}`}
              data-source={it.source}
              draggable
              onDragStart={(e) => {
                // The C->D1 contract — D1's onDrop reads this MIME type
                // + JSON payload via screenToFlowPosition. Uniform across
                // all three sources (built-in / installed / framework)
                // per M08.6.E phase doc E.3.
                e.dataTransfer.setData(
                  'application/x-builder-node',
                  JSON.stringify({ kind: it.kind, ref: it.ref }),
                );
                e.dataTransfer.effectAllowed = 'copy';
              }}
            >
              {it.label}
              {it.source === 'framework' || it.source === 'mcp' ? (
                <span className="builder-palette__source-badge" aria-hidden="true">
                  {' '}
                  · {it.source}
                </span>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
