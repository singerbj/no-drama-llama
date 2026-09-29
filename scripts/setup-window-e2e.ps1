# CI: opens an elevated WebView2 window (the setup wizard, or with -Settings the settings window)
# and checks that WebView2 starts with a profile folder it can write ("Microsoft Edge can't read
# and write to its data directory" otherwise).
#
# WebView2 won't run elevated: it relaunches itself through Explorer, so it runs as whoever
# Explorer runs as. On a real PC that's the signed-in user's unelevated (UAC-filtered) token. A
# runner signs in as the built-in Administrator, whose Explorer is elevated and can write
# anywhere, which hides the problem. So Explorer is first restarted as an ordinary admin account,
# `ndl-user`, with the filtered token UAC gives it, as on a real PC.
#
#   -As SameUser    the window is elevated by UAC as ndl-user, like a normal install.
#   -As OtherAdmin  the window is elevated as another admin account than Explorer's, as Windows
#                   11's Administrator Protection does with its hidden admin account.
#   -Settings       opens the settings window (with its stdin open, as the tray app starts it).
param(
    [Parameter(Mandatory)] [string] $Exe,
    [ValidateSet('SameUser', 'OtherAdmin')] [string] $As = 'SameUser',
    [switch] $Settings,
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
    [DllImport("user32.dll")] static extern IntPtr GetShellWindow();
    [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
    [DllImport("advapi32.dll")] static extern bool OpenProcessToken(IntPtr p, uint access, out IntPtr t);
    [DllImport("advapi32.dll")] static extern bool GetTokenInformation(IntPtr t, int cls, out int info, int len, out int ret);
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
        }, IntPtr.Zero);
        return all;
    }
    // The desktop shell's process, 0 if there's none.
    public static uint ShellPid() {
        var h = GetShellWindow();
        if (h == IntPtr.Zero) return 0;
        uint pid; GetWindowThreadProcessId(h, out pid);
        return pid;
    }
    // 1 elevated, 0 not, -1 unknown (another account's token may not open).
    public static int Elevated(uint pid) {
        var p = OpenProcess(0x1000 /* QUERY_LIMITED_INFORMATION */, false, pid);
        if (p == IntPtr.Zero) return -1;
        try {
            IntPtr t;
            if (!OpenProcessToken(p, 0x8 /* TOKEN_QUERY */, out t)) return -1;
            try {
                int e, n;
                if (!GetTokenInformation(t, 20 /* TokenElevation */, out e, 4, out n)) return -1;
                return e != 0 ? 1 : 0;
            } finally { CloseHandle(t); }
        } finally { CloseHandle(p); }
    }
}
'@

function Titles { [TopWindows]::List() | ForEach-Object { $p, $t = $_ -split "`t", 2; [pscustomobject]@{ Pid = [int]$p; Title = $t } } }

function Owner($processId) {
    $p = Get-CimInstance Win32_Process -Filter "ProcessId = $processId" -ErrorAction SilentlyContinue
    if ($p) { $o = Invoke-CimMethod -InputObject $p -MethodName GetOwner; if ($o.User) { "$($o.Domain)\$($o.User)" } }
}

function Sid($account) { (New-Object Security.Principal.NTAccount $account).Translate([Security.Principal.SecurityIdentifier]).Value }

function Stop-Window {
    Get-Process no-drama-llama, msedgewebview2 -ErrorAction SilentlyContinue | Stop-Process -Force
    Get-Process pwsh -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $PID -and (Owner $_.Id) -like "*\$shellName" } |
        Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep 2
}

$work = 'C:\ndl-e2e'
New-Item -ItemType Directory $work -Force | Out-Null
icacls $work /grant "*S-1-5-32-545:(OI)(CI)M" | Out-Null
$copy = Join-Path $work 'no-drama-llama.exe'
Get-Process no-drama-llama -ErrorAction SilentlyContinue | Stop-Process -Force
Copy-Item $Exe $copy -Force

# The account Explorer will run as: an ordinary admin, not the built-in Administrator.
$shellName = 'ndl-user'
$password = ConvertTo-SecureString ("Ndl-" + [guid]::NewGuid().ToString('N') + "!a1") -AsPlainText -Force
if (-not (Get-LocalUser $shellName -ErrorAction SilentlyContinue)) {
    New-LocalUser $shellName -Password $password -PasswordNeverExpires | Out-Null
    Add-LocalGroupMember Administrators $shellName
} else {
    Set-LocalUser $shellName -Password $password
}
$cred = New-Object Management.Automation.PSCredential ".\$shellName", $password
$shellUser = "$env:COMPUTERNAME\$shellName"
$shellSid = Sid $shellUser
# UAC as on a PC, except that elevating needs no click.
$policies = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System'
Set-ItemProperty $policies ConsentPromptBehaviorAdmin 0
Set-ItemProperty $policies PromptOnSecureDesktop 0

$sessionUser = (Get-CimInstance Win32_ComputerSystem).UserName
if (-not $sessionUser) { $sessionUser = "$env:USERDOMAIN\$env:USERNAME" }
$sessionSid = Sid $sessionUser
"Session $((Get-Process -Id $PID).SessionId), signed-in user $sessionUser ($sessionSid), this process $(whoami)"
Get-ItemProperty $policies | Select-Object EnableLUA, ConsentPromptBehaviorAdmin, PromptOnSecureDesktop | Format-List

Stop-Window

$shell = [TopWindows]::ShellPid()
if (-not $shell -or (Owner $shell) -ne $shellUser) {
    # Winlogon would restart the old Explorer at once.
    Set-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon' AutoRestartShell 0 -Type DWord
    Get-Process explorer -ErrorAction SilentlyContinue | Stop-Process -Force
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Process explorer -ErrorAction SilentlyContinue) -and (Get-Date) -lt $deadline) { Start-Sleep 1 }
    Start-Process "$env:WINDIR\explorer.exe" -Credential $cred -LoadUserProfile -WorkingDirectory $work
    $deadline = (Get-Date).AddSeconds(120)
    while ((Get-Date) -lt $deadline) {
        $shell = [TopWindows]::ShellPid()
        if ($shell -and (Owner $shell) -eq $shellUser) { break }
        Start-Sleep 2
    }
}
"Shell: $shell $((Get-Process -Id $shell -ErrorAction SilentlyContinue).ProcessName) as $(Owner $shell), elevated: $([TopWindows]::Elevated($shell))"
if ((Owner $shell) -ne $shellUser) { throw "Explorer didn't start as $shellUser" }
# What a program started as that account gets: UAC's filtered token, like Explorer's.
Remove-Item "$work\groups.txt" -ErrorAction SilentlyContinue
Start-Process pwsh -Credential $cred -WorkingDirectory $work -WindowStyle Hidden -Wait `
    -ArgumentList '-NoProfile', '-Command', "whoami /groups /fo csv | Set-Content $work\groups.txt"
$groups = Get-Content "$work\groups.txt" -Raw | ConvertFrom-Csv
$label = ($groups | Where-Object 'Group Name' -like 'Mandatory Label\*').'Group Name'
$admins = $groups | Where-Object SID -eq 'S-1-5-32-544'
"$shellUser unelevated: $label; Administrators: $($admins.Attributes)"
if ($label -notlike '*Medium*' -or $admins.Attributes -notlike '*deny only*') { throw "$shellUser doesn't get a UAC-filtered token" }

if ($Settings) {
    $expect, $title, $arg = 'C:\LLM\data\webview2', '^No Drama Llama$', 'settings-window'
    New-Item -ItemType Directory C:\LLM\data -Force | Out-Null
} else {
    $expect, $title, $arg = 'C:\Program Files\No Drama Llama\setup-webview2', '^No Drama Llama Setup$', 'setup'
}
# Where 0.0.3 to 0.0.5 put the profile under Administrator Protection: no longer used.
$oldUserDirs = @($sessionSid, $shellSid) | ForEach-Object {
    $p = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\$_" -ErrorAction SilentlyContinue).ProfileImagePath
    if ($p) { Join-Path $p 'AppData\Local\No Drama Llama\WebView2' }
}
Remove-Item (@($expect) + $oldUserDirs) -Recurse -Force -ErrorAction SilentlyContinue

# Starts the window with its stdin open (the settings window closes when it's closed).
Set-Content "$work\run.ps1" @'
param($Exe, $Arg)
$psi = [Diagnostics.ProcessStartInfo]::new($Exe, $Arg)
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$p = [Diagnostics.Process]::Start($psi)
$p.WaitForExit(180000) | Out-Null
'@
if ($As -eq 'SameUser') {
    # As ndl-user unelevated, then elevated through UAC ("Run as administrator").
    Set-Content "$work\elevate.ps1" "Start-Process pwsh -Verb RunAs -WindowStyle Hidden -ArgumentList '-NoProfile', '-File', '$work\run.ps1', '$copy', '$arg'"
    Start-Process pwsh -Credential $cred -WorkingDirectory $work -WindowStyle Hidden -ArgumentList '-NoProfile', '-File', "$work\elevate.ps1"
    $appUser = $shellUser
} else {
    # As this (elevated) runner account: not Explorer's.
    Start-Process pwsh -WindowStyle Hidden -ArgumentList '-NoProfile', '-File', "$work\run.ps1", $copy, $arg
    $appUser = $sessionUser
}
"Expecting the window as $appUser (elevated), WebView2 as $shellUser (unelevated), its profile in $expect"

$dialog = 'couldn.t create the data directory|can.t read and write'
$deadline = (Get-Date).AddSeconds(90)
$errorDialog = $null
while ((Get-Date) -lt $deadline) {
    $errorDialog = Titles | Where-Object Title -Match $dialog
    if ($errorDialog -or (Test-Path (Join-Path $expect 'EBWebView\Local State'))) { break }
    Start-Sleep 2
}
Start-Sleep 5 # let the page render (or an error dialog appear) for the screenshot
if (-not $errorDialog) { $errorDialog = Titles | Where-Object Title -Match $dialog }

"Visible windows:"
Titles | ForEach-Object { "  [$($_.Pid) $(Owner $_.Pid)] $($_.Title)" }
$apps = @(Get-Process no-drama-llama -ErrorAction SilentlyContinue)
$webviews = @(Get-Process msedgewebview2 -ErrorAction SilentlyContinue)
"Processes:"
@($apps) + @($webviews | Select-Object -First 3) | ForEach-Object {
    "  $($_.Id) $($_.ProcessName) as $(Owner $_.Id), elevated: $([TopWindows]::Elevated($_.Id))"
}
try {
    $b = [Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object Drawing.Bitmap $b.Width, $b.Height
    [Drawing.Graphics]::FromImage($bmp).CopyFromScreen($b.Location, [Drawing.Point]::Empty, $b.Size)
    $bmp.Save($Screenshot)
} catch { "No screenshot: $_" }

$failures = @()
if ($errorDialog) { $failures += "WebView2 showed: $($errorDialog.Title)" }
if (-not (Test-Path (Join-Path $expect 'EBWebView\Local State'))) { $failures += "no WebView2 profile in $expect" }
foreach ($d in $oldUserDirs) { if (Test-Path $d) { $failures += "the window used $d" } }
if (-not (Titles | Where-Object Title -Match $title)) { $failures += "no window titled $title" }
# That the case under test is the real one: the window elevated as $appUser, WebView2 not.
if (-not $apps) { $failures += 'the app is not running' }
foreach ($a in $apps | Select-Object -First 1) {
    if ((Owner $a.Id) -ne $appUser) { $failures += "the app runs as $(Owner $a.Id), not $appUser" }
    if ([TopWindows]::Elevated($a.Id) -eq 0) { $failures += 'the app is not elevated' }
}
if (-not $webviews) { $failures += 'no WebView2 processes' }
foreach ($w in $webviews | Select-Object -First 1) {
    if ((Owner $w.Id) -ne $shellUser) { $failures += "WebView2 runs as $(Owner $w.Id), not $shellUser" }
    if ([TopWindows]::Elevated($w.Id) -eq 1) { $failures += 'WebView2 runs elevated' }
}
if (Test-Path $expect) {
    $acl = Get-Acl $expect
    "Permissions of ${expect}:"
    $acl.Access | ForEach-Object { "  $($_.AccessControlType) $($_.IdentityReference) $($_.FileSystemRights)" }
    $sidOf = { param($ace) try { $ace.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value } catch { '?' } }
    $modify = [Security.AccessControl.FileSystemRights]::Modify
    $rights = $acl.Access | Where-Object { $_.AccessControlType -eq 'Allow' -and (& $sidOf $_) -eq $shellSid } | ForEach-Object FileSystemRights
    if (-not ($rights | Where-Object { ($_ -band $modify) -eq $modify })) { $failures += "$shellUser can't write $expect" }
    # Nobody else unelevated may: the window is elevated. (The runner's session user is on a
    # real PC the same account as Explorer's; here the app may also name it.)
    $allowed = @('S-1-5-18', 'S-1-5-32-544', $shellSid, $sessionSid)
    $others = $acl.Access | Where-Object {
        $_.AccessControlType -eq 'Allow' -and ($_.FileSystemRights -band [Security.AccessControl.FileSystemRights]::WriteData) -and
        (& $sidOf $_) -notin $allowed
    }
    if ($others) { $failures += "others can write ${expect}: $(($others | ForEach-Object IdentityReference) -join ', ')" }
}

Stop-Window
if ($failures) { throw ($failures -join "`n") }
"OK: the window opened, elevated as $appUser, with WebView2 unelevated as $shellUser and its profile in $expect"
