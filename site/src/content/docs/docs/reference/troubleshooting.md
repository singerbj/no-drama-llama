---
title: Troubleshooting
description: Fixes for common problems.
---

Start with **View log** in the tray menu (`C:\LLM\data\tray.log`). The server's own output is in
`C:\LLM\data\server.log`.

| Symptom | Try |
| --- | --- |
| Red icon, "server keeps crashing" | Read `server.log` (**View log** on the **LLM** tab). The usual cause is running out of VRAM: lower *Context length* or use a smaller model. **Restart LLM** on the **LLM** tab, or **Restart server** in the tray, clears the error. |
| Red icon, model or llama.cpp missing | Re-run the exe to reinstall, or pick another model under *Settings → Model*. It recovers on its own once the file is back. |
| Pauses when no game is running | *Show GPU usage now*, then *Not a game – ignore …* or add the app to `GpuIgnore`, or raise the thresholds. See [Game detection](/docs/guides/game-detection/#when-it-gets-it-wrong). |
| A game doesn't pause it | *Rescan and show detected libraries*, add it to `ExtraGames`, or use `DetectionMode: Both` |
| Can't connect from another device | Set **Access** to *Devices on my network* (**LLM** tab, or the tray's *Settings → Access*), allow it through Windows Firewall on private networks, and send your `ApiKey`. See [Use it from other devices](/docs/guides/network/). |
| Requests fail while playing | That's expected: the models are paused. LLM clients can poll `GET /health`. |
| The decision model shows an error | See [Decision model (Laya)](/docs/guides/laya/#troubleshooting). **Restart Laya** and **View Laya log** are on the **Decision model** tab and in the tray's Laya menu. |
| "Windows protected your PC" | The exe isn't code-signed yet. Choose *More info → Run anyway*, or [build it yourself](/docs/contributing/development/). |
| A download was interrupted | Run the exe again. Downloads resume and are verified before use. |
| The tray icon is gone | Start menu → *No Drama Llama*. Check *Start with Windows* is on. |
| A setting I edited is ignored | It was invalid. `tray.log` says which key and why. See [Settings file](/docs/reference/settings/). |

Still stuck? [Open an issue](https://github.com/singerbj/no-drama-llama/issues) with `tray.log`
and `server.log` attached.
