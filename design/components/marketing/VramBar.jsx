import React from 'react';
/** Stacked VRAM bars that grow in. rows = [{ label, segments: [{ label, tone, pct }] }]. */
export function VramBar({ rows = [], style }) {
  return <div style={{ padding: '1.25rem 1.25rem 0.5rem', display: 'grid', gap: '0.8rem', ...style }}>
    {rows.map((r, i) => <div key={i} style={{ display: 'grid', gridTemplateColumns: '4.5rem 1fr', alignItems: 'center', gap: '0.75rem', fontSize: '0.85rem' }}>
      <span style={{ color: 'var(--text-muted)' }}>{r.label}</span>
      <div style={{ display: 'flex', height: 30, background: 'var(--bg-alt)', border: '1px solid var(--border)', borderRadius: 'var(--radius-md)', overflow: 'hidden' }}>
        {r.segments.map((s, j) => <div key={j} style={{ width: s.pct + '%', display: 'flex', alignItems: 'center', height: '100%', paddingInline: '0.7rem', fontSize: '0.78rem', fontWeight: 600, color: 'var(--on-tone)', whiteSpace: 'nowrap', background: 'var(--tone-' + s.tone + ')', transformOrigin: 'left', animation: 'ndl-grow 1.2s var(--ease-out) both', animationDelay: (i * 0.3) + 's' }}>{s.label}</div>)}
      </div>
    </div>)}
  </div>;
}
