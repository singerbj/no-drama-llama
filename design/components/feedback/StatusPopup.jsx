import React from 'react';
import { StatusDot } from '../core/StatusDot.jsx';
/** The on-screen popup (osd.rs): near-black card, tone dot, title + subtitle. Never takes focus. */
export function StatusPopup({ tone = 'paused', title, subtitle, animated = false, style }) {
  const c = 'var(--tone-' + tone + ')';
  return <div role="status" style={{ position: 'relative', display: 'inline-flex', alignItems: 'flex-start', gap: 12, minWidth: 300, padding: 18, borderRadius: 'var(--radius-xl)', background: 'var(--osd-bg)', border: '1px solid var(--osd-border)', boxShadow: 'var(--shadow-card)', overflow: 'hidden', fontFamily: 'var(--font-sans)', pointerEvents: 'none', animation: animated ? 'ndl-popup 6s ease-in-out infinite' : 'none', ...style }}>
    <StatusDot tone={tone} size="md" style={{ width: 12, height: 12, marginTop: 6 }} />
    <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
      <span style={{ color: '#fff', fontSize: 18, fontWeight: 600, lineHeight: 1.3 }}>{title}</span>
      {subtitle && <span style={{ color: 'var(--osd-sub)', fontSize: 14, lineHeight: 1.3 }}>{subtitle}</span>}
    </div>
  </div>;
}
