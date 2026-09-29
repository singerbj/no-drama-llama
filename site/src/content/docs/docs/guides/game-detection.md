---
title: Game detection
description: How No Drama Llama notices a game, and how to tune it when it gets it wrong.
---

Every 5 seconds, the app decides whether a game is running. If one is, it stops the LLM and the
decision model so the game gets all of your VRAM. When no game has been seen for
`ResumeAfterSec` (60 seconds by default), it starts them again. Both stop, wherever they run: the game gets the CPU as well as the GPU.

## Two detectors

**Launchers.** A running program counts as a game if its exe is inside a folder that a launcher
installed games into. The app reads each launcher's own records:

- Steam, Epic, GOG, EA, Ubisoft, Battle.net, Riot, Rockstar
- Xbox / Game Pass, Heroic, Humble, HoYoPlay, Meta / Oculus

It also matches default game folders, a list of known game names, the games Windows' Game Bar
knows about, and the game Steam reports as running.

**GPU usage.** Another app counts as a game if, on two checks in a row, it uses at least
`GpuVramGB` of VRAM (1.5 GB by default) or at least `GpuLoadPct` of the 3D engine (30% by
default). An exclusive full-screen app in the foreground also counts. This catches games the
launcher check misses.

By default both are on. Set **Detect games by** on the **Game detection** tab, or under
**Settings → Game detection** in the tray, to use only one.

## Emulators

Emulators such as RetroArch, Dolphin, PCSX2, RPCS3, DuckStation, PPSSPP, Cemu, Ryujinx, Xemu
and Xenia count as games by default. Turn off **Count emulators as games** (`DetectEmulators`)
if you'd rather they didn't.

## When it gets it wrong

### Something that isn't a game pauses the model

1. Open the tray menu while it's paused and choose **Not a game – ignore *app***. That adds the
   app to `GpuIgnore`.
2. To see what's using your GPU, choose **Show GPU usage now**.
3. If lots of apps trip it, raise the thresholds under **GPU: VRAM threshold** and
   **GPU: 3D load threshold**, or switch to **Launchers only**.

### A game doesn't pause it

1. Choose **Rescan and show detected libraries** to see which game folders the app found.
2. Add the game's process name (without `.exe`) to `ExtraGames` in the
   [settings file](/docs/reference/settings/).
3. Make sure **Detect games by** is set to **GPU usage + launchers**.

## Turning it off

Untick **Pause while gaming** (`PauseWhileGaming`) to keep the models running no matter what. You
can still turn it off by hand with **Ctrl+Alt+L**.
