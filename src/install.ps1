#Requires -RunAsAdministrator
# One-time setup: llama.cpp (Vulkan), Qwen 3.8 27B Q4, low-power always-on settings,
# a system-tray app that starts at logon, and a Ctrl+Alt+L on/off hotkey.
# Run from an elevated PowerShell in the folder containing these scripts:
#   powershell -ExecutionPolicy Bypass -File .\install.ps1
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'config.ps1')
$Hotkey = 'CTRL+ALT+L'

# 1. Folders + scripts ---------------------------------------------------------
New-Item -ItemType Directory -Force -Path $Root, "$Root\llama", "$Root\models" | Out-Null
Copy-Item "$PSScriptRoot\config.ps1", "$PSScriptRoot\llm-tray.ps1", "$PSScriptRoot\llm-toggle.ps1", "$PSScriptRoot\games.ps1", "$PSScriptRoot\gpu.ps1", "$PSScriptRoot\osd.ps1" $Root -Force
icacls $Root /grant "${env:USERNAME}:(OI)(CI)M" | Out-Null   # hotkey runs non-elevated

# 2. llama.cpp, Vulkan build (avoids the ROCm idle-power bug on RDNA3) ------------
if (-not (Test-Path $ServerExe)) {
  Write-Host 'Downloading latest llama.cpp Vulkan build...'
  $rel   = Invoke-RestMethod 'https://api.github.com/repos/ggml-org/llama.cpp/releases/latest'
  $asset = $rel.assets | Where-Object name -like '*bin-win-vulkan-x64.zip' | Select-Object -First 1
  $zip   = Join-Path $env:TEMP $asset.name
  Invoke-WebRequest $asset.browser_download_url -OutFile $zip -UseBasicParsing
  Expand-Archive $zip "$Root\llama" -Force
  # Some releases nest files in a subfolder; flatten if so
  $exe = Get-ChildItem "$Root\llama" -Recurse -Filter llama-server.exe | Select-Object -First 1
  if ($exe.DirectoryName -ne "$Root\llama") { Move-Item "$($exe.DirectoryName)\*" "$Root\llama" -Force }
}

# 3. Model (17.6 GB, resumable) -----------------------------------------------
if (-not (Test-Path $ModelFile)) {
  Write-Host 'Downloading Qwen3.8-27B-UD-Q4_K_XL.gguf (17.6 GB)...'
  curl.exe -L -C - -o $ModelFile `
    'https://huggingface.co/unsloth/Qwen3.8-27B-GGUF/resolve/main/Qwen3.8-27B-UD-Q4_K_XL.gguf'
}

# 3b. Back up current settings so uninstall.ps1 can put them back ---------------
# Only on the first run, so re-running install never overwrites the original values.
if (-not (Test-Path $BackupFile)) {
  Write-Host 'Backing up current power and network settings...'
  $nicsNow = Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' -and $_.MediaType -eq '802.3' }
  $backup = [ordered]@{
    ActiveScheme     = ((powercfg /getactivescheme) -replace '.*GUID:\s*([0-9a-f-]+).*', '$1').Trim()
    HibernateEnabled = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Power' -ErrorAction SilentlyContinue).HibernateEnabled
    Power            = @($PowerSettings | ForEach-Object {
                          @{ Sub = $_.Sub; Set = $_.Set; Value = Get-AcIndex 'SCHEME_BALANCED' $_.Sub $_.Set } })
    Nics             = @($nicsNow | ForEach-Object {
                          $n = $_.Name
                          @{ Name  = $n
                             Magic = [string](Get-NetAdapterPowerManagement -Name $n -ErrorAction SilentlyContinue).WakeOnMagicPacket
                             Adv   = @(Get-NetAdapterAdvancedProperty -Name $n -ErrorAction SilentlyContinue |
                                       Where-Object { $_.DisplayName -match 'Wake.*Magic|Shutdown Wake' } |
                                       ForEach-Object { @{ DisplayName = $_.DisplayName; DisplayValue = $_.DisplayValue } }) } })
  }
  $backup | ConvertTo-Json -Depth 5 | Set-Content $BackupFile
}

# 4. Always-on, low-power Windows settings -------------------------------------
Write-Host 'Configuring power plan...'
powercfg /setactive SCHEME_BALANCED
powercfg /change standby-timeout-ac 0      # never sleep
powercfg /change hibernate-timeout-ac 0
powercfg /change monitor-timeout-ac 10     # screen off after 10 min
powercfg /change disk-timeout-ac 20
powercfg /hibernate off
powercfg /setacvalueindex SCHEME_CURRENT SUB_PCIEXPRESS ASPM 2              # PCIe link power: max savings
powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR PROCTHROTTLEMIN 5    # let CPU clock down
powercfg /setacvalueindex SCHEME_CURRENT 2a737441-1930-4402-8d77-b2bebba308a3 48e6b7a6-50f5-4782-a5d4-53bb8f07e226 1  # USB selective suspend
powercfg /setactive SCHEME_CURRENT

# 4b. Wake-on-LAN, so you can power the PC back on remotely after a shutdown -------
Write-Host 'Enabling Wake-on-LAN on wired adapters...'
$nics = Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' -and $_.MediaType -eq '802.3' }
foreach ($nic in $nics) {
  Set-NetAdapterPowerManagement -Name $nic.Name -WakeOnMagicPacket Enabled -ErrorAction SilentlyContinue
  Get-NetAdapterAdvancedProperty -Name $nic.Name -ErrorAction SilentlyContinue |
    Where-Object { $_.DisplayName -match 'Wake.*Magic|Shutdown Wake' } |
    ForEach-Object { Set-NetAdapterAdvancedProperty -Name $nic.Name -DisplayName $_.DisplayName -DisplayValue 'Enabled' -ErrorAction SilentlyContinue }
  Write-Host "  $($nic.Name)  MAC for your WoL app: $($nic.MacAddress)"
}

# 5. Tray app: starts at logon, elevated (so it can see elevated game processes)
$action    = New-ScheduledTaskAction -Execute 'conhost.exe' `
  -Argument "--headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$Root\llm-tray.ps1`""
$trigger   = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Highest
$settings  = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -AllowStartIfOnBatteries `
  -DontStopIfGoingOnBatteries -RestartCount 99 -RestartInterval (New-TimeSpan -Minutes 1)
Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger `
  -Principal $principal -Settings $settings -Force | Out-Null

# 6. Hotkey: Start Menu shortcut with a global key combo ----------------------
$lnkPath = Join-Path ([Environment]::GetFolderPath('Programs')) $ShortcutName
$sh  = New-Object -ComObject WScript.Shell
$lnk = $sh.CreateShortcut($lnkPath)
$lnk.TargetPath  = 'conhost.exe'
$lnk.Arguments   = "--headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$Root\llm-toggle.ps1`""
$lnk.Hotkey      = $Hotkey
$lnk.WindowStyle = 7
$lnk.Save()

# Start Menu entry to relaunch the tray app if you exit it (runs the elevated task, no UAC prompt)
$tray = $sh.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Programs')) $TrayShortcutName))
$tray.TargetPath  = 'schtasks.exe'
$tray.Arguments   = "/run /tn `"$TaskName`""
$tray.WindowStyle = 7
$tray.Save()

# Stop any copy already running (re-install / upgrade from the old guard script)
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
  Where-Object { $_.CommandLine -match 'llm-(guard|tray)\.ps1' } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Get-Process -Name 'llama-server' -ErrorAction SilentlyContinue | Stop-Process -Force
Start-ScheduledTask -TaskName $TaskName
$cfg = Get-Settings
Write-Host "`nDone. Look for the dot in the system tray (click ^ if hidden, drag it onto the taskbar to pin)."
Write-Host "Chat + OpenAI-compatible API: http://127.0.0.1:$($cfg.Port)   Hotkey: $Hotkey   Log: $LogFile"
