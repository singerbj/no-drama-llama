import React from 'react';
/** Inverted pill notice. floating = fixed 72px above the bottom, centred. */
export function Toast({ error = false, floating = false, children, style }) {
  const pos = floating ? { position: 'fixed', bottom: 72, left: '50%', transform: 'translateX(-50%)', animation: 'ndl-toast 0.15s var(--ease-out)' } : { display: 'inline-block' };
  return <div role="status" style={{ ...pos, maxWidth: 'min(640px, 90vw)', padding: '10px 16px', borderRadius: 'var(--radius-md)', background: error ? 'var(--danger)' : 'var(--text)', color: error ? 'var(--on-danger)' : 'var(--bg)', boxShadow: 'var(--shadow-toast)', ...style }}>{children}</div>;
}
