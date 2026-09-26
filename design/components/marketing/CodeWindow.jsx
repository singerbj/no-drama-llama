import React from 'react';
/** Dark code panel with a filename bar. Always dark, in both themes. */
export function CodeWindow({ filename, children, style }) {
  return <div style={{ background: 'var(--code-bg)', color: 'var(--code-text)', borderRadius: 'var(--radius-card)', boxShadow: 'var(--shadow-card)', overflow: 'hidden', minWidth: 0, ...style }}>
    {filename && <div style={{ padding: '0.6rem 1rem', font: '0.8rem var(--font-mono)', color: 'var(--code-muted)', borderBottom: '1px solid var(--wool-700)' }}>{filename}</div>}
    <pre style={{ margin: 0, padding: '1.25rem', overflowX: 'auto', font: '0.86rem/1.65 var(--font-mono)' }}><code>{children}</code></pre>
  </div>;
}
