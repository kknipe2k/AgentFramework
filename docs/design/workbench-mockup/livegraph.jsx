/* =====================================================================
   livegraph.jsx — the centerpiece. Presentational pieces for the live-
   graph execution view: NodeView, Edges, GraphCanvas, Transport,
   RunBanner, BudgetBar, OutputRail. Driven by props (run state lives in
   tester.jsx). Build it, then watch it actually run.
   ===================================================================== */
const { useState: useStateLG, useMemo: useMemoLG, useRef: useRefLG } = React;

const NODE_W = 140, PARENT_H = 50;
const nodeKindOf = (id) => (window.ARData.NODES.find(n => n.id === id) || {}).kind || 'agent';
const nodeOf = (id) => window.ARData.NODES.find(n => n.id === id);

/* ---- a single node on the canvas --------------------------------- */
function NodeView({ node, status, io, selected, onSelect }) {
  const st = status || 'idle';
  const showIO = node.kind === 'tool' && io && (st === 'active' || st === 'complete' || st === 'gap');
  return (
    <div
      className={`node node--${node.kind} s-${st}${selected ? ' is-selected' : ''}`}
      style={{ left: node.x, top: node.y, ...kindVars(node.kind) }}
      onClick={(e) => { e.stopPropagation(); onSelect(node.id); }}
      data-screen-label={`node:${node.id}`}
    >
      <div className="node__top">
        <Glyph kind={node.kind} />
        <span className="node__name" title={node.label}>{node.label}</span>
        <span className="node__statusdot" title={st} />
      </div>
      <div className="node__sub">{node.sub}</div>
      {showIO && (
        <div className="node__io">
          {io.input && <div className="row"><span className="k">in</span><span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{shortJSON(io.input)}</span></div>}
          {io.output != null && <div className="row"><span className="k">out</span><span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{String(io.output).split('\n')[0].slice(0, 22)}</span></div>}
          {io.duration != null && <div className="row"><span className="k">·</span><span className="dur">{io.duration}ms</span></div>}
        </div>
      )}
      {st === 'gap' && <span className="node__badge"><Badge kind="error" dot>capability gap</Badge></span>}
      {st === 'violation' && <span className="node__badge"><Badge kind="error" dot>blocked</Badge></span>}
      {st === 'blocked' && <span className="node__badge"><Badge kind="warn" dot>awaiting human</Badge></span>}
    </div>
  );
}
const shortJSON = (o) => { try { const s = JSON.stringify(o); return s.length > 26 ? s.slice(0, 24) + '…' : s; } catch { return String(o); } };

/* ---- edges layer -------------------------------------------------- */
function Edges({ edges, status }) {
  return (
    <svg className="edges" aria-hidden="true">
      {edges.map((e, i) => {
        const a = nodeOf(e.from), b = nodeOf(e.to);
        if (!a || !b) return null;
        const x1 = a.x, y1 = a.y + PARENT_H, x2 = b.x, y2 = b.y;
        const my = (y1 + y2) / 2;
        const d = `M ${x1} ${y1} C ${x1} ${my}, ${x2} ${my}, ${x2} ${y2}`;
        const tst = status[e.to] || 'idle';
        const flow = (e.kind === 'tool' || e.kind === 'hook' || e.kind === 'agent') && tst === 'active';
        const done = ['complete', 'gap', 'violation'].includes(tst);
        const cls = `edge edge--${e.kind}${flow ? ' is-flow' : ''}${done ? ' is-done' : tst === 'idle' ? ' is-idle' : ''}`;
        const portColor = `var(--kind-${e.kind})`;
        return (
          <g key={i}>
            <path className={cls} d={d} />
            <circle className="port" cx={x2} cy={y2} r={flow ? 4 : 3} fill={done || flow || tst !== 'idle' ? portColor : 'var(--surface-0)'} stroke={portColor} strokeWidth="1.5" />
          </g>
        );
      })}
    </svg>
  );
}

/* ---- canvas ------------------------------------------------------- */
function GraphCanvas({ nodes, edges, status, io, selected, onSelect, runStatus }) {
  return (
    <div className="canvas-wrap" onClick={() => onSelect(null)}>
      <div className="canvas-toolbar">
        <Badge kind={runStatus === 'suspended' ? 'error' : runStatus === 'running' ? 'info' : runStatus === 'done' ? 'ok' : 'muted'} dot>
          {runStatus === 'running' ? 'executing' : runStatus === 'suspended' ? 'suspended' : runStatus === 'done' ? 'complete' : 'ready'}
        </Badge>
        <span className="spacer" style={{ flex: 1 }} />
        <Button sm variant="ghost" icon="fit">Fit</Button>
        <div className="seg" role="group" aria-label="legend" style={{ background: 'var(--surface-0)', border: '1px solid var(--border)' }}>
          {['agent', 'tool', 'skill', 'hook', 'hitl'].map(k => (
            <span key={k} style={{ display: 'inline-flex', alignItems: 'center', gap: 5, padding: '4px 8px', fontSize: 11, fontWeight: 600, color: KIND[k].t }}>
              <span style={{ width: 8, height: 8, borderRadius: 2, background: KIND[k].c }} />{KIND[k].label}
            </span>
          ))}
        </div>
      </div>
      <div className="canvas-field">
        <Edges edges={edges} status={status} />
        {nodes.map(n => <NodeView key={n.id} node={n} status={status[n.id]} io={io[n.id]} selected={selected === n.id} onSelect={onSelect} />)}
      </div>
    </div>
  );
}

/* ---- run banner --------------------------------------------------- */
function RunBanner({ runStatus, gaps, onResume }) {
  if (runStatus === 'suspended') {
    const g = gaps[gaps.length - 1];
    return (
      <div className="run-banner run-banner--suspended">
        <Icon name="alert" size={18} />
        <div>
          <strong>Session suspended — capability gap.</strong>{' '}
          {g ? <span>{g.action}</span> : 'A required capability is missing.'}
        </div>
        <span className="spacer" />
        <Button sm icon="plus">Install &amp; Resume</Button>
      </div>
    );
  }
  if (runStatus === 'running') return <div className="run-banner run-banner--running"><span className="pulse" /><span>Framework executing — events streaming live into the graph.</span></div>;
  if (runStatus === 'done') return <div className="run-banner run-banner--done"><Icon name="check" size={16} /><span>Run complete.</span></div>;
  return null;
}

/* ---- budget bar --------------------------------------------------- */
function BudgetBar({ budget, tokens, cost }) {
  return (
    <div className={`budget ${budget.status}`}>
      <span className="t-label" style={{ margin: 0 }}>Budget</span>
      <div className="budget__track"><div className="budget__fill" style={{ width: `${budget.percent}%` }} /></div>
      <span className="nums">${budget.spent.toFixed(2)} / ${budget.cap.toFixed(2)}</span>
      <Badge kind={budget.status === 'ok' ? 'muted' : budget.status === 'warn' ? 'warn' : 'error'} dot>
        {budget.status === 'suspended' ? 'spend halted' : budget.status === 'warn' ? `${budget.percent}%` : `${budget.percent}%`}
      </Badge>
      <span className="spacer" style={{ flex: 1 }} />
      <span className="nums">{tokens.toLocaleString()} tok</span>
    </div>
  );
}

/* ---- transport / timeline ---------------------------------------- */
function Transport({ playhead, lastT, playing, onToggle, onScrub, onRestart, speed, setSpeed, marks }) {
  const trackRef = useRefLG(null);
  const pct = lastT ? Math.min(100, (playhead / lastT) * 100) : 0;
  const seek = (clientX) => {
    const r = trackRef.current.getBoundingClientRect();
    onScrub(Math.max(0, Math.min(1, (clientX - r.left) / r.width)) * lastT);
  };
  const onDown = (e) => {
    seek(e.clientX);
    const mv = (ev) => seek(ev.clientX);
    const up = () => { window.removeEventListener('mousemove', mv); window.removeEventListener('mouseup', up); };
    window.addEventListener('mousemove', mv); window.addEventListener('mouseup', up);
  };
  return (
    <div className="transport">
      <button className="play" onClick={onToggle} aria-label={playing ? 'Pause' : 'Play'}><Icon name={playing ? 'pause' : 'play'} size={16} /></button>
      <button className="btn btn--ghost btn--sm" onClick={onRestart} aria-label="Restart" style={{ padding: 6 }}><Icon name="restart" size={15} /></button>
      <div className="scrub">
        <div className="scrub__track" ref={trackRef} onMouseDown={onDown}>
          <div className="scrub__fill" style={{ width: `${pct}%` }} />
          <div className="scrub__ticks">
            {marks.map((m, i) => <span key={i} className={`scrub__tick t-${m.type}`} style={{ left: `${(m.t / lastT) * 100}%` }} title={m.label} />)}
          </div>
          <div className="scrub__head" style={{ left: `${pct}%` }} />
        </div>
      </div>
      <span className="clock">{(playhead / 1000).toFixed(1)}s</span>
      <div className="speed">
        {[0.5, 1, 2].map(s => <button key={s} className={speed === s ? 'on' : ''} onClick={() => setSpeed(s)}>{s}×</button>)}
      </div>
    </div>
  );
}

/* ---- transcript / output rail (reply lives here, keyed to node) --- */
function StreamEntry({ e }) {
  const k = e.node ? nodeKindOf(e.node) : 'agent';
  const who = e.node ? (nodeOf(e.node)?.label || e.who) : e.who;
  const chipStyle = { ...kindVars(k) };
  if (e.kind === 'reply') {
    return (
      <div className="entry">
        <div className="entry__rail"><span className="entry__chip" style={chipStyle}>{KIND[k].glyph}</span><span className="entry__line" /></div>
        <div className="entry__body">
          <div><span className="entry__who" style={{ color: KIND[k].t }}>{who}</span><span className="entry__when">{e.tStr}</span></div>
          <div className="entry__text reply">{e.text}</div>
        </div>
      </div>
    );
  }
  if (e.kind === 'tool') {
    return (
      <div className="entry">
        <div className="entry__rail"><span className="entry__chip" style={kindVars('tool')}>T</span><span className="entry__line" /></div>
        <div className="entry__body">
          <div><span className="entry__who" style={{ color: KIND.tool.t }}>{who}</span>
            <span className="entry__when">{e.phase === 'call' ? 'invoked' : 'result'} · {e.tStr}</span>
            {e.source && <span style={{ marginLeft: 6 }} className={`src src--${e.source}`}>{e.source}{e.server ? `:${e.server}` : ''}</span>}
          </div>
          <div className="entry__code">{e.code}</div>
          {(e.duration != null || e.tokensIn != null) && <div className="entry__meta">{e.duration != null && <span>⏱ {e.duration}ms</span>}{e.tokensIn ? <span>↓ {e.tokensIn} tok</span> : null}</div>}
        </div>
      </div>
    );
  }
  if (e.kind === 'hitl') {
    return (
      <div className="entry">
        <div className="entry__rail"><span className="entry__chip" style={kindVars('hitl')}>‖</span><span className="entry__line" /></div>
        <div className="entry__body">
          <div><span className="entry__who" style={{ color: KIND.hitl.t }}>{who}</span><span className="entry__when">{e.tStr}</span></div>
          <div className="entry__text" style={{ fontWeight: 500 }}>{e.text}</div>
          <div style={{ display: 'flex', gap: 6, marginTop: 8 }}>
            {(e.options || []).map(o => <span key={o} className="btn btn--sm" style={{ pointerEvents: 'none' }}>{o}</span>)}
          </div>
        </div>
      </div>
    );
  }
  if (e.kind === 'incident' || e.kind === 'gap') {
    return (
      <div className={`entry entry--${e.kind}`}>
        <div className="entry__rail"><span className="entry__chip" style={kindVars(e.kind === 'gap' ? 'gap' : 'hook')}>{e.kind === 'gap' ? '!' : 'H'}</span><span className="entry__line" /></div>
        <div className="entry__body">
          <div><span className="entry__who" style={{ color: e.kind === 'gap' ? KIND.gap.t : 'var(--error-text)' }}>{e.kind === 'gap' ? 'capability gap' : 'capability violation'}</span><span className="entry__when">{e.tStr}</span></div>
          <div className="entry__text" style={{ fontWeight: 500 }}>{e.text}</div>
          {e.detail && <div className="entry__meta" style={{ color: 'var(--text-1)' }}>{e.detail}</div>}
        </div>
      </div>
    );
  }
  // generic event
  return (
    <div className="entry entry--event">
      <div className="entry__rail"><span className="entry__chip" style={{ ...chipStyle, opacity: e.spawn ? 1 : .85 }}>{e.node ? KIND[k].glyph : '•'}</span><span className="entry__line" /></div>
      <div className="entry__body">
        <div><span className="entry__who" style={{ color: e.tone === 'warn' ? 'var(--warn-text)' : (e.node ? KIND[k].t : 'var(--text-2)') }}>{who}</span><span className="entry__when">{e.tStr}</span></div>
        <div className="entry__text">{e.text}</div>
        {e.duration != null && <div className="entry__meta">⏱ {e.duration}ms</div>}
      </div>
    </div>
  );
}

function OutputRail({ transcript, selected, setSelected }) {
  const filtered = selected ? transcript.filter(e => e.node === selected) : transcript;
  const node = selected ? nodeOf(selected) : null;
  return (
    <>
      {selected && (
        <div className="between" style={{ padding: '10px var(--s4)', borderBottom: '1px solid var(--border-subtle)', background: 'var(--surface-1)' }}>
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 8, fontSize: 12.5, fontWeight: 600 }}>
            <span style={{ width: 8, height: 8, borderRadius: 2, background: KIND[node.kind].c }} />
            Output · {node.label}
          </span>
          <button className="btn btn--ghost btn--sm" onClick={() => setSelected(null)}>Show all</button>
        </div>
      )}
      <div className="rail-body">
        {filtered.length === 0
          ? <div className="empty-hint">{selected ? 'No output yet for this node — scrub forward or press play.' : 'Press play to watch the framework run. Agent replies, tool calls and results stream here live.'}</div>
          : <div className="stream">{filtered.map((e, i) => <StreamEntry key={e.id + '-' + i} e={e} />)}</div>}
      </div>
    </>
  );
}

Object.assign(window, { GraphCanvas, Transport, RunBanner, BudgetBar, OutputRail, NodeView, Edges, StreamEntry, nodeKindOf, nodeOf });
