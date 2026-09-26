One of our own line icons (assets/icons/), tinted with a token colour — feature cards, CTAs, nav. Only these exist: api, box, chevron-down, eye, gpu, key, moon, shield, theme, tray, windows. Don't pull in another icon set; draw a new one on the same 24px grid at stroke 1.8.
```jsx
<Icon name="gpu" size={28} color="var(--accent)" />
<Icon name="windows" />
```
Stroke icons with round caps (theme and windows are filled). Accent-coloured on cards; currentColor in buttons.
