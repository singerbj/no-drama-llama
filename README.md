# No Drama Llama

[![CI](https://github.com/singerbj/no-drama-llama/actions/workflows/ci.yml/badge.svg)](https://github.com/singerbj/no-drama-llama/actions/workflows/ci.yml)

**[Website](https://nodramallama.benjaminjsinger.com/) · [Docs](https://nodramallama.benjaminjsinger.com/docs/)**

Turns a Windows gaming PC into an always-on local AI server that **gets out of the way when
you play**. It runs two kinds of models side by side:

- **An LLM**, which writes text: [llama.cpp](https://github.com/ggml-org/llama.cpp) with a
  [Qwen 3.8](https://huggingface.co/unsloth/Qwen3.8-27B-GGUF) model sized for your GPU, an
  OpenAI-compatible API and a chat page at `http://127.0.0.1:8080`.
- **A decision model**, which answers typed questions about a text (pick an option, score it,
  yes or no) in milliseconds: [Laya](https://ollaya.dev/library/laya), served by
  [Ollaya](https://ollaya.dev), at `http://127.0.0.1:11435` (`/api/decide`, `/v1/systemone`).
  It's off by default; turn it on in the setup wizard or on the settings window's
  *Decision model* tab.

Both share the same on/off switch, game pausing, access setting and API key. When a game starts,
the app stops them so the game gets all of your VRAM, and it brings them back once you quit.

- **Works with any GPU llama.cpp supports** (AMD, NVIDIA, Intel). It installs the fastest
  llama.cpp build for your card, picks a model that fits, and lets llama.cpp size the context
  and GPU layers to your free VRAM.
- **Choose any LLM** from the tray (*Settings → Model → Download a model*) or the settings
  window's *LLM* tab. Each one is marked with how well it fits your PC, and you can also drop in
  your own `.gguf`.
- **Decision model managed like the LLM**: the app installs and updates Ollaya, downloads Laya
  (English, multilingual or another Ollaya model), and pauses and resumes it with the LLM. It runs on an NVIDIA GPU (CUDA 13) or the CPU;
  Ollaya has no AMD or Intel GPU build yet, so on those it uses the CPU.

- **Tray icon** with status (🟢 running · 🟠 loading · 🔵 paused for a game · ⚪ off · 🔴 error)
  and every setting in its right-click menu
- **Settings window** (*Open settings window...* in the tray menu) with every setting, the
  model downloads, and the on/off, restart and update controls
- **Game detection** that works across launchers. It reads the library records of Steam, Epic,
  GOG, EA, Ubisoft, Battle.net, Riot, Rockstar, Xbox/Game Pass, Heroic, Humble, HoYoPlay,
  Meta/Oculus and more. It also watches per-process GPU usage, which catches anything the
  launcher check misses. Emulators are optional.
- **On-screen popup** when the models pause or resume. It doesn't take focus and clicks pass
  through it.
- **Ctrl+Alt+L** turns the LLM and the decision model on or off from anywhere
- **Low-power, always-on settings**: no sleep, the screen turns off after 10 minutes, PCIe/CPU/USB
  power saving, and Wake-on-LAN. The uninstaller puts all of it back.
- **Updates itself** from signed GitHub releases

## Requirements

- Windows 10 or 11 (x64), signed in with an **administrator** account
- Any GPU with a Vulkan or CUDA driver. More VRAM means a better model, but everything from
  8 GB cards up works (see [Models](#models)). Without a GPU it still runs on the CPU, slowly.
- Free space on `C:` for the LLM: 8 – 30 GB for Qwen 3.8 27B, 75 – 115 GB for Flash-Next. The
  decision model needs about 1 GB more (plus 1.1 GB for Ollaya's GPU pack on NVIDIA).

## Install

1. Download `no-drama-llama.exe` from the [latest release](https://github.com/singerbj/no-drama-llama/releases/latest).
2. Run it and approve the administrator prompt. The setup wizard opens.

The wizard walks through:
1. **System check:** Windows version, GPU and driver, memory, free disk space, the connection to
   GitHub and Hugging Face, the `C:\LLM` folder and the API port. Anything that would stop the
   install is shown in red with what to do about it.
2. **Model:** the LLMs that fit your GPU, with the best one selected. Keep your current model
   on an upgrade, or install without one.
3. **Options:** the llama.cpp build (CUDA on NVIDIA, Vulkan on AMD and Intel, picked for you),
   always-on power settings, Wake-on-LAN, starting with Windows, and the decision model (Laya).
4. **Review and install:** a summary with the download size and disk space, then progress for
   each step and download.

If a download is interrupted, run the exe again and it resumes. Every download is checked
against its published SHA-256. When the install finishes, the tray icon appears and the model
starts loading.

The app installs to `C:\Program Files\No Drama Llama`, starts with Windows, and appears in
**Settings → Apps**. Upgrading from the PowerShell edition is automatic: your settings and
models are kept, and the old scripts, task and shortcuts are removed.

For scripted installs, `install` does the same without the wizard (from a terminal):

```powershell
.\no-drama-llama.exe models                          # what fits this PC
.\no-drama-llama.exe install --model qwen3.8-27b:UD-Q5_K_XL
.\no-drama-llama.exe install --model none            # bring your own .gguf
.\no-drama-llama.exe install --backend vulkan        # force a llama.cpp build (vulkan, cuda12, cuda13)
.\no-drama-llama.exe install --skip-power-settings --skip-wake-on-lan
.\no-drama-llama.exe install --llama-cpp-tag b6500   # pin a llama.cpp build
.\no-drama-llama.exe install --update-llama-cpp      # re-download llama.cpp
.\no-drama-llama.exe install --laya                  # also run the decision model (installs Ollaya)
```

## Usage

| | |
| --- | --- |
| **Chat** | Double-click the tray icon, or open <http://127.0.0.1:8080> |
| **LLM API** | `http://127.0.0.1:8080/v1` (OpenAI-compatible) |
| **Decision model API** | `http://127.0.0.1:11435/api/decide`, and `/v1/systemone` for the TypeSafe SDK (when Laya is on) |
| **On/off** | Ctrl+Alt+L, or *Turn off / Turn on* in the tray menu |
| **Tray app closed?** | Start menu → *No Drama Llama* |
| **Updates** | Automatic (daily), or *Check for updates* in the menu, or `no-drama-llama.exe update` |

```bash
# The LLM
curl http://127.0.0.1:8080/v1/chat/completions -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Hello!"}]}'

# The decision model
curl http://127.0.0.1:11435/api/decide -d '{
  "model": "laya",
  "state": "I was charged twice this month. Please refund the second charge.",
  "questions": {
    "department": {
      "type": "choice",
      "instructions": "Which team should handle this ticket?",
      "criteria": { "billing": "Payments and refunds", "technical": "Bugs and outages" }
    }
  }
}'
```

The [decision model guide](https://nodramallama.benjaminjsinger.com/docs/guides/laya/) has the
question types, models and settings.

### Settings window

*Open settings window...* in the tray menu opens a small window that has every setting from
`settings.json`, with the same controls as the menu:
- **Overview**: GPU, the LLM and the decision model (status and address), version, updates and
  download progress
- **LLM**: status, API address, llama.cpp build and context in use (*Restart LLM*, *View log*);
  your models and *Download a model*; reasoning and context length; access, port and API key
- **Decision model**: *Run Laya alongside the LLM*; status, API address and Ollaya version
  (*Restart Laya*, *Check for Ollaya updates*, *View Laya log*); Laya's model, how long it stays
  loaded, port and device
- **Game detection**: every detection setting, `ExtraGames` and `GpuIgnore`
- **App**: *Start with Windows*, *Update automatically*, privacy, popups, files and *Exit*

Access and the API key (on the **LLM** tab) apply to both models. Changes apply when you click
**Save** (or press Ctrl+S). Changes to the LLM's model or server settings restart the LLM, and
changes to Laya's settings, access or the API key restart Laya. The window uses the Microsoft Edge WebView2 Runtime, which Windows 10 and 11
include.

### Tray menu

- **Settings → Model**: switch between your models, or *Download a model* (see [Models](#models)).
- **Settings → Reasoning / Context length / Access**: switching restarts the server. *Context
  length → Auto* (the default) lets llama.cpp use the largest context that fits your GPU. The
  status line shows what it chose, for example *Running - Qwen3.8 27B Q4_K_XL · 96K context*.
  *Access → Devices on my network* listens on `0.0.0.0`, so other devices on your network can use
  it. Set an `ApiKey` first (see below).
- **Settings → Laya (decision model)**: *Run Laya alongside the LLM*, its model, what it runs on,
  how long it stays loaded, *Restart Laya*, *Check for Ollaya updates* and *View Laya log*.
- **Settings → Game detection**: turn pausing on or off, pick the detection method, tune the GPU
  thresholds and the resume delay, and choose whether emulators count as games. It also has
  *Show GPU usage now* and *Rescan and show detected libraries*.
  - If something that isn't a game pauses the LLM, open the menu while it's paused and choose
    **Not a game – ignore *app***.
- **Settings → On-screen popups · Start with Windows · Update automatically · Edit settings file**
- **Check for updates · Open folder · View log · Exit**

### Settings file

Everything in the menu is saved to `C:\LLM\data\settings.json`. You can also edit the file by
hand, and changes apply within a few seconds. Invalid values are ignored and logged in
`C:\LLM\data\tray.log`.

| Key | Default | Notes |
| --- | --- | --- |
| `Model` | `Qwen3.8-27B-UD-Q4_K_XL.gguf` | File name in `C:\LLM\models` |
| `Reasoning` | `low` | `none` · `low` · `medium` · `xhigh` |
| `Context` | `auto` | `auto` = the largest that fits your GPU, or a number of tokens (512 – 1048576) |
| `ListenHost` | `127.0.0.1` | `0.0.0.0` makes both models reachable from your network |
| `Port` | `8080` | The LLM's port, 1024 – 65535 |
| `ApiKey` | *(empty)* | If set, clients of both models must send `Authorization: Bearer <key>` |
| `PauseWhileGaming` | `true` | |
| `DetectionMode` | `Both` | `Both` · `Gpu` · `Launchers` |
| `GpuVramGB` / `GpuLoadPct` | `1.5` / `30` | Pause when another app uses this much VRAM or 3D load, on 2 checks in a row |
| `ResumeAfterSec` | `60` | How long to wait after the game closes (0 – 3600) |
| `ExtraGames` / `GpuIgnore` | `[]` | Process names (without `.exe`) to always or never treat as games |
| `DetectEmulators` / `UseWindowsGameList` | `true` / `true` | |
| `Popups` / `PopupPosition` | `true` / `TopCenter` | `TopCenter` · `TopRight` · `BottomRight` · `BottomCenter` |
| `AutoUpdate` | `true` | Install signed updates automatically |
| `StartWithWindows` | `true` | Start the app when you sign in. When off, the Start menu entry still starts it. |
| `RunLaya` | `false` | Run the decision model (Laya, served by Ollaya) alongside the LLM |
| `LayaModel` | `laya` | `laya` · `laya:en` · `laya:multilingual` · `laya:typed-decisions`, or another [Ollaya model](https://ollaya.dev/search) |
| `LayaPort` | `11435` | 1024 – 65535, not the same as `Port` |
| `LayaDevice` | `auto` | `auto` · `cpu` · `cuda`. `auto` uses an NVIDIA GPU when there is one, else the CPU (AMD and Intel GPUs aren't supported by Ollaya yet). |
| `LayaKeepAlive` | `-1` | How long the model stays loaded: `-1` = always, a duration (`5m`), or `0` = unload after each request |
| `SendCrashReports` / `ShareUsageStats` | `false` / `false` | Opt-in crash reports and anonymous usage statistics (see [Privacy](#privacy)) |

### Models

The LLMs in the tray's *Settings → Model → Download a model*, the *LLM* tab's *Download a
model* and `no-drama-llama.exe models`:

| Model | Sizes | Best for |
| --- | --- | --- |
| **Qwen 3.8 27B** (dense) | Q8_0 29 GB → UD-IQ2_XXS 7.3 GB | Anything that fits entirely on the GPU: fast |
| **Qwen 3.8 Flash-Next** (125B MoE, 6B active) | 72 – 111 GB | PCs with lots of RAM (96 GB+). Most of it lives in RAM and it stays fast, because only 6B parameters run per token. |

Each model is marked *fits your GPU*, *GPU + RAM*, *partly in RAM - slow* or *too big for this
PC*, and the best one for your PC gets a ★. For example:

| GPU memory | Recommended |
| --- | --- |
| 48 GB | 27B Q8_0 |
| 32 GB (RTX 5090) | 27B UD-Q6_K_XL |
| 24 GB (RX 7900 XTX, RTX 4090) | 27B UD-Q4_K_XL |
| 20 GB (RX 7900 XT) | 27B UD-IQ4_XS |
| 16 GB (RTX 4080, RX 7800 XT) | 27B UD-IQ3_XXS |
| 8 – 12 GB | 27B UD-IQ2_XXS, or Flash-Next if you have 96 GB+ of RAM |

Downloads run in the background (progress shows in the menu, and you can click to cancel) and
switch over when they finish. Any other GGUF you put in `C:\LLM\models` also shows up. Qwen's
sampling and reasoning settings are only applied to Qwen models; other models use their own
defaults.

## Optional: survive power cuts and restarts

- **BIOS:** set *Restore on AC Power Loss* to *Power On*, disable *ErP/EuP*, and enable *Wake on
  LAN / Power On by PCI-E*.
- **Automatic sign-in:** use [Sysinternals Autologon](https://learn.microsoft.com/sysinternals/downloads/autologon).
  After an automatic sign-in right after boot, the app locks the screen straight away.
- **Wake-on-LAN:** the installer prints each wired adapter's MAC address for your WoL app.

## Privacy

Your prompts, chats, model output, files and settings never leave your PC. On first run the app
asks, separately, whether to send **crash reports** and **anonymous usage statistics**. Both
default to No, nothing is sent until you answer, and you can change them any time in
*Settings → App → Privacy*.

Crash reports carry the error and where it happened, with your user name, PC name and profile
folder removed. Usage statistics are a short list of feature events (never settings values),
the app version and your GPU model, with a random ID that turning them off deletes. Data goes
to PostHog with IP addresses discarded. The full list is in the
[privacy guide](https://nodramallama.benjaminjsinger.com/docs/guides/privacy/) and in
[`src/posthog.rs`](src/posthog.rs).

## Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by
[SignPath Foundation](https://signpath.org).

- Only `no-drama-llama.exe` is signed. It's built from this repository by the
  [Release workflow](.github/workflows/release.yml) on GitHub-hosted runners, never on a
  developer's machine.
- Committers and reviewers: [singerbj](https://github.com/singerbj)
- Approvers: [singerbj](https://github.com/singerbj)
- Privacy: see [Privacy](#privacy). The app sends nothing to other networked systems unless
  you turn on crash reports or usage statistics, or use features that download llama.cpp,
  models or updates.

## Uninstall

Use **Settings → Apps → No Drama Llama → Uninstall**. The uninstall wizard asks whether to keep
your models (they're moved to Downloads) and, if automatic sign-in is on, whether to turn it
off. Then it restores your original power and Wake-on-LAN settings and deletes `C:\LLM` and the
app. From a terminal, for scripts:

```powershell
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" uninstall                # asks first
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" uninstall --keep-models  # models go to Downloads
```

## Troubleshooting

| Symptom | Try |
| --- | --- |
| Red icon, "server keeps crashing" | *View log*, then read `C:\LLM\data\server.log`. The usual cause is running out of VRAM: lower *Context length* or use a smaller model. |
| Pauses when no game is running | *Show GPU usage now*, then *Not a game – ignore …* or `GpuIgnore`, or raise the thresholds |
| A game doesn't pause it | *Rescan and show detected libraries*, add it to `ExtraGames`, or use `DetectionMode: Both` |
| "Windows protected your PC" | The exe isn't code-signed yet. Choose *More info → Run anyway*, or build it yourself (below). |

## How it works

[docs/architecture.md](docs/architecture.md) covers the threads, the state machine, game
detection, the install layout and security model, how signed auto-update works, and why the
app moved from PowerShell to Rust.

## Development

```
src/        the app: platform-independent logic, and win/ for the Windows code (+ its tests)
scripts/    check.sh: every PR check, also run by the pre-commit hook in .githooks/
ui/         the settings window's page (React + TypeScript + Vite), served by Tauri from ui/dist
tests/      integration tests and fixtures
examples/   fake_llama_server.rs, the test double the Windows tests run
docs/       architecture, releasing
site/       the website and docs (Astro + Starlight), deployed to GitHub Pages
```

```sh
(cd ui && npm ci && npm run build)   # the settings window's page; needed before any Windows build
scripts/check.sh                 # every PR check (also the pre-commit hook, see below)
cargo test                       # platform-independent logic, on any OS
cargo build --examples && cargo test   # on Windows: also Win32, process, HTTP and end-to-end tests
cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings   # Windows lint from any OS
cargo build --release            # target/release/no-drama-llama.exe
(cd ui && npm run dev)           # the settings page in a browser, with sample data
```

### Checks and the pre-commit hook

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

The test suite includes:
- unit tests for every module
- property tests (proptest) for settings validation, detection, the state machine's safety
  rules, update verification and CLI parsing
- lifecycle simulations (a game launches, closes, launcher hand-offs, false positives)
- compatibility tests against files written by the old PowerShell edition, whose installs
  are migrated automatically
- Windows-only tests: Win32 helpers, process handling, resumable downloads against a local HTTP
  server, and an end-to-end run of the worker against a fake `llama-server`

CI runs all of it on Linux and Windows. Releases are cut by pushing a `v*` tag, as described in
[docs/releasing.md](docs/releasing.md).

## License

[MIT](LICENSE)
