# Website UI kit

Recreation of the landing page at https://nodramallama.benjaminjsinger.com/ (Astro, `site/` in the repo). Structure and class values from `site/src/styles/landing.css`; copy taken verbatim from the live page. Restyled with this system's night-first palette, Unbounded headlines and Google Sans Code body.

- `Top.jsx` — sticky blurred nav (logo, Docs, API, GitHub, theme toggle, Download), hero with the animated OSD popup over the VRAM window, How it works.
- `Sections.jsx` — features grid + launcher chips, Talks OpenAI (code window), model table, Careful with your machine, FAQ, final CTA, footer; mounts the page.

Interactive: theme toggle (dark ⇄ light, persisted), FAQ disclosures, popup + bar-grow animations. The flat dot grid in the hero is this system's addition; the source's radial glows are removed (no-glow rule).
