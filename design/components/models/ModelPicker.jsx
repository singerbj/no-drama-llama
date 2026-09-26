import React, { useState } from 'react';
const gb = (b) => (b / 1e9).toFixed(1) + ' GB';
function Opt({ m, checked, onPick }) {
  const [h, setH] = useState(false);
  return <label onMouseEnter={() => setH(true)} onMouseLeave={() => setH(false)} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '7px 8px', borderRadius: 'var(--radius-sm)', cursor: 'pointer', background: h ? 'var(--hover)' : 'transparent' }}>
    <input type="radio" checked={checked} onChange={onPick} style={{ accentColor: 'var(--accent)', margin: 0 }} />
    <span style={{ flex: 1, overflowWrap: 'anywhere' }}>{m.name}</span>
    <small style={{ color: m.size < 0 ? 'var(--tone-error)' : 'var(--text-muted)', fontSize: 'var(--text-hint)' }}>{m.size < 0 ? 'missing' : gb(m.size)}</small>
  </label>;
}
/** Radio list of installed .gguf models. size in bytes; size < 0 = missing. */
export function ModelPicker({ models = [], value, onChange, style }) {
  return <div role="radiogroup" aria-label="Installed models" style={{ display: 'flex', flexDirection: 'column', paddingTop: 6, ...style }}>
    {models.map((m) => <Opt key={m.name} m={m} checked={m.name === value} onPick={() => onChange && onChange(m.name)} />)}
  </div>;
}
