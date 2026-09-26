# Game detection.
# 1. Get-GameLibraries reads every launcher's own records (config files + registry) to find
#    where games are installed on ANY drive, plus Windows' own list of known game exes.
# 2. Find-RunningGame matches running processes against those folders, a set of
#    default-folder patterns (fallback), known game/emulator process names, and Steam's
#    "currently running app" registry value.

# ---------------------------------------------------------------- fallback folder patterns
# Default install layouts for each launcher (any drive). Used in addition to discovered libraries.
$GameFolderPatterns = @(
  @{ L = 'Steam';            P = '\\steamapps\\common\\' },
  @{ L = 'Epic';             P = '\\Epic Games\\' },
  @{ L = 'Xbox / Game Pass'; P = '\\XboxGames\\' },
  @{ L = 'Xbox / Game Pass'; P = '\\ModifiableWindowsApps\\' },
  @{ L = 'GOG';              P = '\\GOG Galaxy\\Games\\' },
  @{ L = 'GOG';              P = '\\GOG Games\\' },
  @{ L = 'EA';               P = '\\EA Games\\' },
  @{ L = 'EA (Origin)';      P = '\\Origin Games\\' },
  @{ L = 'Ubisoft';          P = '\\Ubisoft Game Launcher\\games\\' },
  @{ L = 'Riot';             P = '\\Riot Games\\' },
  @{ L = 'Rockstar';         P = '\\Rockstar Games\\' },
  @{ L = 'Amazon';           P = '\\Amazon Games\\Library\\' },
  @{ L = 'itch.io';          P = '\\itch\\apps\\' },
  @{ L = 'Heroic';           P = '\\Games\\Heroic\\' },
  @{ L = 'HoYoPlay';         P = '\\HoYoPlay\\games\\' },
  @{ L = 'Meta / Oculus';    P = '\\Oculus\\Software\\Software\\' },
  @{ L = 'Bethesda';         P = '\\Bethesda\.net Launcher\\games\\' },
  @{ L = 'Battlestate (Tarkov)'; P = '\\Battlestate Games\\' },
  @{ L = 'Games folder';     P = '^[A-Za-z]:\\Games\\' }            # Wargaming, standalone installs, custom libraries
)

# Folders inside game locations that are launchers, redistributables or anti-cheat - never games.
$NotGameFolderPatterns = @(
  '\\steamapps\\common\\(Steamworks Shared|wallpaper_engine|Lossless Scaling)\\',
  '\\Epic Games\\(Launcher|Epic Online Services)\\',
  '\\Riot Games\\Riot Client\\', '\\Riot Vanguard\\',
  '\\Rockstar Games\\(Launcher|Social Club)\\',
  '\\Electronic Arts\\EA Desktop\\', '\\Origin\\',
  '\\Ubisoft Game Launcher\\(?!games\\)',
  '\\GOG Galaxy\\(?!Games\\)',
  '\\Battle\.net\\', '\\HoYoPlay\\(?!games\\)',
  '\\Oculus\\Support\\', '\\Amazon Games\\App\\', '\\Battlestate Games\\BsgLauncher\\',
  '\\_CommonRedist\\', '\\__Installer\\', '\\Redist\\', '\\DirectX\\', '\\Support\\',
  '\\EasyAntiCheat', '\\BattlEye\\'
)

# Process names (no .exe) that are never games, even inside a game folder.
$NotGameProcesses = @(
  # launchers / clients / helpers
  'steam','steamwebhelper','steamservice','GameOverlayUI','steamerrorreporter',
  'EpicGamesLauncher','EpicWebHelper','EOSOverlayRenderer-Win64-Shipping',
  'GalaxyClient','GalaxyClientService','GalaxyCommunication','GOG Galaxy Notifications Renderer',
  'EADesktop','EABackgroundService','EALocalHostSvc','Link2EA',
  'UbisoftConnect','upc','UplayWebCore','UbisoftGameLauncher','UbisoftGameLauncher64',
  'Battle.net','Agent','BlizzardError','Battle.net Update Helper',
  'XboxPcApp','GamingServices','GamingServicesNet','gamelaunchhelper','GameBar','GameBarFTServer',
  'RiotClientServices','RiotClientUx','RiotClientUxRender','RiotClientCrashHandler',
  'LeagueClient','LeagueClientUx','LeagueClientUxRender','VALORANT','vgc','vgtray',
  'Launcher','LauncherPatcher','RockstarService','RockstarErrorHandler','SocialClubHelper','PlayGTAV',
  'Amazon Games','Amazon Games UI','itch','butler','Heroic','legendary','gogdl','nile',
  'OVRServer_x64','OculusClient','OVRRedir','HYP','launcher','wgc','wgc_renderer_host','BsgLauncher',
  'Playnite.DesktopApp','Playnite.FullscreenApp','Humble App',
  'wallpaper32','wallpaper64','webwallpaper32',
  # anti-cheat / crash reporters / installers / embedded browsers
  'EasyAntiCheat','EasyAntiCheat_EOS','EasyAntiCheat_EOS_Setup','start_protected_game','BEService','BEService_x64',
  'UnityCrashHandler64','UnityCrashHandler32','CrashReportClient','crashpad_handler','CrashHandler',
  'vc_redist.x64','vc_redist.x86','DXSETUP','UE4PrereqSetup_x64','UEPrereqSetup_x64','unins000',
  'QtWebEngineProcess','CefSharp.BrowserSubprocess'
)

# Games that are easy to miss by folder (custom install paths, launcher-managed).
$KnownGameProcesses = @(
  'League of Legends','VALORANT-Win64-Shipping',
  'EscapeFromTarkov','EscapeFromTarkov_BE','EscapeFromTarkovArena','EscapeFromTarkovArena_BE',
  'GenshinImpact','YuanShen','StarRail','ZenlessZoneZero','BH3',
  'GTA5','GTA5_Enhanced','RDR2','Warframe.x64','PathOfExile','PathOfExile_x64','PathOfExileSteam'
)

$EmulatorProcesses = @(
  'retroarch','Dolphin','pcsx2-qt','pcsx2','rpcs3','duckstation-qt-x64-ReleaseLTCG','duckstation-qt','PPSSPPWindows64',
  'Cemu','Ryujinx','yuzu','suyu','eden','citron','sudachi','citra-qt','azahar','lime3ds',
  'xemu','xenia','xenia_canary','mGBA','melonDS','mame','Project64','snes9x-x64','bsnes',
  'flycast','redream','Vita3K','shadPS4','ares','EmuHawk','scummvm','dosbox','dosbox-x'
)

# Steam app IDs that set RunningAppID but aren't games.
$NotGameSteamAppIds = @(431960, 993090, 228980)   # Wallpaper Engine, Lossless Scaling, Steamworks Redistributables

# ---------------------------------------------------------------- discovery
function Add-GameLib([System.Collections.ArrayList]$list, $dir, $launcher, $name) {
  if (-not $dir) { return }
  try { $d = [IO.Path]::GetFullPath(("$dir" -replace '/', '\').Trim('"')).TrimEnd('\') + '\' } catch { return }
  if (($d.Split('\', [StringSplitOptions]::RemoveEmptyEntries)).Count -lt 2) { return }       # never a whole drive
  if ($d -match '^[A-Za-z]:\\(Program Files( \(x86\))?|Windows|ProgramData|Users\\[^\\]+|Users)\\$') { return }
  foreach ($x in $NotGameFolderPatterns) { if ($d -match $x) { return } }
  if ($list | Where-Object { $_.Dir -eq $d }) { return }
  [void]$list.Add(@{ Dir = $d; Launcher = $launcher; Name = $name })
}

function Get-RegValues($path) {
  Get-ChildItem $path -ErrorAction SilentlyContinue | ForEach-Object { Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue }
}

function Read-JsonFile($path) {
  if (Test-Path $path) { try { Get-Content $path -Raw | ConvertFrom-Json } catch { $null } }
}

function Get-GameLibraries {
  $libs = New-Object System.Collections.ArrayList
  $exes = @{}
  $script:SteamRoots = @()

  # Steam: every library in libraryfolders.vdf
  $steam = (Get-ItemProperty 'HKCU:\Software\Valve\Steam' -ErrorAction SilentlyContinue).SteamPath
  if (-not $steam) { $steam = (Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Valve\Steam' -ErrorAction SilentlyContinue).InstallPath }
  if ($steam) {
    $roots = @($steam -replace '/', '\')
    $vdf = Join-Path $steam 'steamapps\libraryfolders.vdf'
    if (Test-Path $vdf) {
      $roots += Select-String -Path $vdf -Pattern '"path"\s+"([^"]+)"' |
        ForEach-Object { $_.Matches[0].Groups[1].Value -replace '\\\\', '\' }
    }
    $script:SteamRoots = $roots | Select-Object -Unique
    foreach ($r in $script:SteamRoots) { Add-GameLib $libs "$r\steamapps\common" 'Steam' $null }
  }

  # Epic: launcher manifests (skip Unreal Engine installs)
  Get-ChildItem "$env:ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item" -ErrorAction SilentlyContinue | ForEach-Object {
    $m = Read-JsonFile $_.FullName
    if ($m -and $m.AppName -notlike 'UE_*') { Add-GameLib $libs $m.InstallLocation 'Epic' $m.DisplayName }
  }

  # GOG Galaxy
  Get-RegValues 'HKLM:\SOFTWARE\WOW6432Node\GOG.com\Games' | ForEach-Object { Add-GameLib $libs $_.path 'GOG' $_.gameName }

  # EA app / Origin
  foreach ($k in 'HKLM:\SOFTWARE\WOW6432Node\Electronic Arts', 'HKLM:\SOFTWARE\WOW6432Node\EA Games', 'HKLM:\SOFTWARE\WOW6432Node\Origin Games') {
    Get-ChildItem $k -ErrorAction SilentlyContinue | ForEach-Object {
      $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
      Add-GameLib $libs $p.'Install Dir' 'EA' $_.PSChildName
    }
  }

  # Ubisoft Connect
  Get-RegValues 'HKLM:\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs' | ForEach-Object { Add-GameLib $libs $_.InstallDir 'Ubisoft' $null }
  $yml = "$env:LOCALAPPDATA\Ubisoft Game Launcher\settings.yml"   # custom default library folder
  if (Test-Path $yml) {
    $m = Select-String -Path $yml -Pattern 'game_installation_path:\s*"?([^"\r\n]+)' | Select-Object -First 1
    if ($m) { Add-GameLib $libs $m.Matches[0].Groups[1].Value.Trim() 'Ubisoft' $null }
  }

  # Rockstar
  Get-ChildItem 'HKLM:\SOFTWARE\WOW6432Node\Rockstar Games' -ErrorAction SilentlyContinue | ForEach-Object {
    $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
    Add-GameLib $libs $p.InstallFolder 'Rockstar' $_.PSChildName
  }

  # Riot: client installs json + per-product settings
  $riot = Read-JsonFile "$env:ProgramData\Riot Games\RiotClientInstalls.json"
  if ($riot -and $riot.associated_client) {
    foreach ($prop in $riot.associated_client.PSObject.Properties) { Add-GameLib $libs $prop.Name 'Riot' $null }
  }
  Get-ChildItem "$env:ProgramData\Riot Games\Metadata" -Recurse -Filter '*.product_settings.yaml' -ErrorAction SilentlyContinue | ForEach-Object {
    $m = Select-String -Path $_.FullName -Pattern 'product_install_full_path:\s*"?([^"\r\n]+)' | Select-Object -First 1
    if ($m) { Add-GameLib $libs $m.Matches[0].Groups[1].Value 'Riot' $null }
  }

  # Battle.net and other launchers that register games as installed programs
  $uninstall = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall',
               'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall',
               'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall'
  foreach ($u in $uninstall) {
    Get-RegValues $u | ForEach-Object {
      $us = "$($_.UninstallString)"
      $l = switch -Regex ($us) {
        'Blizzard Uninstaller|Battle\.net' { 'Battle.net'; break }
        'RiotClientServices'               { 'Riot'; break }
        'Uplay|Ubisoft'                    { 'Ubisoft'; break }
        'EAInstaller|EA Desktop|Origin'    { 'EA'; break }
        'Amazon Games'                     { 'Amazon'; break }
        'Rockstar'                         { 'Rockstar'; break }
        'GOG Galaxy|GalaxyClient'          { 'GOG'; break }
        default                            { $null }
      }
      if (-not $l -and "$($_.Publisher)" -match 'Battlestate Games') { $l = 'Battlestate (Tarkov)' }
      if (-not $l -and $_.PSChildName -like 'Uplay Install *') { $l = 'Ubisoft' }
      if ($l) { Add-GameLib $libs $_.InstallLocation $l $_.DisplayName }
    }
  }

  # Xbox / Game Pass: every drive with a .GamingRoot file names its games folder
  foreach ($drv in [IO.DriveInfo]::GetDrives() | Where-Object { $_.IsReady -and $_.DriveType -eq 'Fixed' }) {
    $gr = Join-Path $drv.RootDirectory.FullName '.GamingRoot'
    if (Test-Path $gr -ErrorAction SilentlyContinue) {
      try {
        $bytes = [IO.File]::ReadAllBytes($gr)
        $names = [Text.Encoding]::Unicode.GetString($bytes, 8, $bytes.Length - 8).Split([char]0) | Where-Object { $_ -match '\w' }
        foreach ($n in $names) { Add-GameLib $libs (Join-Path $drv.RootDirectory.FullName $n) 'Xbox / Game Pass' $null }
      } catch { }
    }
  }

  # Meta / Oculus libraries
  Get-RegValues 'HKCU:\Software\Oculus VR, LLC\Oculus\Libraries' | ForEach-Object {
    if ($_.OriginalPath) { Add-GameLib $libs (Join-Path $_.OriginalPath 'Software') 'Meta / Oculus' $null }
  }

  # Heroic / Legendary (Epic, GOG and Amazon games installed through Heroic)
  foreach ($f in "$env:APPDATA\heroic\legendaryConfig\legendary\installed.json", "$env:USERPROFILE\.config\legendary\installed.json") {
    $j = Read-JsonFile $f
    if ($j) { foreach ($p in $j.PSObject.Properties) { Add-GameLib $libs $p.Value.install_path 'Heroic (Epic)' $p.Value.title } }
  }
  $j = Read-JsonFile "$env:APPDATA\heroic\gog_store\installed.json"
  if ($j -and $j.installed) { foreach ($g in $j.installed) { Add-GameLib $libs $g.install_path 'Heroic (GOG)' $null } }
  $j = Read-JsonFile "$env:APPDATA\heroic\nile_config\nile\installed.json"
  if ($j) { foreach ($g in @($j)) { Add-GameLib $libs $g.path 'Heroic (Amazon)' $null } }

  # Humble App
  $j = Read-JsonFile "$env:APPDATA\Humble App\config.json"
  if ($j -and $j.'game-collection-4') {
    foreach ($g in $j.'game-collection-4') { if ($g.filePath) { Add-GameLib $libs $g.filePath 'Humble' $g.gameName } }
  }

  # HoYoPlay
  foreach ($k in 'HKCU:\Software\Cognosphere\HYP\1_0', 'HKCU:\Software\miHoYo\HYP\1_0') {
    Get-RegValues $k | ForEach-Object { Add-GameLib $libs $_.GameInstallPath 'HoYoPlay' $null }
  }

  # Windows' own list of exes it recognises as games (Game Bar)
  Get-RegValues 'HKCU:\System\GameConfigStore\Children' | ForEach-Object {
    if ($_.MatchedExeFullPath) { $exes[$_.MatchedExeFullPath.ToLower()] = $true }
  }

  @{ Libs = $libs; Exes = $exes; ScannedAt = Get-Date }
}

# ---------------------------------------------------------------- matching
function Get-SteamAppName($id) {
  foreach ($r in $script:SteamRoots) {
    $acf = Join-Path $r "steamapps\appmanifest_$id.acf"
    if (Test-Path $acf) {
      $m = Select-String -Path $acf -Pattern '"name"\s+"([^"]+)"' | Select-Object -First 1
      if ($m) { return $m.Matches[0].Groups[1].Value }
    }
  }
  "Steam app $id"
}

function Get-FriendlyName($p, $known) {
  $n = if ($known) { $known } elseif ($p.MainWindowTitle) { $p.MainWindowTitle } elseif ($p.Description) { $p.Description } else { $p.ProcessName }
  if ($n.Length -gt 40) { $n.Substring(0, 40) + '...' } else { $n }
}

# Returns @{ Name; Launcher } for the first running game, or $null.
function Find-RunningGame($scan, $settings) {
  $appId = (Get-ItemProperty 'HKCU:\Software\Valve\Steam' -ErrorAction SilentlyContinue).RunningAppID
  if ($appId -and $appId -ne 0 -and $NotGameSteamAppIds -notcontains [int]$appId) {
    return @{ Name = (Get-SteamAppName $appId); Launcher = 'Steam' }
  }
  $extra     = @($settings.ExtraGames)
  $emulators = [bool]$settings.DetectEmulators
  $useWin    = [bool]$settings.UseWindowsGameList

  foreach ($p in Get-Process -ErrorAction SilentlyContinue) {
    $n = $p.ProcessName
    if ($NotGameProcesses -contains $n) { continue }
    if ($extra -contains $n)              { return @{ Name = (Get-FriendlyName $p); Launcher = 'your list' } }
    if ($KnownGameProcesses -contains $n) { return @{ Name = (Get-FriendlyName $p); Launcher = 'known game' } }
    if ($emulators -and ($EmulatorProcesses -contains $n -or $n -like 'xenia*' -or $n -like 'duckstation*')) {
      return @{ Name = (Get-FriendlyName $p); Launcher = 'emulator' }
    }
    $path = $p.Path
    if (-not $path) { continue }
    $skip = $false
    foreach ($x in $NotGameFolderPatterns) { if ($path -match $x) { $skip = $true; break } }
    if ($skip) { continue }
    if ($useWin -and $scan.Exes.ContainsKey($path.ToLower())) { return @{ Name = (Get-FriendlyName $p); Launcher = 'Windows game list' } }
    foreach ($lib in $scan.Libs) {
      if ($path.StartsWith($lib.Dir, [StringComparison]::OrdinalIgnoreCase)) {
        return @{ Name = (Get-FriendlyName $p $lib.Name); Launcher = $lib.Launcher }
      }
    }
    foreach ($g in $GameFolderPatterns) { if ($path -match $g.P) { return @{ Name = (Get-FriendlyName $p); Launcher = $g.L } } }
  }
  $null
}

# Best label for a process found by GPU detection: its launcher/library name if its exe
# lives in a known game location, otherwise its window title / description.
function Get-LaunchLabel($p, $scan) {
  $path = $p.Path
  if ($path -and $scan) {
    foreach ($lib in $scan.Libs) {
      if ($path.StartsWith($lib.Dir, [StringComparison]::OrdinalIgnoreCase)) {
        return @{ Name = (Get-FriendlyName $p $lib.Name); Launcher = $lib.Launcher }
      }
    }
    foreach ($g in $GameFolderPatterns) { if ($path -match $g.P) { return @{ Name = (Get-FriendlyName $p); Launcher = $g.L } } }
  }
  @{ Name = (Get-FriendlyName $p); Launcher = 'GPU' }
}
