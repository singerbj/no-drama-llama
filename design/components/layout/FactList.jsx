import React from 'react';
/** Two-column definition list panel (170px term column). items = [[term, value], ...]. */
export function FactList({ items = [], style }) {
  return <dl style={{ display: 'grid', gridTemplateColumns: '170px 1fr', gap: '8px 16px', margin: '0 0 8px', padding: '14px 16px', background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-md)', ...style }}>
    {items.map(([k, v], i) => <React.Fragment key={i}><dt style={{ color: 'var(--text-muted)' }}>{k}</dt><dd style={{ margin: 0, overflowWrap: 'anywhere' }}>{v}</dd></React.Fragment>)}
  </dl>;
}
