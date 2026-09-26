BeforeAll {
  . (Join-Path $PSScriptRoot '..\src\config.ps1')
  $SettingsFile = Join-Path $TestDrive 'settings.json'
  function Write-SettingsJson([hashtable]$h) { $h | ConvertTo-Json | Set-Content $SettingsFile }
}

Describe 'Get-Settings' {
  BeforeEach { Remove-Item $SettingsFile -ErrorAction SilentlyContinue }

  It 'returns the defaults when there is no settings file' {
    $s = Get-Settings
    foreach ($k in $DefaultSettings.Keys) { "$($s[$k])" | Should -Be "$($DefaultSettings[$k])" }
    $SettingsWarnings | Should -BeNullOrEmpty
  }

  It 'applies valid values' {
    Write-SettingsJson @{ Model = 'My Model-Q4_K_M.gguf'; Reasoning = 'none'; Context = 65536; Port = 9000
                          ListenHost = '0.0.0.0'; Popups = $false; GpuVramGB = 2.5; ExtraGames = @('foo', 'bar') }
    $s = Get-Settings
    $s.Model      | Should -Be 'My Model-Q4_K_M.gguf'
    $s.Reasoning  | Should -Be 'none'
    $s.Context    | Should -Be 65536
    $s.Port       | Should -Be 9000
    $s.ListenHost | Should -Be '0.0.0.0'
    $s.Popups     | Should -BeFalse
    $s.GpuVramGB  | Should -Be 2.5
    $s.ExtraGames | Should -Be @('foo', 'bar')
    $SettingsWarnings | Should -BeNullOrEmpty
  }

  It 'rejects <Key> = <Value> (falls back to default)' -ForEach @(
    @{ Key = 'Model';      Value = '..\..\Windows\evil.gguf' }
    @{ Key = 'Model';      Value = 'x.gguf" --log-file C:\Windows\x "' }
    @{ Key = 'Model';      Value = 'notamodel.exe' }
    @{ Key = 'Reasoning';  Value = 'extreme' }
    @{ Key = 'Context';    Value = 'lots' }
    @{ Key = 'Context';    Value = 100 }
    @{ Key = 'ListenHost'; Value = '0.0.0.0 --api-key x' }
    @{ Key = 'Port';       Value = 80 }
    @{ Key = 'Popups';     Value = 'yes' }
    @{ Key = 'ApiKey';     Value = 'abc def' }
    @{ Key = 'GpuLoadPct'; Value = 150 }
  ) {
    Write-SettingsJson @{ $Key = $Value }
    $s = Get-Settings
    "$($s[$Key])" | Should -Be "$($DefaultSettings[$Key])"
    $SettingsWarnings | Should -HaveCount 1
  }

  It 'ignores unknown keys' {
    Write-SettingsJson @{ Bogus = 1 }
    (Get-Settings).Contains('Bogus') | Should -BeFalse
  }

  It 'wraps a single ExtraGames string into a list' {
    Set-Content $SettingsFile '{ "ExtraGames": "mygame" }'
    @((Get-Settings).ExtraGames) | Should -Be @('mygame')
  }

  It 'survives a corrupt file and reports it' {
    Set-Content $SettingsFile '{ not json'
    (Get-Settings).Model | Should -Be $DefaultModel
    $SettingsWarnings | Should -HaveCount 1
  }

  It 'round-trips through Save-Settings' {
    $s = Get-Settings
    $s.Reasoning = 'medium'; $s.GpuIgnore = @('MyApp')
    Save-Settings $s
    $r = Get-Settings
    $r.Reasoning | Should -Be 'medium'
    @($r.GpuIgnore) | Should -Be @('MyApp')
  }
}

Describe 'Layout' {
  It 'keeps user-writable files out of the admin-only app folder' {
    foreach ($p in $OffFlag, $LogFile, $ServerLog, $SettingsFile) { $p | Should -Not -BeLike "$AppDir*" }
    foreach ($p in $OffFlag, $LogFile, $ServerLog) { $p | Should -BeLike "$DataDir*" }
  }
}
