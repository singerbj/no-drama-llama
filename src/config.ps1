# Shared paths and settings for the local LLM setup.
# User-changeable settings live in C:\LLM\data\settings.json and are edited from the tray menu.
#
# Layout (install.ps1 sets the ACLs):
#   C:\LLM\          admin-only. The tray app runs elevated, so the code it loads must not be
#     app\            user-writable - otherwise any program could get silently elevated at logon.
#     llama\          llama.cpp binaries
#     settings-backup.json
#   C:\LLM\data\     user-writable: settings, on/off flag (hotkey runs non-elevated), logs
#   C:\LLM\models\   user-writable: drop extra .gguf files here
$AppVersion   = '1.1.0'
$Root         = 'C:\LLM'
$AppDir       = "$Root\app"
$ServerExe    = "$Root\llama\llama-server.exe"
$ModelsDir    = "$Root\models"
$DataDir      = "$Root\data"
$OffFlag      = "$DataDir\off.flag"          # exists = turned off (tray menu or hotkey)
$LogFile      = "$DataDir\tray.log"
$ServerLog    = "$DataDir\server.log"
$SettingsFile = "$DataDir\settings.json"
$BackupFile   = "$Root\settings-backup.json" # original Windows settings, used by uninstall.ps1
$TaskName     = 'Local LLM Guard'
$ShortcutName = 'Toggle Local LLM.lnk'                  # Ctrl+Alt+L hotkey
$TrayShortcutName = 'Local LLM.lnk'                     # Start Menu entry to relaunch the tray app
$PollSeconds  = 5

$DefaultModel = 'Qwen3.8-27B-UD-Q4_K_XL.gguf'
$ModelFile    = "$ModelsDir\$DefaultModel"         # what install.ps1 downloads
$ModelRepo    = 'unsloth/Qwen3.8-27B-GGUF'              # Hugging Face repo it comes from

$DefaultSettings = [ordered]@{
  Model            = $DefaultModel
  Reasoning        = 'low'        # none | low | medium | xhigh  (Qwen 3.8 defaults to xhigh and overthinks)
  Context          = 32768
  ListenHost       = '127.0.0.1'  # '0.0.0.0' = reachable from your LAN
  Port             = 8080
  PauseWhileGaming = $true
  DetectEmulators  = $true
  UseWindowsGameList = $true      # also treat exes Windows Game Bar knows as games
  Popups           = $true        # on-screen popup when the LLM starts/stops
  PopupPosition    = 'TopCenter'  # TopCenter | TopRight | BottomRight | BottomCenter
  ExtraGames       = @()          # extra game process names (no .exe) the scanner misses
  DetectionMode    = 'Both'       # Both | Gpu | Launchers
  GpuVramGB        = 1.5          # pause when another app holds this much VRAM...
  GpuLoadPct       = 30           # ...or keeps the 3D engine this busy (seen on 2 checks in a row)
  ResumeAfterSec   = 60           # wait this long after the game is gone before reloading
  GpuIgnore        = @()          # your own "not a game" process names for GPU detection
  ApiKey           = ''           # if set, clients must send it (Authorization: Bearer <key>)
}

# settings.json is user-writable but read by the elevated tray app and turned into
# llama-server arguments, so every value is checked before use. Anything invalid
# falls back to its default (and is reported in $script:SettingsWarnings).
$SettingsRules = @{
  Model              = { param($v) $v -is [string] -and $v -match '^[\w.\-+ ()\[\]]+\.gguf$' }
  Reasoning          = { param($v) $v -in 'none', 'low', 'medium', 'high', 'xhigh' }
  Context            = { param($v) $v -as [int] -and [int]$v -ge 512 -and [int]$v -le 1048576 }
  ListenHost         = { param($v) $v -in '127.0.0.1', '0.0.0.0' }
  Port               = { param($v) $v -as [int] -and [int]$v -ge 1024 -and [int]$v -le 65535 }
  PopupPosition      = { param($v) $v -in 'TopCenter', 'TopRight', 'BottomRight', 'BottomCenter' }
  DetectionMode      = { param($v) $v -in 'Both', 'Gpu', 'Launchers' }
  GpuVramGB          = { param($v) $null -ne ($v -as [double]) -and [double]$v -gt 0 -and [double]$v -le 256 }
  GpuLoadPct         = { param($v) $null -ne ($v -as [double]) -and [double]$v -gt 0 -and [double]$v -le 100 }
  ResumeAfterSec     = { param($v) $null -ne ($v -as [int]) -and [int]$v -ge 0 -and [int]$v -le 3600 }
  ApiKey             = { param($v) $v -is [string] -and $v -match '^[\w.\-~]{0,128}$' }
  ExtraGames         = { param($v) @($v | Where-Object { $_ -isnot [string] }).Count -eq 0 }
  GpuIgnore          = { param($v) @($v | Where-Object { $_ -isnot [string] }).Count -eq 0 }
}
foreach ($k in 'PauseWhileGaming', 'DetectEmulators', 'UseWindowsGameList', 'Popups') { $SettingsRules[$k] = { param($v) $v -is [bool] } }

function Get-Settings {
  $script:SettingsWarnings = @()
  $s = [ordered]@{}
  foreach ($kv in $DefaultSettings.GetEnumerator()) { $s[$kv.Key] = $kv.Value }
  if (Test-Path $SettingsFile) {
    try {
      $j = Get-Content $SettingsFile -Raw | ConvertFrom-Json
      foreach ($p in $j.PSObject.Properties) {
        if (-not $DefaultSettings.Contains($p.Name)) { continue }             # unknown key: ignore
        $v = $p.Value
        if ($v -is [array] -or $DefaultSettings[$p.Name] -is [array]) { $v = @($v | Where-Object { $null -ne $_ }) }
        if (& $SettingsRules[$p.Name] $v) { $s[$p.Name] = $v }
        else { $script:SettingsWarnings += "invalid $($p.Name) '$v' in settings.json - using default" }
      }
    } catch {
      $script:SettingsWarnings += "settings.json unreadable ($($_.Exception.Message)) - using defaults"
    }
  }
  $s
}
function Save-Settings($s) { $s | ConvertTo-Json -Depth 3 | Set-Content $SettingsFile -Encoding UTF8 }

# Game detection lists and launcher discovery live in games.ps1

# --- Power-plan settings install.ps1 changes (backed up first, restored by uninstall.ps1)
$PowerSettings = @(
  @{ Sub = 'SUB_SLEEP';      Set = 'STANDBYIDLE' },
  @{ Sub = 'SUB_SLEEP';      Set = 'HIBERNATEIDLE' },
  @{ Sub = 'SUB_VIDEO';      Set = 'VIDEOIDLE' },
  @{ Sub = 'SUB_DISK';       Set = 'DISKIDLE' },
  @{ Sub = 'SUB_PCIEXPRESS'; Set = 'ASPM' },
  @{ Sub = 'SUB_PROCESSOR';  Set = 'PROCTHROTTLEMIN' },
  @{ Sub = '2a737441-1930-4402-8d77-b2bebba308a3'; Set = '48e6b7a6-50f5-4782-a5d4-53bb8f07e226' }  # USB selective suspend
)
function Get-AcIndex($scheme, $sub, $set) {
  $m = (powercfg /q $scheme $sub $set) | Select-String 'Current AC Power Setting Index:\s*0x([0-9a-fA-F]+)'
  if ($m) { [Convert]::ToInt32($m.Matches[0].Groups[1].Value, 16) } else { $null }
}
