# CI: opens an elevated WebView2 window (the setup wizard, or with -Settings the settings window)
# and checks that WebView2 starts with a profile folder it can write ("Microsoft Edge can't read
# and write to its data directory" otherwise).
#
# WebView2 won't run elevated: it relaunches itself through Explorer, so it runs as whoever
# Explorer runs as. On a real PC that's the signed-in user's unelevated token. A runner signs in
# as the built-in Administrator, whose Explorer is elevated and can write anywhere, which hides
# the problem, so Explorer is first restarted with an unelevated token.
#
#   -AsOtherAdmin  runs the wizard elevated as a second admin account in this desktop session,
#                  as Windows 11's Administrator Protection does with its hidden admin account.
#   -Settings      opens the settings window (elevated, as the tray app does) instead.
param(
    [Parameter(Mandatory)] [string] $Exe,
    [switch] $AsOtherAdmin,
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
    // 1 elevated, 0 not, -1 unknown.
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
    if ($p) { $o = Invoke-CimMethod -InputObject $p -MethodName GetOwner; "$($o.Domain)\$($o.User)" }
}

function Stop-Window {
    Get-Process no-drama-llama, msedgewebview2 -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep 2
}

# Explorer as on a real PC: the signed-in user, unelevated (a Basic User token: no admin rights,
# medium integrity, like UAC's filtered token).
function Use-UnelevatedShell {
    $shell = [TopWindows]::ShellPid()
    if ($shell -and [TopWindows]::Elevated($shell) -eq 0) { return }
    # Winlogon would restart it elevated at once.
    Set-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon' AutoRestartShell 0 -Type DWord
    Get-Process explorer -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep 3
    Start-Process runas.exe -ArgumentList '/trustlevel:0x20000', "$env:WINDIR\explorer.exe" -WindowStyle Hidden
    $deadline = (Get-Date).AddSeconds(60)
    while (-not ($shell = [TopWindows]::ShellPid()) -and (Get-Date) -lt $deadline) { Start-Sleep 1 }
    if (-not $shell) { throw 'Explorer did not come back as the shell' }
    $elevated = [TopWindows]::Elevated($shell)
    "Shell: $shell $((Get-Process -Id $shell).ProcessName) as $(Owner $shell), elevated: $elevated"
    if ($elevated -ne 0) { throw "the shell is still elevated ($elevated)" }
}

$sessionUser = (Get-CimInstance Win32_ComputerSystem).UserName
if (-not $sessionUser) { $sessionUser = "$env:USERDOMAIN\$env:USERNAME" }
$sessionSid = (New-Object Security.Principal.NTAccount $sessionUser).Translate([Security.Principal.SecurityIdentifier]).Value
$profileDir = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\$sessionSid").ProfileImagePath
# Where 0.0.3 to 0.0.5 put the profile under Administrator Protection: no longer used.
$oldUserDir = Join-Path $profileDir 'AppData\Local\No Drama Llama\WebView2'
"Session $((Get-Process -Id $PID).SessionId), signed-in user $sessionUser ($sessionSid), this process $(whoami)"
Get-ItemProperty HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System |
    Select-Object EnableLUA, ConsentPromptBehaviorAdmin, PromptOnSecureDesktop | Format-List

Stop-Window
Use-UnelevatedShell

if ($Settings) {
    $expect, $title = 'C:\LLM\data\webview2', '^No Drama Llama$'
    Remove-Item $expect -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory C:\LLM\data -Force | Out-Null
    # As the tray app starts it: elevated (this shell is), with its stdin open until it's closed.
    $psi = [Diagnostics.ProcessStartInfo]::new($Exe, 'settings-window')
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $settingsProcess = [Diagnostics.Process]::Start($psi)
} else {
    $expect, $title = 'C:\Program Files\No Drama Llama\setup-webview2', '^No Drama Llama Setup$'
    Remove-Item $expect, $oldUserDir -Recurse -Force -ErrorAction SilentlyContinue
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
    } else {
        Start-Process $Exe -ArgumentList 'setup'
    }
}
"Expecting the WebView2 profile in $expect"

$deadline = (Get-Date).AddSeconds(90)
$errorDialog = $null
while ((Get-Date) -lt $deadline) {
    $errorDialog = Titles | Where-Object Title -Match 'couldn.t create the data directory|can.t read and write'
    if ($errorDialog -or (Test-Path (Join-Path $expect 'EBWebView\Local State'))) { break }
    Start-Sleep 2
}
Start-Sleep 5 # let the page render (or an error dialog appear) for the screenshot
if (-not $errorDialog) { $errorDialog = Titles | Where-Object Title -Match 'couldn.t create the data directory|can.t read and write' }

"Visible windows:"
Titles | ForEach-Object { "  [$($_.Pid) $(Owner $_.Pid)] $($_.Title)" }
$webviews = @(Get-Process msedgewebview2 -ErrorAction SilentlyContinue)
"Processes:"
Get-Process no-drama-llama -ErrorAction SilentlyContinue | ForEach-Object { "  $($_.Id) $($_.ProcessName) as $(Owner $_.Id), elevated: $([TopWindows]::Elevated($_.Id))" }
$webviews | Select-Object -First 4 | ForEach-Object { "  $($_.Id) $($_.ProcessName) as $(Owner $_.Id), elevated: $([TopWindows]::Elevated($_.Id))" }
try {
    $b = [Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object Drawing.Bitmap $b.Width, $b.Height
    [Drawing.Graphics]::FromImage($bmp).CopyFromScreen($b.Location, [Drawing.Point]::Empty, $b.Size)
    $bmp.Save($Screenshot)
} catch { "No screenshot: $_" }

$failures = @()
if ($errorDialog) { $failures += "WebView2 showed: $($errorDialog.Title)" }
if (-not (Test-Path (Join-Path $expect 'EBWebView\Local State'))) { $failures += "no WebView2 profile in $expect" }
if (-not $Settings -and (Test-Path $oldUserDir)) { $failures += "the wizard used $oldUserDir" }
if (-not (Titles | Where-Object Title -Match $title)) { $failures += "no window titled $title" }
# The case this test is about: WebView2 running unelevated, as the signed-in user.
if (-not $webviews) { $failures += 'no WebView2 processes' }
foreach ($w in $webviews | Select-Object -First 1) {
    if ([TopWindows]::Elevated($w.Id) -ne 0) { $failures += "WebView2 ($($w.Id)) runs elevated: not the case under test" }
    if ((Owner $w.Id) -ne $sessionUser) { $failures += "WebView2 ($($w.Id)) runs as $(Owner $w.Id), not $sessionUser" }
}
if (Test-Path $expect) {
    $acl = Get-Acl $expect
    "Permissions of ${expect}:"
    $acl.Access | ForEach-Object { "  $($_.AccessControlType) $($_.IdentityReference) $($_.FileSystemRights)" }
    $rights = $acl.Access | Where-Object {
        $_.AccessControlType -eq 'Allow' -and
        $(try { $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value } catch { '' }) -eq $sessionSid
    } | ForEach-Object { $_.FileSystemRights }
    $modify = [Security.AccessControl.FileSystemRights]::Modify
    if (-not ($rights | Where-Object { ($_ -band $modify) -eq $modify })) {
        $failures += "$sessionUser can't write $expect"
    }
    # Nobody else unelevated may: the window is elevated.
    $others = $acl.Access | Where-Object {
        $_.AccessControlType -eq 'Allow' -and ($_.FileSystemRights -band [Security.AccessControl.FileSystemRights]::WriteData) -and
        $(try { $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value } catch { '?' }) -notin @('S-1-5-18', 'S-1-5-32-544', $sessionSid)
    }
    if ($others) { $failures += "others can write ${expect}: $(($others | ForEach-Object IdentityReference) -join ', ')" }
}

if ($settingsProcess) { $settingsProcess.StandardInput.Close() }
Stop-Window
if ($failures) { throw ($failures -join "`n") }
"OK: the window opened with its WebView2 profile in $expect, WebView2 running unelevated as $sessionUser"
