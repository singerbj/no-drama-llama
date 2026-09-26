import React from 'react';
import { StatusDot } from '../core/StatusDot.jsx';
/** Top of the settings window: big tone dot, status line, chat URL, actions. */
export function StatusHeader({ tone = 'running', title, subtitle, actions, style }) {
  return <header style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '14px 20px', background: 'var(--surface)', borderBottom: '1px solid var(--border)', ...style }}>
    <StatusDot tone={tone} size="lg" />
    <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }}>
      <strong style={{ fontSize: 'var(--text-status)', fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{title}</strong>
      {subtitle && <small style={{ color: 'var(--text-muted)', fontSize: 'var(--text-hint)' }}>{subtitle}</small>}
    </div>
    {actions && <div style={{ display: 'flex', gap: 8 }}>{actions}</div>}
  </header>;
}
