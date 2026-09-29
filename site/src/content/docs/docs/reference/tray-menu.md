---
title: Tray menu
description: Every item in the tray icon's right-click menu.
---

Right-click the llama in the system tray. Double-click it to open the chat UI. **Open settings
window...** opens the [settings window](/docs/first-steps/#the-settings-window), which has the
same settings in tabs.

## Top level

| Item | What it does |
| --- | --- |
| *Status line* | What the model is doing, which model, and the context llama.cpp chose |
| **Turn off / Turn on** | Stops or starts the model. Same as **Ctrl+Alt+L**. |
| **Open chat** | Opens <http://127.0.0.1:8080> |
| **Restart server** | Restarts `llama-server`, and clears an error |
| **Not a game – ignore *app*** | Shown while paused: adds the app that triggered the pause to `GpuIgnore` |
| **Settings** | See below |
| **Check for updates** | Checks GitHub now. Changes to *Update available* or *Installing update* when there is one. |
| **Open folder** | Opens `C:\LLM` |
| **View log** | Opens `C:\LLM\data\tray.log` |
| **Exit (stops the model)** | Closes the tray app and stops the model. Start it again from the Start menu. |

## Settings

| Submenu | Options |
| --- | --- |
| **Model** | Every model in `C:\LLM\models`, **Download a model** (the [catalog](/docs/guides/models/)), **Open models folder** |
| **Reasoning** | `none`, `low`, `medium`, `xhigh`. Restarts the server. |
| **Context length** | *Auto (largest that fits your GPU)*, or 8K, 16K, 32K, 64K, 128K tokens. Restarts the server. |
| **Access** | *This PC only* (`127.0.0.1`) or *Devices on my network* (`0.0.0.0`). Set an `ApiKey` first: see [Use it from other devices](/docs/guides/network/). |
| **Laya (decision model)** | Laya's status; *Run Laya alongside the LLM*; *Model* (`laya`, `laya:en`, `laya:multilingual`, `laya:typed-decisions`); *Run on* (Auto, CPU only, NVIDIA GPU only); *Keep the model loaded*; *Restart Laya*; *Check for Ollaya updates*; *View Laya log*. See [Decision model (Laya)](/docs/guides/laya/). |
| **Game detection** | *Pause while gaming*; *Detect games by* (GPU usage + launchers, GPU usage only, Launchers only); *GPU: VRAM threshold* (1 – 4 GB); *GPU: 3D load threshold* (20 – 70%); *Resume after game closes* (15 seconds – 5 min); *Show GPU usage now*; *Count emulators as games*; *Use Windows' game list (Game Bar)*; *Rescan and show detected libraries* |
| **On-screen popups** | *Show popups*, position, *Test popup* |
| **Start with Windows** | Starts the tray app when you sign in |
| **Update automatically** | Installs signed updates daily |
| **Edit settings file** | Opens [`settings.json`](/docs/reference/settings/) |
