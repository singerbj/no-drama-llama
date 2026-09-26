---
title: Privacy
description: What No Drama Llama sends, only if you agree, and how to turn it off.
---

No Drama Llama runs your model entirely on your PC. Your prompts, chats, model output, files and
settings never leave it.

The first time it starts, the app asks two separate questions. Both default to **No**, and
nothing is sent until you answer:

- **Send crash reports?** Helps fix crashes.
- **Share anonymous usage statistics?** Shows which features are used, so work goes where it matters.

You can change either one at any time in **Settings → App → Privacy**, or with `SendCrashReports`
and `ShareUsageStats` in the [settings file](/docs/reference/settings/).

## What's sent

**Crash reports** (only with *Send crash reports* on):

- The error message and the place in the app's code where it happened
- The app version and your GPU model (for example `NVIDIA GeForce RTX 4090 (24 GB)`)

Before sending, your Windows user name, PC name, domain and profile folder are replaced with
placeholders like `%USERNAME%`. Each crash report gets a new random ID, so reports can't be
linked to each other or to usage statistics.

**Usage statistics** (only with *Share anonymous usage statistics* on):

- These events, when you do them: `llm_toggled`, `server_restarted`, `settings_saved`,
  `model_download_requested`, `model_download_cancelled`, `update_check_requested`,
  `laya_restarted`, `laya_update_requested`. Settings values are never included.
- Three log lines: the app started, a settings change was saved, and the app is exiting
- The app version and your GPU model
- A random install ID, so repeat use can be counted. It isn't derived from your PC, hardware or
  account. It's stored in `C:\LLM\data\analytics-id`, and turning usage statistics off deletes
  it. If you turn them back on, you get a new, unconnected ID.

Nothing else is collected: no IP address or location, no prompts or chats, no model names you
typed, no file names, and the app's log files stay on your PC.

## Where it goes

Data is sent to [PostHog](https://posthog.com), an analytics service, over HTTPS. PostHog
processes it in the United States under the EU–US Data Privacy Framework and the EU's Standard
Contractual Clauses. The project is set to discard IP addresses and not to look up locations,
and data is deleted after 30 days.

The app is open source: everything above is in
[`src/posthog.rs`](https://github.com/singerbj/no-drama-llama/blob/main/src/posthog.rs).
Builds you make yourself don't send anything unless you configure your own PostHog project.

## Your data

Because the data isn't linked to you, it usually can't be looked up by person. If you want
your usage statistics removed, send the ID in `C:\LLM\data\analytics-id` with a request on
[GitHub](https://github.com/singerbj/no-drama-llama/issues) before turning them off. Turning
usage statistics off stops all further collection immediately, and everything already sent is
deleted within 30 days.
