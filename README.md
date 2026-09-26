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

## Requirements

- Windows 10 or 11 (x64), signed in with an **administrator** account. The logon task is created
  for the account that runs the installer.
- A GPU with Vulkan support. The default model (17.6 GB) plus a 32K context fits a **24 GB** card,
  for example a Radeon RX 7900 XTX or GeForce RTX 3090/4090. For 16 GB cards, use a smaller quant
  from the [same repo](https://huggingface.co/unsloth/Qwen3.8-27B-GGUF) (see
  [Models](#models)).
- About 25 GB of free space on `C:`
- Internet access during install (GitHub and Hugging Face)

## Install

1. Download the latest [release zip](https://github.com/singerbj/no-drama-llama/releases) and
   extract it, or clone the repo and use the `src` folder.
2. Open **PowerShell as administrator** in that folder and run:

   ```powershell
   powershell -ExecutionPolicy Bypass -File .\install.ps1
   ```

The model download is about 17.6 GB. If it gets interrupted, run the same command again and it
resumes where it stopped. Every download is checked against its published SHA-256. When the
install finishes, the tray icon appears and the model starts loading.

Useful switches (combine as needed):

| Switch | Effect |
| --- | --- |
| `-SkipModel` | Don't download the default model. Put your own `.gguf` in `C:\LLM\models` instead. |
| `-SkipPowerSettings` | Leave your power plan alone |
| `-SkipWakeOnLan` | Leave network adapter wake settings alone |
| `-LlamaCppTag b6500` | Install that specific llama.cpp build instead of the latest one |
| `-UpdateLlamaCpp` | Download llama.cpp again (for example to upgrade it) |

Running `install.ps1` again is safe. That's also how you upgrade: settings, the model and the
backup of your original Windows settings are all kept.

## Usage

| | |
| --- | --- |
| **Chat** | Double-click the tray icon, or open <http://127.0.0.1:8080> |
| **API** | `http://127.0.0.1:8080/v1` (OpenAI-compatible, any API key if `ApiKey` is unset) |
| **On/off** | Ctrl+Alt+L, or *Turn off / Turn on* in the tray menu |
| **Tray app closed?** | Start menu → *Local LLM* |

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
  *Show GPU usage now* and *Rescan and show detected libraries* for troubleshooting.
  - If something that isn't a game pauses the LLM, open the menu while it's paused and choose
    **Not a game – ignore *app***.
- **Settings → On-screen popups**: turn popups on or off, pick their position, or show a test popup
- **Settings → Start with Windows**
- **View log / Open folder**

### Settings file

Everything in the menu is saved to `C:\LLM\data\settings.json`. You can also edit the file by hand
(*Settings → Edit settings file*). Changes apply within a few seconds. Invalid values are ignored
and logged.

| Key | Default | Notes |
| --- | --- | --- |
| `Model` | `Qwen3.8-27B-UD-Q4_K_XL.gguf` | File name in `C:\LLM\models` |
| `Reasoning` | `low` | `none` · `low` · `medium` · `xhigh` |
| `Context` | `32768` | Tokens. More context uses more VRAM. |
| `ListenHost` | `127.0.0.1` | `0.0.0.0` makes it reachable from your network |
| `Port` | `8080` | |
| `ApiKey` | *(empty)* | If set, clients must send `Authorization: Bearer <key>`. Letters, digits, `.` `-` `_` `~` |
| `PauseWhileGaming` | `true` | |
| `DetectionMode` | `Both` | `Both` · `Gpu` · `Launchers` |
| `GpuVramGB` / `GpuLoadPct` | `1.5` / `30` | Pause when another app uses this much VRAM or 3D load, on 2 checks in a row |
| `ResumeAfterSec` | `60` | How long to wait after the game closes |
| `ExtraGames` | `[]` | Extra process names (without `.exe`) to treat as games |
| `GpuIgnore` | `[]` | Process names to never treat as games |
| `DetectEmulators` / `UseWindowsGameList` | `true` / `true` | |
| `Popups` / `PopupPosition` | `true` / `TopCenter` | `TopCenter` · `TopRight` · `BottomRight` · `BottomCenter` |

### Models

Any GGUF file in `C:\LLM\models` shows up under *Settings → Model*. For a smaller card:

```powershell
curl.exe -L -o C:\LLM\models\Qwen3.8-27B-UD-IQ3_XXS.gguf `
  https://huggingface.co/unsloth/Qwen3.8-27B-GGUF/resolve/main/Qwen3.8-27B-UD-IQ3_XXS.gguf
```

The *Reasoning* setting and the sampling defaults are tuned for Qwen 3.8. Other model families
work, but may ignore the reasoning setting.

## Optional: survive power cuts and restarts

For a PC that should come back on its own after a power cut or an update restart:

- **BIOS:** set *Restore on AC Power Loss* to *Power On*, disable *ErP/EuP*, and enable *Wake on
  LAN / Power On by PCI-E*.
- **Automatic sign-in:** use [Sysinternals Autologon](https://learn.microsoft.com/sysinternals/downloads/autologon)
  (`winget install Microsoft.Sysinternals.Autologon`). After an automatic sign-in right after
  boot, the tray app locks the screen straight away, so the desktop isn't left open.
- **Wake-on-LAN:** the installer prints the MAC address of each wired adapter for your WoL app.

## Uninstall

```powershell
powershell -ExecutionPolicy Bypass -File C:\LLM\app\uninstall.ps1            # remove everything
powershell -ExecutionPolicy Bypass -File C:\LLM\app\uninstall.ps1 -KeepModel # move models to Downloads first
```

This stops the tray app and the server, removes the task and shortcuts, restores your original
power and Wake-on-LAN settings, offers to turn off automatic sign-in, and deletes `C:\LLM`. BIOS
changes are yours to undo.

## Troubleshooting

| Symptom | Try |
| --- | --- |
| Red icon, "server keeps crashing" | *View log*, then read `C:\LLM\data\server.log`. The usual cause is running out of VRAM: lower *Context length* or use a smaller model. |
| Pauses when no game is running | Use *Show GPU usage now* to find the app, then *Not a game – ignore …* or add it to `GpuIgnore`. You can also raise the GPU thresholds. |
| A game doesn't pause it | Use *Rescan and show detected libraries*. Add the game's process name to `ExtraGames`, or use `DetectionMode: Both`. |
| Hotkey does nothing | The shortcut must stay in the Start menu (`Toggle Local LLM`). Sign out and back in once after installing. |
| Antivirus flags it | It's a hidden, elevated PowerShell script that compiles C# at startup, which matches common malware heuristics. Read the scripts and allow `C:\LLM\app` if you're comfortable with them. |

## How it works

See [docs/architecture.md](docs/architecture.md) for the state machine, the game-detection
methods, the security model (why `C:\LLM` is admin-only), and why the project is written in
PowerShell.

## Development

```
src/        the scripts (what install.ps1 copies to C:\LLM\app)
tests/      Pester tests
tools/      Invoke-Checks.ps1: lint + tests, same as CI
docs/       architecture and design notes
```

```powershell
./tools/Invoke-Checks.ps1 -InstallDependencies   # first run: installs Pester 5 + PSScriptAnalyzer
./tools/Invoke-Checks.ps1
```

The checks run on Windows PowerShell 5.1 (the runtime the app uses) and on PowerShell 7 on
Windows and Linux. The logic tests work on any OS. Compiling the embedded C# needs Windows
PowerShell. Keep `src/` **ASCII-only**, because Windows PowerShell 5.1 reads BOM-less UTF-8 as
ANSI (a test enforces this).

**Releasing:** bump `$AppVersion` in `src/config.ps1`, add a `CHANGELOG.md` entry, and push a tag
`v<version>`. The release workflow runs the checks and publishes the zip.
