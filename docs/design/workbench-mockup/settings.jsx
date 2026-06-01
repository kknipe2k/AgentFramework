/* =====================================================================
   settings.jsx — Settings surface. Demonstrates the rule fixes:
     • tier: truthful labels, no "Promote to Promoted" (rule 8 / IRL #20)
     • API key: shows ACTIVE state, not an empty field (rule 2 / IRL #1)
     • budget cap: save gives feedback + states persistence (IRL #21/#22)
     • MCP add modal: above chrome, scrollable, complete labels (#23/#24)
   ===================================================================== */
const { useState: useStateS } = React;

function McpAddModal({ onClose }) {
  const push = useToast();
  const [transport, setTransport] = useStateS('stdio');
  return (
    <Modal title="Add MCP server" onClose={onClose}
      footer={<>
        <Button variant="ghost" onClick={onClose}>Cancel</Button>
        <Button onClick={() => push({ kind: 'info', title: 'Testing connection…', msg: 'Spawning server to verify initialize handshake.' })}>Test connection</Button>
        <Button variant="primary" onClick={() => { push({ kind: 'ok', title: 'Server added', msg: 'github is connected — 14 tools available.' }); onClose(); }}>Add server</Button>
      </>}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--s4)' }}>
        <div className="field"><span className="t-label">Server name</span><input className="input mono" defaultValue="github" placeholder="lowercase-dns-label" /></div>
        <div className="field"><span className="t-label">Transport</span>
          <div className="seg" style={{ alignSelf: 'flex-start' }}>
            <button className={transport === 'stdio' ? 'on' : ''} onClick={() => setTransport('stdio')}>stdio</button>
            <button className={transport === 'http' ? 'on' : ''} onClick={() => setTransport('http')}>http</button>
          </div>
        </div>
        {transport === 'stdio'
          ? <div className="field"><span className="t-label">Command</span><input className="input mono" defaultValue="npx -y @modelcontextprotocol/server-github" /></div>
          : <div className="field"><span className="t-label">Endpoint URL</span><input className="input mono" defaultValue="https://mcp.example.com/sse" /></div>}
        <div className="field"><span className="t-label">Auth secret (optional)</span><input className="input mono" type="password" defaultValue="ghp_xxxxxxxxxxxx" /><span className="muted t-small">Stored in the OS keychain — never written to the framework file.</span></div>
      </div>
    </Modal>
  );
}

function SettingsView() {
  const push = useToast();
  const [tier, setTier] = useStateS('promoted');
  const [cap, setCap] = useStateS('0.30');
  const [showModal, setShowModal] = useStateS(false);
  const [servers, setServers] = useStateS([{ name: 'filesystem', status: 'connected', tools: 6 }, { name: 'github', status: 'disconnected', tools: 0 }]);

  const changeTier = (t) => { if (t === tier) return; setTier(t); push({ kind: 'ok', title: `Tier set to ${t === 'promoted' ? 'Promoted' : 'Novice'}`, msg: t === 'promoted' ? 'Full capability surface; L1 still narrows.' : 'Curated allowlist — read + HTTPS only.' }); };
  const saveCap = () => push({ kind: 'ok', title: 'Budget cap saved', msg: `Hard stop at $${(+cap).toFixed(2)} — persists across restarts.` });

  return (
    <div className="center scroll-y">
      <div className="settings">
        <h2 className="t-title" style={{ margin: '0 0 4px' }}>Settings</h2>
        <p className="muted" style={{ margin: 0, fontSize: 13 }}>Single-session · Windows · v0.1</p>

        {/* Tier — truthful labels (IRL #20) */}
        <div className="panel" style={{ padding: 'var(--s4) var(--s5)' }}>
          <div className="set-row" style={{ borderBottom: 0, padding: 0 }}>
            <div className="set-label">
              <div className="t">Capability tier</div>
              <div className="d">{tier === 'promoted' ? 'Promoted (active) — full capability surface; the L1 enforcer still narrows per agent.' : 'Novice (active) — curated allowlist: read + HTTPS-only network.'}</div>
            </div>
            <div className="seg">
              <button className={tier === 'novice' ? 'on' : ''} onClick={() => changeTier('novice')}>Novice</button>
              <button className={tier === 'promoted' ? 'on' : ''} onClick={() => changeTier('promoted')}>Promoted</button>
            </div>
          </div>
        </div>

        {/* API key — ACTIVE state, not empty (rule 2 / IRL #1) */}
        <div className="panel" style={{ padding: 'var(--s4) var(--s5)' }}>
          <div className="set-row" style={{ borderBottom: 0, padding: 0 }}>
            <div className="set-label">
              <div className="t" style={{ display: 'flex', alignItems: 'center', gap: 8 }}><Icon name="key" size={15} />Anthropic API key</div>
              <div className="d">Loaded from the OS keychain at startup.</div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <input className="input mono input--active" style={{ width: 200 }} value="sk-ant-•••••••••••••3f9a" readOnly />
              <Badge kind="ok" dot>active</Badge>
              <Button sm onClick={() => push({ kind: 'info', title: 'Replace key', msg: 'Enter a new key to override the stored one.' })}>Replace</Button>
            </div>
          </div>
        </div>

        {/* Budget cap — save feedback + persistence (IRL #21/#22) */}
        <div className="panel" style={{ padding: 'var(--s4) var(--s5)' }}>
          <div className="set-row" style={{ borderBottom: 0, padding: 0 }}>
            <div className="set-label">
              <div className="t">Budget cap (USD)</div>
              <div className="d">Hard stop on session spend. Warn at 50%, downshift at 75%, suspend at 90%.</div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span className="mono muted">$</span>
              <input className="input mono" style={{ width: 96 }} value={cap} onChange={e => setCap(e.target.value)} />
              <Button sm variant="primary" onClick={saveCap}>Save</Button>
            </div>
          </div>
        </div>

        {/* MCP servers — progressive disclosure + add modal */}
        <Panel title="MCP servers" defaultOpen right={<Badge kind="muted">{servers.filter(s => s.status === 'connected').length} connected</Badge>}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8, paddingTop: 12 }}>
            {servers.map(s => (
              <div key={s.name} className="between" style={{ padding: '9px 11px', border: '1px solid var(--border)', borderRadius: 'var(--r-sm)' }}>
                <span style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
                  <span style={{ width: 9, height: 9, borderRadius: '50%', background: s.status === 'connected' ? 'var(--ok)' : 'var(--text-3)' }} />
                  <span className="mono" style={{ fontWeight: 600, fontSize: 12.5 }}>{s.name}</span>
                  <span className="src src--mcp">{s.tools} tools</span>
                </span>
                <span style={{ display: 'flex', gap: 6 }}>
                  <Badge kind={s.status === 'connected' ? 'ok' : 'muted'}>{s.status}</Badge>
                  <Button sm variant="ghost" onClick={() => { setServers(srv => srv.filter(x => x.name !== s.name)); push({ kind: 'ok', title: 'Server removed', msg: `${s.name} uninstalled.` }); }}>Remove</Button>
                </span>
              </div>
            ))}
            <Button icon="plus" onClick={() => setShowModal(true)} style={{ alignSelf: 'flex-start', marginTop: 4 }}>Add server</Button>
          </div>
        </Panel>
      </div>
      {showModal && <McpAddModal onClose={() => setShowModal(false)} />}
    </div>
  );
}

Object.assign(window, { SettingsView, McpAddModal });
