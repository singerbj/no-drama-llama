import React from 'react';
/** Settings group: 8px panel with an inline legend. Disabled dims to 55%. */
export function Fieldset({ legend, disabled = false, children, style }) {
  return <fieldset disabled={disabled} style={{ border: '1px solid var(--border)', borderRadius: 'var(--radius-md)', background: 'var(--surface)', margin: '0 0 16px', padding: '4px 16px 12px', opacity: disabled ? 0.55 : 1, minWidth: 0, ...style }}>
    {legend && <legend style={{ fontWeight: 600, padding: '0 6px', marginLeft: -6 }}>{legend}</legend>}
    {children}
  </fieldset>;
}
