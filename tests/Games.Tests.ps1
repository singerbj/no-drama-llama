BeforeAll {
  . (Join-Path $PSScriptRoot '..\src\games.ps1')
  function New-Lib { New-Object System.Collections.ArrayList }
  function New-Proc($name, $path, $title = '') {
    [pscustomobject]@{ ProcessName = $name; Path = $path; MainWindowTitle = $title; Description = ''; Id = 1234 }
  }
  $script:NoScan   = @{ Libs = (New-Lib); Exes = @{}; ScannedAt = Get-Date }
  $script:Defaults = @{ ExtraGames = @(); DetectEmulators = $true; UseWindowsGameList = $true }
}

Describe 'Add-GameLib' -Skip:(-not $IsWindows -and $PSVersionTable.PSVersion.Major -gt 5) {
  # Uses Windows path rules ([IO.Path]::GetFullPath on C:\...), so Windows only.
  It 'normalises and adds a library folder' {
    $l = New-Lib
    Add-GameLib $l 'D:/SteamLibrary/steamapps/common' 'Steam' $null
    $l[0].Dir | Should -Be 'D:\SteamLibrary\steamapps\common\'
  }
  It 'never adds <_>' -ForEach 'C:\', 'C:\ProgramData', 'C:\Program Files', 'C:\Users\me', 'C:\Windows' {
    $l = New-Lib
    Add-GameLib $l $_ 'x' $null
    $l.Count | Should -Be 0
  }
  It 'skips launcher / redistributable folders' {
    $l = New-Lib
    Add-GameLib $l 'C:\Program Files\Epic Games\Launcher' 'Epic' $null
    $l.Count | Should -Be 0
  }
  It 'de-duplicates' {
    $l = New-Lib
    Add-GameLib $l 'E:\Epic Games\Fortnite' 'Epic' 'Fortnite'
    Add-GameLib $l 'E:\Epic Games\Fortnite\' 'Epic' 'Fortnite'
    $l.Count | Should -Be 1
  }
}

Describe 'Get-FriendlyName' {
  It 'prefers the launcher name, then window title, then process name' {
    Get-FriendlyName (New-Proc 'game' 'x' 'Window') 'Library Name' | Should -Be 'Library Name'
    Get-FriendlyName (New-Proc 'game' 'x' 'Window') | Should -Be 'Window'
    Get-FriendlyName (New-Proc 'game' 'x') | Should -Be 'game'
  }
  It 'truncates long names' {
    (Get-FriendlyName (New-Proc ('a' * 60) 'x')).Length | Should -Be 43
  }
}

Describe 'Find-RunningGame' {
  BeforeEach {
    Mock Get-ItemProperty { $null }   # no Steam RunningAppID
  }

  It 'returns null when nothing looks like a game' {
    Mock Get-Process { New-Proc 'notepad' 'C:\Windows\notepad.exe' }
    Find-RunningGame $NoScan $Defaults | Should -BeNullOrEmpty
  }

  It 'finds a game by default folder pattern' {
    Mock Get-Process { New-Proc 'eldenring' 'D:\SteamLibrary\steamapps\common\ELDEN RING\Game\eldenring.exe' 'ELDEN RING' }
    $g = Find-RunningGame $NoScan $Defaults
    $g.Name     | Should -Be 'ELDEN RING'
    $g.Launcher | Should -Be 'Steam'
  }

  It 'ignores launchers and redistributables inside game folders' {
    Mock Get-Process {
      New-Proc 'EpicGamesLauncher' 'C:\Program Files\Epic Games\Launcher\EpicGamesLauncher.exe'
      New-Proc 'vc_redist.x64' 'D:\SteamLibrary\steamapps\common\Foo\_CommonRedist\vc_redist.x64.exe'
      New-Proc 'setup' 'D:\SteamLibrary\steamapps\common\Steamworks Shared\setup.exe'
    }
    Find-RunningGame $NoScan $Defaults | Should -BeNullOrEmpty
  }

  It 'matches a discovered library folder' {
    $scan = @{ Libs = [System.Collections.ArrayList]@(@{ Dir = 'F:\MyGames\'; Launcher = 'GOG'; Name = 'Witcher 3' }); Exes = @{} }
    Mock Get-Process { New-Proc 'witcher3' 'F:\MyGames\bin\witcher3.exe' }
    $g = Find-RunningGame $scan $Defaults
    $g.Name | Should -Be 'Witcher 3'; $g.Launcher | Should -Be 'GOG'
  }

  It "uses Windows' game list only when enabled" {
    $scan = @{ Libs = (New-Lib); Exes = @{ 'x:\odd\place\game.exe' = $true } }
    Mock Get-Process { New-Proc 'game' 'X:\Odd\Place\game.exe' }
    (Find-RunningGame $scan $Defaults).Launcher | Should -Be 'Windows game list'
    Find-RunningGame $scan @{ ExtraGames = @(); DetectEmulators = $true; UseWindowsGameList = $false } | Should -BeNullOrEmpty
  }

  It 'counts emulators only when enabled' {
    Mock Get-Process { New-Proc 'retroarch' 'C:\RetroArch\retroarch.exe' }
    (Find-RunningGame $NoScan $Defaults).Launcher | Should -Be 'emulator'
    Find-RunningGame $NoScan @{ ExtraGames = @(); DetectEmulators = $false; UseWindowsGameList = $true } | Should -BeNullOrEmpty
  }

  It 'honours ExtraGames from settings' {
    Mock Get-Process { New-Proc 'mygame' 'C:\Stuff\mygame.exe' }
    (Find-RunningGame $NoScan @{ ExtraGames = @('mygame'); DetectEmulators = $false; UseWindowsGameList = $false }).Launcher | Should -Be 'your list'
  }

  It 'reports the running Steam app' {
    Mock Get-ItemProperty { [pscustomobject]@{ RunningAppID = 1245620 } }
    Mock Get-SteamAppName { 'ELDEN RING' }
    $g = Find-RunningGame $NoScan $Defaults
    $g.Name | Should -Be 'ELDEN RING'; $g.Launcher | Should -Be 'Steam'
  }

  It 'ignores Steam apps that are not games (Wallpaper Engine)' {
    Mock Get-ItemProperty { [pscustomobject]@{ RunningAppID = 431960 } }
    Mock Get-Process { }
    Find-RunningGame $NoScan $Defaults | Should -BeNullOrEmpty
  }
}

Describe 'Get-LaunchLabel' {
  It 'labels a GPU-detected process by its library' {
    $l = Get-LaunchLabel (New-Proc 'game' 'D:\XboxGames\Forza\Content\forza.exe' 'Forza') $NoScan
    $l.Launcher | Should -Be 'Xbox / Game Pass'
  }
  It "falls back to 'GPU'" {
    (Get-LaunchLabel (New-Proc 'blender' 'C:\Blender\blender.exe') $NoScan).Launcher | Should -Be 'GPU'
  }
}
