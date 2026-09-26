---
title: Always-on PC
description: Low-power settings, Wake-on-LAN, and surviving power cuts and restarts.
---

## What the installer changes

So the model is there when you need it, the installer sets:

- **No sleep**, and the screen turns off after 10 minutes
- **Power saving** for PCIe, the CPU and USB, so an idle PC draws little power
- **Wake-on-LAN** on each wired network adapter. It prints each adapter's MAC address for your
  WoL app.

Your original settings are saved to `C:\LLM\settings-backup.json`, and uninstalling puts them
back. To leave them alone, install with `--skip-power-settings` and/or `--skip-wake-on-lan`.

## Survive power cuts and restarts

These are optional, but they let the PC come back on its own:

- **BIOS:** set *Restore on AC Power Loss* to *Power On*, disable *ErP/EuP*, and enable
  *Wake on LAN / Power On by PCI-E*.
- **Automatic sign-in:** use
  [Sysinternals Autologon](https://learn.microsoft.com/sysinternals/downloads/autologon). The
  tray app starts when you sign in. After an automatic sign-in right after boot, it locks the
  screen straight away, so the PC isn't left signed in and unlocked.
