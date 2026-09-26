---
title: Models and GPUs
description: How No Drama Llama picks a llama.cpp build and a model for your hardware, and how to change them.
---

## The model catalog

Choose **Settings → Model → Download a model** in the tray, or run `no-drama-llama.exe models`,
to see the catalog:

| Model | Sizes | Best for |
| --- | --- | --- |
| **Qwen 3.8 27B** (dense) | Q8_0 29 GB → UD-IQ2_XXS 7.3 GB | Anything that fits entirely on the GPU: fast |
| **Qwen 3.8 Flash-Next** (125B MoE, 6B active) | 72 – 111 GB | PCs with lots of RAM (96 GB+). Most of it lives in RAM and it stays fast, because only 6B parameters run per token. |

Each model is marked with how well it fits your PC:

| Mark | Meaning |
| --- | --- |
| *fits your GPU* | The whole model fits in VRAM with room for a long context. Fast. |
| *GPU + RAM* | Part of a MoE model lives in system RAM. Still fast. |
| *partly in RAM - slow* | Part of a dense model lives in system RAM. It works, but slowly. |
| *too big for this PC* | Doesn't fit in VRAM plus RAM. |

The best one for your PC gets a ★.

## What you'll get

| GPU memory | Recommended |
| --- | --- |
| 48 GB | 27B Q8_0 |
| 32 GB (RTX 5090) | 27B UD-Q6_K_XL |
| 24 GB (RX 7900 XTX, RTX 4090) | 27B UD-Q4_K_XL |
| 20 GB (RX 7900 XT) | 27B UD-IQ4_XS |
| 16 GB (RTX 4080, RX 7800 XT) | 27B UD-IQ3_XXS |
| 8 – 12 GB | 27B UD-IQ2_XXS, or Flash-Next if you have 96 GB+ of RAM |

The recommendation uses your main GPU's memory (integrated GPUs are skipped) and your RAM:

1. The best Qwen 3.8 27B quant (IQ3_XXS or better) that fits in VRAM with 5 GiB to spare for
   context.
2. Otherwise Flash-Next, if it fits in VRAM plus RAM (keeping 12 GiB of RAM for Windows).
3. Otherwise the best 27B quant that fits in VRAM at all.
4. Otherwise the smallest 27B quant that fits in VRAM plus RAM.

## Switching models

- **From the tray:** *Settings → Model* lists every model in `C:\LLM\models`. Pick one and the
  server restarts with it.
- **Downloading:** *Download a model* runs in the background. Progress shows in the menu (click
  it to cancel), and it switches over when the download finishes and verifies.
- **At install:** `no-drama-llama.exe install --model qwen3.8-27b:UD-Q5_K_XL`.

## Bring your own model

Put any `.gguf` file in `C:\LLM\models` and it appears under *Settings → Model*. For split
GGUFs, the first part is listed. You can install without downloading a model at all with
`install --model none`.

## GPU backends

The installer reads your GPU and installs the fastest llama.cpp build for it:

| GPU | Build |
| --- | --- |
| NVIDIA, driver 580+ and compute capability 7.5+ | CUDA 13 |
| NVIDIA, driver 528+ and compute capability 6.0+ | CUDA 12 |
| AMD, Intel, older NVIDIA | Vulkan |

If a llama.cpp release lacks that build, it falls back to CUDA 12 and then Vulkan. If you change
your GPU, the next install notices and reinstalls the right build. To force one:

```powershell
.\no-drama-llama.exe install --backend vulkan    # or cuda12, cuda13
```

## How it fits your VRAM

llama.cpp's `--fit` sizes everything the app leaves unset to your free GPU memory: the GPU
layers, the context (when *Context length* is *Auto*), and which MoE experts run on the CPU. The
tray's status line shows the context it chose. Older llama.cpp builds without `--fit` get all
layers on the GPU and a 32K context.
