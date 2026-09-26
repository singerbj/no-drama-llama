Single-line text input for string settings; put it in a stacked SettingRow.
```jsx
<SettingRow label="API key" hint="Optional. Clients must send it as a Bearer token." stacked>
  <div style={{display:"flex",gap:8}}><TextInput placeholder="(none)" /><Button>Generate</Button></div>
</SettingRow>
```
