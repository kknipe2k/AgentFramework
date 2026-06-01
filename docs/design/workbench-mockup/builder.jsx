/* =====================================================================
   builder.jsx — the Builder: Palette (draggable node kinds), Canvas
   (build mode) | JSON view switch, Inspector with Load/Save/Validate/
   Test, a clear-canvas recovery affordance (rule 5), and the plain-
   English validation/error surface (rules 6 + 7).
   ===================================================================== */
const { useState: useStateB, useMemo: useMemoB } = React;

const PALETTE = [
  { kind: 'agent', name: 'Agent', hint: 'spawns' },
  { kind: 'tool',  name: 'Tool',  hint: 'builtin' },
  { kind: 'skill', name: 'Skill', hint: 'context' },
  { kind: 'hook',  name: 'Hook',  hint: 'verify' },
  { kind: 'hitl',  name: 'HITL',  hint: 'gate' },
];

function highlightJSON(src) {
  const esc = src.replace(/&/g, '&amp;').replace(/</g, '&lt;');
  return esc
    .replace(/("(?:[^"\\]|\\.)*")(\s*:)/g, '<span class="tok-key">$1</span>$2')
    .replace(/:\s*("(?:[^"\\]|\\.)*")/g, ': <span class="tok-str">$1</span>')
    .replace(/\b(-?\d+\.?\d*)\b/g, '<span class="tok-num">$1</span>')
    .replace(/([{}\[\],])/g, '<span class="tok-punc">$1</span>');
}

function Palette() {
  return (
    <div className="palette">
      <div className="t-label" style={{ padding: '4px 6px 8px' }}>Palette · drag to canvas</div>
      <div className="palette__group">
        {PALETTE.map(p => (
          <div key={p.kind} className="palette__item" style={kindVars(p.kind)} draggable>
            <span className="pglyph" style={kindVars(p.kind)}>{KIND[p.kind].glyph}</span>
            <span className="pname">{p.name}</span>
            <span className="phint">{p.hint}</span>
          </div>
        ))}
      </div>
      <div className="divider" />
      <div className="t-label" style={{ padding: '4px 6px 8px' }}>Built-in tools</div>
      {['read_file', 'write_file', 'run_tests', 'git_commit'].map(t => (
        <div key={t} className="palette__item" style={kindVars('tool')} draggable>
          <span className="pglyph" style={kindVars('tool')}>T</span><span className="pname mono" style={{ fontSize: 12 }}>{t}</span><span className="src src--builtin">builtin</span>
        </div>
      ))}
    </div>
  );
}

function ValidationSurface({ result, onValidate }) {
  if (!result) {
    return <div className="empty-hint" style={{ padding: 'var(--s4)' }}>Not validated yet. Run <strong>Validate</strong> to check capability narrowing, references and schema — results surface per node, in plain language.</div>;
  }
  if (result.ok) {
    return <div className="err-card" style={{ border: '1px solid var(--ok-border)', background: 'var(--ok-bg)' }}><div className="err-plain" style={{ color: 'var(--ok-text)' }}>✓ Framework is valid. 11 nodes, 10 edges, capability narrowing satisfied.</div></div>;
  }
  return (
    <div>
      <div className="between" style={{ marginBottom: 10 }}>
        <Badge kind="error" dot>{result.errors.length} issue{result.errors.length === 1 ? '' : 's'}</Badge>
        <span className="muted t-small">click a node to locate</span>
      </div>
      {result.errors.map((e, i) => <ErrorCard key={i} e={e} />)}
    </div>
  );
}

function ErrorCard({ e }) {
  const [raw, setRaw] = useStateB(false);
  return (
    <div className="err-card" style={{ marginBottom: 10 }}>
      <div className="between" style={{ marginBottom: 6 }}>
        <span style={{ display: 'inline-flex', alignItems: 'center', gap: 7, fontWeight: 600, fontSize: 12.5 }}><span className="insp-glyph" style={{ ...kindVars('agent'), width: 20, height: 20, fontSize: 11 }}>A</span>{e.node}</span>
        <Badge kind="error">schema</Badge>
      </div>
      <div className="err-plain">{e.plain}</div>
      <div className="err-fix">→ {e.fix}</div>
      <button className="btn btn--ghost btn--sm" style={{ marginTop: 8, padding: '3px 7px' }} onClick={() => setRaw(r => !r)}>{raw ? 'Hide' : 'Show'} raw error</button>
      {raw && <div className="err-raw">{e.raw}</div>}
    </div>
  );
}

function BuilderView() {
  const [view, setView] = useStateB('canvas');     // canvas | json
  const [selected, setSelected] = useStateB(null);
  const [validation, setValidation] = useStateB(null);
  const [hasInvalid, setHasInvalid] = useStateB(true);
  const push = useToast();

  // builder graph = framework nodes (+ one invalid demo node to exercise
  // the error surface, until Clear removes it — rule 5 recovery).
  const nodes = useMemoB(() => {
    const base = window.ARData.NODES.map(n => ({ ...n }));
    if (hasInvalid) base.push({ id: 'demo-agent', kind: 'agent', label: 'demo-agent@1.0.0', sub: 'agent · unsaved', x: 690, y: 470 });
    return base;
  }, [hasInvalid]);
  const edges = window.ARData.EDGES;
  const idleStatus = useMemoB(() => { const s = {}; nodes.forEach(n => s[n.id] = n.id === 'demo-agent' ? 'error' : 'idle'); return s; }, [nodes]);

  const validate = () => {
    if (hasInvalid) {
      const t = window.ARData.ERROR_TRANSLATIONS[0];
      setValidation({ ok: false, errors: [t] });
      setSelected('demo-agent');
      push({ kind: 'warn', title: 'Validation found 1 issue', msg: 'demo-agent is missing its capabilities block. See the Inspector.' });
    } else {
      setValidation({ ok: true });
      push({ kind: 'ok', title: 'Validation passed', msg: 'Framework “aria” is valid.' });
    }
  };
  const save = () => {
    if (hasInvalid) push({ kind: 'error', title: 'Can’t save', msg: '1 node is invalid. Fix it and re-validate first.' });
    else push({ kind: 'ok', title: 'Saved', msg: 'Wrote framework.json + 11 companion files to disk.' });
  };
  const clearCanvas = () => {
    setHasInvalid(false); setValidation(null); setSelected(null);
    push({ kind: 'ok', title: 'Removed invalid node', msg: 'demo-agent cleared. Undo available.', });
  };

  return (
    <>
      <aside className="rail-left"><Palette /></aside>
      <div className="center">
        <div className="budget" style={{ justifyContent: 'space-between' }}>
          <div className="seg">
            <button className={view === 'canvas' ? 'on' : ''} onClick={() => setView('canvas')}>Canvas</button>
            <button className={view === 'json' ? 'on' : ''} onClick={() => setView('json')}>JSON</button>
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button sm icon="save" onClick={() => push({ kind: 'info', title: 'Loaded', msg: 'examples/aria/ loaded from disk.' })}>Load</Button>
            <Button sm onClick={clearCanvas}>Clear</Button>
            <Button sm onClick={validate}>Validate</Button>
            <Button sm onClick={save}>Save</Button>
            <Button sm variant="primary" icon="bolt" onClick={() => push({ kind: 'info', title: 'Switch to Tester', msg: 'Open the Tester tab to run this framework.' })}>Test</Button>
          </div>
        </div>
        {view === 'canvas'
          ? <GraphCanvas nodes={nodes} edges={edges} status={idleStatus} io={{}} selected={selected} onSelect={setSelected} runStatus="idle" />
          : <div className="jsonview"><pre dangerouslySetInnerHTML={{ __html: highlightJSON(window.ARData.FRAMEWORK_JSON) }} /></div>}
      </div>
      <aside className="rail-right">
        <div className="rail-tabs" role="tablist">
          <button className="rail-tab" role="tab" aria-selected={true}>Inspector</button>
        </div>
        <div className="rail-body">
          {selected ? <NodeInspector selected={selected} status={idleStatus} io={{}} nodes={nodes} /> : <NodeInspector selected={null} status={{}} io={{}} nodes={nodes} />}
          <div className="divider" />
          <div className="t-label" style={{ marginBottom: 10 }}>Validation</div>
          <ValidationSurface result={validation} onValidate={validate} />
        </div>
      </aside>
    </>
  );
}

Object.assign(window, { BuilderView, Palette, highlightJSON });
