import React from 'react';
import { Button } from '../core/Button.jsx';
/** Unsaved-changes bar with Revert + Save. floating = fixed to the window bottom. */
export function SaveBar({ message, onRevert, onSave, saveDisabled = false, floating = false, style }) {
  return <footer style={{ ...(floating ? { position: 'fixed', left: 0, right: 0, bottom: 0 } : {}), display: 'flex', alignItems: 'center', gap: 10, padding: '12px 20px', background: 'var(--surface)', borderTop: '1px solid var(--border)', boxShadow: 'var(--shadow-savebar)', ...style }}>
    <span style={{ flex: 1 }}>{message}</span>
    <Button onClick={onRevert}>Revert</Button>
    <Button variant="primary" disabled={saveDisabled} onClick={onSave}>Save</Button>
  </footer>;
}
