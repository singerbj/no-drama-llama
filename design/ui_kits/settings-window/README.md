# Settings window UI kit

Recreation of the tray app's settings window (`ui/` in the repo — React + Vite, served by Tauri in a WebView2 window). Built from `ui/src/App.tsx`, `panels.tsx`, `fields.tsx`, `style.css` and the sample state in `ui/src/mock.ts`, restyled with this design system's tokens.

- `index.html` — loads React, the DS bundle, `data.js`, then the JSX.
- `data.js` — sample tray state (copied from `mock.ts`).
- `Panels.jsx` — the six tabs: Overview, Model, Server & API, Laya, Game detection, App. Every setting from `settings.json` has one field.
- `App.jsx` — status header, side nav, save bar, toast, fake `send()` commands.

Interactions: switch tabs (remembered), edit any field → save bar with change count (and "saving restarts the model" for server keys), Save / Ctrl+S → "Saved" toast, invalid port / API key → "Fix the highlighted value to save", Turn off/on, Restart (loading pulse), Check for updates, Download/Cancel a catalog model, Test popup. Press **G** (outside a field) to simulate a game launch → paused tone + banner; "Ignore this app" resumes. The G shortcut is a demo aid and not part of the real product.
