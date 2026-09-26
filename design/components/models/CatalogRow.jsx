import React from 'react';
import { Button } from '../core/Button.jsx';
const FIT = { gpu: 'running', gpuAndRam: 'loading', tooBig: 'off' };
/** Catalog entry: label, fit note (tone-coloured), and Download / Installed / Cancel. */
export function CatalogRow({ label, note, fit = 'gpu', recommended = false, state = 'download', onAction, last = false }) {
  const tone = 'var(--tone-' + FIT[fit] + ')';
  const btn = state === 'installed' ? <Button disabled>Installed</Button> : state === 'downloading' ? <Button onClick={onAction}>Cancel</Button> : <Button disabled={fit === 'tooBig' || state === 'unavailable'} onClick={onAction}>Download</Button>;
  return <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, padding: '8px 0', borderBottom: last ? 'none' : '1px solid var(--border)' }}>
    <div style={{ display: 'flex', flexDirection: 'column', minWidth: 0 }}>
      <span style={{ color: fit === 'tooBig' ? 'var(--text-muted)' : 'var(--text)' }}>{label}</span>
      <small style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 'var(--text-hint)', color: recommended ? 'var(--accent)' : 'var(--text-muted)', fontWeight: recommended ? 600 : 400 }}><span style={{ width: 6, height: 6, borderRadius: '50%', background: tone }} />{note}{recommended && ' · recommended ★'}</small>
    </div>
    {btn}
  </div>;
}
