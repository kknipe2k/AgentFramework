/* =====================================================================
   components.jsx — shared primitives (icons, badges, buttons, panels,
   toasts, modals), the KIND meta map, and the run-state reducer that
   folds the scripted event stream into derived graph + transcript state.
   Exposed on window for the other babel files.
   ===================================================================== */
const { useState, useEffect, useRef, useMemo, useCallback, createContext, useContext } = React;

/* ---- Kind metadata ------------------------------------------------ */
const KIND = {
  agent: { glyph: 'A',  label: 'Agent', c: 'var(--kind-agent)',  t: 'var(--kind-agent-text)',  bg: 'var(--kind-agent-bg)' },
  tool:  { glyph: 'T',  label: 'Tool',  c: 'var(--kind-tool)',   t: 'var(--kind-tool-text)',   bg: 'var(--kind-tool-bg)' },
  skill: { glyph: 'S',  label: 'Skill', c: 'var(--kind-skill)',  t: 'var(--kind-skill-text)',  bg: 'var(--kind-skill-bg)' },
  hook:  { glyph: 'H',  label: 'Hook',  c: 'var(--kind-hook)',   t: 'var(--kind-hook-text)',   bg: 'var(--kind-hook-bg)' },
  hitl:  { glyph: '\u2016', label: 'HITL', c: 'var(--kind-hitl)', t: 'var(--kind-hitl-text)',  bg: 'var(--kind-hitl-bg)' },
  gap:   { glyph: '!',  label: 'Gap',   c: 'var(--kind-gap)',    t: 'var(--kind-gap-text)',    bg: 'var(--kind-gap-bg)' },
  mcp:   { glyph: 'M',  label: 'MCP',   c: 'var(--kind-mcp)',    t: 'var(--kind-mcp-text)',    bg: 'var(--kind-mcp-bg)' },
};
const kindVars = (kind) => ({ '--kc': KIND[kind].c, '--kt': KIND[kind].t, '--kbg': KIND[kind].bg });

/* ---- Icons (simple geometric SVG) -------------------------------- */
function Icon({ name, className, size = 16 }) {
  const p = { width: size, height: size, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round', strokeLinejoin: 'round', className };
  switch (name) {
    case 'play':    return <svg {...p} fill="currentColor" stroke="none"><path d="M8 5v14l11-7z" /></svg>;
    case 'pause':   return <svg {...p} fill="currentColor" stroke="none"><rect x="6" y="5" width="4" height="14" rx="1" /><rect x="14" y="5" width="4" height="14" rx="1" /></svg>;
    case 'restart': return <svg {...p}><path d="M3 12a9 9 0 1 0 3-6.7" /><path d="M3 4v4h4" /></svg>;
    case 'chevron': return <svg {...p}><path d="M9 6l6 6-6 6" /></svg>;
    case 'close':   return <svg {...p}><path d="M6 6l12 12M18 6L6 18" /></svg>;
    case 'check':   return <svg {...p}><path d="M5 12l4.5 4.5L19 7" /></svg>;
    case 'x':       return <svg {...p}><path d="M6 6l12 12M18 6L6 18" /></svg>;
    case 'plus':    return <svg {...p}><path d="M12 5v14M5 12h14" /></svg>;
    case 'alert':   return <svg {...p}><path d="M12 3l9 16H3z" /><path d="M12 10v4M12 17h.01" /></svg>;
    case 'info':    return <svg {...p}><circle cx="12" cy="12" r="9" /><path d="M12 11v5M12 8h.01" /></svg>;
    case 'fit':     return <svg {...p}><path d="M4 9V5a1 1 0 0 1 1-1h4M20 9V5a1 1 0 0 0-1-1h-4M4 15v4a1 1 0 0 0 1 1h4M20 15v4a1 1 0 0 1-1 1h-4" /></svg>;
    case 'save':    return <svg {...p}><path d="M5 4h11l3 3v13H5z" /><path d="M8 4v5h7" /><rect x="8" y="13" width="8" height="5" /></svg>;
    case 'bolt':    return <svg {...p} fill="currentColor" stroke="none"><path d="M13 2L4 14h6l-1 8 9-12h-6z" /></svg>;
    case 'key':     return <svg {...p}><circle cx="8" cy="8" r="4" /><path d="M11 11l8 8M16 16l2-2M18 18l2-2" /></svg>;
    case 'dot':     return <svg {...p} fill="currentColor" stroke="none"><circle cx="12" cy="12" r="4" /></svg>;
    default: return null;
  }
}

/* ---- Glyph chip --------------------------------------------------- */
function Glyph({ kind, size = 22 }) {
  return <span className="node__glyph" style={{ ...kindVars(kind), width: size, height: size, flexBasis: size, fontSize: size * 0.52 }}>{KIND[kind].glyph}</span>;
}

/* ---- Badge / Button ----------------------------------------------- */
function Badge({ kind = 'muted', dot, mono, children }) {
  return <span className={`badge badge--${kind}${mono ? ' badge--mono' : ''}`}>{dot && <span className="g" />}{children}</span>;
}
function Button({ variant, sm, icon, children, ...rest }) {
  return <button className={`btn${variant ? ' btn--' + variant : ''}${sm ? ' btn--sm' : ''}`} {...rest}>{icon && <Icon name={icon} size={sm ? 13 : 14} className="ic" />}{children}</button>;
}

/* ---- Collapsible panel (progressive disclosure; rule 3) ----------- */
function Panel({ title, defaultOpen = false, right, children }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className={`panel${open ? ' panel--open' : ''}`} style={{ marginBottom: 'var(--s3)' }}>
      <button className="panel__head" onClick={() => setOpen(o => !o)} aria-expanded={open}>
        <Icon name="chevron" className="chevron" size={12} />
        <span className="panel__title">{title}</span>
        <span className="spacer" />
        {right}
      </button>
      {open && <div className="panel__body">{children}</div>}
    </div>
  );
}

/* ---- Toast system ------------------------------------------------- */
const ToastCtx = createContext(() => {});
const useToast = () => useContext(ToastCtx);
function ToastProvider({ children }) {
  const [toasts, setToasts] = useState([]);
  const push = useCallback((t) => {
    const id = Math.random().toString(36).slice(2);
    setToasts(ts => [...ts, { id, kind: 'info', ttl: 4200, ...t }]);
    setTimeout(() => setToasts(ts => ts.filter(x => x.id !== id)), (t.ttl ?? 4200));
  }, []);
  const dismiss = (id) => setToasts(ts => ts.filter(x => x.id !== id));
  const icon = k => k === 'ok' ? 'check' : k === 'error' ? 'alert' : k === 'warn' ? 'alert' : 'info';
  return (
    <ToastCtx.Provider value={push}>
      {children}
      <div className="toast-stack">
        {toasts.map(t => (
          <div key={t.id} className={`toast toast--${t.kind}`} role="status">
            <span className="toast__ic" style={{ color: `var(--${t.kind === 'ok' ? 'ok' : t.kind === 'error' ? 'error' : t.kind === 'warn' ? 'warn' : 'accent'})` }}><Icon name={icon(t.kind)} size={18} /></span>
            <div className="toast__body"><div className="toast__title">{t.title}</div>{t.msg && <div className="toast__msg">{t.msg}</div>}</div>
            <button className="toast__x" onClick={() => dismiss(t.id)} aria-label="Dismiss"><Icon name="close" size={14} /></button>
          </div>
        ))}
      </div>
    </ToastCtx.Provider>
  );
}

/* ---- Modal -------------------------------------------------------- */
function Modal({ title, onClose, children, footer, width }) {
  useEffect(() => {
    const h = e => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', h); return () => window.removeEventListener('keydown', h);
  }, [onClose]);
  return (
    <div className="overlay" onMouseDown={e => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="modal" style={width ? { width } : undefined} role="dialog" aria-modal="true" aria-label={title}>
        <div className="modal__head"><h3>{title}</h3><button className="toast__x" onClick={onClose} aria-label="Close"><Icon name="close" size={16} /></button></div>
        <div className="modal__body">{children}</div>
        {footer && <div className="modal__foot">{footer}</div>}
      </div>
    </div>
  );
}

/* =====================================================================
   Run-state reducer — folds events[0..index] into derived state.
   Reversible: always recomputed from scratch so scrubbing works.
   ===================================================================== */
function targetNode(item) {
  if (item._node) return item._node;
  const ev = item.ev;
  if (ev.agent_id && ['agent_spawned', 'agent_complete', 'agent_error', 'stream_text', 'decision_record'].includes(ev.type)) return ev.agent_id;
  if (ev.tool_name) return ev.tool_name;
  if (ev.skill_name) return ev.skill_name;
  if (ev.hook_id) return ev.hook_id;
  return null;
}

function foldRun(events, index) {
  const status = {};            // nodeId -> status string
  const io = {};                // toolId -> { input, output, duration, tokensIn, tokensOut }
  const activeEdges = {};       // "from>to" -> true (currently flowing)
  const transcript = [];
  let budget = { spent: 0, cap: 0.30, status: 'ok', percent: 0 };
  let tokens = 0, cost = 0;
  let runStatus = 'idle';
  let gaps = [];
  let lastT = 0;
  const fmtT = ms => `${(ms / 1000).toFixed(1)}s`;

  for (let i = 0; i <= index && i < events.length; i++) {
    const item = events[i]; const ev = item.ev; lastT = item.t;
    const node = targetNode(item);
    const push = (e) => transcript.push({ id: i, t: item.t, tStr: fmtT(item.t), ...e });

    switch (ev.type) {
      case 'session_start': runStatus = 'running'; break;
      case 'agent_spawned':
        status[ev.agent_id] = 'active';
        push({ kind: 'event', node: ev.agent_id, who: ev.agent_name, text: ev.parent_id ? `Spawned by ${ev.parent_id}` : 'Session root spawned', spawn: true, narrowed: ev.narrowed_from });
        break;
      case 'agent_complete': status[ev.agent_id] = 'complete'; break;
      case 'agent_error': status[ev.agent_id] = 'error'; break;
      case 'stream_text':
        push({ kind: 'reply', node: ev.agent_id, who: ev.agent_id, text: ev.text });
        break;
      case 'tool_invoked':
        status[ev.tool_name] = 'active';
        io[ev.tool_name] = { input: ev.input, source: ev.source, server: ev.server };
        push({ kind: 'tool', node: ev.tool_name, who: ev.tool_name, phase: 'call', source: ev.source, server: ev.server, code: JSON.stringify(ev.input) });
        break;
      case 'tool_result':
        status[ev.tool_name] = 'complete';
        io[ev.tool_name] = { ...(io[ev.tool_name] || {}), output: ev.output, duration: ev.duration_ms, tokensIn: ev.tokens_in, tokensOut: ev.tokens_out };
        push({ kind: 'tool', node: ev.tool_name, who: ev.tool_name, phase: 'result', code: String(ev.output), duration: ev.duration_ms, tokensIn: ev.tokens_in });
        break;
      case 'tool_error': status[ev.tool_name] = 'error'; break;
      case 'skill_loaded':
        status[node] = 'loaded';
        push({ kind: 'event', node, who: ev.skill_name, text: `Skill loaded into context · ${ev.mode || 'manual'}` });
        break;
      case 'plan_created':
        push({ kind: 'event', node: 'planner', who: 'planner', text: `Plan “${ev.title}” created — ${ev.task_count} tasks${ev.approval_required ? ', approval required' : ''}` });
        break;
      case 'hitl_requested':
        status[node] = 'blocked';
        push({ kind: 'hitl', node, who: 'human gate', text: ev.question, options: ev.options });
        break;
      case 'hitl_resolved':
        status[node] = 'complete';
        push({ kind: 'event', node, who: 'human gate', text: `Resolved: ${ev.choice} · ${ev.duration_ms}ms` });
        break;
      case 'plan_approved': break;
      case 'verify_started':
        status[ev.hook_id] = 'active';
        push({ kind: 'event', node: ev.hook_id, who: ev.hook_id, text: `Hook started · ${ev.firing_point}${ev.level ? ' · ' + ev.level : ''}` });
        break;
      case 'verify_passed':
        status[ev.hook_id] = 'complete';
        push({ kind: 'event', node: ev.hook_id, who: ev.hook_id, text: `Passed — ${ev.output_preview}`, duration: ev.duration_ms });
        break;
      case 'verify_failed': status[ev.hook_id] = 'error'; break;
      case 'capability_violation':
        status[node] = 'violation';
        push({ kind: 'incident', node, who: ev.agent_id, text: ev.requested_action, detail: ev.declared_scope, capKind: ev.capability_kind });
        break;
      case 'tool_missing':
        status[ev.tool_name] = 'gap';
        gaps.push({ name: ev.tool_name, severity: ev.severity, action: ev.suggested_action, agent: ev.agent_id });
        push({ kind: 'gap', node: ev.tool_name, who: ev.agent_id, text: `Tool “${ev.tool_name}” not found`, detail: ev.suggested_action, severity: ev.severity });
        if (ev.severity === 'critical') runStatus = 'suspended';
        break;
      case 'token_usage':
        tokens += ev.input + ev.output; cost = +(cost + ev.cost_usd).toFixed(3); budget.spent = cost;
        break;
      case 'budget_warn':
        budget = { ...budget, spent: ev.spent_usd, cap: ev.cap_usd, percent: ev.percent, status: 'warn' };
        push({ kind: 'event', node: null, who: 'budget', text: `Budget at ${ev.percent}% of cap ($${ev.spent_usd.toFixed(2)} / $${ev.cap_usd.toFixed(2)})`, tone: 'warn' });
        break;
      case 'budget_suspended':
        budget = { ...budget, spent: ev.spent_usd, cap: ev.cap_usd, status: 'suspended' };
        break;
      default: break;
    }
  }
  budget.percent = budget.cap ? Math.min(100, Math.round((budget.spent / budget.cap) * 100)) : 0;
  return { status, io, activeEdges, transcript, budget, tokens, cost, runStatus, gaps, lastT };
}

Object.assign(window, { KIND, kindVars, Icon, Glyph, Badge, Button, Panel, ToastProvider, useToast, Modal, foldRun, targetNode });
