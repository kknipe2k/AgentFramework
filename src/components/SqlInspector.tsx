import { useState } from 'react';
import { invokeQuerySessionDb } from '../lib/ipc';
import { unwrapCmdError } from '../lib/ipc';

/**
 * Renderer-side ad-hoc SQL inspector. The user pastes a SELECT
 * statement and sees the resulting rows as a table. The drone-side
 * validator (`runtime_drone::vdr::is_select_only`) is the security
 * boundary — DDL/DML/PRAGMA + compound statements are rejected before
 * SQL touches the database.
 *
 * Stage E ships read-only SELECT only; M03+ may extend with EXPLAIN /
 * parameterized queries / WITH RECURSIVE.
 */
export function SqlInspector(): JSX.Element {
  const [sql, setSql] = useState('SELECT * FROM signals LIMIT 10;');
  const [rows, setRows] = useState<Record<string, unknown>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleExecute(): Promise<void> {
    if (loading) {
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const result = await invokeQuerySessionDb(sql);
      setRows(result);
    } catch (e) {
      // Always log the structured object — `unwrapCmdError` collapses to
      // a human string for display, but DevTools needs the raw shape per
      // docs/gotchas.md #30.
      console.error('SQL inspector error:', e);
      setError(unwrapCmdError(e));
      setRows([]);
    } finally {
      setLoading(false);
    }
  }

  const firstRow = rows[0];
  const columns = firstRow !== undefined ? Object.keys(firstRow) : [];

  return (
    <section aria-label="SQL inspector" className="sql-inspector" data-testid="sql-inspector">
      <header className="sql-inspector__header">
        <h2 className="sql-inspector__title">SQL inspector</h2>
      </header>
      <label className="sql-inspector__label">
        <span className="sql-inspector__label-text">SQL query</span>
        <textarea
          className="sql-inspector__textarea"
          aria-label="SQL query"
          value={sql}
          onChange={(e) => setSql(e.target.value)}
          rows={3}
          disabled={loading}
        />
      </label>
      <button
        type="button"
        className="sql-inspector__execute"
        onClick={() => void handleExecute()}
        disabled={loading || sql.trim().length === 0}
      >
        {loading ? 'Executing…' : 'Execute'}
      </button>
      {error !== null && (
        <p className="sql-inspector__error" role="alert">
          {error}
        </p>
      )}
      {rows.length > 0 && (
        <table className="sql-inspector__results">
          <thead>
            <tr>
              {columns.map((c) => (
                <th key={c} scope="col">
                  {c}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr key={i}>
                {columns.map((c) => (
                  <td key={c}>{formatCell(row[c])}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function formatCell(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  // Primitive — number / boolean / string. String coercion is safe here
  // because we've ruled out objects (which would default-stringify to
  // '[object Object]' per docs/gotchas.md #30).
  if (typeof value === 'string') {
    return value;
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return value.toString();
  }
  return JSON.stringify(value);
}
