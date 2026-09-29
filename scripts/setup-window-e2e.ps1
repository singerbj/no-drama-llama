# CI: opens the setup wizard and checks that WebView2 starts with the profile folder it can write
# ("Microsoft Edge can't read and write to its data directory" otherwise).
#
#   -AsOtherAdmin  runs the wizard elevated as a second admin account in this desktop session,
#                  as Windows 11's Administrator Protection does with its hidden admin account.
#                  WebView2 then runs as the signed-in user, so the profile must be one they
#                  can write: their own %LOCALAPPDATA%, not the admin-only Program Files.
param(
    [Parameter(Mandatory)] [string] $Exe,
    [switch] $AsOtherAdmin,
    [string] $Screenshot = 'setup-window.png'
)
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class TopWindows {
    delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    public static List<string> List() {
        var all = new List<string>();
        EnumWindows((h, l) => {
            if (!IsWindowVisible(h)) return true;
            var s = new StringBuilder(512);
            GetWindowText(h, s, s.Capacity);
            if (s.Length == 0) return true;
            uint pid; GetWindowThreadProcessId(h, out pid);
            all.Add(pid + "\t" + s);
            return true;
        });
        return all;
    }
}
'@

function Titles { [TopWindows]::List() | ForEach-Object { $p, $t = $_ -split "`t", 2; [pscustomobject]@{ Pid = [int]$p; Title = $t } } }

function Owner($processId) {
    $p = Get-CimInstance Win32_Process -Filter "ProcessId = $processId" -ErrorAction SilentlyContinue
    if ($p) { $o = Invoke-CimMethod -InputObject $p -MethodName GetOwner; "$($o.Domain)\$($o.User)" }
}

function Stop-Wizard {
    Get-Process no-drama-llama, msedgewebview2 -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep 2
}

$sessionUser = (Get-CimInstance Win32_ComputerSystem).UserName
if (-not $sessionUser) { $sessionUser = "$env:USERDOMAIN\$env:USERNAME" }
$sessionSid = (New-Object Security.Principal.NTAccount $sessionUser).Translate([Security.Principal.SecurityIdentifier]).Value
$profileDir = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\$sessionSid").ProfileImagePath
$adminOnly = 'C:\Program Files\No Drama Llama\setup-webview2'
$userDir = Join-Path $profileDir 'AppData\Local\No Drama Llama\WebView2\setup'
"Session $((Get-Process -Id $PID).SessionId), signed-in user $sessionUser ($sessionSid), this process $(whoami)"
Get-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System |
    Select-Object EnableLUA, ConsentPromptBehaviorAdmin, PromptOnSecureDesktop | Format-List

Stop-Wizard
Remove-Item $adminOnly, $userDir -Recurse -Force -ErrorAction SilentlyContinue

if ($AsOtherAdmin) {
    # Should the second account get a filtered token and ask for elevation, don't prompt.
    Set-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System ConsentPromptBehaviorAdmin 0
    Set-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System PromptOnSecureDesktop 0
    $name = 'ndl-elevated'
    $password = ConvertTo-SecureString ("Ndl-" + [guid]::NewGuid().ToString('N') + "!a1") -AsPlainText -Force
    if (-not (Get-LocalUser $name -ErrorAction SilentlyContinue)) {
        New-LocalUser $name -Password $password -PasswordNeverExpires | Out-Null
        Add-LocalGroupMember Administrators $name
    } else {
        Set-LocalUser $name -Password $password
    }
    $cred = New-Object Management.Automation.PSCredential ".\$name", $password
    # The copy must be readable by that account (the workspace is under the runner's profile).
    $copy = 'C:\ndl-e2e\no-drama-llama.exe'
    New-Item -ItemType Directory C:\ndl-e2e -Force | Out-Null
    Copy-Item $Exe $copy -Force
    icacls C:\ndl-e2e /grant "*S-1-5-32-545:(OI)(CI)RX" | Out-Null
    Start-Process $copy -ArgumentList 'setup' -Credential $cred -LoadUserProfile -WorkingDirectory C:\ndl-e2e
    $expect, $forbid = $userDir, $adminOnly
} else {
    Start-Process $Exe -ArgumentList 'setup'
    $expect, $forbid = $adminOnly, $userDir
}
"Expecting the WebView2 profile in $expect"

$deadline = (Get-Date).AddSeconds(90)
$errorDialog = $null
while ((Get-Date) -lt $deadline) {
    $errorDialog = Titles | Where-Object Title -Match "couldn't create the data directory|can't read and write"
    if ($errorDialog -or (Test-Path (Join-Path $expect 'EBWebView\Local State'))) { break }
    Start-Sleep 2
}
Start-Sleep 5 # let the page render for the screenshot

"Visible windows:"
Titles | ForEach-Object { "  [$($_.Pid) $(Owner $_.Pid)] $($_.Title)" }
"Processes:"
Get-Process no-drama-llama, msedgewebview2 -ErrorAction SilentlyContinue |
    Select-Object -First 4 | ForEach-Object { "  $($_.Id) $($_.ProcessName) as $(Owner $_.Id)" }
try {
    $b = [Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object Drawing.Bitmap $b.Width, $b.Height
    [Drawing.Graphics]::FromImage($bmp).CopyFromScreen($b.Location, [Drawing.Point]::Empty, $b.Size)
    $bmp.Save($Screenshot)
} catch { "No screenshot: $_" }

$failures = @()
if ($errorDialog) { $failures += "WebView2 showed: $($errorDialog.Title)" }
if (-not (Test-Path (Join-Path $expect 'EBWebView'))) { $failures += "no WebView2 profile in $expect" }
if (Test-Path $forbid) { $failures += "the wizard used $forbid" }
if (-not (Titles | Where-Object Title -Match 'No Drama Llama Setup')) { $failures += 'no setup window' }
if ($AsOtherAdmin -and (Test-Path $expect)) {
    # What WebView2 needs once it drops elevation: the signed-in user's own write access,
    # which the admin-only folder only gives to (elevated) administrators.
    $rights = (Get-Acl $expect).Access | Where-Object {
        $_.AccessControlType -eq 'Allow' -and
        $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -eq $sessionSid
    } | ForEach-Object { $_.FileSystemRights }
    "Rights of $sessionUser on ${expect}: $($rights -join ', ')"
    $modify = [Security.AccessControl.FileSystemRights]::Modify
    if (-not ($rights | Where-Object { ($_ -band $modify) -eq $modify })) {
        $failures += "$sessionUser can't write $expect"
    }
}

Stop-Wizard
if ($failures) { throw ($failures -join "`n") }
"OK: the setup window opened with its WebView2 profile in $expect"
