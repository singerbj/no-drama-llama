---
title: Updates
description: How No Drama Llama updates itself, and how updates are verified.
---

With **Settings → Update automatically** on (the default), the app checks GitHub for a new
release a minute after it starts and every 6 hours after that, and installs it. The model keeps running while it updates: the new version
takes over the running server without reloading it.

To update now, choose **Check for updates** in the tray menu, or run:

```powershell
& "C:\Program Files\No Drama Llama\no-drama-llama.exe" update
```

## How updates are verified

The updater only installs a release that:

1. is newer than the installed version and isn't a pre-release
2. carries a valid **minisign signature** from the project's key, which is built into the app
3. has a signature that names that exact version, so an old signed build can't be passed off
   as a new one

Builds made without the key (your own builds, or forks) only tell you that an update exists.
The [architecture notes](/docs/contributing/architecture/#auto-update) have the details.

## llama.cpp

The app doesn't update llama.cpp on its own. To get the latest build:

```powershell
.\no-drama-llama.exe install --update-llama-cpp
```

## Ollaya

When the [decision model](/docs/guides/laya/#updates) is on, **Update automatically** also
checks Ollaya's releases once a day and installs a newer one, keeping the downloaded models.
**Check for Ollaya updates** on the **Decision model** tab checks now.
