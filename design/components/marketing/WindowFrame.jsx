import React from 'react';
/** Faux app window for marketing demos: three grey dots + mono caption. */
export function WindowFrame({ caption, children, style }) {
  return <div style={{ background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-card)', boxShadow: 'var(--shadow-card)', overflow: 'hidden', ...style }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '0.7rem 1rem', borderBottom: '1px solid var(--border)', background: 'var(--bg-alt)' }}>
      {[0, 1, 2].map((i) => <span key={i} style={{ width: 10, height: 10, borderRadius: '50%', background: 'var(--border-strong)' }} />)}
      {caption && <em style={{ marginLeft: 'auto', fontStyle: 'normal', fontSize: '0.8rem', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>{caption}</em>}
    </div>
    {children}
  </div>;
}
