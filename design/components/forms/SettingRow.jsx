import React from 'react';
/** One setting: label + muted hint on the left, control on the right (or below when stacked). */
export function SettingRow({ label, hint, stacked = false, htmlFor, last = false, children }) {
  return <div style={{ display: 'flex', flexDirection: stacked ? 'column' : 'row', alignItems: stacked ? 'stretch' : 'center', justifyContent: 'space-between', gap: stacked ? 6 : 16, padding: '10px 0', borderBottom: last ? 'none' : '1px solid var(--border)' }}>
    <label htmlFor={htmlFor} style={{ display: 'flex', flexDirection: 'column', minWidth: 0 }}>{label}{hint && <small style={{ color: 'var(--text-muted)', fontSize: 'var(--text-hint)' }}>{hint}</small>}</label>
    {children}
  </div>;
}
