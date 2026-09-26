# Architecture

No Drama Llama ships as one Windows executable, `no-drama-llama.exe` (Rust, in `rust/`).
The original PowerShell edition (`src/`) is still in the repo and still tested. It's described
at the end of this page.

## Components (`rust/src`)

| Module | Platform | Role |
| --- | --- | --- |
| `settings.rs` | any | `settings.json`: defaults, validation of every value, atomic save. Same keys as the PowerShell edition. |
| `server.rs` | any | Builds the `llama-server` command line (flags, chat-template kwargs, sampling) |
| `detect.rs` | any | Game matching: launcher folders, name lists, Windows' game list, Steam's running app, GPU-use decision |
| `state.rs` | any | The state machine (below) |
| `update.rs` | any | Update rules: version comparison, asset selection, digest check, minisign verification |
| `setup.rs` | any | Install/uninstall logic: task XML, settings backup and restore, `powercfg` parsing, PowerShell-edition migration |
| `cli.rs` | any | Command-line parsing |
| `win/tray.rs` | Windows | UI thread: tray icon and menu, Ctrl+Alt+L hotkey, popups |
| `win/worker.rs` | Windows | Worker thread: polls every 5 s, runs detection, the state machine, `llama-server` and updates |
| `win/osd.rs` | Windows | Layered, click-through, never-focused popup window (GDI) |
| `win/gpu.rs` | Windows | Per-process GPU counters (PDH), full-screen check |
| `win/libraries.rs` | Windows | Reads each launcher's records to find game folders |
| `win/procs.rs` | Windows | Process list and exe paths (sysinfo) |
| `win/install.rs` | Windows | `install` / `uninstall` subcommands |
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

## Game detection

- **Launchers**: the process's exe is inside a folder found in a launcher's own records (Steam
  via `steamlocate`, Epic manifests, GOG/EA/Ubisoft/Rockstar/Battle.net registry, Riot YAML,
  Xbox `.GamingRoot`, Heroic/Humble JSON, HoYoPlay, Oculus). Also matched: default folder
  patterns, known game and emulator names, Windows Game Bar's list, and Steam's `RunningAppID`.
- **GPU**: another process holds at least `GpuVramGB` of VRAM or at least `GpuLoadPct` of the 3D
  engine on two checks in a row, or an exclusive full-screen D3D app is in the foreground.
- The name lists match the PowerShell edition's, and a test enforces that.

## Install layout and trust

```
%ProgramFiles%\No Drama Llama\no-drama-llama.exe   admin-only (Windows default)
C:\LLM\                 Administrators + SYSTEM: full, Users: read (inheritance from C:\ removed)
  llama\                llama.cpp (run elevated)
  data\                 settings.json, off.flag, tray.log, server.log
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
4. It swaps the running exe (`self-replace`), updates the Apps & features version, and relaunches
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
  `windows` crate, `sysinfo`, `winreg`, `steamlocate`, `ureq` with SChannel TLS, `self-replace`,
  and `minisign-verify`.
- Porting to Linux/SteamOS later is feasible.

The cost was re-implementing the popup and the GPU counter reader in Win32, about 300 lines.

## PowerShell edition (`src/`)

Same behaviour, as scripts: `install.ps1`/`uninstall.ps1`, and `llm-tray.ps1` run elevated at
logon, with `llm-toggle.ps1` for the hotkey via a Start-menu shortcut and a flag file. It's
covered by Pester tests and PSScriptAnalyzer in CI. Installing the Rust edition migrates it
automatically: it removes the old task, scripts and shortcuts, and keeps settings and models.
