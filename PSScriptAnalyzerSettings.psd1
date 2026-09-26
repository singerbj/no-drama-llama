@{
  Severity     = @('Error', 'Warning')
  ExcludeRules = @(
    'PSAvoidUsingWriteHost',                        # installer / uninstaller talk to a human
    'PSUseDeclaredVarsMoreThanAssignments',         # config.ps1 is dot-sourced; its variables are used elsewhere
    'PSUseShouldProcessForStateChangingFunctions',  # internal helpers, not a published module
    'PSUseSingularNouns',
    'PSAvoidUsingEmptyCatchBlock',                  # best-effort probes (registry, counters) are meant to fail quietly
    'PSAvoidGlobalVars'
  )
  Rules = @{
    # The app runs on Windows PowerShell 5.1 (powershell.exe); keep syntax compatible.
    PSUseCompatibleSyntax = @{ Enable = $true; TargetVersions = @('5.1', '7.4') }
  }
}
