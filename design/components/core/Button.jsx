import React, { useState } from 'react';
function useHover(){const [h,setH]=useState(false);return [h,{onMouseEnter:()=>setH(true),onMouseLeave:()=>setH(false)}];}
/** Settings-window button: default, primary, danger, or a side-nav tab. Values from ui/src/style.css. */
export function Button({ variant = 'default', active = false, disabled = false, type = 'button', onClick, children, style, ...rest }) {
  const [h, hp] = useHover();
  const base = { font: 'inherit', fontFamily: 'var(--font-sans)', fontSize: 'var(--text-app)', lineHeight: 'var(--leading-app)', color: 'var(--text)', background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-sm)', padding: '5px 14px', cursor: disabled ? 'default' : 'pointer', whiteSpace: 'nowrap', opacity: disabled ? 0.5 : 1, transition: 'background var(--dur-fast), filter var(--dur-fast)' };
  const live = h && !disabled;
  const v = {
    default: { background: live ? 'var(--hover)' : 'var(--surface)' },
    primary: { background: 'var(--accent)', borderColor: 'var(--accent)', color: 'var(--on-accent)', fontWeight: 600, filter: live ? 'brightness(1.08)' : 'none' },
    danger: { color: 'var(--danger)', borderColor: 'var(--danger)', background: live ? 'color-mix(in srgb, var(--danger) 10%, var(--surface))' : 'var(--surface)' },
    tab: { textAlign: 'left', border: '1px solid ' + (active ? 'color-mix(in srgb, var(--accent) 40%, transparent)' : 'transparent'), background: active ? 'var(--accent-soft)' : live ? 'var(--hover)' : 'none', padding: '7px 11px', fontWeight: active ? 600 : 400, color: active ? 'var(--accent-strong)' : 'var(--text-muted)' },
  }[variant];
  return <button type={type} disabled={disabled} onClick={onClick} style={{ ...base, ...v, ...style }} {...hp} {...rest}>{children}</button>;
}
