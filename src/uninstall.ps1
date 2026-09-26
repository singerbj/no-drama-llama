#Requires -RunAsAdministrator
# Removes everything install.ps1 did and restores your original Windows settings.
# Run from an elevated PowerShell in the folder containing these scripts:
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1            # remove everything
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1 -KeepModel # keep your models (moved to Downloads)
# A copy of this script is also installed as C:\LLM\app\uninstall.ps1.
#Requires -Version 5.1
param([switch]$KeepModel)
$ErrorActionPreference = 'Continue'
. (Join-Path $PSScriptRoot 'config.ps1')
if ($PWD.Path -like "$Root*") { Set-Location $env:USERPROFILE }   # can't delete a folder we're standing in

# 1. Stop the tray app and the model ---------------------------------------------
Write-Host 'Stopping tray app and model server...'
Stop-ScheduledTask   -TaskName $TaskName -ErrorAction SilentlyContinue
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
  Where-Object { $_.CommandLine -match 'llm-(guard|tray)\.ps1' } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Get-Process -Name 'llama-server' -ErrorAction SilentlyContinue |
  Where-Object { $_.Path -eq $ServerExe } | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# 2. Hotkey shortcut ------------------------------------------------------------
$lnk = Join-Path ([Environment]::GetFolderPath('Programs')) $ShortcutName
if (Test-Path $lnk) { Remove-Item $lnk -Force; Write-Host 'Removed Ctrl+Alt+L hotkey shortcut.' }
$lnk = Join-Path ([Environment]::GetFolderPath('Programs')) $TrayShortcutName
if (Test-Path $lnk) { Remove-Item $lnk -Force; Write-Host 'Removed Local LLM Start Menu entry.' }

# 3. Restore power + network settings --------------------------------------------
if (Test-Path $BackupFile) {
  Write-Host 'Restoring original power and network settings from backup...'
  $b = Get-Content $BackupFile -Raw | ConvertFrom-Json

  foreach ($p in $b.Power) {
    if ($null -ne $p.Value) { powercfg /setacvalueindex SCHEME_BALANCED $p.Sub $p.Set $p.Value }
  }
  if ($b.HibernateEnabled -eq 1) { powercfg /hibernate on }
  if ($b.ActiveScheme) { powercfg /setactive $b.ActiveScheme } else { powercfg /setactive SCHEME_BALANCED }

  foreach ($n in $b.Nics) {
    if ($n.Magic) { Set-NetAdapterPowerManagement -Name $n.Name -WakeOnMagicPacket $n.Magic -ErrorAction SilentlyContinue }
    foreach ($a in $n.Adv) {
      Set-NetAdapterAdvancedProperty -Name $n.Name -DisplayName $a.DisplayName -DisplayValue $a.DisplayValue -ErrorAction SilentlyContinue
    }
  }
} else {
  Write-Warning 'No settings backup found (install ran before backups were added).'
  $ans = Read-Host 'Reset ALL power plans to Windows defaults instead? This also undoes any other power tweaks you made. (y/N)'
  if ($ans -match '^[yY]') {
    powercfg -restoredefaultschemes
    powercfg /hibernate on
    Write-Host 'Power plans reset to Windows defaults.'
  } else {
    Write-Host 'Power settings left as they are. Change them in Settings > System > Power.'
  }
  Write-Host 'Wake-on-LAN left enabled (harmless; turn off in Device Manager > network adapter > Power Management).'
}

# 4. Auto sign-in -----------------------------------------------------------------
$wl = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon'
if ((Get-ItemProperty $wl -ErrorAction SilentlyContinue).AutoAdminLogon -eq '1') {
  $ans = Read-Host 'Automatic sign-in is on. Turn it off? (Y/n)'
  if ($ans -notmatch '^[nN]') {
    Set-ItemProperty $wl -Name AutoAdminLogon -Value '0'
    Remove-ItemProperty $wl -Name DefaultPassword -ErrorAction SilentlyContinue
    Write-Host 'Auto sign-in disabled. Also open Autologon and click "Disable" to erase the stored password,'
    Write-Host 'then uninstall it: winget uninstall Microsoft.Sysinternals.Autologon'
  }
}

# 5. Files -------------------------------------------------------------------------
if ($KeepModel) {
  $dest = Join-Path ([Environment]::GetFolderPath('UserProfile')) 'Downloads'
  foreach ($m in Get-ChildItem $ModelsDir -Filter *.gguf -ErrorAction SilentlyContinue) {
    Move-Item $m.FullName $dest -Force
    Write-Host "Model kept: $dest\$($m.Name)"
  }
}
if (Test-Path $Root) {
  Remove-Item $Root -Recurse -Force
  Write-Host "Deleted $Root (llama.cpp, scripts, logs$(if (-not $KeepModel) { ', model' }))."
}

Write-Host @'

Done. Windows-side changes are reverted. Undo these BIOS settings by hand if you changed them:
  - Restore on AC Power Loss  -> Power Off (or Last State)
  - ErP / EuP                 -> Enabled
  - Wake on LAN / Power On by PCI-E -> Disabled
  - Power On by RTC Alarm     -> Disabled
Adrenalin tweaks (FreeSync, power limit, undervolt) are yours to keep or reset.
'@
