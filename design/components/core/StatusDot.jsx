import React from 'react';
/** The tray's status tone as a dot. loading pulses (1.2s). */
export function StatusDot({ tone = 'off', size = 'md', style }) {
  const px = size === 'lg' ? 14 : size === 'sm' ? 8 : 10;
  const c = 'var(--tone-' + tone + ')';
  return <span role="img" aria-label={tone} style={{ display: 'inline-block', flex: 'none', width: px, height: px, borderRadius: '50%', background: c, animation: tone === 'loading' ? 'ndl-pulse 1.2s ease-in-out infinite' : 'none', verticalAlign: size === 'lg' ? 'middle' : '-1px', ...style }} />;
}
