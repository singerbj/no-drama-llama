import React, { useState } from 'react';
function useHover(){const [h,setH]=useState(false);return [h,{onMouseEnter:()=>setH(true),onMouseLeave:()=>setH(false)}];}
/** Website CTA: pill-shaped, mint fill or ghost outline. From site .btn / .btn-ghost / .btn-sm. */
export function PillButton({ variant = 'primary', size = 'md', href, icon, onClick, children, style }) {
  const [h, hp] = useHover();
  const s = { display: 'inline-flex', alignItems: 'center', gap: '0.5rem', padding: size === 'sm' ? '0.45rem 0.95rem' : '0.8rem 1.3rem', fontSize: size === 'sm' ? '0.9rem' : 'inherit', fontFamily: 'var(--font-sans)', borderRadius: 'var(--radius-pill)', fontWeight: 600, textDecoration: 'none', cursor: 'pointer', transition: 'background var(--dur-fast), transform var(--dur-fast)', transform: h ? 'translateY(-1px)' : 'none', whiteSpace: 'nowrap' };
  const v = variant === 'ghost'
    ? { background: h ? 'var(--bg-alt)' : 'transparent', color: 'var(--text)', border: '1px solid var(--border)' }
    : { background: h ? 'var(--accent-strong)' : 'var(--accent)', color: 'var(--on-accent)', border: '1px solid var(--accent)' };
  const Tag = href ? 'a' : 'button';
  return <Tag href={href} onClick={onClick} style={{ ...s, ...v, ...style }} {...hp}>{icon}{children}</Tag>;
}
