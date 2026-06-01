/* =====================================================================
   app.jsx — workbench shell: top chrome (brand + tab nav + tier chip),
   the active surface (Builder · Tester · Settings), and Tweaks wiring
   (accent hue · density · visual direction · node-diff style).
   ===================================================================== */
const { useState: useStateA, useEffect: useEffectA } = React;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "accent": "#2563eb",
  "direction": "refined",
  "density": "regular",
  "nodediff": "bar"
}/*EDITMODE-END*/;

const ACCENT_MAP = { '#2563eb': 'blue', '#0891b2': 'cyan', '#6d28d9': 'violet', '#4338ca': 'indigo' };

const TABS = [
  { id: 'builder', label: 'Builder' },
  { id: 'tester', label: 'Tester' },
  { id: 'settings', label: 'Settings' },
];

function Shell() {
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const [tab, setTab] = useStateA('tester');

  // apply tweak-driven theming to the document root so every surface
  // (incl. modals + toasts) picks up the tokens.
  useEffectA(() => {
    const r = document.documentElement;
    r.setAttribute('data-accent', ACCENT_MAP[t.accent] || 'blue');
    r.setAttribute('data-direction', t.direction);
    r.setAttribute('data-nodediff', t.nodediff);
    if (t.density === 'regular') r.removeAttribute('data-density'); else r.setAttribute('data-density', t.density);
  }, [t.accent, t.direction, t.nodediff, t.density]);

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="mark" />
          <span><span className="name">Agent Runtime</span> <span className="ver mono-micro">workbench · v0.1</span></span>
        </div>
        <div className="tabs" role="tablist">
          {TABS.map(x => (
            <button key={x.id} className="tab" role="tab" aria-selected={tab === x.id} onClick={() => setTab(x.id)}>
              <span className="tdot" />{x.label}
            </button>
          ))}
        </div>
        <span className="spacer" />
        <span className="tier-chip"><span className="swatch" style={{ background: 'var(--warn)' }} />Promoted</span>
        <span className="tier-chip mono-micro"><span className="swatch" style={{ background: 'var(--ok)' }} />key active</span>
      </header>

      <div className="workspace">
        {tab === 'builder' && <BuilderView />}
        {tab === 'tester' && <TesterView />}
        {tab === 'settings' && <SettingsView />}
      </div>

      <TweaksPanel title="Tweaks">
        <TweakSection label="Brand" />
        <TweakColor label="Accent" value={t.accent}
          options={['#2563eb', '#0891b2', '#6d28d9', '#4338ca']}
          onChange={(v) => setTweak('accent', v)} />
        <TweakSection label="Visual direction" />
        <TweakSelect label="Direction" value={t.direction}
          options={[{ value: 'bythebook', label: 'By-the-book' }, { value: 'refined', label: 'Refined (default)' }, { value: 'expressive', label: 'Expressive' }]}
          onChange={(v) => setTweak('direction', v)} />
        <TweakRadio label="Density" value={t.density}
          options={['compact', 'regular', 'comfortable']}
          onChange={(v) => setTweak('density', v)} />
        <TweakSection label="Node differentiation" />
        <TweakSelect label="Style" value={t.nodediff}
          options={[{ value: 'bar', label: 'Accent bar + glyph' }, { value: 'minimal', label: 'Minimal outline' }, { value: 'fill', label: 'Tinted fill' }, { value: 'shape', label: 'Distinct shapes' }]}
          onChange={(v) => setTweak('nodediff', v)} />
      </TweaksPanel>
    </div>
  );
}

function App() {
  return <ToastProvider><Shell /></ToastProvider>;
}

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);
