The OSD popup: "LLM paused / Example Game (Steam) detected · GPU freed". Always dark, regardless of theme. The tone lives in the dot only — the source's 4px left accent edge is dropped (no one-sided borders).
```jsx
<StatusPopup tone="paused" title="LLM paused" subtitle="Example Game (Steam) detected · GPU freed" />
<StatusPopup tone="running" title="LLM back" subtitle="Qwen3.8-27B-UD-Q4_K_XL is ready" />
```
Holds 2.6s, fades in (+0.12 alpha/15ms) and out (-0.06).
