Label + hint + control row — the atom of every settings panel; stack them inside a Fieldset.
```jsx
<Fieldset legend="GPU usage">
  <SettingRow label="VRAM threshold" hint="Another app using this many GB or more counts as a game."><NumberInput value={1.5} step={0.1} /></SettingRow>
</Fieldset>
```

Invalid values: pass `invalid` to the control (tomato border); the SaveBar says "Fix the highlighted value to save".
