import React from 'react';
/** Horizontal tone timeline: Running → Paused → Loading → Running. */
export function StateTimeline({ steps = [], style }) {
  return <ol style={{ listStyle: 'none', margin: 0, padding: '1rem 1.25rem 1.25rem', display: 'grid', gridTemplateColumns: 'repeat(' + steps.length + ', minmax(0,1fr))', gap: '0.5rem', fontSize: '0.8rem', ...style }}>
    {steps.map((s, i) => { const c = 'var(--tone-' + s.tone + ')'; return <li key={i} style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem', padding: '0.65rem', borderRadius: 'var(--radius-lg)', background: 'var(--bg-alt)' }}>
      <span style={{ width: 10, height: 10, borderRadius: '50%', background: c, animation: s.tone === 'loading' ? 'ndl-pulse 1.2s ease-in-out infinite' : 'none' }} />
      <strong style={{ fontSize: '0.82rem', lineHeight: 1.25, marginTop: 4 }}>{s.title}</strong>
      <em style={{ fontStyle: 'normal', color: 'var(--text-muted)' }}>{s.note}</em>
    </li>; })}
  </ol>;
}
