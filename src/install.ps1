#Requires -RunAsAdministrator
#Requires -Version 5.1
<#
.SYNOPSIS
  One-time setup: llama.cpp (Vulkan), Qwen 3.8 27B Q4, low-power always-on settings,
  a system-tray app that starts at logon, and a Ctrl+Alt+L on/off hotkey.

.DESCRIPTION
  Safe to re-run: downloads resume, the settings backup is only taken once, and an
  existing install (including the older flat C:\LLM layout) is upgraded in place.
  Run from an elevated PowerShell in the folder containing these scripts:
    powershell -ExecutionPolicy Bypass -File .\install.ps1

.PARAMETER LlamaCppTag
  llama.cpp release to install, e.g. 'b6500'. Default 'latest'.
.PARAMETER UpdateLlamaCpp
  Re-download llama.cpp even if it is already installed.
.PARAMETER SkipModel
  Don't download the default model (bring your own .gguf into C:\LLM\models and pick it from the tray).
.PARAMETER SkipPowerSettings
  Leave the Windows power plan alone.
.PARAMETER SkipWakeOnLan
  Leave network adapter wake settings alone.
#>
[CmdletBinding()]
param(
  [string]$LlamaCppTag = 'latest',
  [switch]$UpdateLlamaCpp,
  [switch]$SkipModel,
  [switch]$SkipPowerSettings,
  [switch]$SkipWakeOnLan
)
$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'   # Invoke-WebRequest is many times slower with the progress bar
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
. (Join-Path $PSScriptRoot 'config.ps1')
$Hotkey   = 'CTRL+ALT+L'
$AppFiles = 'config.ps1', 'server.ps1', 'llm-tray.ps1', 'llm-toggle.ps1', 'games.ps1', 'gpu.ps1', 'osd.ps1', 'uninstall.ps1'
Write-Host "No Drama Llama $AppVersion"

function Assert-Sha256($file, $expected) {
  if (-not $expected) { return }
  Write-Host "  Verifying SHA-256 of $(Split-Path $file -Leaf)..."
  $actual = (Get-FileHash $file -Algorithm SHA256).Hash
  if ($actual -ne $expected) {
    Remove-Item $file -Force -ErrorAction SilentlyContinue
    throw "Checksum mismatch for $file (expected $expected, got $actual). The file was deleted; re-run install.ps1."
  }
}

# 0. Stop a running copy (re-install / upgrade) so files aren't locked ---------------
Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
  Where-Object { $_.CommandLine -match 'llm-(guard|tray)\.ps1' } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Get-Process -Name 'llama-server' -ErrorAction SilentlyContinue |
  Where-Object { $_.Path -eq $ServerExe } | Stop-Process -Force -ErrorAction SilentlyContinue

# 1. Folders + scripts ---------------------------------------------------------
New-Item -ItemType Directory -Force -Path $Root, $AppDir, $DataDir, $ModelsDir, "$Root\llama" | Out-Null
foreach ($f in $AppFiles) { Copy-Item (Join-Path $PSScriptRoot $f) $AppDir -Force }

# Upgrade from the flat layout (scripts, settings and logs directly in C:\LLM)
foreach ($f in $AppFiles) { Remove-Item (Join-Path $Root $f) -Force -ErrorAction SilentlyContinue }
foreach ($f in 'settings.json', 'off.flag', 'tray.log', 'server.log', 'gpu-usage.txt', 'game-libraries.txt') {
  $old = Join-Path $Root $f
  if (Test-Path $old) {
    if (Test-Path (Join-Path $DataDir $f)) { Remove-Item $old -Force } else { Move-Item $old $DataDir }
  }
}

# Permissions. The tray app runs elevated, so C:\LLM (its scripts and llama-server.exe)
# must be admin-only. Folders created under C:\ let every signed-in user modify them
# by default, so replace the inherited ACL: Administrators + SYSTEM full, Users read.
# Only data\ (settings, on/off flag, logs) and models\ are writable by you.
$acl = New-Object System.Security.AccessControl.DirectorySecurity   # fresh object: only the DACL is written
$acl.SetSecurityDescriptorSddlForm('D:PAI(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;0x1200a9;;;BU)', 'Access')
Set-Acl $Root $acl
icacls "$Root\*" /reset /T /C /Q | Out-Null   # drop explicit grants left by older installs
$me = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
foreach ($d in $DataDir, $ModelsDir) {
  icacls $d /grant "*${me}:(OI)(CI)M" /Q | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "icacls failed on $d (exit $LASTEXITCODE)" }
}

# 2. llama.cpp, Vulkan build (avoids the ROCm idle-power bug on RDNA3) ------------
if ($UpdateLlamaCpp -or -not (Test-Path $ServerExe)) {
  $api = if ($LlamaCppTag -eq 'latest') { 'https://api.github.com/repos/ggml-org/llama.cpp/releases/latest' }
         else { "https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/$LlamaCppTag" }
  $rel   = Invoke-RestMethod $api -Headers @{ 'User-Agent' = 'no-drama-llama' }
  $asset = $rel.assets | Where-Object name -like '*bin-win-vulkan-x64.zip' | Select-Object -First 1
  if (-not $asset) { throw "llama.cpp release $($rel.tag_name) has no *bin-win-vulkan-x64.zip asset. Try -LlamaCppTag <older tag>." }
  Write-Host "Downloading llama.cpp $($rel.tag_name) (Vulkan)..."
  $zip = Join-Path $env:TEMP $asset.name
  Invoke-WebRequest $asset.browser_download_url -OutFile $zip -UseBasicParsing
  if ("$($asset.digest)" -match '^sha256:([0-9a-fA-F]{64})$') { Assert-Sha256 $zip $Matches[1] }
  Get-ChildItem "$Root\llama" -Force | Remove-Item -Recurse -Force
  Expand-Archive $zip "$Root\llama" -Force
  Remove-Item $zip -Force
  # Some releases nest files in a subfolder; flatten if so
  $exe = Get-ChildItem "$Root\llama" -Recurse -Filter llama-server.exe | Select-Object -First 1
  if (-not $exe) { throw "llama-server.exe not found in $($asset.name)." }
  if ($exe.DirectoryName -ne "$Root\llama") { Move-Item "$($exe.DirectoryName)\*" "$Root\llama" -Force }
}

# 3. Model (17.6 GB, resumable, verified) -------------------------------------
if (-not $SkipModel -and -not (Test-Path $ModelFile)) {
  $url  = "https://huggingface.co/$ModelRepo/resolve/main/$DefaultModel"
  $part = "$ModelFile.part"
  # Expected size + SHA-256 from Hugging Face's file listing (LFS metadata)
  $meta = $null
  try {
    $meta = (Invoke-RestMethod "https://huggingface.co/api/models/$ModelRepo/tree/main" -UseBasicParsing) |
      Where-Object path -eq $DefaultModel | Select-Object -First 1
  } catch { Write-Warning "Couldn't read model checksum from Hugging Face ($($_.Exception.Message)); skipping verification." }
  $size = if ($meta.lfs) { [long]$meta.lfs.size } else { 0 }

  if (-not ($size -and (Test-Path $part) -and (Get-Item $part).Length -eq $size)) {
    Write-Host "Downloading $DefaultModel ($([Math]::Round($size / 1e9, 1)) GB). Interrupted? Re-run install.ps1 to resume."
    curl.exe -L --fail --retry 5 --retry-delay 5 -C - -o $part $url
    if ($LASTEXITCODE -ne 0) { throw "Model download failed (curl exit $LASTEXITCODE). Re-run install.ps1 to resume." }
  }
  if ($size -and (Get-Item $part).Length -ne $size) {
    throw "Model download is incomplete ($((Get-Item $part).Length) of $size bytes). Re-run install.ps1 to resume."
  }
  Assert-Sha256 $part $meta.lfs.oid
  Move-Item $part $ModelFile -Force
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
if (-not $SkipPowerSettings) {
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
}

# 4b. Wake-on-LAN, so you can power the PC back on remotely after a shutdown -------
if (-not $SkipWakeOnLan) {
  Write-Host 'Enabling Wake-on-LAN on wired adapters...'
  $nics = Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' -and $_.MediaType -eq '802.3' }
  foreach ($nic in $nics) {
    Set-NetAdapterPowerManagement -Name $nic.Name -WakeOnMagicPacket Enabled -ErrorAction SilentlyContinue
    Get-NetAdapterAdvancedProperty -Name $nic.Name -ErrorAction SilentlyContinue |
      Where-Object { $_.DisplayName -match 'Wake.*Magic|Shutdown Wake' } |
      ForEach-Object { Set-NetAdapterAdvancedProperty -Name $nic.Name -DisplayName $_.DisplayName -DisplayValue 'Enabled' -ErrorAction SilentlyContinue }
    Write-Host "  $($nic.Name)  MAC for your WoL app: $($nic.MacAddress)"
  }
}

# 5. Tray app: starts at logon, elevated (so it can see elevated game processes)
$action    = New-ScheduledTaskAction -Execute 'conhost.exe' `
  -Argument "--headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$AppDir\llm-tray.ps1`""
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
$lnk.Arguments   = "--headless powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$AppDir\llm-toggle.ps1`""
$lnk.Hotkey      = $Hotkey
$lnk.WindowStyle = 7
$lnk.Save()

# Start Menu entry to relaunch the tray app if you exit it (runs the elevated task, no UAC prompt)
$tray = $sh.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Programs')) $TrayShortcutName))
$tray.TargetPath  = 'schtasks.exe'
$tray.Arguments   = "/run /tn `"$TaskName`""
$tray.WindowStyle = 7
$tray.Save()

Start-ScheduledTask -TaskName $TaskName
$cfg = Get-Settings
Write-Host "`nDone. Look for the dot in the system tray (click ^ if hidden, drag it onto the taskbar to pin)."
Write-Host "Chat + OpenAI-compatible API: http://127.0.0.1:$($cfg.Port)   Hotkey: $Hotkey   Log: $LogFile"
Write-Host "To remove everything later: powershell -ExecutionPolicy Bypass -File `"$AppDir\uninstall.ps1`""
