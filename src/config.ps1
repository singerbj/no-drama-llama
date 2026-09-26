# Shared paths and settings for the local LLM setup.
# User-changeable settings live in C:\LLM\settings.json and are edited from the tray menu.
$Root         = 'C:\LLM'
$ServerExe    = Join-Path $Root 'llama\llama-server.exe'
$ModelsDir    = Join-Path $Root 'models'
$OffFlag      = Join-Path $Root 'off.flag'              # exists = turned off (tray menu or hotkey)
$LogFile      = Join-Path $Root 'tray.log'
$SettingsFile = Join-Path $Root 'settings.json'
$BackupFile   = Join-Path $Root 'settings-backup.json'  # original Windows settings, used by uninstall.ps1
$TaskName     = 'Local LLM Guard'
$ShortcutName = 'Toggle Local LLM.lnk'                  # Ctrl+Alt+L hotkey
$TrayShortcutName = 'Local LLM.lnk'                     # Start Menu entry to relaunch the tray app
$PollSeconds  = 5

$DefaultModel = 'Qwen3.8-27B-UD-Q4_K_XL.gguf'
$ModelFile    = Join-Path $ModelsDir $DefaultModel      # what install.ps1 downloads

$DefaultSettings = [ordered]@{
  Model            = $DefaultModel
  Reasoning        = 'low'        # none | low | medium | high | xhigh  (Qwen 3.8 defaults to xhigh and overthinks)
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
}

function Get-Settings {
  $s = [ordered]@{}
  foreach ($kv in $DefaultSettings.GetEnumerator()) { $s[$kv.Key] = $kv.Value }
  if (Test-Path $SettingsFile) {
    try {
      $j = Get-Content $SettingsFile -Raw | ConvertFrom-Json
      foreach ($p in $j.PSObject.Properties) { $s[$p.Name] = $p.Value }
    } catch { }
  }
  $s
}
function Save-Settings($s) { $s | ConvertTo-Json | Set-Content $SettingsFile }

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
