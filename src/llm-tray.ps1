# Local LLM tray app: runs llama-server, pauses it while gaming, and puts
# status + controls + settings in a system-tray icon's right-click menu.
# Started at logon (elevated, hidden) by the scheduled task install.ps1 creates.
#Requires -Version 5.1

# Runs elevated: only auto-load modules from admin-only locations, never from the
# user-writable Documents\WindowsPowerShell\Modules folder.
$env:PSModulePath = @("$PSHOME\Modules", "$env:ProgramFiles\WindowsPowerShell\Modules") -join ';'

. (Join-Path $PSScriptRoot 'config.ps1')
. (Join-Path $PSScriptRoot 'server.ps1')
. (Join-Path $PSScriptRoot 'games.ps1')
. (Join-Path $PSScriptRoot 'gpu.ps1')
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
. (Join-Path $PSScriptRoot 'osd.ps1')
[LlmOsd]::EnableDpiAwareness()
[System.Windows.Forms.Application]::EnableVisualStyles()

# Single instance
$created = $false
$script:Mutex = New-Object System.Threading.Mutex($true, 'Global\LocalLLMTray', [ref]$created)
if (-not $created) { exit }

New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
function Log($msg) { "$(Get-Date -Format s)  $msg" | Add-Content -Path $LogFile }
# Keep the log from growing forever on an always-on PC: one 5 MB generation.
if ((Test-Path $LogFile) -and (Get-Item $LogFile).Length -gt 5MB) { Move-Item $LogFile "$LogFile.1" -Force }
Log "tray started (v$AppVersion)"

# After an automatic sign-in right after boot (power loss, update restart), lock the screen.
# Only when auto sign-in is on - someone who just typed their password shouldn't get locked out.
$uptime = (Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime
$autoLogon = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon' -ErrorAction SilentlyContinue).AutoAdminLogon -eq '1'
if ($autoLogon -and $uptime.TotalMinutes -lt 3) {
  Log "boot logon detected (uptime $([int]$uptime.TotalSeconds)s) - locking screen"
  rundll32.exe user32.dll,LockWorkStation
}

$script:S          = Get-Settings
foreach ($w in $script:SettingsWarnings) { Log $w }
$script:Status     = ''
$script:Game       = $null
$script:Failed     = 0       # consecutive starts that never became ready
$script:Pending    = $false  # started, not yet ready
$script:ErrorMsg   = ''
$script:LastFlag   = Test-Path $OffFlag
$script:AnnounceReady = $false   # show a "ready" popup after the next load finishes
$script:LoadStart  = Get-Date
$script:Scan       = Get-GameLibraries
Log "game scan: $($script:Scan.Libs.Count) library folders, $($script:Scan.Exes.Count) Windows-listed game exes"

# ---------------------------------------------------------------- icons
function New-DotIcon([int]$r, [int]$g, [int]$b) {
  $bmp = New-Object System.Drawing.Bitmap 16, 16
  $gfx = [System.Drawing.Graphics]::FromImage($bmp)
  $gfx.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $gfx.Clear([System.Drawing.Color]::Transparent)
  $gfx.FillEllipse((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb($r, $g, $b))), 2, 2, 12, 12)
  $gfx.DrawEllipse((New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(170, 0, 0, 0)), 1), 2, 2, 12, 12)
  $gfx.Dispose()
  [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
}
$Icons = @{
  Running = New-DotIcon 46 160 67     # green
  Loading = New-DotIcon 210 153 34    # amber
  Paused  = New-DotIcon 88 166 255    # blue  (gaming)
  Off     = New-DotIcon 140 140 140   # gray
  Error   = New-DotIcon 218 54 51     # red
}

# ---------------------------------------------------------------- server
# Only our own llama-server (by path): leave any other copy the user runs alone.
function Get-Server { Get-Process -Name 'llama-server' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $ServerExe } }
function Stop-Server { Get-Server | Stop-Process -Force -ErrorAction SilentlyContinue; $script:Pending = $false }

function Start-Server {
  $model = Join-Path $ModelsDir $script:S.Model
  if (-not (Test-Path $model)) { $script:ErrorMsg = "Model missing: $($script:S.Model)"; $script:Failed = 99; return }
  if (-not (Test-Path $ServerExe)) { $script:ErrorMsg = 'llama-server.exe missing'; $script:Failed = 99; return }
  $a = Get-ServerArgs $script:S $model
  Start-Process -FilePath $ServerExe -ArgumentList $a -WindowStyle Hidden `
    -RedirectStandardError $ServerLog
  $script:Pending = $true
  Log "started server ($($script:S.Model), reasoning=$($script:S.Reasoning), ctx=$($script:S.Context), host=$($script:S.ListenHost))"
}

function Test-Ready {
  try {
    $req = [System.Net.WebRequest]::Create("http://127.0.0.1:$($script:S.Port)/health")
    $req.Timeout = 700
    $resp = $req.GetResponse(); $ok = ([int]$resp.StatusCode -eq 200); $resp.Close(); $ok
  } catch { $false }
}

$script:LastGame   = $null
$script:LastGameAt = [datetime]::MinValue

function Test-Gaming {
  if (-not $script:S.PauseWhileGaming) { $script:LastGame = $null; return $null }
  if (((Get-Date) - $script:Scan.ScannedAt).TotalMinutes -ge 10) { $script:Scan = Get-GameLibraries }  # pick up new installs
  $mode = "$($script:S.DetectionMode)"
  $g = $null
  if ($mode -ne 'Gpu') { $g = Find-RunningGame $script:Scan $script:S }
  if ($mode -ne 'Launchers') {
    $gpu = Find-GpuGame $script:S $script:Scan @(Get-Server | ForEach-Object { $_.Id })   # sample every tick
    if (-not $g) { $g = $gpu }
  }
  if ($g) { $script:LastGame = $g; $script:LastGameAt = Get-Date; return $g }
  # Hold the pause a while after the game disappears (loading screens, launcher hand-offs, quick restarts)
  if ($script:LastGame -and ((Get-Date) - $script:LastGameAt).TotalSeconds -lt [double]$script:S.ResumeAfterSec) { return $script:LastGame }
  $script:LastGame = $null
  $null
}

$Colors = @{
  Running = @(46, 160, 67); Loading = @(210, 153, 34); Paused = @(88, 166, 255)
  Off = @(140, 140, 140); Error = @(218, 54, 51)
}
function Show-Popup($title, $sub, $colorKey) {
  if (-not $script:S.Popups) { return }
  $c = $Colors[$colorKey]
  try { [LlmOsd]::Popup($title, $sub, [System.Drawing.Color]::FromArgb($c[0], $c[1], $c[2]), 2600, $script:S.PopupPosition) }
  catch { Log "popup error: $_" }
}
function Get-ModelShort { ($script:S.Model -replace '\.gguf$', '' -replace '-UD-', ' ' -replace '-', ' ') }

# Popup for each state change (not on the tray's own startup)
function Announce($prev, $status, $detail) {
  $m = Get-ModelShort
  switch ($status) {
    'Paused'  {
      $who = "$($detail.Name) ($($detail.Launcher))"
      if ($prev -eq 'Off') { Show-Popup 'LLM on - waiting' "$who is running. Starts when you quit." 'Paused' }
      else { Show-Popup 'LLM paused' "$who detected  -  GPU freed" 'Paused' }
    }
    'Off'     { Show-Popup 'LLM off' 'Turned off  -  GPU freed' 'Off' }
    'Loading' {
      $script:AnnounceReady = $true; $script:LoadStart = Get-Date
      switch ($prev) {
        'Paused'  { Show-Popup 'Game closed' "Reloading $m..." 'Loading' }
        'Running' { Show-Popup 'LLM restarting' "Applying new settings..." 'Loading' }
        default   { Show-Popup 'LLM on' "Loading $m..." 'Loading' }
      }
    }
    'Running' {
      if ($script:AnnounceReady) {
        $secs = [int]((Get-Date) - $script:LoadStart).TotalSeconds
        Show-Popup 'LLM ready' "$m  -  loaded in ${secs}s" 'Running'
        $script:AnnounceReady = $false
      }
    }
    'Error'   { Show-Popup 'LLM error' "$detail" 'Error' }
  }
}

# ---------------------------------------------------------------- state machine
function Set-Status($status, $detail) {
  $prev = $script:Status
  $script:Status = $status
  if ($prev -and $prev -ne $status) { Announce $prev $status $detail }
  $modelShort = Get-ModelShort
  $text = switch ($status) {
    'Running' { "Running - $modelShort" }
    'Loading' { "Loading - $modelShort" }
    'Paused'  { "Paused - $($detail.Name)" }
    'Off'     { 'Off' }
    'Error'   { "Error - $detail" }
  }
  $script:Tray.Icon = $Icons[$status]
  $tip = "Local LLM: $text"
  $script:Tray.Text = $tip.Substring(0, [Math]::Min(63, $tip.Length))
  $script:MiStatus.Text = $text
  $script:MiToggle.Text = if ($status -eq 'Off') { 'Turn on' } else { 'Turn off' }
}

# Settings that change llama-server's command line (need a restart)
$ServerKeys = 'Model', 'Reasoning', 'Context', 'ListenHost', 'Port', 'ApiKey'

# Pick up hand edits to settings.json ("Edit settings file") without restarting the tray.
function Save-TraySettings {
  Save-Settings $script:S
  $script:SettingsStamp = (Get-Item $SettingsFile).LastWriteTimeUtc
}
$script:SettingsStamp = if (Test-Path $SettingsFile) { (Get-Item $SettingsFile).LastWriteTimeUtc }
function Sync-SettingsFile {   # returns $true if a server setting changed
  if (-not (Test-Path $SettingsFile)) { return $false }
  $t = (Get-Item $SettingsFile).LastWriteTimeUtc
  if ($t -eq $script:SettingsStamp) { return $false }
  $script:SettingsStamp = $t
  $old = $script:S
  $script:S = Get-Settings
  Log 'settings.json changed - reloaded'
  foreach ($w in $script:SettingsWarnings) { Log $w }
  foreach ($k in $ServerKeys) { if ("$($old[$k])" -ne "$($script:S[$k])") { return $true } }
  $false
}

function Update-State {
  if (Sync-SettingsFile) {
    if (Get-Server) { Stop-Server; Log 'stopped server (settings changed)'; Start-Sleep -Milliseconds 500 }
    $script:Failed = 0; $script:ErrorMsg = ''
  }
  $off  = Test-Path $OffFlag
  $script:LastFlag = $off
  $game = Test-Gaming
  $srv  = Get-Server

  if ($off) {
    if ($srv) { Stop-Server; Log 'stopped server (turned off)' }
    $script:Game = $null
    Set-Status 'Off'; return
  }
  if ($game) {
    if ($srv) { Stop-Server; Log "stopped server (gaming: $($game.Name) via $($game.Launcher))" }
    $script:Game = $game
    Set-Status 'Paused' $game; return
  }
  if ($script:Game) { $script:Game = $null; Log 'game closed' }

  if (-not $srv) {
    if ($script:Pending) { $script:Failed++; $script:Pending = $false; Log "server exited before ready (fail $($script:Failed))" }
    if ($script:Failed -ge 3) {
      if (-not $script:ErrorMsg) { $script:ErrorMsg = 'server keeps crashing (see log)' }
      Set-Status 'Error' $script:ErrorMsg; return
    }
    Start-Server
    if ($script:Failed -ge 3) { Set-Status 'Error' $script:ErrorMsg; return }
    Set-Status 'Loading'; return
  }
  if (Test-Ready) {
    if ($script:Pending) { Log 'server ready' }
    $script:Pending = $false; $script:Failed = 0; $script:ErrorMsg = ''
    Set-Status 'Running'
  } else { Set-Status 'Loading' }
}

function Restart-Server {
  Stop-Server
  Start-Sleep -Milliseconds 500
  $script:Failed = 0; $script:ErrorMsg = ''
  Update-State
}

function Set-Setting($key, $value) {
  $script:S[$key] = $value
  Save-TraySettings
  Log "setting $key = $value"
  if ($key -in $ServerKeys) { Restart-Server } else { Update-State }
}

function Open-Url($url) { Start-Process explorer.exe $url }   # explorer = opens un-elevated, in your normal browser

# ---------------------------------------------------------------- menu
$script:Tray = New-Object System.Windows.Forms.NotifyIcon
$menu = New-Object System.Windows.Forms.ContextMenuStrip

function Add-Item($parent, $text, [scriptblock]$onClick) {
  $i = New-Object System.Windows.Forms.ToolStripMenuItem $text
  if ($onClick) { $i.add_Click($onClick) }
  [void]$parent.Items.Add($i)   # works for ContextMenuStrip
  $i
}
function Add-Sub($parent, $text) {
  $i = New-Object System.Windows.Forms.ToolStripMenuItem $text
  [void]$parent.DropDownItems.Add($i); $i
}
function Add-Choice($sub, $text, $key, $value) {
  $i = New-Object System.Windows.Forms.ToolStripMenuItem $text
  $i.Tag = @{ Key = $key; Value = $value }
  $i.add_Click({ Set-Setting $this.Tag.Key $this.Tag.Value })
  [void]$sub.DropDownItems.Add($i); $i
}

$script:MiStatus = Add-Item $menu 'Starting...' $null
$script:MiStatus.Enabled = $false
[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

$script:MiToggle = Add-Item $menu 'Turn off' {
  if (Test-Path $OffFlag) { Remove-Item $OffFlag -Force; $script:Failed = 0; $script:ErrorMsg = '' }
  else { New-Item -ItemType File -Path $OffFlag -Force | Out-Null }
  Update-State
}
$miOpen    = Add-Item $menu 'Open chat' { Open-Url "http://127.0.0.1:$($script:S.Port)" }
$miOpen.Font = New-Object System.Drawing.Font($miOpen.Font, [System.Drawing.FontStyle]::Bold)
$miRestart = Add-Item $menu 'Restart server' { Restart-Server }
[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

$settings = Add-Item $menu 'Settings' $null
$subModel  = Add-Sub $settings 'Model'
$subReason = Add-Sub $settings 'Reasoning'
foreach ($r in 'none', 'low', 'medium', 'xhigh') { [void](Add-Choice $subReason $r 'Reasoning' $r) }
$subCtx    = Add-Sub $settings 'Context length'
foreach ($c in 8192, 16384, 32768, 65536, 131072) { [void](Add-Choice $subCtx "$($c / 1024)K tokens" 'Context' $c) }
$subNet    = Add-Sub $settings 'Access'
[void](Add-Choice $subNet 'This PC only' 'ListenHost' '127.0.0.1')
[void](Add-Choice $subNet 'Devices on my network' 'ListenHost' '0.0.0.0')
[void]$settings.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
$subGames = Add-Sub $settings 'Game detection'
$miPause  = Add-Sub $subGames 'Pause while gaming'
$miPause.add_Click({ Set-Setting 'PauseWhileGaming' (-not $script:S.PauseWhileGaming) })
[void]$subGames.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
$subMode = Add-Sub $subGames 'Detect games by'
[void](Add-Choice $subMode 'GPU usage + launchers (recommended)' 'DetectionMode' 'Both')
[void](Add-Choice $subMode 'GPU usage only' 'DetectionMode' 'Gpu')
[void](Add-Choice $subMode 'Launchers only' 'DetectionMode' 'Launchers')
$subVram = Add-Sub $subGames 'GPU: VRAM threshold'
foreach ($v in 1, 1.5, 2, 3, 4) { [void](Add-Choice $subVram "Another app uses $v GB+" 'GpuVramGB' $v) }
$subLoad = Add-Sub $subGames 'GPU: 3D load threshold'
foreach ($v in 20, 30, 50, 70) { [void](Add-Choice $subLoad "Another app uses $v%+" 'GpuLoadPct' $v) }
$subResume = Add-Sub $subGames 'Resume after game closes'
foreach ($v in 15, 30, 60, 120, 300) { [void](Add-Choice $subResume $(if ($v -lt 60) { "$v seconds" } else { "$($v / 60) min" }) 'ResumeAfterSec' $v) }
$miNotGame = Add-Sub $subGames 'Not a game - ignore this app'
$miNotGame.add_Click({
  $proc = $script:Game.Process
  if (-not $proc) { return }
  $script:S.GpuIgnore = @(@($script:S.GpuIgnore) + $proc | Where-Object { $_ } | Select-Object -Unique)
  Save-TraySettings
  Log "added $proc to GpuIgnore"
  $script:LastGame = $null; $script:GpuHits = 0
  Update-State
})
[void](Add-Sub $subGames 'Show GPU usage now').add_Click({
  $out = Join-Path $DataDir 'gpu-usage.txt'
  $hdr = "GPU users right now (pause when VRAM >= $($script:S.GpuVramGB) GB or 3D >= $($script:S.GpuLoadPct)% on 2 checks in a row)`r`n" +
         "Ignore list: edit GpuIgnore in the settings file, or use 'Not a game - ignore this app' while paused.`r`n"
  $hdr | Set-Content $out
  Get-GpuReport $script:S @(Get-Server | ForEach-Object { $_.Id }) | Format-Table -AutoSize | Out-String -Width 200 | Add-Content $out
  Start-Process notepad.exe $out
})
[void]$subGames.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
$miEmu    = Add-Sub $subGames 'Count emulators as games'
$miEmu.add_Click({ Set-Setting 'DetectEmulators' (-not $script:S.DetectEmulators) })
$miWinList = Add-Sub $subGames "Use Windows' game list (Game Bar)"
$miWinList.add_Click({ Set-Setting 'UseWindowsGameList' (-not $script:S.UseWindowsGameList) })
[void]$subGames.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
[void](Add-Sub $subGames 'Rescan and show detected libraries').add_Click({
  $script:Scan = Get-GameLibraries
  $out = Join-Path $DataDir 'game-libraries.txt'
  $lines = @("Scanned $($script:Scan.ScannedAt)", '', 'Game library folders (any process running from these = gaming):')
  $lines += $script:Scan.Libs | Sort-Object { $_.Launcher } | ForEach-Object { "  [{0}] {1}{2}" -f $_.Launcher, $_.Dir, $(if ($_.Name) { "  ($($_.Name))" } else { '' }) }
  $lines += '', 'Fallback folder patterns (any drive):'
  $lines += $GameFolderPatterns | ForEach-Object { "  [{0}] {1}" -f $_.L, $_.P }
  $lines += '', "Windows Game Bar list: $($script:Scan.Exes.Count) exes"
  $lines += $script:Scan.Exes.Keys | Sort-Object | ForEach-Object { "  $_" }
  $lines += '', "Extra games from settings: $(@($script:S.ExtraGames) -join ', ')"
  $lines | Set-Content $out
  Start-Process notepad.exe $out
  Log "game scan: $($script:Scan.Libs.Count) library folders, $($script:Scan.Exes.Count) Windows-listed game exes"
})

$subPopup = Add-Sub $settings 'On-screen popups'
$miPopups = Add-Sub $subPopup 'Show popups'
$miPopups.add_Click({ Set-Setting 'Popups' (-not $script:S.Popups) })
[void]$subPopup.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
foreach ($pos in @(@('Top center','TopCenter'), @('Top right','TopRight'), @('Bottom right','BottomRight'), @('Bottom center','BottomCenter'))) {
  [void](Add-Choice $subPopup $pos[0] 'PopupPosition' $pos[1])
}
[void]$subPopup.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
[void](Add-Sub $subPopup 'Test popup').add_Click({
  $was = $script:S.Popups; $script:S.Popups = $true
  Show-Popup 'LLM paused' 'Example Game (Steam) detected  -  GPU freed' 'Paused'
  $script:S.Popups = $was
})
$miBoot   = Add-Sub $settings 'Start with Windows'
$miBoot.add_Click({
  $t = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
  if ($t.State -eq 'Disabled') { Enable-ScheduledTask -TaskName $TaskName | Out-Null }
  else { Disable-ScheduledTask -TaskName $TaskName | Out-Null }
})
[void]$settings.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
[void](Add-Sub $settings 'Edit settings file').add_Click({ Start-Process notepad.exe $SettingsFile })

[void](Add-Item $menu 'Open folder' { Open-Url $Root })
[void](Add-Item $menu 'View log' { Start-Process notepad.exe $LogFile })
[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))
[void](Add-Item $menu 'Exit (stops the model)' {
  $script:Timer.Stop(); $script:FlagTimer.Stop(); Stop-Server; Log 'tray exited'
  $script:Tray.Visible = $false; $script:Tray.Dispose()
  [System.Windows.Forms.Application]::Exit()
})

# Refresh checkmarks / model list each time the menu opens
$menu.add_Opening({
  if (-not (Test-Path $SettingsFile)) { Save-TraySettings }
  $subModel.DropDownItems.Clear()
  Get-ChildItem $ModelsDir -Filter *.gguf -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notmatch '^mmproj' } |
    ForEach-Object {
      $i = Add-Choice $subModel ("{0}  ({1:N1} GB)" -f $_.Name, ($_.Length / 1GB)) 'Model' $_.Name
      $i.Checked = ($_.Name -eq $script:S.Model)
    }
  foreach ($sub in $subReason, $subCtx, $subNet, $subPopup, $subMode, $subVram, $subLoad, $subResume) {
    foreach ($i in $sub.DropDownItems) {
      if ($i.Tag -is [hashtable]) { $i.Checked = ("$($script:S[$i.Tag.Key])" -eq "$($i.Tag.Value)") }
    }
  }
  $miPause.Checked   = [bool]$script:S.PauseWhileGaming
  $miEmu.Checked     = [bool]$script:S.DetectEmulators
  $miWinList.Checked = [bool]$script:S.UseWindowsGameList
  $miPopups.Checked  = [bool]$script:S.Popups
  $miNotGame.Visible = ($script:Status -eq 'Paused' -and [bool]$script:Game.Process)
  if ($miNotGame.Visible) { $miNotGame.Text = "Not a game - ignore $($script:Game.Process)" }
  $gpuOn = ("$($script:S.DetectionMode)" -ne 'Launchers')
  $subVram.Enabled = $gpuOn; $subLoad.Enabled = $gpuOn
  $miBoot.Checked   = ((Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue).State -ne 'Disabled')
  $miOpen.Enabled = ($script:Status -eq 'Running')
  $miRestart.Enabled = ($script:Status -ne 'Off' -and $script:Status -ne 'Paused')
})

$script:Tray.ContextMenuStrip = $menu
$script:Tray.add_DoubleClick({ if ($script:Status -eq 'Running') { Open-Url "http://127.0.0.1:$($script:S.Port)" } })
$script:Tray.Icon = $Icons.Loading
$script:Tray.Text = 'Local LLM'
$script:Tray.Visible = $true

# ---------------------------------------------------------------- loop
$script:Timer = New-Object System.Windows.Forms.Timer
$script:Timer.Interval = $PollSeconds * 1000
$script:Timer.add_Tick({ try { Update-State } catch { Log "error: $_" } })
try { Update-State } catch { Log "error: $_" }
$script:Timer.Start()

# Fast check (0.5s) so the Ctrl+Alt+L hotkey reacts instantly
$script:FlagTimer = New-Object System.Windows.Forms.Timer
$script:FlagTimer.Interval = 500
$script:FlagTimer.add_Tick({
  if ((Test-Path $OffFlag) -ne $script:LastFlag) { try { Update-State } catch { Log "error: $_" } }
})
$script:FlagTimer.Start()
[System.Windows.Forms.Application]::Run()
