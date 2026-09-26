import React from 'react';
import { Icon } from '../core/Icon.jsx';
/** Feature tile: accent line icon, display-font title, muted copy. */
export function FeatureCard({ icon, title, children, tone = 'running', style }) {
  return <div style={{ padding: '1.4rem', background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-card)', ...style }}>
    {icon && <Icon name={icon} size={28} color={'var(--tone-' + tone + ')'} />}
    <h3 style={{ margin: '0.8rem 0 0.35rem', fontFamily: 'var(--font-display)', fontWeight: 600, fontSize: 'var(--text-h3)', lineHeight: 1.15, letterSpacing: '-0.02em' }}>{title}</h3>
    <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.96rem', lineHeight: 1.6 }}>{children}</p>
  </div>;
}
