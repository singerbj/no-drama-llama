---
title: Command line
description: Every command and option of no-drama-llama.exe.
---

```
no-drama-llama.exe [command]
```

After installing, the exe lives at `C:\Program Files\No Drama Llama\no-drama-llama.exe`.

| Command | What it does |
| --- | --- |
| *(none)* | Installs if needed, otherwise starts the tray app |
| `run` | Starts the tray app |
| `install` | Installs or upgrades. Needs admin, and asks for it. |
| `uninstall` | Removes everything and restores your settings |
| `update` | Checks for a new version and installs it |
| `models` | Lists downloadable models and what fits this PC |
| `version` | Prints the version |
| `help` | Prints help |

## `install`

| Option | What it does |
| --- | --- |
| `--model <id\|auto\|none>` | Model to download. `auto` (the default) picks the best for your GPU. Ids come from `models`, e.g. `qwen3.8-27b:UD-Q4_K_XL` or `qwen3.8-flash-next:UD-Q2_K_XL`. |
| `--skip-model` | Same as `--model none` |
| `--backend <name>` | llama.cpp build: `auto` (the default), `vulkan`, `cuda12`, `cuda13` |
| `--skip-power-settings` | Leaves the power plan alone |
| `--skip-wake-on-lan` | Leaves network adapter wake settings alone |
| `--llama-cpp-tag <tag>` | Installs this llama.cpp build (the default is the latest), e.g. `b6500` |
| `--update-llama-cpp` | Downloads llama.cpp again |
| `--laya` | Turns on [Laya](/docs/guides/laya/) and installs Ollaya now (the tray app downloads the model when it starts) |

## `uninstall`

| Option | What it does |
| --- | --- |
| `--keep-models` | Moves your models to Downloads first |
| `--yes`, `-y` | Doesn't ask for confirmation |

## Exit codes

`0` on success, `2` for an unknown command or option.
