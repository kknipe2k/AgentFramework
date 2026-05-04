import { useState } from 'react';

interface Props {
  onSave: (key: string) => Promise<void>;
}

export function SetupPanel({ onSave }: Props): JSX.Element {
  const [key, setKey] = useState('');
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  async function handleSave(): Promise<void> {
    setSaving(true);
    try {
      await onSave(key);
      setKey('');
      setSaved(true);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section aria-label="api key setup">
      <label>
        Anthropic API key:
        <input
          type="password"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="sk-ant-..."
          disabled={saving}
        />
      </label>{' '}
      <button onClick={() => void handleSave()} disabled={saving || key.length < 10}>
        {saving ? 'Saving…' : 'Save key'}
      </button>
      {saved && <span aria-label="saved"> ✓ stored in OS keychain</span>}
    </section>
  );
}
