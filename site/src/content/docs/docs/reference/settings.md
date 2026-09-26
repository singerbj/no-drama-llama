---
title: Settings file
description: Every key in settings.json, with defaults and allowed values.
---

Everything in the tray menu is saved to `C:\LLM\data\settings.json`. You can also edit the file
by hand (**Settings → Edit settings file**), and changes apply within a few seconds, restarting
the server if needed.

Invalid values are ignored, and the default is used instead. Each one is logged in
`C:\LLM\data\tray.log`. Key names aren't case-sensitive.

## Keys

| Key | Default | Allowed values |
| --- | --- | --- |
| `Model` | `Qwen3.8-27B-UD-Q4_K_XL.gguf` | A `.gguf` file name in `C:\LLM\models` |
| `Reasoning` | `low` | `none` · `low` · `medium` · `xhigh` |
| `Context` | `auto` | `auto` = the largest that fits your GPU, or a number of tokens (512 – 1048576) |
| `ListenHost` | `127.0.0.1` | `127.0.0.1`, or `0.0.0.0` to make it reachable from your network |
| `Port` | `8080` | 1024 – 65535 |
| `ApiKey` | *(empty)* | Up to 128 of `A–Z a–z 0–9 . _ - ~`. If set, clients must send `Authorization: Bearer <key>`. |
| `PauseWhileGaming` | `true` | `true` · `false` |
| `DetectionMode` | `Both` | `Both` · `Gpu` · `Launchers` |
| `GpuVramGB` | `1.5` | 0.1 – 256. Pause when another app uses this much VRAM on 2 checks in a row. |
| `GpuLoadPct` | `30` | 1 – 100. Pause when another app uses this much of the 3D engine on 2 checks in a row. |
| `ResumeAfterSec` | `60` | 0 – 3600. How long to wait after the game closes. |
| `ExtraGames` | `[]` | Process names (without `.exe`) to always treat as games |
| `GpuIgnore` | `[]` | Process names (without `.exe`) to never treat as games |
| `DetectEmulators` | `true` | `true` · `false` |
| `UseWindowsGameList` | `true` | `true` · `false`. Use the games Windows' Game Bar knows about. |
| `Popups` | `true` | `true` · `false` |
| `PopupPosition` | `TopCenter` | `TopCenter` · `TopRight` · `BottomRight` · `BottomCenter` |
| `AutoUpdate` | `true` | `true` · `false`. Install signed updates automatically (and newer Ollaya releases when Laya is on). |
| `StartWithWindows` | `true` | `true` · `false`. Start the app when you sign in. |
| `RunLaya` | `false` | `true` · `false`. Run [Laya](/docs/guides/laya/) (Ollaya) alongside the LLM. |
| `LayaModel` | `laya` | An Ollaya model name: `laya`, `laya:en`, `laya:multilingual`, `laya:typed-decisions`, or another [Ollaya model](https://ollaya.dev/search) (`name`, `name:tag`, `namespace/name:tag`) |
| `LayaPort` | `11435` | 1024 – 65535, not the same as `Port` |
| `LayaDevice` | `auto` | `auto` · `cpu` · `cuda`. With `cpu`, Laya keeps running while you play. |
| `LayaKeepAlive` | `-1` | How long the model stays loaded after a request: `-1` = always, a duration (`5m`, `1h30m`), a number of seconds, or `0` = unload right away |
| `SendCrashReports` | `false` | `true` · `false`. Send scrubbed crash reports. See [Privacy](/docs/guides/privacy/). |
| `ShareUsageStats` | `false` | `true` · `false`. Send anonymous usage statistics. See [Privacy](/docs/guides/privacy/). |
| `PrivacyAsked` | `false` | `true` · `false`. Set once you've answered the first-run privacy questions. |

## Example

```json
{
  "Model": "Qwen3.8-27B-UD-Q4_K_XL.gguf",
  "Reasoning": "medium",
  "Context": "auto",
  "ListenHost": "0.0.0.0",
  "Port": 8080,
  "ApiKey": "pick-a-long-random-string",
  "ExtraGames": ["MyIndieGame"],
  "GpuIgnore": ["obs64", "Resolve"],
  "ResumeAfterSec": 120
}
```

Keys you leave out keep their defaults.

## Other files in `C:\LLM\data`

| File | What |
| --- | --- |
| `tray.log` | The app's log (rotates at 5 MB) |
| `server.log` | `llama-server`'s output |
| `laya.log` | Ollaya's output, when Laya is on |
| `off.flag` | Present while the model is turned off |
