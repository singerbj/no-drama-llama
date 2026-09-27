---
title: Uninstall
description: Remove No Drama Llama and restore your original settings.
---

Use **Settings → Apps → No Drama Llama → Uninstall**. The uninstall wizard asks whether to
keep your models (they're moved to Downloads) and, if automatic sign-in is on, whether to turn
it off. For scripts, use the terminal instead:

```powershell
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" uninstall                # asks first
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" uninstall --keep-models  # models go to Downloads
```

Uninstalling:

- restores your original power and Wake-on-LAN settings
- offers to turn off automatic sign-in (if you used Sysinternals Autologon, also click
  *Disable* in it to erase the stored password)
- deletes `C:\LLM` and the app

Add `--yes` to skip the question.
