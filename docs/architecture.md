# Architecture

## Components

| File | Runs as | Role |
| --- | --- | --- |
| `install.ps1` | admin, once | Downloads llama.cpp and the model, sets folder ACLs, power and Wake-on-LAN settings, the logon task, and the hotkey shortcut |
| `uninstall.ps1` | admin, once | Reverses all of the above from `settings-backup.json` |
| `llm-tray.ps1` | **elevated**, at logon (scheduled task) | Tray icon, state machine, starts and stops `llama-server` |
| `llm-toggle.ps1` | you (not elevated), via the Ctrl+Alt+L shortcut | Flips `data\off.flag` |
| `config.ps1` | dot-sourced | Paths, default settings, settings validation, list of power settings |
| `server.ps1` | dot-sourced | Builds the `llama-server` command line (pure function, unit-tested) |
| `games.ps1` | dot-sourced | Finds launcher libraries and matches running processes against them |
| `gpu.ps1` | dot-sourced | Embedded C#: per-process VRAM and 3D-engine load from Windows performance counters, and whether a full-screen D3D app is running |
| `osd.ps1` | dot-sourced | Embedded C#: click-through popup that doesn't steal focus |

## State machine (`Update-State`, every 5 s, plus a 0.5 s check of the flag file)

```
            off.flag exists                     game detected
  any ───────────────────────► Off      any ───────────────────► Paused   (server killed, VRAM freed)
                                                                   │
                                                  no game for ResumeAfterSec
                                                                   ▼
  no server ──► Start-Server ──► Loading ──/health = 200──► Running
                     ▲               │
                     └── exited before ready (3 times) ──► Error
```

Order of precedence: **Off > Paused > Loading/Running**. The server process is killed rather than
having its model unloaded, which guarantees that all of its VRAM goes back to the game.

Game detection (`DetectionMode`):

- **Launchers**: the process's exe is inside a folder found by reading each launcher's own records
  (Steam `libraryfolders.vdf`, Epic manifests, GOG/EA/Ubisoft/Rockstar registry keys, Riot YAML,
  Xbox `.GamingRoot`, Heroic/Humble JSON, …). Also matched: default folder patterns, known
  game and emulator names, Windows Game Bar's list of games, and Steam's `RunningAppID`.
- **GPU**: another process holds at least `GpuVramGB` of VRAM or at least `GpuLoadPct` of the 3D
  engine on two checks in a row, or an exclusive full-screen D3D app is in the foreground.
  Browsers, chat apps, overlays and similar programs are on an ignore list.
- Once a game is gone, the pause is held for `ResumeAfterSec`. This covers loading screens and
  launcher hand-offs.

## Folders and trust boundaries

```
C:\LLM\                 Administrators + SYSTEM: full   Users: read   (inheritance from C:\ removed)
  app\                  scripts the elevated tray app loads
  llama\                llama.cpp binaries (run elevated)
  settings-backup.json  your original Windows settings (read by the elevated uninstaller)
  data\                 + you: modify    settings.json, off.flag, tray.log, server.log
  models\               + you: modify    *.gguf
```

The tray app is elevated, and the hotkey is not. The only channel between them is a file in
`data\`, so everything the elevated side reads from there is treated as untrusted:

- `off.flag`: only whether it exists is checked.
- `settings.json`: every key is checked against `$SettingsRules` in `config.ps1`. Unknown keys
  are dropped, and invalid values fall back to their defaults, before anything reaches a command
  line.

**Why elevated at all?** Without admin rights, `Get-Process` can't read the path of an elevated
process, and many games and anti-cheat launchers run elevated. Launcher-based detection needs
that path.

## Decision record: why PowerShell?

**Status:** kept, with conditions for revisiting (below).

The install and uninstall scripts are clearly a good fit for PowerShell. Everything they touch is
first-class in it: `powercfg`, the NetAdapter and ScheduledTasks modules, the registry, ACLs and
`.lnk` shortcuts. It also ships with every copy of Windows 10 and 11, so there's nothing extra
to install.

For the tray app it's a reasonable choice, with trade-offs:

**For**
- No dependencies: no runtime, no build step, no installer. The source you read is what runs,
  which matters for something that runs elevated at every logon.
- It's a small program (about 1,500 lines). The parts PowerShell can't do (performance counters,
  P/Invoke, a custom popup window) are embedded C# compiled with `Add-Type`, so no language
  boundary leaks into the design.
- The pure logic (settings, argument building, game matching) can be unit-tested with Pester on
  any OS, and CI does that.

**Against**
- **Weight:** a hidden `powershell.exe` with WinForms uses roughly 100–150 MB of RAM, and
  `Add-Type` compiles the C# with `csc` on every start, which slows startup by a few seconds.
- **Everything runs on one UI thread.** The 5-second poll lists every process and reads each
  one's `.Path` (a module lookup per process). The `/health` check can block for up to 700 ms,
  and the menu doesn't respond during that time.
- **It looks like malware to security software.** A hidden, elevated PowerShell process started by
  a scheduled task, run with `-ExecutionPolicy Bypass` and compiling C# at runtime, matches
  common malware heuristics. Some antivirus and EDR products will flag or block it, and the
  scripts aren't signed.
- The tray app itself is hard to test automatically (WinForms event loop, global state).

**Revisit if** the project gets shared more widely (antivirus false positives and signing start
to matter), grows much more UI, or background CPU becomes a concern. The natural next step is a
small C# (.NET 8) tray app. The two hardest parts, `GpuWatch` and `LlmOsd`, are already C# and
would move over unchanged. `install.ps1` would stay in PowerShell. Other languages (Rust, Go,
AutoHotkey) would mean rewriting those parts for no clear benefit.

**Alternatives considered for the LLM runtime:** Ollama and LM Studio have their own tray apps and
can unload models, but neither has game-aware pausing. Running llama.cpp directly gives an exact
choice of backend (Vulkan avoids the ROCm idle-power problem on RDNA3) and exact flags, and it's
one fewer background service. The pause-while-gaming logic is what this project adds, and it
could later drive another backend.

## Known limitations and possible improvements

- `llama-server` inherits the tray app's elevation, so an elevated network service is exposed to
  your LAN when access is set to "Devices on my network". Set `ApiKey` in that case. A better fix
  is to start it without admin rights (for example through a second, non-elevated scheduled task).
- Replacing `Process.Path` with `QueryFullProcessImageName(PROCESS_QUERY_LIMITED_INFORMATION)`
  would make polling cheaper. That call usually works across integrity levels, so it might remove
  the need for elevation entirely. This needs testing with anti-cheat games.
- `llama.cpp` defaults to the latest release, whose flags can change. Use
  `install.ps1 -LlamaCppTag bNNNN` to pin a build you've tested.
