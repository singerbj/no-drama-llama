import React from 'react';
import { Button } from '../core/Button.jsx';
const gb = (b) => (b / 1e9).toFixed(1) + ' GB';
/** Background download panel with an 8px progress bar. done/total in bytes. */
export function DownloadProgress({ label, done = 0, total = 0, onCancel, note, style }) {
  const p = total ? done / total : 0;
  return <div style={{ padding: '12px 16px', border: '1px solid var(--border)', borderRadius: 'var(--radius-md)', background: 'var(--surface)', marginTop: 8, ...style }}>
    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: 12, marginBottom: 8 }}><span>Downloading <b>{label}</b></span>{onCancel && <Button onClick={onCancel}>Cancel</Button>}</div>
    <div style={{ height: 8, borderRadius: 'var(--radius-pill)', background: 'var(--bg-alt)', overflow: 'hidden', border: '1px solid var(--border)' }}><div style={{ width: (p * 100) + '%', height: '100%', background: 'var(--tone-running)', transition: 'width 0.3s var(--ease-out)' }} /></div>
    <small style={{ display: 'block', marginTop: 6, color: 'var(--text-muted)', fontSize: 'var(--text-hint)' }}>{total ? gb(done) + ' of ' + gb(total) + ' (' + Math.floor(p * 100) + '%).' : 'Starting...'}{note ? ' ' + note : ''}</small>
  </div>;
}
