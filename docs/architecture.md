# Architecture

No Drama Llama ships as one Windows executable, `no-drama-llama.exe`, written in Rust. It
replaced an earlier set of PowerShell scripts (see the decision record below). Installs of that
edition are migrated automatically.

## Components (`src/`)

| Module | Platform | Role |
| --- | --- | --- |
| `settings.rs` | any | `settings.json`: defaults, validation of every value, atomic save. Same keys as the PowerShell edition. |
| `server.rs` | any | Builds the `llama-server` command line (flags, chat-template kwargs, sampling) |
| `detect.rs` | any | Game matching: launcher folders, name lists, Windows' game list, Steam's running app, GPU-use decision |
| `state.rs` | any | The state machine (below) |
| `update.rs` | any | Update rules: version comparison, asset selection, digest check, minisign verification |
| `hardware.rs` | any | `--list-devices` / `nvidia-smi` parsing, backend choice, llama.cpp asset selection, context from the log |
| `catalog.rs` | any | Downloadable models, how well each fits this PC, the recommendation |
| `laya.rs` | any | Laya via Ollaya: settings values, the daemon's environment, release assets and checksums, pull progress, Laya's state machine |
| `setup.rs` | any | Install/uninstall logic: task XML, settings backup and restore, `powercfg` parsing, PowerShell-edition migration |
| `cli.rs` | any | Command-line parsing |
| `control.rs` | any | Messages between the tray and the settings window |
| `win/tray.rs` | Windows | UI thread: tray icon and menu, Ctrl+Alt+L hotkey, popups |
| `win/worker.rs` | Windows | Worker thread: polls every 5 s, runs detection, the state machine, `llama-server` and updates |
| `win/osd.rs` | Windows | Layered, click-through, never-focused popup window (GDI) |
| `win/settings_host.rs` | Windows | Tray side of the settings window: starts it, sends it state, receives its requests |
| `win/settings_app.rs` | Windows | The settings window itself: a Tauri app (`settings-window`) that shows `ui/` |
| `win/gpu.rs` | Windows | Per-process GPU counters (PDH), full-screen check |
| `win/libraries.rs` | Windows | Reads each launcher's records to find game folders |
| `win/procs.rs` | Windows | Process list and exe paths (sysinfo) |
| `win/install.rs` | Windows | `install` / `uninstall` subcommands |
| `win/probe.rs` | Windows | Runs `nvidia-smi`, `llama-server --list-devices` / `--help` (with timeouts); reads RAM |
| `win/models.rs` | Windows | Hugging Face model downloads: resumable, verified, cancellable, multi-part |
| `win/laya.rs` | Windows | Installs Ollaya from its GitHub releases, starts `ollaya serve`, pulls and loads the model over its API |
| `win/updater.rs`, `win/net.rs` | Windows | GitHub Releases, verified resumable downloads, `/health` checks |

Everything that makes a decision is platform-independent and unit-tested on Linux and Windows.
The Windows modules gather inputs and carry out actions.

## Threads

```
 UI thread (message loop, 100 ms timer)          worker thread (every 5 s, or right after a command)
 ─────────────────────────────────────           ─────────────────────────────────────────────────────
 tray icon + menu  ── Cmd (Toggle, Edit…) ──►    reload settings.json if edited by hand
 Ctrl+Alt+L hotkey                               list processes, sample GPU counters, detect game
 popups            ◄── UiMsg (State, Popup) ──   /health check (≤ 700 ms)
                                                 Machine::step → start/stop llama-server
                                                 daily update check (on its own thread)
```

The UI thread never blocks, so the menu stays responsive while a model loads.

## Settings window

The settings window is a separate process running the same exe (`no-drama-llama.exe
settings-window`), so the tray app itself never loads a webview. It's built with Tauri 2 and
WebView2. Its page (`ui/`: React, TypeScript and Vite, about 75 KB gzipped) is compiled into the
exe.

```
 tray (UI thread)                         settings window (Tauri)            page (ui/)
 ────────────────                         ───────────────────────            ──────────
 settings_host  ── Event (JSON line) ──►  stdin  ── "state" event ────────►  renders
   state every tick, Saved, Focus
                ◄─ Request (JSON line) ─  stdout ◄─ send command ─────────  buttons, Save
 → the same actions as the menu (Cmd to the worker)
```

- The tray starts the window with its stdin and stdout piped. Nothing listens on a port or a
  named pipe, so no other program can drive the elevated app through it. When the tray exits,
  the window's stdin closes and the window closes too.
- The window inherits the tray's elevation. It loads only its own bundled page under a strict
  CSP, and its capability allows only event listening plus its two commands.
- The tray parses every request (`control::Request`) and runs it as the matching menu action.
  *Save* sends only the changed settings. `Settings::with_patch` applies each valid value and
  reports the invalid ones, which keep their current value.
- Every `settings.json` key has exactly one field in `ui/src/panels.tsx` (`k="Key"`), and a test
  checks that. The field components in `ui/src/fields.tsx` only accept keys whose type matches
  (`Toggle` takes boolean settings, `NumberField` numeric ones, and so on), and `ui/src/types.ts`
  mirrors `Settings`.

## Start with Windows

`StartWithWindows` (default `true`) turns the logon task's trigger on or off. The task itself
stays enabled, so the Start menu entry can still start the app without a UAC prompt. The worker
applies the setting at startup and whenever settings change, including hand edits. Older
versions disabled the whole task from the menu; when `settings.json` has no `StartWithWindows`
yet, the setting takes the task's current state so that choice is kept.

## State machine (`state.rs`)

```
            off.flag exists                     game detected (or seen < ResumeAfterSec ago)
  any ───────────────────────► Off      any ───────────────────► Paused   (llama-server killed, VRAM freed)

  no server ──► StartServer ──► Loading ──/health = 200──► Running
                     ▲              │
                     └── exited before ready (3 times) ──► Error   (cleared by Turn on / Restart)
  model or llama-server.exe missing ──► Error   (clears by itself once the file is back)
```

Precedence is **Off > Paused > Loading/Running**. The app stays silent while it first loads at
startup, so no popup appears over a game at boot. Property tests check the safety rules across
thousands of random event sequences: never run while off or while a game is held, never start
twice, and report Running only when `/health` says ready.

## Laya (Ollaya) alongside the LLM

[Ollaya](https://ollaya.dev) serves decision models such as Laya from one daemon,
`ollaya serve`. The worker runs it as a second supervised service with its own state machine
(`laya::Machine`), stepped right after the LLM's each tick:

```
  RunLaya off ──► Disabled        settings problem (port clash) ──► Error
  off.flag ──► Off                LLM Paused and LayaDevice ≠ cpu ──► Paused     (daemon killed)
  not installed / GPU pack wanted / update ──► Installing (job) ──► Starting ──► GET / = 200
      ──► model not in /api/tags ──► Downloading (job: POST /api/pull, NDJSON progress)
      ──► not loaded ──► Loading (job: POST /api/decide without state = load + keep_alive)
      ──► Ready
```

- **One job at a time** (install, pull, load) runs on its own thread and reports back with a
  `Cmd::LayaJobDone`. A job's result is ignored if the daemon it was for has been stopped since
  (a generation counter). A failed job shows as an error and retries after 2 minutes, doubling up
  to an hour. Three exits before ready is *keeps crashing*, like llama-server.
- **Configuration** is `ollaya serve`'s environment: `OLLAYA_HOST` (`ListenHost:LayaPort`),
  `OLLAYA_MODELS`, `OLLAYA_KEEP_ALIVE`, `OLLAYA_DEVICE`, `OLLAYA_API_KEY` (the LLM's key). Any
  other `OLLAYA_*` variable is removed, so a user's own Ollaya setup can't leak in.
- **Processes:** the daemon starts one runner per model (a copy of the exe in the GPU pack on
  NVIDIA). Everything running from `C:\LLM\ollaya\` is ours: it's what stop kills, and it's
  excluded from GPU game detection.
- **Install:** the latest release's `ollaya-windows-amd64.zip`, plus `ollaya-windows-amd64-cuda.zip`
  when an NVIDIA driver R580+ and a Turing or newer card are there and `LayaDevice` isn't `cpu`.
  Each archive must match the release's `sha256sum.txt` (and GitHub's digest). An unchanged GPU
  pack is kept (its `FILES.sha256` list and every file are checked), as Ollaya's own installer
  does. Files are staged, then swapped in; `models\` is kept. `install.json` records the version.

## Fitting any GPU

- **Backend:** `hardware::choose_backend` reads `nvidia-smi`:
  - NVIDIA with driver 580+ and compute capability 7.5+ → CUDA 13
  - NVIDIA with driver 528+ and compute capability 6.0+ → CUDA 12
  - anything else (AMD, Intel, old NVIDIA) → Vulkan

  The installer downloads that llama.cpp build, plus the CUDA runtime zip for CUDA. If a
  release lacks the build, it falls back to CUDA 12 and then Vulkan. It records the result in
  `llama\backend.txt` and reinstalls if the GPU changes.
- **Sizing:** llama.cpp's `--fit` (on by default, detected from `llama-server --help`) sets
  everything we leave unset to fit the free device memory:
  - GPU layers
  - context, when *Context* is `auto`
  - CPU offload of MoE experts

  So the app passes no `-ngl`, and no `-c` in auto mode. Builds without `--fit` get the old
  `-ngl 99 -c 32768`. The context llama.cpp chose is read back from `server.log`.
- **Model choice:** `catalog::recommend` uses the primary GPU's memory as llama.cpp reports it
  (`--list-devices`, which skips integrated GPUs) and system RAM:
  1. The best Qwen 3.8 27B quant (IQ3_XXS or better) that fits in VRAM with 5 GiB to spare
     for context.
  2. Otherwise Flash-Next (MoE) if it fits in VRAM plus RAM.
  3. Otherwise the best 27B quant that fits in VRAM at all.
  4. Otherwise the smallest 27B quant that fits in VRAM plus RAM.

  Split GGUFs are downloaded part by part, verified, and listed once.

## Game detection

- **Launchers**: the process's exe is inside a folder found in a launcher's own records (Steam
  via `steamlocate`, Epic manifests, GOG/EA/Ubisoft/Rockstar/Battle.net registry, Riot YAML,
  Xbox `.GamingRoot`, Heroic/Humble JSON, HoYoPlay, Oculus). Also matched: default folder
  patterns, known game and emulator names, Windows Game Bar's list, and Steam's `RunningAppID`.
- **GPU**: another process holds at least `GpuVramGB` of VRAM or at least `GpuLoadPct` of the 3D
  engine on two checks in a row, or an exclusive full-screen D3D app is in the foreground.

## Install layout and trust

```
%ProgramFiles%\No Drama Llama\no-drama-llama.exe   admin-only (Windows default)
C:\LLM\                 Administrators + SYSTEM: full, Users: read (inheritance from C:\ removed)
  llama\                llama.cpp (run elevated)
  ollaya\               Ollaya when Laya is on (run elevated): bin\, lib\, models\, install.json
  data\                 settings.json, off.flag, tray.log, server.log, laya.log
  models\               + you: modify (drop in .gguf files)
  settings-backup.json  your original power/network settings
```

- The app runs **elevated** from a logon scheduled task (no UAC prompt). It needs admin rights
  to see the exe paths of elevated games and anti-cheat processes.
- Because it runs elevated, everything it executes is admin-only. `settings.json` is still
  validated before any value reaches a command line.
- The hotkey is handled inside the app, so nothing non-elevated needs to write to `C:\LLM`.
- Downloads are checked against published SHA-256 values (llama.cpp from GitHub's asset digest,
  the model from Hugging Face's LFS metadata). The llama.cpp zip is extracted with zip-slip
  protection.

## Auto-update

1. The app checks `releases/latest` daily if *Update automatically* is on, or on demand from the menu.
2. It only takes a newer, non-prerelease semver release that has both `no-drama-llama.exe` and
   `no-drama-llama.exe.minisig`.
3. It downloads both and checks GitHub's asset digest. It then verifies the **minisign
   signature** against the public key baked in at build time (`NDL_UPDATE_PUBKEY`). The
   signature's trusted comment must be exactly `no-drama-llama <that version>`, so a validly
   signed old build can't be passed off under a newer tag.
4. It swaps the running exe inside the admin-only install folder (running exe renamed to
   `.old`, deleted at the next start), updates the Apps & features version, and relaunches
   with `run --after-update <old>`. The new instance waits for the single-instance mutex and
   takes over the still-running `llama-server` without reloading it.

A build without a key (local or fork builds) only reports that an update exists.
[releasing.md](releasing.md) covers key setup.

## Decision record: PowerShell → Rust

The PowerShell edition worked, but it was a poor way to ship the app to other people:
- users had to run it with `-ExecutionPolicy Bypass`
- a hidden, elevated script compiling C# looks like malware to antivirus software
- it had no installer entry and no update path
- it used about 100–150 MB of RAM

Rust was chosen over C# because:
- It builds to a single static exe of a few MB that uses little memory and needs no runtime.
- Mature crates cover the platform plumbing: `tray-icon` + `muda`, `global-hotkey`, Microsoft's
  `windows` crate, `sysinfo`, `winreg`, `steamlocate`, `ureq` with SChannel TLS,
  and `minisign-verify`.
- Porting to Linux/SteamOS later is feasible.

The cost was re-implementing the popup and the GPU counter reader in Win32, about 300 lines.

**Migration:** installing the app over the PowerShell edition removes its logon task, scripts
and shortcuts, and moves its settings into `data\`. The `settings.json` keys, the `C:\LLM`
layout and the settings backup format are unchanged, so models, settings and the original
power settings carry over. The PowerShell code itself was removed from the repository in 2.0.0.
