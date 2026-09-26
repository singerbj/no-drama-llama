Outline pill for a row of names, e.g. the launchers it reads.
```jsx
<div style={{display:"flex",flexWrap:"wrap",gap:8}}>{["Steam","Epic","GOG"].map(n => <Chip key={n}>{n}</Chip>)}</div>
```
