import React from 'react';
/** Tone-tinted notice with an optional action (default tone: paused). */
export function Banner({ tone = 'paused', action, children, style }) {
  const c = 'var(--tone-' + tone + ')';
  return <div role="status" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, padding: '10px 14px', marginBottom: 14, borderRadius: 'var(--radius-md)', border: '1px solid ' + c, background: 'color-mix(in srgb, ' + c + ' 12%, var(--surface))' }}>
    <span>{children}</span>{action}
  </div>;
}
