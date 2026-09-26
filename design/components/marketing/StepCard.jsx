import React from 'react';
/** Numbered "How it works" step. */
export function StepCard({ n, title, children, style }) {
  return <div style={{ position: 'relative', padding: '1.5rem', border: '1px solid var(--border)', borderRadius: 'var(--radius-card)', background: 'var(--surface)', ...style }}>
    <span style={{ display: 'grid', placeItems: 'center', width: 36, height: 36, borderRadius: '50%', background: 'var(--accent-soft)', color: 'var(--accent)', fontWeight: 700, fontFamily: 'var(--font-display)' }}>{n}</span>
    <h3 style={{ margin: '0.9rem 0 0.4rem', fontFamily: 'var(--font-display)', fontWeight: 600, fontSize: 'var(--text-h3)', lineHeight: 1.15 }}>{title}</h3>
    <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.96rem', lineHeight: 1.6 }}>{children}</p>
  </div>;
}
