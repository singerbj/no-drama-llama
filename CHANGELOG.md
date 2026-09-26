# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). The version lives in `$AppVersion` in `src/config.ps1`.

## [1.1.0]

### Security
- **The tray app could be used to get admin rights without a UAC prompt.** It runs elevated at
  logon from `C:\LLM`, but every signed-in user could modify that folder. Folders created under
  `C:\` allow this by default, and the installer also granted Modify explicitly. Any program
  running as you could edit a script and run as admin at your next sign-in. `C:\LLM` is now
  admin-only. Only `C:\LLM\data` (settings, on/off flag, logs) and `C:\LLM\models` are writable
  by you.
- `settings.json` is validated before its values reach the elevated `llama-server` command line.
  This blocks argument injection such as `--log-file` pointing at a system folder, and path
  traversal in `Model`.
- The tray app only loads PowerShell modules from admin-only locations.
- New `ApiKey` setting: requires `Authorization: Bearer <key>` from clients. Recommended when
  access is set to "Devices on my network".

### Fixed
- **Reasoning = none broke every chat request.** Qwen 3.8's chat template rejects
  `reasoning_effort: none`, so thinking is now turned off with `enable_thinking: false`.
  `high` was removed from the menu because the template treats it as `xhigh`.
- **An interrupted model download could leave a broken model in place for good.** The installer
  ignored curl's exit code, and on the next run it saw the partial file and skipped the
  download. Downloads now go to a `.part` file, are checked against Hugging Face's size and
  SHA-256, and are renamed only once they verify. The llama.cpp zip is checked against GitHub's
  asset digest.
- The screen no longer locks after a normal manual sign-in within 3 minutes of boot. The
  lock-after-boot now happens only when automatic sign-in is on.
- The tray app no longer stops or adopts `llama-server` processes that it didn't start
  (matched by path).
- Hand edits to `settings.json` are now picked up live (restarting the server if needed). Before,
  they were ignored and then overwritten by the next change made from the menu.
- Invalid or corrupt `settings.json` values are logged instead of being silently ignored.
- `tray.log` rotates at 5 MB instead of growing forever.
- Installer: TLS 1.2 is forced on Windows PowerShell 5.1, downloads are no longer slowed down by
  the progress bar, and a missing llama.cpp release asset gives a clear error.
- `uninstall.ps1 -KeepModel` now keeps every `.gguf` in `models\`, not only the default model.

### Changed
- Sampling now follows the model card: thinking-mode settings for low, medium and xhigh, and
  non-thinking settings only for `none`. Before, `low` used the non-thinking settings.
- New folder layout: `C:\LLM\app` (scripts), `C:\LLM\data` (settings and logs). Existing installs
  are migrated automatically by re-running `install.ps1`.
- New `install.ps1` switches: `-LlamaCppTag`, `-UpdateLlamaCpp`, `-SkipModel`,
  `-SkipPowerSettings`, `-SkipWakeOnLan`.
- `uninstall.ps1` is also installed to `C:\LLM\app`.

### Added
- Pester test suite, PSScriptAnalyzer config, `tools/Invoke-Checks.ps1`, and GitHub Actions CI on
  Windows PowerShell 5.1, PowerShell 7 on Windows, and PowerShell 7 on Linux.
- Tag-triggered release workflow that publishes a ready-to-run zip.

## [1.0.0]

- First version: llama.cpp (Vulkan) + Qwen 3.8 27B, a system-tray app with game detection
  (launchers + GPU usage) and an on-screen popup, a Ctrl+Alt+L hotkey, low-power always-on
  settings, Wake-on-LAN, and an uninstaller that restores the original settings.
