import React from 'react';
/** Multi-line list field, one entry per line, code font 13px. */
export function TextArea({ value, onChange, rows = 3, invalid = false, style, ...rest }) {
  return <textarea value={value} onChange={onChange} rows={rows} spellCheck={false} style={{ ...{ fontFamily: 'var(--font-sans)', fontSize: 'var(--text-app)', color: 'var(--text)', background: 'var(--field)', border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'), borderRadius: 'var(--radius-sm)', padding: '5px 8px', minWidth: 0, outlineColor: invalid ? 'var(--danger)' : 'var(--accent)' }, width: '100%', resize: 'vertical', fontFamily: 'var(--font-mono)', fontSize: 'var(--text-code)', ...style }} {...rest} />;
}
