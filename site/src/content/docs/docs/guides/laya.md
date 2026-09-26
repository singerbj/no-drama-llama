---
title: Laya (decision model)
description: Run Laya with Ollaya next to the LLM, for fast typed answers to choice, score and yes/no questions.
---

[Laya](https://ollaya.dev/library/laya) is a *decision model*. Instead of writing text, it reads a
message, email, ticket or any JSON and answers typed questions about it: pick one of these
options, score it on a scale, yes or no. It does that in a single pass, in milliseconds, with a
calibrated confidence for each answer. [Ollaya](https://ollaya.dev) is the runtime that serves it,
the way llama.cpp serves the LLM.

No Drama Llama can run Ollaya alongside the LLM. It installs Ollaya, downloads the model, starts
and stops it with the LLM, and pauses it while you play.

## Turn it on

- **Settings window:** the **Laya** tab → *Run Laya alongside the LLM* → **Save**.
- **Tray:** **Settings → Laya (decision model) → Run Laya alongside the LLM**.
- **Installer:** `no-drama-llama.exe install --laya`.

The first time, the app:

1. downloads Ollaya from its [GitHub releases](https://github.com/ollaya-dev/ollaya/releases)
   (about 25 MB) and checks it against the release's `sha256sum.txt`. On an NVIDIA GPU with a
   driver from R580 on (RTX 20 series or newer), it also installs Ollaya's GPU pack, about
   1.1 GB.
2. starts `ollaya serve` on port **11435**.
3. downloads the model (`laya`, about 1 GB) and loads it.

The tray tooltip and the **Laya** tab show each step, with download progress. A popup says
*Laya ready* when it's done.

## Use it

Ollaya's API is at `http://127.0.0.1:11435`. Ask Laya questions with `POST /api/decide`:

```bash
curl http://127.0.0.1:11435/api/decide -d '{
  "model": "laya",
  "state": "I was charged twice for my subscription this month. Please refund the second charge.",
  "questions": {
    "department": {
      "type": "choice",
      "instructions": "Which team should handle this ticket?",
      "criteria": {
        "billing": "Payments, invoices and refunds",
        "technical": "Bugs, errors and outages",
        "account": "Login, profile and settings"
      }
    },
    "urgency": {
      "type": "score",
      "instructions": "How urgent is this ticket?",
      "criteria": ["Can wait", "Needs attention this week", "Needs attention today"]
    },
    "refund": { "type": "noul", "instructions": "The customer asks for money back." }
  }
}'
```

Each answer comes back with its probabilities: `department` → `billing`, `urgency` → a score
from 0 to 2, `refund` → the probability of *yes*.

- **TypeSafe-compatible:** `POST /v1/systemone` and `GET /v1/models` work with the TypeSafe SDK.
  Set `TYPESAFE_BASE_URL=http://127.0.0.1:11435`.
- **Other endpoints:** `/api/tags`, `/api/ps`, `/api/show` and the rest are in
  [Ollaya's API reference](https://github.com/ollaya-dev/ollaya/blob/main/docs/api.md).
- **Agents:** Ollaya's own CLI has an MCP server (`ollaya mcp`) for Claude Code, Cursor and other
  MCP clients. Install the CLI separately if you want it; No Drama Llama doesn't need it.

A typical pattern is to let Laya route or triage, and call the LLM only when you need text
written.

## Settings

All of them are on the **Laya** tab and in [`settings.json`](/docs/reference/settings/). Saving
one restarts Laya, which takes a second or two.

| Setting | What it does |
| --- | --- |
| **Run Laya alongside the LLM** (`RunLaya`) | Off by default |
| **Model** (`LayaModel`) | `laya` picks the English or the multilingual model for each request. `laya:en` (421M) is the fastest; `laya:multilingual` (322M) covers 100+ languages; `laya:typed-decisions` is fine-tuned for typed-decisions workflows. Any other [Ollaya model](https://ollaya.dev/search) works from the settings file, e.g. `nli` or `decider:0.8b`. |
| **Keep the model loaded** (`LayaKeepAlive`) | *Always* (`-1`, the default), for a time after the last request (`5m`, `30m`, `1h`), or *Unload after each request* (`0`). |
| **Port** (`LayaPort`) | `11435`. It can't be the LLM's port. |
| **Run on** (`LayaDevice`) | *Auto* uses the NVIDIA GPU when the GPU pack is installed, else the CPU. *CPU only* never touches the GPU. *NVIDIA GPU only* fails without it. |

**Access** and the **API key** are shared with the LLM (**Server & API**). With *Devices on my
network*, Ollaya listens on `0.0.0.0` too, and with an API key set, clients send it as
`Authorization: Bearer <key>` (the TypeSafe SDK sends `TYPESAFE_API_KEY` that way). See
[Use it from other devices](/docs/guides/network/).

## While you play

Laya follows the LLM:

- **Turn off** (or **Ctrl+Alt+L**) stops both.
- When a game starts, Laya stops with the LLM so the game gets the GPU, and comes back when the
  game closes. The model is already downloaded, so it only reloads (a second or two).
- With **Run on → CPU only**, Laya keeps running while you play, since it uses no VRAM.

Ollaya's processes never count as a game.

## Updates

With **Update automatically** on, the app checks Ollaya's releases once a day and installs a
newer one, keeping the downloaded models. You can also use **Check for Ollaya updates** in the
tray's Laya menu or on the Laya tab.

## Files

| Path | What |
| --- | --- |
| `C:\LLM\ollaya\bin\ollaya.exe` | Ollaya (admin-only, like llama.cpp) |
| `C:\LLM\ollaya\lib\ollaya\` | llama.cpp libraries for GGUF models, and the NVIDIA GPU pack (`cuda_v13`) |
| `C:\LLM\ollaya\models\` | Downloaded Ollaya models (`OLLAYA_MODELS`) |
| `C:\LLM\ollaya\install.json` | The installed version and whether the GPU pack is there |
| `C:\LLM\data\laya.log` | Ollaya's output (**View Laya log**) |

Uninstalling No Drama Llama removes all of it. An Ollaya you installed yourself (in
`%LOCALAPPDATA%\Programs\Ollaya`) is separate and untouched, but it can't use port 11435 at the
same time: stop it, or give one of them another port.

## Troubleshooting

| Status | What to do |
| --- | --- |
| *Error - port 11435 is used by another program* | Another Ollaya (the desktop app or `ollaya serve`) is running. Stop it, or change **Port**. |
| *Error - install failed / download failed ... (retrying in N min)* | Usually no internet. It retries by itself, waiting longer each time. **Restart Laya** tries now. |
| *Error - Ollaya keeps crashing* | Check **View Laya log**. **Restart Laya** tries again. |
| *Error - LayaPort ... is also the LLM's port* | Pick another **Port**. |
