# Static checks over every script in src/.
BeforeDiscovery {
  $script:Files = Get-ChildItem (Join-Path $PSScriptRoot '..\src') -Filter *.ps1 |
    ForEach-Object { @{ Name = $_.Name; Path = $_.FullName } }
}

Describe '<Name>' -ForEach $Files {
  It 'parses without errors' {
    $errors = $null
    [void][System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$null, [ref]$errors)
    $errors | Should -BeNullOrEmpty
  }

  It 'is plain ASCII (Windows PowerShell 5.1 misreads BOM-less UTF-8)' {
    $bytes = [IO.File]::ReadAllBytes($Path)
    @($bytes | Where-Object { $_ -gt 127 }).Count | Should -Be 0
  }
}

Describe 'install.ps1' {
  BeforeAll {
    $src     = Join-Path $PSScriptRoot '..\src'
    $install = Get-Content (Join-Path $src 'install.ps1') -Raw
    $m = [regex]::Match($install, '\$AppFiles\s*=\s*([^\r\n]+)')
    $script:AppFiles = [regex]::Matches($m.Groups[1].Value, "'([^']+)'") | ForEach-Object { $_.Groups[1].Value }
  }

  It 'copies every script the installed app dot-sources' {
    foreach ($f in $AppFiles) {
      $text = Get-Content (Join-Path $src $f) -Raw
      foreach ($dep in [regex]::Matches($text, "\. \(Join-Path \`$PSScriptRoot '([^']+)'\)")) {
        $AppFiles | Should -Contain $dep.Groups[1].Value -Because "$f dot-sources it"
      }
    }
  }

  It 'lists only files that exist' {
    foreach ($f in $AppFiles) { Join-Path $src $f | Should -Exist }
  }

  It 'requires elevation' {
    $install | Should -Match '#Requires -RunAsAdministrator'
  }
}

Describe 'Versioning' {
  It 'CHANGELOG.md has an entry for $AppVersion' {
    . (Join-Path $PSScriptRoot '..\src\config.ps1')
    $changelog = Get-Content (Join-Path $PSScriptRoot '..\CHANGELOG.md') -Raw
    $changelog | Should -Match ("## \[" + [regex]::Escape($AppVersion) + "\]")
  }
}
