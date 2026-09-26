import React, { useState } from 'react';
/** Disclosure row with a mint +/− marker. */
export function FaqItem({ question, children, defaultOpen = false, style }) {
  const [open, setOpen] = useState(defaultOpen);
  return <div style={{ background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: 'var(--radius-xl)', padding: '0 1.25rem', ...style }}>
    <button type="button" onClick={() => setOpen(!open)} aria-expanded={open} style={{ all: 'unset', boxSizing: 'border-box', width: '100%', cursor: 'pointer', padding: '1rem 0', fontWeight: 600, display: 'flex', justifyContent: 'space-between', gap: '1rem' }}>
      {question}<span style={{ color: 'var(--accent)', fontSize: '1.3rem', lineHeight: 1 }}>{open ? '−' : '+'}</span>
    </button>
    {open && <p style={{ margin: 0, paddingBottom: '1.1rem', color: 'var(--text-muted)' }}>{children}</p>}
  </div>;
}
