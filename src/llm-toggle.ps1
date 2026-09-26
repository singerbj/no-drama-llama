# Hotkey target: flips the LLM on/off and shows a notification.
. (Join-Path $PSScriptRoot 'config.ps1')

if (Test-Path $OffFlag) {
  Remove-Item $OffFlag -Force
  $msg = 'LLM ON - but the tray app is not running. Start "Local LLM" from the Start menu.'
} else {
  New-Item -ItemType File -Path $OffFlag -Force | Out-Null
  $msg = 'LLM OFF (tray app not running)'
}

# The tray app shows its own on-screen popup within half a second. Only fall back
# to a notification if the tray app isn't running.
$trayRunning = $true
try { [System.Threading.Mutex]::OpenExisting('Global\LocalLLMTray').Dispose() }
catch [System.Threading.WaitHandleCannotBeOpenedException] { $trayRunning = $false }
catch { }   # access denied = exists (tray runs elevated)
if ($trayRunning) { exit }

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$n = New-Object System.Windows.Forms.NotifyIcon
$n.Icon = [System.Drawing.SystemIcons]::Information
$n.Visible = $true
$n.ShowBalloonTip(3000, 'Local LLM', $msg, 'Info')
Start-Sleep -Seconds 4
$n.Dispose()
