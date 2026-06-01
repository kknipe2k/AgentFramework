/* =====================================================================
   tester.jsx — the run + test surface (the centerpiece host). Owns
   playback state, folds the event stream, and lays out:
     left  : task input + Run + live results + gaps
     center: budget bar + live graph canvas + run banner + transport
     right : Output / Inspector rail (agent reply lives here, per node)
   ===================================================================== */
const { useState: useStateT, useEffect: useEffectT, useRef: useRefT, useMemo: useMemoT } = React;

const PLAY_RATE = 1.3; // 1× speed multiplier (mild compression for demo feel)

function useRunState() {
  const events = window.ARData.EVENTS;
  const lastT = events[events.length - 1].t;
  const [playhead, setPlayhead] = useStateT(0);
  const [playing, setPlaying] = useStateT(false);
  const [speed, setSpeed] = useStateT(1);
  const raf = useRefT(0); const last = useRefT(0);

  useEffectT(() => {
    if (!playing) return;
    last.current = performance.now();
    const tick = (now) => {
      const dt = (now - last.current) * PLAY_RATE * speed; last.current = now;
      setPlayhead(p => {
        const np = p + dt;
        if (np >= lastT) { setPlaying(false); return lastT; }
        return np;
      });
      raf.current = requestAnimationFrame(tick);
    };
    raf.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf.current);
  }, [playing, speed, lastT]);

  const index = useMemoT(() => {
    let i = -1; for (let k = 0; k < events.length; k++) { if (events[k].t <= playhead) i = k; else break; } return i;
  }, [playhead, events]);
  const derived = useMemoT(() => foldRun(events, index), [index, events]);

  return {
    playhead, lastT, playing, speed, index, ...derived,
    toggle: () => { if (playhead >= lastT) { setPlayhead(0); setPlaying(true); } else setPlaying(p => !p); },
    scrub: (t) => { setPlaying(false); setPlayhead(t); },
    restart: () => { setPlayhead(0); setPlaying(false); },
    setSpeed,
  };
}

/* ---- Inspector tab (node config / framework summary) ------------- */
function NodeInspector({ selected, status, io, nodes }) {
  const resolve = (id) => (nodes && nodes.find(n => n.id === id)) || nodeOf(id);
  if (!selected) {
    const counts = window.ARData.NODES.reduce((a, n) => (a[n.kind] = (a[n.kind] || 0) + 1, a), {});
    return (
      <div>
        <div className="insp-head"><span className="insp-glyph" style={kindVars('agent')}>α</span><div><div style={{ fontWeight: 600, fontSize: 14 }}>aria</div><div className="mono-micro muted">framework · v1.0.0</div></div></div>
        <dl className="kv">
          <dt>model</dt><dd>claude-sonnet-4-6</dd>
          <dt>fallback</dt><dd>claude-haiku-4-5</dd>
          <dt>mode</dt><dd>STANDARD</dd>
        </dl>
        <div className="t-label section-label">Composition</div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
          {Object.entries(counts).map(([k, n]) => <span key={k} className="badge badge--muted"><span style={{ width: 7, height: 7, borderRadius: 2, background: KIND[k].c }} />{n} {KIND[k].label}</span>)}
        </div>
        <div className="empty-hint" style={{ paddingBottom: 0 }}>Select a node to inspect its config and output.</div>
      </div>
    );
  }
  const n = resolve(selected); const st = status[selected] || 'idle'; const d = io[selected];
  const stBadge = { active: 'info', complete: 'ok', gap: 'error', violation: 'error', blocked: 'warn', loaded: 'info', idle: 'muted' }[st] || 'muted';
  return (
    <div>
      <div className="insp-head">
        <span className="insp-glyph" style={kindVars(n.kind)}>{KIND[n.kind].glyph}</span>
        <div><div style={{ fontWeight: 600, fontSize: 14 }}>{n.label}</div><div className="mono-micro muted">{KIND[n.kind].label}</div></div>
        <span style={{ marginLeft: 'auto' }}><Badge kind={stBadge} dot>{st}</Badge></span>
      </div>
      <dl className="kv">
        <dt>id</dt><dd>{n.id}</dd>
        <dt>kind</dt><dd>{n.kind}</dd>
        <dt>source</dt><dd>{n.sub}</dd>
      </dl>
      {n.kind === 'tool' && d && (
        <>
          <div className="t-label section-label">Last invocation</div>
          <div className="entry__code">{JSON.stringify(d.input, null, 2)}</div>
          {d.output != null && <><div className="t-label section-label">Result</div><div className="entry__code">{String(d.output)}</div></>}
          {d.duration != null && <div className="entry__meta" style={{ marginTop: 8 }}><span>⏱ {d.duration}ms</span>{d.tokensIn ? <span>↓ {d.tokensIn} tok</span> : null}</div>}
        </>
      )}
      {n.kind === 'agent' && (
        <>
          <div className="t-label section-label">Capabilities</div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            <Badge kind="info" mono>read:src/**</Badge><Badge kind="info" mono>write:src/**</Badge>
          </div>
          <div className="err-card" style={{ marginTop: 12, border: '1px solid var(--accent-border)', background: 'var(--accent-bg)' }}>
            <div style={{ fontSize: 12.5, color: 'var(--text-1)' }}><strong>Narrowed on spawn.</strong> Inherited from <span className="mono-micro">orchestrator</span> and narrowed to <span className="mono-micro">src/**</span> per L2a.</div>
          </div>
        </>
      )}
      {st === 'gap' && <div className="err-card" style={{ marginTop: 12 }}><div className="err-plain">This tool reference did not resolve at load. The session is suspended until the capability is installed.</div><div className="err-fix">Install an MCP server providing <span className="mono-micro">fetch_prs</span>, then Resume.</div></div>}
      {st === 'violation' && <div className="err-card" style={{ marginTop: 12 }}><div className="err-plain">Blocked by the capability enforcer (network egress not granted).</div></div>}
    </div>
  );
}

/* ---- Left rail: task + results + gaps ---------------------------- */
function TesterLeft({ run, onRun }) {
  const verifyDone = run.status.verify_std === 'complete';
  const tasks = run.index >= 5 ? 4 : 0;
  return (
    <div style={{ padding: 'var(--s4)' }}>
      <div className="t-label" style={{ marginBottom: 8 }}>Task</div>
      <textarea className="textarea" rows={3} defaultValue="Implement a config parser in src/config that honours the existing snake_case convention, with tests." />
      <div style={{ display: 'flex', gap: 8, marginTop: 10 }}>
        <Button variant="primary" icon="play" onClick={onRun} style={{ flex: 1 }}>{run.playing ? 'Running…' : 'Run framework'}</Button>
        <Button icon="save" aria-label="Save">Save</Button>
      </div>

      <div className="divider" />
      <div className="t-label" style={{ marginBottom: 10 }}>Run summary</div>
      <div className="metrics" style={{ gridTemplateColumns: '1fr 1fr' }}>
        <div className="metric"><div className="label t-label">Result</div><div className={`value ${run.runStatus === 'suspended' ? 'bad' : verifyDone ? 'ok' : ''}`} style={{ fontSize: 16 }}>{run.runStatus === 'suspended' ? 'Suspended' : run.runStatus === 'running' ? 'Running' : 'Ready'}</div><div className="delta">{run.gaps.length} gap{run.gaps.length === 1 ? '' : 's'}</div></div>
        <div className="metric"><div className="label t-label">Verify</div><div className={`value ${verifyDone ? 'ok' : ''}`} style={{ fontSize: 16 }}>{verifyDone ? '12 ✓' : '—'}</div><div className="delta">0 failed</div></div>
        <div className="metric"><div className="label t-label">Tokens</div><div className="value" style={{ fontSize: 16 }}>{(run.tokens / 1000).toFixed(1)}k</div><div className="delta">in + out</div></div>
        <div className="metric"><div className="label t-label">Spend</div><div className={`value ${run.budget.status === 'suspended' ? 'bad' : ''}`} style={{ fontSize: 16 }}>${run.budget.spent.toFixed(2)}</div><div className="delta">/ ${run.budget.cap.toFixed(2)} cap</div></div>
      </div>

      {run.gaps.length > 0 && (
        <>
          <div className="t-label section-label">Capability gaps</div>
          {run.gaps.map((g, i) => (
            <div key={i} className="err-card" style={{ marginBottom: 8 }}>
              <div className="between"><span className="mono" style={{ fontWeight: 600, color: 'var(--error-text)' }}>{g.name}</span><Badge kind="error">{g.severity}</Badge></div>
              <div className="err-fix" style={{ borderTop: 0, paddingTop: 6, marginTop: 4 }}>{g.action}</div>
            </div>
          ))}
        </>
      )}
    </div>
  );
}

/* ---- Tester view -------------------------------------------------- */
function TesterView() {
  const run = useRunState();
  const [selected, setSelected] = useStateT(null);
  const [tab, setTab] = useStateT('output');
  const push = useToast();
  const prevSuspend = useRefT(false);

  // Test affordance (mirrors the app's own __graphStore expose pattern):
  // lets verification drive playback without relying on rAF in a
  // backgrounded iframe. Carries no secrets.
  useEffectT(() => { window.__run = run; window.__setSelected = setSelected; });

  // toast on suspension (rule 1: every consequential state change is visible)
  useEffectT(() => {
    if (run.runStatus === 'suspended' && !prevSuspend.current) {
      prevSuspend.current = true;
      push({ kind: 'error', title: 'Session suspended', msg: 'Capability gap: tool “fetch_prs” not found. Install it and Resume.' });
    }
    if (run.runStatus !== 'suspended') prevSuspend.current = false;
  }, [run.runStatus]);

  const marks = [
    { t: 2100, type: 'hitl', label: 'plan approval (HITL)' },
    { t: 14200, type: 'incident', label: 'capability violation' },
    { t: 15900, type: 'gap', label: 'capability gap → suspend' },
  ];

  const onRun = () => { run.restart(); setTimeout(() => run.toggle(), 30); push({ kind: 'info', title: 'Run started', msg: 'Executing framework “aria” in STANDARD mode.' }); };

  return (
    <>
      <aside className="rail-left">
        <TesterLeft run={run} onRun={onRun} />
      </aside>
      <div className="center">
        <BudgetBar budget={run.budget} tokens={run.tokens} cost={run.cost} />
        <GraphCanvas nodes={window.ARData.NODES} edges={window.ARData.EDGES} status={run.status} io={run.io} selected={selected} onSelect={setSelected} runStatus={run.runStatus} />
        <RunBanner runStatus={run.runStatus} gaps={run.gaps} />
        <Transport playhead={run.playhead} lastT={run.lastT} playing={run.playing} onToggle={run.toggle} onScrub={run.scrub} onRestart={run.restart} speed={run.speed} setSpeed={run.setSpeed} marks={marks} />
      </div>
      <aside className="rail-right">
        <div className="rail-tabs" role="tablist">
          <button className="rail-tab" role="tab" aria-selected={tab === 'output'} onClick={() => setTab('output')}>Output</button>
          <button className="rail-tab" role="tab" aria-selected={tab === 'inspector'} onClick={() => setTab('inspector')}>Inspector</button>
        </div>
        {tab === 'output'
          ? <OutputRail transcript={run.transcript} selected={selected} setSelected={setSelected} />
          : <div className="rail-body"><NodeInspector selected={selected} status={run.status} io={run.io} /></div>}
      </aside>
    </>
  );
}

Object.assign(window, { TesterView, useRunState, NodeInspector });
