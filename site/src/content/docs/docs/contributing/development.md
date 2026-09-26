---
title: Development
description: Build, test and lint No Drama Llama, and work on this docs site.
---

## Layout

```
src/        the app: platform-independent logic, and win/ for the Windows code (+ its tests)
ui/         the settings window's page (React + TypeScript + Vite), served by Tauri from ui/dist
scripts/    check.sh: every PR check, also run by the pre-commit hook in .githooks/
tests/      integration tests and fixtures
examples/   fake_llama_server.rs, the test double the Windows tests run
docs/       architecture, releasing
site/       this website (Astro + Starlight)
```

## Build and test

You need the Rust toolchain pinned in `rust-toolchain.toml` and Node.js 22 or later.

```sh
(cd ui && npm ci && npm run build)   # the settings window's page; needed before any Windows build
scripts/check.sh                 # every PR check (also the pre-commit hook, see below)
cargo test                       # platform-independent logic, on any OS
cargo build --examples && cargo test   # on Windows: also Win32, process, HTTP and end-to-end tests
cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings   # Windows lint from any OS
cargo build --release            # target/release/no-drama-llama.exe
```

The test suite includes:

- unit tests for every module
- property tests (proptest) for settings validation, detection, the state machine's safety
  rules, update verification and CLI parsing
- lifecycle simulations (a game launches, closes, launcher hand-offs, false positives)
- compatibility tests against files written by the old PowerShell edition
- Windows-only tests: Win32 helpers, process handling, resumable downloads against a local HTTP
  server, and an end-to-end run of the worker against a fake `llama-server`

## Checks and the pre-commit hook

`scripts/check.sh` runs every pull request check:

- **UI** (`ui/`): `better-npm-audit`, `oxfmt --check`, `oxlint`, `tsc`, `vitest` and the build
- **Rust**: `cargo fmt --check`, clippy for this OS and for the Windows target, the tests, and
  `cargo deny check advisories`
- **Windows only**: the Windows tests, plus the release build and its smoke test
- **Docs site**: its build, when `site/`, `docs/` or `CHANGELOG.md` changed (`--all` builds it regardless)

The same script is the pre-commit hook in `.githooks/pre-commit`. Running `npm ci` in `ui/`
turns the hook on (`git config core.hooksPath .githooks`). Skip it once with `git commit --no-verify`.

You also need `cargo install cargo-deny --locked`. Checking the Windows target from Linux or macOS
also needs LLVM's `llvm-rc` on your PATH (Debian/Ubuntu: `sudo apt-get install llvm`).

Inside `ui/` you can run each check alone: `npm run audit`, `format:check` (or `format` to fix),
`lint`, `typecheck`, `test`, `build`, or `check` for all of them. `npm run dev` shows the page in
a browser with sample data.

CI runs all of it on Linux and Windows. See [Architecture](/docs/contributing/architecture/) for
how the code fits together, and [Releasing](/docs/contributing/releasing/) to cut a release.

## The website

The landing page and these docs live in `site/`, built with
[Astro](https://astro.build) and [Starlight](https://starlight.astro.build). You need Node.js 22.18
or newer.

```sh
cd site
npm ci
npm run dev       # http://localhost:4321/no-drama-llama/
npm run build     # type-checks and builds to site/dist
```

| Path | What |
| --- | --- |
| `site/src/pages/index.astro` | The landing page |
| `site/src/content/docs/docs/` | The docs pages |
| `site/astro.config.ts` | Site settings and the sidebar |

The Architecture, Releasing and Changelog pages are generated from `docs/architecture.md`,
`docs/releasing.md` and `CHANGELOG.md` by `site/scripts/sync-docs.ts`, so edit those files
instead. Links in docs pages start with `/docs/` and get the site's base path added
automatically.

Pushes to `main` that touch the site or those files deploy it to GitHub Pages.
