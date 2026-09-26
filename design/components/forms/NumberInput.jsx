import React from 'react';
/** Numeric field (110px). Use for ports, thresholds, seconds. */
export function NumberInput({ value, onChange, min, max, step = 1, invalid = false, width = 110, style, ...rest }) {
  return <input type="number" value={value} onChange={onChange} min={min} max={max} step={step} aria-invalid={invalid} style={{ ...{ fontFamily: 'var(--font-sans)', fontSize: 'var(--text-app)', color: 'var(--text)', background: 'var(--field)', border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'), borderRadius: 'var(--radius-sm)', padding: '5px 8px', minWidth: 0, outlineColor: invalid ? 'var(--danger)' : 'var(--accent)' }, width, ...style }} {...rest} />;
}
