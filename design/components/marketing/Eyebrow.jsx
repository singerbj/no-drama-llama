import React from 'react';
/** Mint pill label above a hero headline. */
export function Eyebrow({ children, style }) {
  return <span style={{ display: 'inline-block', fontSize: '0.85rem', fontWeight: 600, color: 'var(--accent)', background: 'var(--accent-soft)', padding: '0.3rem 0.8rem', borderRadius: 'var(--radius-pill)', ...style }}>{children}</span>;
}
