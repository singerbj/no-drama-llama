import React from 'react';
import { Icon } from '../core/Icon.jsx';
/** Native select (220px min). options = [[value, label], ...]. */
export function Select({ value, onChange, options = [], invalid = false, minWidth = 220, style, ...rest }) {
  return <span style={{ position: 'relative', display: 'inline-flex', alignItems: 'center', minWidth: 0 }}>
    <select value={value} onChange={onChange} style={{ ...{ fontFamily: 'var(--font-sans)', fontSize: 'var(--text-app)', color: 'var(--text)', background: 'var(--field)', border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'), borderRadius: 'var(--radius-sm)', padding: '5px 8px', minWidth: 0, outlineColor: invalid ? 'var(--danger)' : 'var(--accent)' }, minWidth, appearance: 'none', WebkitAppearance: 'none', paddingRight: 30, cursor: 'pointer', ...style }} {...rest}>{options.map(([v, l]) => <option key={String(v)} value={String(v)}>{l}</option>)}</select>
    <Icon name="chevron-down" size={16} color="var(--text-muted)" style={{ position: 'absolute', right: 9, pointerEvents: 'none' }} />
  </span>;
}
