# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). The version lives in `Cargo.toml`.

## [Unreleased]

### Changed
- **GitHub API calls can use a token.** With `GITHUB_TOKEN` set, the installer's lookups of
  llama.cpp and Ollaya releases send it to the GitHub API (and nowhere else), so a busy network
  or a CI runner doesn't hit GitHub's limit of 60 unauthenticated calls an hour ("http status:
  403").

## [0.0.8]

### Changed
- **Updates work like rekt clipz's and TunedUp's.** The app reads the latest release's
  `latest.json` (Tauri's updater manifest) instead of the GitHub API, so it never hits the
  API's rate limit, and checks a minute after it starts and every 6 hours (30 minutes after a
  failed check) instead of once a day. Update signatures use Tauri's trusted comment, which
  names the version, and any minisign key encoding works.
- **Release pipeline shared with the other desktop apps**: `scripts/release/` bumps the
  version, signs updates and writes `latest.json`; Prepare release also cuts betas; a failed
  release can be re-run on its tag; code signing also takes a `.pfx` certificate; and releases
  now need the updater signing key (docs/releasing.md).

## [0.0.7]

### Fixed
- **No console window after setup.** Once the setup window closed, a console window running
  `ping` popped up for a few seconds: the step that deletes setup's leftover files after it
  exits asked Windows for no console *and* a detached process, and Windows ignores the first
  when given the second, so `ping` opened a console of its own. It now runs in a hidden one.

## [0.0.6]

### Fixed
- **"Microsoft Edge can't read and write to its data directory", for good.** WebView2 never
  runs elevated: it relaunches itself through Explorer, as the signed-in user's unelevated
  token, with plain UAC as much as under Administrator Protection. That token can't write the
  admin-only profile folders (`C:\Program Files\No Drama Llama\setup-webview2`,
  `C:\LLM\data\webview2`), so the setup and settings windows failed on most PCs. Those folders
  now also give write access to the account Explorer runs as, and nobody else unelevated.
  CI missed it because runners sign in as the built-in Administrator, whose Explorer is
  elevated. CI now runs Explorer as an ordinary admin account with UAC's filtered token and
  opens both windows elevated by UAC and as another admin, checking that WebView2 runs
  unelevated and starts without the error (0.0.5 fails all of these).

## [0.0.5]

### Fixed
- **Windows 11 Administrator Protection, again.** The setup window could still fail with
  "Microsoft Edge can't read and write to its data directory" (`C:\Program Files\No Drama
  Llama\setup-webview2`): 0.0.3 only recognised Administrator Protection when the elevated
  token's linked token was the signed-in user, which isn't always so. It's now also recognised
  when the elevated account isn't the one signed in to the Windows session, so WebView2's
  profile goes to the signed-in user's `%LOCALAPPDATA%`. The logon task and the models folder
  use that user too.

### Added
- **Setup window CI.** Windows runners open the setup wizard, elevated both as the signed-in
  user and as a second admin account, and check that WebView2 starts with a profile folder it
  can write.

## [0.0.4]

### Fixed
- **Your own models with small attention heads.** llama-server refused to load a `.gguf` whose
  attention head size isn't a multiple of 32 (tiny and some small models), because the KV cache
  was always `q8_0`. The app now reads the head size from the model file and uses an `f16` KV
  cache for those models.
- **Uninstall and network adapters without Wake-on-LAN.** Restoring an adapter whose Wake-on-LAN
  was backed up as "Unsupported" failed, and stopped the rest of the network settings from being
  restored. Those adapters are now skipped, and one adapter can no longer stop the others.

### Added
- **End-to-end CI on Windows.** Every change is installed on Windows Server 2022 and 2025
  runners, runs a small model through the tray app and API, and is uninstalled again.

## [0.0.3]

### Fixed
- **Windows 11 Administrator Protection.** The setup and settings windows failed with
  "Microsoft Edge can't read and write to its data directory": WebView2 drops elevation there
  and runs as the signed-in user, who can't write to the admin-only profile folder. Under
  Administrator Protection the profile now lives in
  `%LOCALAPPDATA%\No Drama Llama\WebView2\` of the signed-in user. The logon task and the
  models folder's permissions also use the signed-in user rather than the hidden admin account.

## [0.0.2]

### Added
- **Setup wizard.** Running the downloaded exe opens a branded, step-by-step installer instead
  of a console window: a system check (Windows version, GPU and driver, memory, disk space,
  GitHub and Hugging Face, the `C:\LLM` folder, the API port, laptop or desktop), a model picker
  with the best fit selected, options (llama.cpp build, always-on power, Wake-on-LAN, start with
  Windows, Laya, privacy), a review with download and disk sizes, and step-by-step progress with
  download speed, time left and a *Stop* button. Failures show the reason with *Try again*, and
  a `setup.log` in `C:\LLM\data`.
- **Uninstall wizard.** *Settings → Apps → Uninstall* opens a matching wizard that asks whether
  to keep your models and to turn off automatic sign-in.
- `setup` and `setup --uninstall` commands. `install` and `uninstall` stay for scripts.
- The GPU and its memory are read through DirectX before llama.cpp is installed, so AMD and
  Intel PCs get a model recommendation on the first install too.
- The Apps & features entry has an install date, size, help link and a quiet uninstall command.

### Fixed
- Installing failed with "llama.cpp has no Windows build for Vulkan": llama.cpp now publishes
  its builds as pre-releases, and its "latest" release carries no Windows files. The installer
  now picks the newest release that has a build for your GPU.

## [0.0.1]

### Added
- **Opt-in crash reports and anonymous usage statistics** (off by default). On first run the
  app asks two separate questions, both defaulting to No, and sends nothing until they're
  answered. Crash reports have the user name, PC name and profile folder removed. Usage
  statistics are a short list of feature events with a random install ID that's deleted when
  they're turned off. See the [privacy guide](https://nodramallama.benjaminjsinger.com/docs/guides/privacy/).
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
- A **Prepare release** workflow cuts a release from the Actions tab: it bumps the version,
  moves the changelog's [Unreleased] notes under it, commits to `main` and pushes the tag that
  starts the Release workflow (see `docs/releasing.md`).
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
