# No Drama Llama — Design System

No Drama Llama turns a Windows gaming PC into an always-on local LLM server that **gets out of the way when you play**. It runs llama.cpp with a Qwen 3.8 model sized for the GPU; when a game starts it stops the model so the game gets all the VRAM, and brings it back after you quit. Free, open source (MIT), one exe.

## Sources
- GitHub: **https://github.com/singerbj/no-drama-llama** (branch `main`) — explore it further for anything not captured here. Key paths: `ui/src/` (settings window, React + TS), `site/src/styles/` (landing + docs theme), `src/win/osd.rs` (on-screen popup), `src/win/tray.rs` (tray menu), `icons/logo.svg`.
- Live site: https://nodramallama.benjaminjsinger.com/ (docs at `/docs/`).
- Visual direction references from the brief (not product sources): Dribbble shots "Stealth Mode Backend Developer Portfolio", "Logistics Management Dashboard", "Automotive Lead-Gen Website", "Yeib 3D WebGL Studio Portfolio". These pushed the system night-first, mono-typed and techy.

## Surfaces
1. **Tray icon + menu** (Win32, native). The icon's disc is tinted by status tone. Not recreated as HTML (native menu).
2. **Settings window** — small WebView2 window: status header, 180px tab nav (Overview, Model, Server & API, Laya, Game detection, App), fieldsets of setting rows, save bar, toast. → `ui_kits/settings-window/`
3. **On-screen popup (OSD)** — borderless, click-through, dark card with a tone dot, fades in, holds 2.6s, fades out. → `StatusPopup` component.
4. **Website + docs** — Astro landing + Starlight docs. → `ui_kits/website/`

## What changed vs. the repo
The repo uses Segoe UI / system fonts, a cool grey app palette and a warm "llama wool" site palette. Per the brief this system:
- sets **Google Sans Code** as the main font everywhere (UI, body, code) and **Unbounded** (geometric, rounded-corner display face — not a grotesk) for headlines;
- goes **dark-first** with the warm wool neutrals deepened, keeps a full `[data-theme="light"]` scope;
- makes the **status tones** the palette: mint (running), marigold (loading), sky (paused), pebble (off), tomato (error), grape (game).
All numeric values (paddings, radii, sizes) are copied from the source CSS.

---

## HARD RULES
1. **No glows on anything.** No radial-gradient glows, blurred colour halos, soft tone rings around dots, or coloured glow shadows. The dot grid is used flat on a solid ground. Neutral drop shadows on floating elements (`--shadow-card`, `--shadow-toast`) are allowed.
2. **No one-sided borders.** No left-only (or any single-side) accent bars — not on tabs, cards, callouts or popups, and not faked with `inset` box-shadows or absolutely positioned strips. Selection and tone use a full border and/or a fill instead (active tab = `--accent-soft` fill + full 1px border + `--accent-strong` text). Row dividers between list items (`border-bottom`) are structure, not accents, and remain.

## CONTENT FUNDAMENTALS
- **Voice:** plain, calm, practical — literally "no drama". Short declarative sentences that say what happens: "Launch a game and the model stops, so the game gets all of your VRAM. Quit, and it comes back a minute later."
- **Person:** second person — "your GPU", "you play". The product is "it" ("It detects your GPU…", "It steps aside"). No "we".
- **Casing:** sentence case for everything — headings ("How it works", "Careful with your machine"), buttons ("Check for updates", "Open models folder", "Download for Windows"), tabs, legends. Product names keep their casing (llama.cpp, Qwen 3.8, Ollaya, Laya, `.gguf`).
- **Hints** explain the consequence, not the control: "Stop the model so the game gets all of your VRAM." "Who can reach the chat page and the API." Ranges are stated literally: "1024 - 65535", "Seconds, 0 - 3600".
- **Status strings** use a hyphen and middot: `Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context`. Middots (·) separate facts everywhere: "One exe · installs in minutes · uninstall restores every setting".
- **Ellipses** as three dots for in-progress/opens-something: "Checking...", "Open settings window...", "Custom...".
- **Honesty over hype:** tells you the downside ("It runs on the CPU, slowly.", "The exe isn't code-signed yet."). Numbers are specific (60 seconds, 10 minutes, 25 MB, 1.1 GB).
- **Headline wit** is dry and llama-flavoured, used sparingly: "Your gaming PC is an AI server. It just knows when to step aside." "Game on. The llama will wait."
- **FAQ answers** open with "Yes." or "No." then the reason.
- **Emoji:** not in UI. The repo README uses 🟢🟠🔵⚪🔴 once as a legend for tray colours; in designs use `StatusDot` instead. ★ marks the recommended model in the tray.
- Keyboard shortcuts written as `Ctrl+Alt+L`, `Ctrl+S`. Menu paths in italics with arrows: *Settings → Model → Download a model*.

## VISUAL FOUNDATIONS
- **Colour:** night ground (`--bg` wool-900 #13100D), warm brown-grey neutrals — never cool grey. The fun comes from the six **status tones**, bright candy hues that are the only saturated colours on a screen. Mint `--accent` is the single brand/action colour (it is also "running"). Tones are used as solid dots, bar fills, banner borders + 12–14% tinted fills (`color-mix`), and icon tints. Code panels are always dark (`--code-bg`), even in light theme; the OSD popup is always near-black `rgb(28 28 32)`.
- **Type:** Google Sans Code for all running text — UI 14/1.45, site body 17/1.6, hints 12.5 muted. Unbounded 600–700 for h1/h2/h3 and app section titles only, tracking −0.02em (h1 −0.035em), line-height 1.1–1.15. Small caps labels: 0.85rem 600 uppercase +0.08em muted.
- **Spacing:** the source's own off-grid values — rows `10px 0`, fieldset `4px 16px 12px`, buttons `5px 14px`, header `14px 20px`, gaps 8/12/16. Site sections `clamp(3.5rem, 8vw, 6rem)` vertical; content wrap 1120px, narrow 760px.
- **Radii:** 6 controls, 8 panels/fieldsets/banners/bars, 10 timeline tiles, 12 popup/FAQ, 14 site cards/windows/code, 999 site pills + chips. App buttons are rectangles (6px); site CTAs are pills.
- **Cards:** 1px `--border` + `--surface` fill, no shadow in-app. Site cards: 14px radius, 1px border, no shadow; *floating* things (demo window, code window, popup, toast) get `--shadow-card`/`--shadow-toast`. No one-sided borders anywhere (the source's OSD 4px tone edge and inset tab bar were removed).
- **Backgrounds:** flat grounds. The hero may use the flat 22px dot grid (`--pattern-grid`); no glows. Alternate sections switch to `--bg-alt` with top/bottom borders. No photos, no illustrations beyond the logo.
- **Borders:** 1px everywhere; dividers between rows; the active tab is an `--accent-soft` fill with a full 1px mint border and mint text.
- **Hover:** app buttons → `--hover` fill; primary → `brightness(1.08)`; site pills → deeper fill + `translateY(-1px)`; ghost → `--bg-alt`; list rows → `--hover`; links → underline. No press shrink.
- **Focus:** 2px solid accent outline, 1px offset (3px on site). Invalid fields: tomato border.
- **Disabled:** 50% opacity (buttons), 55% (fieldsets).
- **Motion:** quick and functional. 0.15s for colour/transform; loading dot pulses opacity 1 → 0.45 over 1.2s ease-in-out; VRAM bars grow from the left over 1.2s `cubic-bezier(0.2,0.8,0.2,1)`; OSD fades in (~120ms), holds 2.6s, fades out. No bounces. `prefers-reduced-motion` kills all animation.
- **Transparency/blur:** only the sticky site nav (`bg` at 85% + `blur(10px)`) and tone tints via `color-mix`.
- **Layout:** settings window = fixed header, fixed 180px nav, scrolling main, fixed bottom save bar when dirty, toast 72px above bottom. Site = sticky 64px nav, two-column hero/split sections that collapse under ~960px.
- **Imagery:** none besides the logo. If imagery is needed, prefer screenshots of the product on dark grounds; warm, not cool.

## ICONOGRAPHY
- **Logo:** `assets/logo.svg` — a white llama in dark shades on a mint disc (`#3DDC84`, identical to `--tone-running`); the `<g class="status">` disc is recoloured per tone for the tray icon. `assets/icon.ico` is the Windows icon. Don't redraw or restyle it.
- **UI icons:** the settings window uses none — text buttons only, plus the select chevron. Every other icon is **our own** line set in `assets/icons/`: a 24px grid, stroke 1.8, round caps and joins, `stroke="currentColor"` so it takes any token colour (28px accent- or tone-tinted on feature cards, 18px in buttons). The site's feature icons (`gpu`, `eye`, `api`, `tray`, `key`, `box`, `moon`, `shield`), its theme toggle (`theme`) and Download button (`windows`) come from these files, and so does the app's select `chevron-down`. The `Icon` component renders them inline. Don't substitute another icon set (Lucide, etc.): draw a new icon in the same style and add it to the folder and to `Icon.jsx`.
- **Status is the icon language:** solid round tone dots (14px header, 10px inline, 8px small; never ringed) carry state everywhere. Use `StatusDot`, not emoji.
- **Unicode:** `→` in text links ("Read the docs →"), `·` as separator, `★` recommended model, `+ / −` FAQ markers.
- **Emoji:** not used in UI.

---

## Index
- `styles.css` — entry point (imports only) → `tokens/fonts.css`, `colors.css`, `typography.css`, `spacing.css`, `base.css`
- `fonts/` — Google Sans Code (roman + italic, variable 300–800) and Unbounded (variable 400–800), latin + latin-ext woff2
- `assets/` — `logo.svg`, `icon.ico`, `icons/` (our line icons)
- `guidelines/` — foundation specimen cards (Colors, Type, Spacing, Brand)
- `components/` — React primitives (see below), one `.card.html` per folder
- `ui_kits/settings-window/` — settings window recreation
- `ui_kits/website/` — landing page recreation
- `thumbnail.html`, `SKILL.md`, `github.md`

### Components
- **core/** — Button (default, primary, danger, tab), PillButton, StatusDot, Icon
- **forms/** — SettingRow, Switch, TextInput, NumberInput, Select, TextArea
- **layout/** — StatusHeader, SideNav, Fieldset, FactList
- **feedback/** — Banner, Toast, SaveBar, DownloadProgress, StatusPopup
- **models/** — ModelPicker, CatalogRow
- **marketing/** — Eyebrow, Chip, FeatureCard, StepCard, WindowFrame, VramBar, StateTimeline, CodeWindow, FaqItem, DataTable

### Intentional additions
- **Icon** — renders our own line icons from `assets/icons/`.
- **PillButton** is separate from Button because the site (`.btn`) and app (`button`) buttons are distinct in the source.
- Tone **game** (grape) comes from the site's `--game` token, promoted to a full status tone.
