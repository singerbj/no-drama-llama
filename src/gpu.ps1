# Launcher-agnostic game detection: per-process GPU usage.
# Reads the same Windows counters Task Manager's GPU columns use:
#   "GPU Process Memory\Dedicated Usage"  - VRAM each process holds
#   "GPU Engine\Utilization Percentage"   - 3D-engine load per process
# plus Windows' "a Direct3D full-screen app is running" state.
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;

public class GpuProc
{
    public int Pid;
    public double Engine3D;       // % of the 3D engine (summed over engines, capped at 100)
    public long DedicatedBytes;   // VRAM held
}

public static class GpuWatch
{
    static Dictionary<string, CounterSample> prev = new Dictionary<string, CounterSample>();
    static readonly Regex PidRx = new Regex(@"^pid_(\d+)_", RegexOptions.Compiled);

    public static List<GpuProc> Sample()
    {
        Dictionary<int, GpuProc> map = new Dictionary<int, GpuProc>();

        try
        {
            InstanceDataCollectionCollection data = new PerformanceCounterCategory("GPU Engine").ReadCategory();
            InstanceDataCollection util = data["Utilization Percentage"];
            Dictionary<string, CounterSample> next = new Dictionary<string, CounterSample>();
            if (util != null)
            {
                foreach (InstanceData d in util.Values)
                {
                    string name = d.InstanceName;
                    if (!name.EndsWith("engtype_3D", StringComparison.OrdinalIgnoreCase)) continue;
                    Match m = PidRx.Match(name);
                    if (!m.Success) continue;
                    CounterSample s = d.Sample;
                    next[name] = s;
                    CounterSample old;
                    if (prev.TryGetValue(name, out old))
                    {
                        GpuProc p = Get(map, int.Parse(m.Groups[1].Value));
                        p.Engine3D = Math.Min(100.0, p.Engine3D + CounterSample.Calculate(old, s));
                    }
                }
            }
            prev = next;
        }
        catch { }

        try
        {
            InstanceDataCollectionCollection data = new PerformanceCounterCategory("GPU Process Memory").ReadCategory();
            InstanceDataCollection ded = data["Dedicated Usage"];
            if (ded != null)
            {
                foreach (InstanceData d in ded.Values)
                {
                    Match m = PidRx.Match(d.InstanceName);
                    if (!m.Success) continue;
                    Get(map, int.Parse(m.Groups[1].Value)).DedicatedBytes += d.RawValue;
                }
            }
        }
        catch { }

        return new List<GpuProc>(map.Values);
    }

    static GpuProc Get(Dictionary<int, GpuProc> map, int pid)
    {
        GpuProc p;
        if (!map.TryGetValue(pid, out p)) { p = new GpuProc(); p.Pid = pid; map[pid] = p; }
        return p;
    }

    [DllImport("shell32.dll")] static extern int SHQueryUserNotificationState(out int state);
    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);

    // 3 = QUNS_RUNNING_D3D_FULL_SCREEN (exclusive full-screen Direct3D app)
    public static int NotificationState()
    {
        int s;
        try { if (SHQueryUserNotificationState(out s) == 0) return s; } catch { }
        return 0;
    }

    public static int ForegroundPid()
    {
        uint pid;
        GetWindowThreadProcessId(GetForegroundWindow(), out pid);
        return (int)pid;
    }
}
'@

# Apps that use the GPU but aren't games. Your own additions go in settings (GpuIgnore).
$GpuAlwaysIgnore = @(
  'llama-server','dwm','csrss','explorer','ShellExperienceHost','StartMenuExperienceHost','SearchHost','SearchApp',
  'TextInputHost','Taskmgr','Widgets','WidgetService','LockApp','ApplicationFrameHost','SystemSettings','PhoneExperienceHost',
  'msedgewebview2','chrome','msedge','firefox','brave','opera','vivaldi','arc','Claude','ChatGPT',
  'Discord','Spotify','Teams','ms-teams','Zoom','slack','Code','Cursor','obs64','Streamlabs OBS',
  'wallpaper32','wallpaper64','webwallpaper32','Lively','LosslessScaling',
  'RadeonSoftware','AMDRSServ','AMDRSSrcExt','amdow','atieclxx','cncmd','NVIDIA Overlay','nvcontainer','NVIDIA app',
  'vlc','mpc-hc64','mpv','Microsoft.Media.Player','Video.UI','Photos','PowerToys.PowerLauncher','ScreenClippingHost'
)

$script:GpuHits = 0

# Returns @{ Name; Launcher; Process } for a process that looks like a game by GPU use, or $null.
function Find-GpuGame($settings, $scan, [int[]]$excludePids) {
  $procs  = [GpuWatch]::Sample()
  $ignore = @($GpuAlwaysIgnore) + @($NotGameProcesses) + @($settings.GpuIgnore)
  $vram   = [double]$settings.GpuVramGB * 1GB
  $load   = [double]$settings.GpuLoadPct

  $best = $null
  foreach ($g in $procs) {
    if ($g.Pid -le 4 -or $excludePids -contains $g.Pid) { continue }
    if ($g.DedicatedBytes -lt $vram -and $g.Engine3D -lt $load) { continue }
    $p = Get-Process -Id $g.Pid -ErrorAction SilentlyContinue
    if (-not $p -or $ignore -contains $p.ProcessName) { continue }
    if (-not $best -or $g.DedicatedBytes -gt $best.G.DedicatedBytes) { $best = @{ P = $p; G = $g } }
  }

  if (-not $best) {
    # Exclusive full-screen Direct3D app in the foreground: pause right away
    if ([GpuWatch]::NotificationState() -eq 3) {
      $fp = Get-Process -Id ([GpuWatch]::ForegroundPid()) -ErrorAction SilentlyContinue
      if ($fp -and $ignore -notcontains $fp.ProcessName) {
        $label = Get-LaunchLabel $fp $scan
        return @{ Name = $label.Name; Launcher = "$($label.Launcher) - full-screen"; Process = $fp.ProcessName }
      }
    }
    $script:GpuHits = 0
    return $null
  }

  # Must be seen on 2 checks in a row (~5-10 s) so short GPU spikes don't pause the model
  $script:GpuHits++
  if ($script:GpuHits -lt 2) { return $null }

  $label = Get-LaunchLabel $best.P $scan
  $gb = '{0:N1}' -f ($best.G.DedicatedBytes / 1GB)
  @{ Name = $label.Name; Launcher = "$($label.Launcher) - $gb GB VRAM"; Process = $best.P.ProcessName }
}

# Text report of current GPU users (Settings > Game detection > Show GPU usage now)
function Get-GpuReport($settings, [int[]]$excludePids) {
  $ignore = @($GpuAlwaysIgnore) + @($NotGameProcesses) + @($settings.GpuIgnore)
  $vram = [double]$settings.GpuVramGB * 1GB; $load = [double]$settings.GpuLoadPct
  $rows = foreach ($g in [GpuWatch]::Sample()) {
    $p = Get-Process -Id $g.Pid -ErrorAction SilentlyContinue
    if (-not $p) { continue }
    $state = if ($excludePids -contains $g.Pid) { 'the LLM' }
             elseif ($ignore -contains $p.ProcessName) { 'ignored' }
             elseif ($g.DedicatedBytes -ge $vram -or $g.Engine3D -ge $load) { 'WOULD PAUSE' }
             else { '' }
    [pscustomobject]@{ Process = $p.ProcessName; 'VRAM GB' = [Math]::Round($g.DedicatedBytes / 1GB, 2); '3D %' = [Math]::Round($g.Engine3D, 0); Status = $state }
  }
  $rows | Sort-Object 'VRAM GB' -Descending | Select-Object -First 25
}
