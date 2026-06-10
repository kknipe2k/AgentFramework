import { useState } from 'react';
import { useBuilderStore } from '../../lib/builderStore';
import type { Agent, Framework } from '../../types/framework';

/** The Anthropic model set offered in the model dropdown — v0.1 is
 *  Anthropic-only (§0d). */
const MODELS = ['claude-opus-4-7', 'claude-sonnet-4-6', 'claude-haiku-4-5'] as const;

/** Resolve the inline Agent a canvas node id selects, or `undefined`
 *  (a `{ id, path }` $ref agent or a non-agent node — D1 configures
 *  inline Agent nodes; D2 widens to other kinds). */
function findAgent(framework: Framework, nodeId: string): Agent | undefined {
  const entry = framework.agents.find((a) => `agent:${a.id}` === nodeId);
  if (entry === undefined || !('role' in entry)) {
    return undefined;
  }
  return entry;
}

interface AllowedListProps {
  label: string;
  testId: string;
  removeTestIdPrefix: string;
  addInputTestId: string;
  addButtonTestId: string;
  items: string[];
  onChange: (next: string[]) => void;
}

/** An editable name list — the inline alternative to D2's drag-to-connect
 *  edges; both write the same `framework` `allowed_*` field. */
function AllowedList({
  label,
  testId,
  removeTestIdPrefix,
  addInputTestId,
  addButtonTestId,
  items,
  onChange,
}: AllowedListProps): JSX.Element {
  const [draft, setDraft] = useState('');
  function add(): void {
    const value = draft.trim();
    if (value.length === 0 || items.includes(value)) {
      return;
    }
    onChange([...items, value]);
    setDraft('');
  }
  return (
    <div className="builder-node-config__field">
      <span>{label}</span>
      <ul className="builder-node-config__list" data-testid={testId}>
        {items.map((item) => (
          <li key={item} className="builder-node-config__list-item">
            {item}
            <button
              type="button"
              data-testid={`${removeTestIdPrefix}-${item}`}
              onClick={() => onChange(items.filter((i) => i !== item))}
            >
              ×
            </button>
          </li>
        ))}
      </ul>
      <div className="builder-node-config__add">
        <input
          data-testid={addInputTestId}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
        />
        <button type="button" data-testid={addButtonTestId} onClick={add}>
          Add
        </button>
      </div>
    </div>
  );
}

/**
 * The inline node-configuration surface (M08.D1 — spec Phase 9
 * "right-click for properties"). For the selected Agent node: `role`,
 * `model`, and the `allowed_tools` / `allowed_skills` editable lists.
 * Every edit flows through `builderStore.updateNode` → a `framework`
 * mutation → the canvas projection (and the per-node capability
 * disclosure) re-derive. Renders `null` when nothing is selected.
 */
export function NodeConfigPanel(): JSX.Element | null {
  const selectedNodeId = useBuilderStore((s) => s.selectedNodeId);
  const framework = useBuilderStore((s) => s.framework);
  const updateNode = useBuilderStore((s) => s.updateNode);

  if (selectedNodeId === null) {
    return null;
  }
  const agent = findAgent(framework, selectedNodeId);
  if (agent === undefined) {
    return null;
  }

  return (
    <div className="builder-node-config" data-testid="builder-node-config" role="group">
      <h3 className="builder-node-config__title">Configure {agent.id}</h3>
      <label className="builder-node-config__field">
        <span>Role</span>
        <input
          data-testid="node-config-role"
          value={agent.role}
          onChange={(e) => updateNode(selectedNodeId, { role: e.target.value })}
        />
      </label>
      <label className="builder-node-config__field">
        <span>Model</span>
        <select
          data-testid="node-config-model"
          value={agent.model.id}
          onChange={(e) =>
            updateNode(selectedNodeId, { model: { provider: 'anthropic', id: e.target.value } })
          }
        >
          {MODELS.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>
      </label>
      <AllowedList
        label="Tools"
        testId="node-config-tools"
        removeTestIdPrefix="node-config-tool-remove"
        addInputTestId="node-config-add-tool-input"
        addButtonTestId="node-config-add-tool"
        items={agent.allowed_tools}
        onChange={(next) => updateNode(selectedNodeId, { allowed_tools: next })}
      />
      <AllowedList
        label="Skills"
        testId="node-config-skills"
        removeTestIdPrefix="node-config-skill-remove"
        addInputTestId="node-config-add-skill-input"
        addButtonTestId="node-config-add-skill"
        items={agent.allowed_skills}
        onChange={(next) => updateNode(selectedNodeId, { allowed_skills: next })}
      />
      <FileAccessEditor
        capabilities={agent.capabilities}
        onChange={(capabilities) => updateNode(selectedNodeId, { capabilities })}
      />
    </div>
  );
}

interface FileAccessEditorProps {
  capabilities: Agent['capabilities'];
  onChange: (next: Agent['capabilities']) => void;
}

/**
 * The file_access grant editor (M09.B). Two glob lists — Read and Write —
 * over `capabilities.file_access.{read,write}` (the L2 enforcer's scope;
 * E-02 `capability_live_tool.rs`). Each edit recomputes the FULL
 * `Capabilities` immutably (the other required fields — tools_called /
 * network / shell / … — carried through untouched) and writes it via
 * `updateNode`'s `{ capabilities }` patch. Declaration-only: this writes
 * the agent's grant in the document; the enforcer (unchanged) consumes it
 * at run time, and the *enforced* write lands at M09.D. M09 scopes to
 * file_access; the rest of the Capabilities surface widens per the
 * ADR-0032 slice that executes it (spawn_agents→M11, shell→M12,
 * network→M13; tools_called via M09.C).
 */
function FileAccessEditor({ capabilities, onChange }: FileAccessEditorProps): JSX.Element {
  const fa = capabilities.file_access;
  return (
    <div className="builder-node-config__field" data-testid="node-config-file-access">
      <span>File access</span>
      <AllowedList
        label="Read (globs)"
        testId="node-config-fa-read"
        removeTestIdPrefix="node-config-fa-read-remove"
        addInputTestId="node-config-add-fa-read-input"
        addButtonTestId="node-config-add-fa-read"
        items={fa.read}
        onChange={(read) => onChange({ ...capabilities, file_access: { ...fa, read } })}
      />
      <AllowedList
        label="Write (globs)"
        testId="node-config-fa-write"
        removeTestIdPrefix="node-config-fa-write-remove"
        addInputTestId="node-config-add-fa-write-input"
        addButtonTestId="node-config-add-fa-write"
        items={fa.write}
        onChange={(write) => onChange({ ...capabilities, file_access: { ...fa, write } })}
      />
    </div>
  );
}
