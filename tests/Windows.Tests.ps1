# Compiles the embedded C# (GPU counters, on-screen popup). Needs Windows + .NET Framework,
# so it only runs on Windows PowerShell 5.1 (the runtime the app actually uses).
Describe 'Embedded C#' -Tag 'Windows' -Skip:($PSVersionTable.PSEdition -ne 'Desktop') {
  It 'gpu.ps1 compiles and samples without throwing' {
    . (Join-Path $PSScriptRoot '..\src\games.ps1')
    . (Join-Path $PSScriptRoot '..\src\gpu.ps1')
    { [GpuWatch]::Sample() } | Should -Not -Throw
    { [GpuWatch]::NotificationState() } | Should -Not -Throw
  }
  It 'osd.ps1 compiles' {
    Add-Type -AssemblyName System.Windows.Forms, System.Drawing
    . (Join-Path $PSScriptRoot '..\src\osd.ps1')
    [LlmOsd] | Should -Not -BeNullOrEmpty
  }
}
