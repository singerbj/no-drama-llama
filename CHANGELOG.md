# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). The version lives in `Cargo.toml`.

## [Unreleased]

### Added
- **Opt-in crash reports and anonymous usage statistics** (off by default). On first run the
  app asks two separate questions, both defaulting to No, and sends nothing until they're
  answered. Crash reports have the user name, PC name and profile folder removed. Usage
  statistics are a short list of feature events with a random install ID that's deleted when
  they're turned off. See the [privacy guide](https://singerbj.github.io/no-drama-llama/docs/guides/privacy/).
  - New settings `SendCrashReports`, `ShareUsageStats` and `PrivacyAsked`, and a **Privacy**
    section on the settings window's **App** tab.
- **Laya alongside the LLM** (optional, off by default). [Ollaya](https://ollaya.dev) serves
  Laya, a decision model that answers choice, score and yes/no questions about a text in one
  forward pass, on its own port (11435) with an Ollaya and TypeSafe-compatible API. The app
  installs Ollaya from its GitHub releases (checked against `sha256sum.txt`, with the NVIDIA GPU
  pack when the driver supports CUDA 13), downloads and preloads the model, shows progress, and
  runs it with the LLM: the same on/off switch, access and API key, and a pause while gaming
  unless it runs on the CPU. Updates follow *Update automatically*.
  - New settings `RunLaya`, `LayaModel`, `LayaPort`, `LayaDevice` and `LayaKeepAlive`.
  - A **Laya** tab in the settings window, a *Laya (decision model)* submenu in the tray, and
    `install --laya`.
  - `C:\LLM\data\laya.log` has Ollaya's output.
- **Settings window** (*Open settings window...* in the tray menu). It has every setting, the
  model catalog with downloads, and on/off, restart, open chat and update controls. It's built
  with Tauri 2 (WebView2), React and TypeScript. It runs as a child process of the tray
  (`settings-window`), talks to it over stdin/stdout, and is compiled into the same exe.
- New setting `StartWithWindows` (default `true`). The tray's *Start with Windows* item now uses
  it too. Turning it off disables the logon trigger rather than the whole task, so the Start menu
  entry still starts the app. Existing installs keep the choice they made in the menu.
- UI checks in CI: `better-npm-audit`, `oxfmt`, `oxlint` and `vitest` tests of the settings window.
- `scripts/check.sh` runs every pull request check, and the pre-commit hook in `.githooks/` runs it
  before each commit (enabled by `npm ci` in `ui/`).
- The project is licensed under the [MIT License](LICENSE).

### Changed
- New look from the No Drama Llama design system, now in `design/` (tokens, fonts, guidelines,
  reference components and UI kits). The settings window, the landing page and the docs share
  it: dark first with a light theme, Google Sans Code for text and Unbounded for headings, warm
  "llama wool" neutrals and brighter status colors (mint, marigold, sky, pebble, tomato) that the
  tray icon and on-screen popup use too. The logo's circle is mint, and the popup lost its side
  stripe. The popup is set in Google Sans Code, embedded in the exe and loaded for the app only
  (nothing is installed); text in other scripts falls back to Segoe UI.

### Fixed
- Model and llama.cpp downloads no longer fail with "download interrupted" after two minutes. The
  120 s limit was a total budget for the whole body; it now applies only when no data arrives for
  120 s.

### Security
- **Release exes can be code-signed through [SignPath Foundation](https://signpath.org/).**
  Once it's set up (see [docs/releasing.md](docs/releasing.md)), the release workflow has
  SignPath Authenticode-sign the exe and checks that the signed file is the CI build plus a
  valid, timestamped signature. The minisign signature for the updater then covers the signed
  file. Until then, releases stay unsigned as before.
- **Self-update no longer runs a copy of the app from `%TEMP%`.** The update used to move the
  old exe to the user's temp folder and start an elevated helper copy there, where unelevated
  programs could plant DLLs or a fake `cmd.exe` and have them run as administrator. The swap
  now happens inside the admin-only install folder, and leftovers are removed at the next start.
- **`C:\LLM` can't be taken over by another account.** Any user can create folders in `C:\`,
  and whoever creates one stays its owner, able to change its permissions. The installer now
  creates `C:\LLM` admin-only from the start, gives an existing one to Administrators and
  replaces its whole permission list, refuses a `C:\LLM` (or `llama\`, `models\`, `data\`,
  `ollaya\`) that is a link, and downloads llama.cpp and Ollaya again if the folder had another
  owner.
- **Model downloads no longer write inside the user-writable `models\` folder while in
  progress.** Partial files now live in the admin-only `C:\LLM\data\model-downloads\` and only
  a verified file is moved into `models\`. Before, a link in `models\` could make the elevated
  app append to or delete any file on the PC.
- **Elevated programs no longer inherit the user's environment.** The app drops variables that
  make programs load code or change what they run (`COR_*`, `DOTNET_*`, `WEBVIEW2_*`,
  `LLAMA_*`, `GGML_*`, `OLLAYA_*`, `PSModulePath`...), rebuilds `PATH` from the machine-wide
  value, and runs Windows tools (`schtasks`, `icacls`, `powercfg`, `powershell`, `cmd`) by
  their full System32 path, never by name next to the exe (for example in Downloads).
  Program Files and System32 come from Windows APIs instead of environment variables.
- The settings window keeps its WebView2 profile in the admin-only `C:\LLM\data\webview2\`
  instead of `%LOCALAPPDATA%`.
- Start menu shortcuts are never created or deleted through a link.
- A llama.cpp download without a published SHA-256 is refused, and https downloads never
  follow a redirect to plain http.
- Release pipeline: every GitHub Action is pinned to a commit, checkouts don't keep
  credentials, cargo runs with `--locked`, the release build uses no npm cache or install
  scripts, a release must be tagged on `main`, and the signing key lives on a `release`
  environment (see `docs/releasing.md`). cargo-deny also rejects crates from git or other
  registries, and Dependabot waits 7 days before proposing a new version.

## [2.0.0]

A single self-installing app, written in Rust, replaces the PowerShell scripts. It keeps the
same `C:\LLM` layout and settings, and installing it migrates the PowerShell edition
automatically.

### Added
- `no-drama-llama.exe`: tray app, installer (`install`), uninstaller (`uninstall`, also in
  Settings → Apps), updater (`update`). It installs to `C:\Program Files\No Drama Llama`.
- **Auto-update** from GitHub Releases. It only installs newer releases signed with the
  project's minisign key, and the signature must name that exact version. The update happens
  without reloading the model.
- Ctrl+Alt+L is now a real global hotkey handled by the app (no Start-menu shortcut or flag-file
  hand-off).
- Menu items: *Update automatically*, *Check for updates*. New setting: `AutoUpdate`.
- Tests: unit and property tests for all logic, lifecycle simulations, compatibility with
  PowerShell-edition files, and Windows integration tests, including an end-to-end run of the
  worker against a fake llama-server.
- CI/CD: fmt, clippy for Linux and Windows targets, tests on Linux and Windows, a release-build
  smoke test and a dependency advisory check. A tag-triggered release builds, signs and
  publishes. Dependabot keeps dependencies current.

- **Any GPU:**
  - llama.cpp's `--fit` sizes GPU layers, context (`Context: auto`, the new default) and MoE
    offload to your VRAM.
  - The installer picks CUDA 13 or CUDA 12 on NVIDIA and Vulkan on AMD and Intel.
  - The status line shows the context llama.cpp chose.
- **Model choice:**
  - A catalog of Qwen 3.8 27B quants and Qwen 3.8 Flash-Next (125B MoE), each rated for your
    PC, with a recommendation based on GPU memory and RAM.
  - Download from the tray: in the background, resumable, verified and cancellable, and it
    switches when done.
  - Installer options `--model` and `--backend`, and a `models` command.
  - Qwen sampling and reasoning settings only apply to Qwen models.

### Removed
- The PowerShell edition (scripts, Pester tests, PSScriptAnalyzer config, its CI job). The Rust
  project now lives at the repository root.

### Changed
- New logo: a llama in sunglasses on a green circle (`icons/logo.svg`). It's the site's logo
  and favicon and the app icon, and the tray icon is the logo with the circle in the status
  color instead of a bare dot. `scripts/icons.ts` builds every icon from it.
- The menu stays responsive: detection, health checks and server control run on a worker
  thread.
- Settings keys are matched case-insensitively, as PowerShell did.
- `GpuVramGB` must be ≥ 0.1 and `GpuLoadPct` ≥ 1. Property testing found that tiny values
  didn't round-trip, and they aren't meaningful anyway.
- A missing model or llama.cpp now recovers on its own once the file is back.
- Library paths with spaces inside quotes are cleaned up correctly (found by fuzzing).

## [1.1.0]

### Security
- **The tray app could be used to get admin rights without a UAC prompt.** It runs elevated at
  logon from `C:\LLM`, but every signed-in user could modify that folder. Folders created under
  `C:\` allow this by default, and the installer also granted Modify explicitly. Any program
  running as you could edit a script and run as admin at your next sign-in. `C:\LLM` is now
  admin-only. Only `C:\LLM\data` (settings, on/off flag, logs) and `C:\LLM\models` are writable
  by you.
- `settings.json` is validated before its values reach the elevated `llama-server` command line.
  This blocks argument injection such as `--log-file` pointing at a system folder, and path
  traversal in `Model`.
- The tray app only loads PowerShell modules from admin-only locations.
- New `ApiKey` setting: requires `Authorization: Bearer <key>` from clients. Recommended when
  access is set to "Devices on my network".

### Fixed
- **Reasoning = none broke every chat request.** Qwen 3.8's chat template rejects
  `reasoning_effort: none`, so thinking is now turned off with `enable_thinking: false`.
  `high` was removed from the menu because the template treats it as `xhigh`.
- **An interrupted model download could leave a broken model in place for good.** The installer
  ignored curl's exit code, and on the next run it saw the partial file and skipped the
  download. Downloads now go to a `.part` file, are checked against Hugging Face's size and
  SHA-256, and are renamed only once they verify. The llama.cpp zip is checked against GitHub's
  asset digest.
- The screen no longer locks after a normal manual sign-in within 3 minutes of boot. The
  lock-after-boot now happens only when automatic sign-in is on.
- The tray app no longer stops or adopts `llama-server` processes that it didn't start
  (matched by path).
- Hand edits to `settings.json` are now picked up live (restarting the server if needed). Before,
  they were ignored and then overwritten by the next change made from the menu.
- Invalid or corrupt `settings.json` values are logged instead of being silently ignored.
- `tray.log` rotates at 5 MB instead of growing forever.
- Installer: TLS 1.2 is forced on Windows PowerShell 5.1, downloads are no longer slowed down by
  the progress bar, and a missing llama.cpp release asset gives a clear error.
- `uninstall.ps1 -KeepModel` now keeps every `.gguf` in `models\`, not only the default model.

### Changed
- Sampling now follows the model card: thinking-mode settings for low, medium and xhigh, and
  non-thinking settings only for `none`. Before, `low` used the non-thinking settings.
- New folder layout: `C:\LLM\app` (scripts), `C:\LLM\data` (settings and logs). Existing installs
  are migrated automatically by re-running `install.ps1`.
- New `install.ps1` switches: `-LlamaCppTag`, `-UpdateLlamaCpp`, `-SkipModel`,
  `-SkipPowerSettings`, `-SkipWakeOnLan`.
- `uninstall.ps1` is also installed to `C:\LLM\app`.

### Added
- Pester test suite, PSScriptAnalyzer config, `tools/Invoke-Checks.ps1`, and GitHub Actions CI on
  Windows PowerShell 5.1, PowerShell 7 on Windows, and PowerShell 7 on Linux.
- Tag-triggered release workflow that publishes a ready-to-run zip.

## [1.0.0]

- First version: llama.cpp (Vulkan) + Qwen 3.8 27B, a system-tray app with game detection
  (launchers + GPU usage) and an on-screen popup, a Ctrl+Alt+L hotkey, low-power always-on
  settings, Wake-on-LAN, and an uninstaller that restores the original settings.
