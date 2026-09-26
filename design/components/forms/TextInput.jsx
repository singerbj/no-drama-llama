import React from 'react';
/** Single-line text field (260px default). Invalid = tomato border. */
export function TextInput({ value, onChange, placeholder, invalid = false, width = 260, style, ...rest }) {
  return <input type="text" value={value} onChange={onChange} placeholder={placeholder} aria-invalid={invalid} spellCheck={false} autoComplete="off" style={{ ...{ fontFamily: 'var(--font-sans)', fontSize: 'var(--text-app)', color: 'var(--text)', background: 'var(--field)', border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'), borderRadius: 'var(--radius-sm)', padding: '5px 8px', minWidth: 0, outlineColor: invalid ? 'var(--danger)' : 'var(--accent)' }, width, ...style }} {...rest} />;
}
