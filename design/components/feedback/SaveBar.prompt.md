Bottom bar that appears with unsaved edits; Ctrl+S saves.
```jsx
<SaveBar floating message="2 unsaved changes · saving restarts the model" onRevert={revert} onSave={save} />
```
Invalid state: `message="Fix the highlighted value to save"` + `saveDisabled`.
