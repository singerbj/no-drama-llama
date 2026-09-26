import React from 'react';
import { Button } from '../core/Button.jsx';
/** 180px left nav of tab buttons. items = [{ id, title }]. */
export function SideNav({ items = [], active, onSelect, style }) {
  return <nav style={{ width: 'var(--nav-width)', flex: 'none', padding: '12px 8px', display: 'flex', flexDirection: 'column', gap: 2, borderRight: '1px solid var(--border)', ...style }}>
    {items.map((it) => <Button key={it.id} variant="tab" active={it.id === active} onClick={() => onSelect && onSelect(it.id)}>{it.title}</Button>)}
  </nav>;
}
