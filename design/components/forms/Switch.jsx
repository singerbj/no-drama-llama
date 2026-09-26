import React from 'react';
/** 40×22 toggle switch; mint when on. */
export function Switch({ checked = false, onChange, disabled = false, id, style }) {
  return <button type="button" role="switch" id={id} aria-checked={checked} disabled={disabled} onClick={() => onChange && onChange(!checked)} style={{ flex: 'none', width: 40, height: 22, borderRadius: 11, border: 'none', padding: 0, margin: 0, position: 'relative', cursor: disabled ? 'default' : 'pointer', opacity: disabled ? 0.55 : 1, background: checked ? 'var(--accent)' : 'var(--border-strong)', transition: 'background var(--dur-fast)', ...style }}>
    <span style={{ position: 'absolute', top: 3, left: 3, width: 16, height: 16, borderRadius: '50%', background: '#fff', boxShadow: 'var(--shadow-knob)', transform: checked ? 'translateX(18px)' : 'none', transition: 'transform var(--dur-fast)' }} />
  </button>;
}
