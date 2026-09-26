# No Drama Llama

[![CI](https://github.com/singerbj/no-drama-llama/actions/workflows/ci.yml/badge.svg)](https://github.com/singerbj/no-drama-llama/actions/workflows/ci.yml)

Turns a Windows gaming PC into an always-on local LLM server that **gets out of the way when
you play**. It runs [llama.cpp](https://github.com/ggml-org/llama.cpp) with
[Qwen 3.8 27B](https://huggingface.co/unsloth/Qwen3.8-27B-GGUF) on your GPU. When a game starts,
it stops the model so the game gets all of your VRAM, and it brings the model back once you
quit.

- **Tray icon** with status (🟢 running · 🟠 loading · 🔵 paused for a game · ⚪ off · 🔴 error)
  and every setting in its right-click menu
- **Game detection** that works across launchers. It reads the library records of Steam, Epic,
  GOG, EA, Ubisoft, Battle.net, Riot, Rockstar, Xbox/Game Pass, Heroic, Humble, HoYoPlay,
  Meta/Oculus and more. It also watches per-process GPU usage, which catches anything the
  launcher check misses. Emulators are optional.
- **On-screen popup** when the model pauses or resumes. It doesn't take focus and clicks pass
  through it.
- **Ctrl+Alt+L** turns the LLM on or off from anywhere
- **OpenAI-compatible API** and a chat web UI at `http://127.0.0.1:8080`
- **Low-power, always-on settings**: no sleep, the screen turns off after 10 minutes, PCIe/CPU/USB
  power saving, and Wake-on-LAN. The uninstaller puts all of it back.
- **Updates itself** from signed GitHub releases

## Requirements

- Windows 10 or 11 (x64), signed in with an **administrator** account
- A GPU with Vulkan support. The default model (17.6 GB) plus a 32K context fits a **24 GB** card,
  for example a Radeon RX 7900 XTX or GeForce RTX 3090/4090. For 16 GB cards, see [Models](#models).
- About 25 GB of free space on `C:`

## Install

1. Download `no-drama-llama.exe` from the [latest release](https://github.com/singerbj/no-drama-llama/releases/latest).
2. Run it and choose **Yes** to install, then approve the administrator prompt.

A console window shows progress. The model download is about 17.6 GB. If it's interrupted, run
the exe again and it resumes. Every download is checked against its published SHA-256. When the
install finishes, the tray icon appears and the model starts loading.

The app installs to `C:\Program Files\No Drama Llama`, starts with Windows, and appears in
**Settings → Apps**. Upgrading from the PowerShell edition is automatic: your settings and
models are kept, and the old scripts, task and shortcuts are removed.

Installer options (from a terminal):

```powershell
.\no-drama-llama.exe install --skip-model            # bring your own .gguf
.\no-drama-llama.exe install --skip-power-settings --skip-wake-on-lan
.\no-drama-llama.exe install --llama-cpp-tag b6500   # pin a llama.cpp build
.\no-drama-llama.exe install --update-llama-cpp      # re-download llama.cpp
```

## Usage

| | |
| --- | --- |
| **Chat** | Double-click the tray icon, or open <http://127.0.0.1:8080> |
| **API** | `http://127.0.0.1:8080/v1` (OpenAI-compatible) |
| **On/off** | Ctrl+Alt+L, or *Turn off / Turn on* in the tray menu |
| **Tray app closed?** | Start menu → *No Drama Llama* |
| **Updates** | Automatic (daily), or *Check for updates* in the menu, or `no-drama-llama.exe update` |

```bash
curl http://127.0.0.1:8080/v1/chat/completions -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Hello!"}]}'
```

### Tray menu

- **Settings → Model / Reasoning / Context length / Access**: switching restarts the server.
  *Access → Devices on my network* listens on `0.0.0.0`, so other devices on your network can use
  it. Set an `ApiKey` first (see below).
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
| `Context` | `32768` | Tokens (512 – 1048576). More context uses more VRAM. |
| `ListenHost` | `127.0.0.1` | `0.0.0.0` makes it reachable from your network |
| `Port` | `8080` | 1024 – 65535 |
| `ApiKey` | *(empty)* | If set, clients must send `Authorization: Bearer <key>` |
| `PauseWhileGaming` | `true` | |
| `DetectionMode` | `Both` | `Both` · `Gpu` · `Launchers` |
| `GpuVramGB` / `GpuLoadPct` | `1.5` / `30` | Pause when another app uses this much VRAM or 3D load, on 2 checks in a row |
| `ResumeAfterSec` | `60` | How long to wait after the game closes (0 – 3600) |
| `ExtraGames` / `GpuIgnore` | `[]` | Process names (without `.exe`) to always or never treat as games |
| `DetectEmulators` / `UseWindowsGameList` | `true` / `true` | |
| `Popups` / `PopupPosition` | `true` / `TopCenter` | `TopCenter` · `TopRight` · `BottomRight` · `BottomCenter` |
| `AutoUpdate` | `true` | Install signed updates automatically |

### Models

Any GGUF file in `C:\LLM\models` shows up under *Settings → Model*. For a 16 GB card:

```powershell
curl.exe -L -o C:\LLM\models\Qwen3.8-27B-UD-IQ3_XXS.gguf `
  https://huggingface.co/unsloth/Qwen3.8-27B-GGUF/resolve/main/Qwen3.8-27B-UD-IQ3_XXS.gguf
```

## Optional: survive power cuts and restarts

- **BIOS:** set *Restore on AC Power Loss* to *Power On*, disable *ErP/EuP*, and enable *Wake on
  LAN / Power On by PCI-E*.
- **Automatic sign-in:** use [Sysinternals Autologon](https://learn.microsoft.com/sysinternals/downloads/autologon).
  After an automatic sign-in right after boot, the app locks the screen straight away.
- **Wake-on-LAN:** the installer prints each wired adapter's MAC address for your WoL app.

## Uninstall

Use **Settings → Apps → No Drama Llama → Uninstall**, or:

```powershell
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" uninstall                # asks first
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" uninstall --keep-models  # models go to Downloads
```

This restores your original power and Wake-on-LAN settings, offers to turn off automatic
sign-in, and deletes `C:\LLM` and the app.

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
rust/       the app (Rust): src/ (logic + win/ platform code), tests/, examples/fake_llama_server.rs
src/        the original PowerShell edition (still tested)
tests/      Pester tests for the PowerShell edition
docs/       architecture, releasing
```

```sh
cd rust
cargo test                       # platform-independent logic, on any OS
cargo build --examples && cargo test   # on Windows: also Win32, process, HTTP and end-to-end tests
cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings   # Windows lint from any OS
cargo build --release            # rust/target/release/no-drama-llama.exe
```

The test suite includes:
- unit tests for every module
- property tests (proptest) for settings validation, detection, the state machine's safety
  rules, update verification and CLI parsing
- lifecycle simulations (a game launches, closes, launcher hand-offs, false positives)
- compatibility tests against PowerShell-edition files
- Windows-only tests: Win32 helpers, process handling, resumable downloads against a local HTTP
  server, and an end-to-end run of the worker against a fake `llama-server`

CI runs all of it on Linux and Windows. Releases are cut by pushing a `v*` tag, as described in
[docs/releasing.md](docs/releasing.md).
