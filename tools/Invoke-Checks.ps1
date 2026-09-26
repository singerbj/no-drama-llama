<#
.SYNOPSIS
  Runs the same checks as CI: PSScriptAnalyzer lint + Pester tests.
.EXAMPLE
  pwsh ./tools/Invoke-Checks.ps1            # or: powershell -File .\tools\Invoke-Checks.ps1
#>
[CmdletBinding()]
param([switch]$InstallDependencies)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent

if ($InstallDependencies) {
  if ($PSVersionTable.PSEdition -eq 'Desktop') {   # Windows PowerShell 5.1
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    Install-PackageProvider -Name NuGet -MinimumVersion 2.8.5.201 -Scope CurrentUser -Force | Out-Null
  }
  Set-PSRepository PSGallery -InstallationPolicy Trusted
  Install-Module Pester -MinimumVersion 5.5.0 -MaximumVersion 5.99.99 -Scope CurrentUser -Force -SkipPublisherCheck
  Install-Module PSScriptAnalyzer -MinimumVersion 1.21.0 -Scope CurrentUser -Force
}

Write-Host '== PSScriptAnalyzer' -ForegroundColor Cyan
Import-Module PSScriptAnalyzer
$findings = foreach ($dir in 'src', 'tests', 'tools') {
  Invoke-ScriptAnalyzer -Path (Join-Path $repo $dir) -Recurse -Settings (Join-Path $repo 'PSScriptAnalyzerSettings.psd1')
}
$findings | Format-Table -AutoSize RuleName, Severity, ScriptName, Line, Message | Out-String -Width 200 | Write-Host
if ($findings) { throw "PSScriptAnalyzer: $(@($findings).Count) finding(s)" }
Write-Host 'No findings.'

Write-Host '== Pester' -ForegroundColor Cyan
if (-not (Get-Module Pester)) { Import-Module Pester -MinimumVersion 5.5.0 }
$cfg = New-PesterConfiguration
$cfg.Run.Path           = Join-Path $repo 'tests'
$cfg.Run.Exit           = $false
$cfg.Run.PassThru       = $true
$cfg.Output.Verbosity   = 'Detailed'
$cfg.TestResult.Enabled = $true
$cfg.TestResult.OutputPath = Join-Path $repo 'testResults.xml'
$result = Invoke-Pester -Configuration $cfg
if ($result.FailedCount -gt 0 -or $result.FailedContainersCount -gt 0) { throw "Pester: $($result.FailedCount) test(s) failed" }
