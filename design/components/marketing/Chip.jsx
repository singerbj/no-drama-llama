import React from 'react';
/** Outline pill for lists of names (launchers, GPUs). Optional leading tone dot. */
export function Chip({ tone, children, style }) {
  return <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, padding: '0.35rem 0.85rem', border: '1px solid var(--border)', borderRadius: 'var(--radius-pill)', background: 'var(--surface)', fontSize: '0.9rem', lineHeight: 1.3, ...style }}>{tone && <span style={{ width: 7, height: 7, borderRadius: '50%', background: 'var(--tone-' + tone + ')' }} />}{children}</span>;
}
