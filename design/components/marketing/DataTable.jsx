import React from 'react';
/** Bordered table with caption and caps headers. rows = arrays of cells. */
export function DataTable({ caption, columns = [], rows = [], style }) {
  const cell = { textAlign: 'left', padding: '0.75rem 1.25rem', borderBottom: '1px solid var(--border)', verticalAlign: 'top' };
  return <div style={{ background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-card)', overflowX: 'auto', ...style }}>
    <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.95rem' }}>
      {caption && <caption style={{ textAlign: 'left', padding: '1rem 1.25rem 0.25rem', fontWeight: 600, color: 'var(--text-muted)', fontSize: '0.85rem' }}>{caption}</caption>}
      <thead><tr>{columns.map((c, i) => <th key={i} style={{ ...cell, fontSize: '0.8rem', textTransform: 'uppercase', letterSpacing: '0.06em', color: 'var(--text-muted)' }}>{c}</th>)}</tr></thead>
      <tbody>{rows.map((r, i) => <tr key={i}>{r.map((c, j) => <td key={j} style={{ ...cell, borderBottom: i === rows.length - 1 ? 0 : cell.borderBottom }}>{c}</td>)}</tr>)}</tbody>
    </table>
  </div>;
}
