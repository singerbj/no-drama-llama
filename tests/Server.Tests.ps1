BeforeAll {
  . (Join-Path $PSScriptRoot '..\src\config.ps1')
  . (Join-Path $PSScriptRoot '..\src\server.ps1')
  function New-TestSettings([hashtable]$over = @{}) {
    $s = [ordered]@{}
    foreach ($kv in $DefaultSettings.GetEnumerator()) { $s[$kv.Key] = $kv.Value }
    foreach ($kv in $over.GetEnumerator()) { $s[$kv.Key] = $kv.Value }
    $s
  }
  function Get-ArgValue($argList, $name) { $argList[[array]::IndexOf($argList, $name) + 1] }
}

Describe 'Get-ChatTemplateKwargs' {
  It 'turns thinking off for none (the template rejects reasoning_effort=none)' {
    Get-ChatTemplateKwargs 'none' | Should -Be '{"enable_thinking":false}'
  }
  It 'passes <_> through as reasoning_effort' -ForEach 'low', 'medium', 'high', 'xhigh' {
    Get-ChatTemplateKwargs $_ | Should -Be ('{"reasoning_effort":"' + $_ + '"}')
  }
}

Describe 'Get-ServerArgs' {
  It 'quotes the model path' {
    $a = Get-ServerArgs (New-TestSettings) 'C:\LLM\models\My Model.gguf'
    Get-ArgValue $a '-m' | Should -Be '"C:\LLM\models\My Model.gguf"'
  }

  It 'passes host, port and context from settings' {
    $a = Get-ServerArgs (New-TestSettings @{ ListenHost = '0.0.0.0'; Port = 9000; Context = 65536 }) 'm.gguf'
    Get-ArgValue $a '--host' | Should -Be '0.0.0.0'
    Get-ArgValue $a '--port' | Should -Be '9000'
    Get-ArgValue $a '-c'     | Should -Be '65536'
  }

  It 'escapes the chat-template JSON for the Windows command line' {
    $a = Get-ServerArgs (New-TestSettings @{ Reasoning = 'low' }) 'm.gguf'
    Get-ArgValue $a '--chat-template-kwargs' | Should -Be '"{\"reasoning_effort\":\"low\"}"'
  }

  It 'uses non-thinking sampling only when reasoning is none' {
    Get-ArgValue (Get-ServerArgs (New-TestSettings @{ Reasoning = 'none' }) 'm.gguf') '--temp' | Should -Be '0.7'
    Get-ArgValue (Get-ServerArgs (New-TestSettings @{ Reasoning = 'low' })  'm.gguf') '--temp' | Should -Be '1.0'
  }

  It 'adds --api-key only when one is set' {
    Get-ServerArgs (New-TestSettings) 'm.gguf' | Should -Not -Contain '--api-key'
    $a = Get-ServerArgs (New-TestSettings @{ ApiKey = 's3cret' }) 'm.gguf'
    Get-ArgValue $a '--api-key' | Should -Be 's3cret'
  }

  It 'returns only strings' {
    Get-ServerArgs (New-TestSettings) 'm.gguf' | ForEach-Object { $_ | Should -BeOfType [string] }
  }
}
