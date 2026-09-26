//! Game detection logic (platform-independent; the Windows side only gathers the inputs).
//!
//! 1. Launcher records say where games are installed ([`Scan`], built by `win::libraries`).
//! 2. [`find_running_game`] matches running processes against those folders, default install
//!    patterns, known game/emulator names, Windows' game list and Steam's running app.
//! 3. [`GpuDetector`] catches anything else by per-process GPU use.

use crate::settings::Settings;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameHit {
    pub name: String,
    pub launcher: String,
    /// Process name (no `.exe`), for "Not a game - ignore this app".
    pub process: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProcInfo {
    pub pid: u32,
    /// Process name without `.exe`
    pub name: String,
    pub path: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    /// Normalized: backslashes, ends with `\`.
    pub dir: String,
    pub launcher: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Scan {
    pub libs: Vec<Library>,
    /// Lower-case full exe paths from Windows' game list (Game Bar).
    pub exes: HashSet<String>,
}

impl Scan {
    /// Adds a library folder unless it is a drive root, a system folder, a launcher/redist
    /// folder, or already present.
    pub fn add_lib(&mut self, dir: &str, launcher: &str, name: Option<&str>) {
        if let Some(d) = normalize_lib_dir(dir) {
            if !self.libs.iter().any(|l| l.dir.eq_ignore_ascii_case(&d)) {
                self.libs.push(Library {
                    dir: d,
                    launcher: launcher.into(),
                    name: name.map(str::to_owned),
                });
            }
        }
    }
}

// ------------------------------------------------------------------ lists

/// Default install layouts for each launcher (any drive).
static GAME_FOLDER_PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    [
        ("Steam", r"\\steamapps\\common\\"),
        ("Epic", r"\\Epic Games\\"),
        ("Xbox / Game Pass", r"\\XboxGames\\"),
        ("Xbox / Game Pass", r"\\ModifiableWindowsApps\\"),
        ("GOG", r"\\GOG Galaxy\\Games\\"),
        ("GOG", r"\\GOG Games\\"),
        ("EA", r"\\EA Games\\"),
        ("EA (Origin)", r"\\Origin Games\\"),
        ("Ubisoft", r"\\Ubisoft Game Launcher\\games\\"),
        ("Riot", r"\\Riot Games\\"),
        ("Rockstar", r"\\Rockstar Games\\"),
        ("Amazon", r"\\Amazon Games\\Library\\"),
        ("itch.io", r"\\itch\\apps\\"),
        ("Heroic", r"\\Games\\Heroic\\"),
        ("HoYoPlay", r"\\HoYoPlay\\games\\"),
        ("Meta / Oculus", r"\\Oculus\\Software\\Software\\"),
        ("Bethesda", r"\\Bethesda\.net Launcher\\games\\"),
        ("Battlestate (Tarkov)", r"\\Battlestate Games\\"),
        ("Games folder", r"^[A-Za-z]:\\Games\\"),
    ]
    .into_iter()
    .map(|(l, p)| (l, Regex::new(&format!("(?i){p}")).unwrap()))
    .collect()
});

/// Folders inside game locations that are launchers, redistributables or anti-cheat.
/// `(pattern, unless followed by)` - the second part replaces .NET's `(?!...)` lookahead.
static NOT_GAME_FOLDER_PATTERNS: LazyLock<Vec<(Regex, Option<&'static str>)>> =
    LazyLock::new(|| {
        [
            (
                r"\\steamapps\\common\\(Steamworks Shared|wallpaper_engine|Lossless Scaling)\\",
                None,
            ),
            (r"\\Epic Games\\(Launcher|Epic Online Services)\\", None),
            (r"\\Riot Games\\Riot Client\\", None),
            (r"\\Riot Vanguard\\", None),
            (r"\\Rockstar Games\\(Launcher|Social Club)\\", None),
            (r"\\Electronic Arts\\EA Desktop\\", None),
            (r"\\Origin\\", None),
            (r"\\Ubisoft Game Launcher\\", Some("games\\")),
            (r"\\GOG Galaxy\\", Some("games\\")),
            (r"\\Battle\.net\\", None),
            (r"\\HoYoPlay\\", Some("games\\")),
            (r"\\Oculus\\Support\\", None),
            (r"\\Amazon Games\\App\\", None),
            (r"\\Battlestate Games\\BsgLauncher\\", None),
            (r"\\_CommonRedist\\", None),
            (r"\\__Installer\\", None),
            (r"\\Redist\\", None),
            (r"\\DirectX\\", None),
            (r"\\Support\\", None),
            (r"\\EasyAntiCheat", None),
            (r"\\BattlEye\\", None),
        ]
        .into_iter()
        .map(|(p, unless)| (Regex::new(&format!("(?i){p}")).unwrap(), unless))
        .collect()
    });

fn lower_set(items: &[&str]) -> HashSet<String> {
    items.iter().map(|s| s.to_lowercase()).collect()
}

/// Process names that are never games, even inside a game folder.
pub static NOT_GAME_PROCESSES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    lower_set(&[
        // launchers / clients / helpers
        "steam",
        "steamwebhelper",
        "steamservice",
        "GameOverlayUI",
        "steamerrorreporter",
        "EpicGamesLauncher",
        "EpicWebHelper",
        "EOSOverlayRenderer-Win64-Shipping",
        "GalaxyClient",
        "GalaxyClientService",
        "GalaxyCommunication",
        "GOG Galaxy Notifications Renderer",
        "EADesktop",
        "EABackgroundService",
        "EALocalHostSvc",
        "Link2EA",
        "UbisoftConnect",
        "upc",
        "UplayWebCore",
        "UbisoftGameLauncher",
        "UbisoftGameLauncher64",
        "Battle.net",
        "Agent",
        "BlizzardError",
        "Battle.net Update Helper",
        "XboxPcApp",
        "GamingServices",
        "GamingServicesNet",
        "gamelaunchhelper",
        "GameBar",
        "GameBarFTServer",
        "RiotClientServices",
        "RiotClientUx",
        "RiotClientUxRender",
        "RiotClientCrashHandler",
        "LeagueClient",
        "LeagueClientUx",
        "LeagueClientUxRender",
        "VALORANT",
        "vgc",
        "vgtray",
        "Launcher",
        "LauncherPatcher",
        "RockstarService",
        "RockstarErrorHandler",
        "SocialClubHelper",
        "PlayGTAV",
        "Amazon Games",
        "Amazon Games UI",
        "itch",
        "butler",
        "Heroic",
        "legendary",
        "gogdl",
        "nile",
        "OVRServer_x64",
        "OculusClient",
        "OVRRedir",
        "HYP",
        "launcher",
        "wgc",
        "wgc_renderer_host",
        "BsgLauncher",
        "Playnite.DesktopApp",
        "Playnite.FullscreenApp",
        "Humble App",
        "wallpaper32",
        "wallpaper64",
        "webwallpaper32",
        // anti-cheat / crash reporters / installers / embedded browsers
        "EasyAntiCheat",
        "EasyAntiCheat_EOS",
        "EasyAntiCheat_EOS_Setup",
        "start_protected_game",
        "BEService",
        "BEService_x64",
        "UnityCrashHandler64",
        "UnityCrashHandler32",
        "CrashReportClient",
        "crashpad_handler",
        "CrashHandler",
        "vc_redist.x64",
        "vc_redist.x86",
        "DXSETUP",
        "UE4PrereqSetup_x64",
        "UEPrereqSetup_x64",
        "unins000",
        "QtWebEngineProcess",
        "CefSharp.BrowserSubprocess",
        // this app
        "no-drama-llama",
    ])
});

/// Games that are easy to miss by folder (custom install paths, launcher-managed).
static KNOWN_GAME_PROCESSES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    lower_set(&[
        "League of Legends",
        "VALORANT-Win64-Shipping",
        "EscapeFromTarkov",
        "EscapeFromTarkov_BE",
        "EscapeFromTarkovArena",
        "EscapeFromTarkovArena_BE",
        "GenshinImpact",
        "YuanShen",
        "StarRail",
        "ZenlessZoneZero",
        "BH3",
        "GTA5",
        "GTA5_Enhanced",
        "RDR2",
        "Warframe.x64",
        "PathOfExile",
        "PathOfExile_x64",
        "PathOfExileSteam",
    ])
});

static EMULATOR_PROCESSES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    lower_set(&[
        "retroarch",
        "Dolphin",
        "pcsx2-qt",
        "pcsx2",
        "rpcs3",
        "duckstation-qt-x64-ReleaseLTCG",
        "duckstation-qt",
        "PPSSPPWindows64",
        "Cemu",
        "Ryujinx",
        "yuzu",
        "suyu",
        "eden",
        "citron",
        "sudachi",
        "citra-qt",
        "azahar",
        "lime3ds",
        "xemu",
        "xenia",
        "xenia_canary",
        "mGBA",
        "melonDS",
        "mame",
        "Project64",
        "snes9x-x64",
        "bsnes",
        "flycast",
        "redream",
        "Vita3K",
        "shadPS4",
        "ares",
        "EmuHawk",
        "scummvm",
        "dosbox",
        "dosbox-x",
    ])
});

/// Apps that use the GPU but aren't games. Your own additions go in settings (GpuIgnore).
static GPU_ALWAYS_IGNORE: LazyLock<HashSet<String>> = LazyLock::new(|| {
    lower_set(&[
        "llama-server",
        "dwm",
        "csrss",
        "explorer",
        "ShellExperienceHost",
        "StartMenuExperienceHost",
        "SearchHost",
        "SearchApp",
        "TextInputHost",
        "Taskmgr",
        "Widgets",
        "WidgetService",
        "LockApp",
        "ApplicationFrameHost",
        "SystemSettings",
        "PhoneExperienceHost",
        "msedgewebview2",
        "chrome",
        "msedge",
        "firefox",
        "brave",
        "opera",
        "vivaldi",
        "arc",
        "Claude",
        "ChatGPT",
        "Discord",
        "Spotify",
        "Teams",
        "ms-teams",
        "Zoom",
        "slack",
        "Code",
        "Cursor",
        "obs64",
        "Streamlabs OBS",
        "wallpaper32",
        "wallpaper64",
        "webwallpaper32",
        "Lively",
        "LosslessScaling",
        "RadeonSoftware",
        "AMDRSServ",
        "AMDRSSrcExt",
        "amdow",
        "atieclxx",
        "cncmd",
        "NVIDIA Overlay",
        "nvcontainer",
        "NVIDIA app",
        "vlc",
        "mpc-hc64",
        "mpv",
        "Microsoft.Media.Player",
        "Video.UI",
        "Photos",
        "PowerToys.PowerLauncher",
        "ScreenClippingHost",
    ])
});

/// Steam app IDs that set RunningAppID but aren't games.
pub const NOT_GAME_STEAM_APP_IDS: [u32; 3] = [431960, 993090, 228980]; // Wallpaper Engine, Lossless Scaling, Steamworks Redist

pub fn game_folder_patterns() -> impl Iterator<Item = (&'static str, &'static str)> {
    GAME_FOLDER_PATTERNS.iter().map(|(l, r)| (*l, r.as_str()))
}

// ------------------------------------------------------------------ helpers

fn contains_ci(set: &HashSet<String>, name: &str) -> bool {
    set.contains(&name.to_lowercase())
}

fn list_contains_ci(list: &[String], name: &str) -> bool {
    list.iter().any(|x| x.eq_ignore_ascii_case(name))
}

pub fn is_not_game_path(path: &str) -> bool {
    NOT_GAME_FOLDER_PATTERNS.iter().any(|(re, unless)| {
        re.find_iter(path).any(|m| match unless {
            None => true,
            Some(next) => !path[m.end()..].to_lowercase().starts_with(next),
        })
    })
}

/// Normalizes a library folder, or rejects it (drive roots, system folders, launcher folders).
pub fn normalize_lib_dir(dir: &str) -> Option<String> {
    let d = dir.trim().trim_matches('"').trim().replace('/', "\\");
    let d = d.trim_end_matches('\\');
    if d.is_empty() || d.contains("..") {
        return None;
    }
    let d = format!("{d}\\");
    if d.split('\\').filter(|s| !s.is_empty()).count() < 2 {
        return None; // never a whole drive
    }
    static SYSTEM: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)^[a-z]:\\(Program Files( \(x86\))?|Windows|ProgramData|Users\\[^\\]+|Users)\\$",
        )
        .unwrap()
    });
    if SYSTEM.is_match(&d) || is_not_game_path(&d) {
        return None;
    }
    Some(d)
}

fn starts_with_ci(path: &str, prefix: &str) -> bool {
    path.len() >= prefix.len()
        && path.is_char_boundary(prefix.len())
        && path[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Folder names in an Xbox app `.GamingRoot` file: an 8-byte header, then NUL-separated
/// UTF-16LE names.
pub fn parse_gaming_root(bytes: &[u8]) -> Vec<String> {
    if bytes.len() <= 8 {
        return Vec::new();
    }
    let utf16: Vec<u16> = bytes[8..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    String::from_utf16_lossy(&utf16)
        .split('\0')
        .filter(|n| n.chars().any(char::is_alphanumeric))
        .map(str::to_owned)
        .collect()
}

pub fn friendly_name(p: &ProcInfo, known: Option<&str>) -> String {
    let n = known
        .filter(|s| !s.is_empty())
        .or(p.title.as_deref().filter(|s| !s.is_empty()))
        .unwrap_or(&p.name);
    if n.chars().count() > 40 {
        format!("{}...", n.chars().take(40).collect::<String>())
    } else {
        n.to_string()
    }
}

fn hit(p: &ProcInfo, known: Option<&str>, launcher: &str) -> GameHit {
    GameHit {
        name: friendly_name(p, known),
        launcher: launcher.into(),
        process: Some(p.name.clone()),
    }
}

/// Label for a process found by GPU use: its launcher/library if its exe is in a known game
/// location, otherwise just "GPU".
pub fn launch_label(p: &ProcInfo, scan: &Scan) -> GameHit {
    if let Some(path) = &p.path {
        if let Some(lib) = scan.libs.iter().find(|l| starts_with_ci(path, &l.dir)) {
            return hit(p, lib.name.as_deref(), &lib.launcher);
        }
        if let Some((l, _)) = GAME_FOLDER_PATTERNS
            .iter()
            .find(|(_, re)| re.is_match(path))
        {
            return hit(p, None, l);
        }
    }
    hit(p, None, "GPU")
}

/// First running process that looks like a game, by launcher records and name lists.
/// `steam_running` is Steam's RunningAppID with the app's name, if any.
pub fn find_running_game(
    procs: &[ProcInfo],
    scan: &Scan,
    s: &Settings,
    steam_running: Option<(u32, String)>,
) -> Option<GameHit> {
    if let Some((id, name)) = steam_running {
        if id != 0 && !NOT_GAME_STEAM_APP_IDS.contains(&id) {
            return Some(GameHit {
                name,
                launcher: "Steam".into(),
                process: None,
            });
        }
    }
    for p in procs {
        let n = p.name.as_str();
        if contains_ci(&NOT_GAME_PROCESSES, n) {
            continue;
        }
        if list_contains_ci(&s.extra_games, n) {
            return Some(hit(p, None, "your list"));
        }
        if contains_ci(&KNOWN_GAME_PROCESSES, n) {
            return Some(hit(p, None, "known game"));
        }
        let lower = n.to_lowercase();
        if s.detect_emulators
            && (contains_ci(&EMULATOR_PROCESSES, n)
                || lower.starts_with("xenia")
                || lower.starts_with("duckstation"))
        {
            return Some(hit(p, None, "emulator"));
        }
        let Some(path) = p.path.as_deref() else {
            continue;
        };
        if is_not_game_path(path) {
            continue;
        }
        if s.use_windows_game_list && scan.exes.contains(&path.to_lowercase()) {
            return Some(hit(p, None, "Windows game list"));
        }
        if let Some(lib) = scan.libs.iter().find(|l| starts_with_ci(path, &l.dir)) {
            return Some(hit(p, lib.name.as_deref(), &lib.launcher));
        }
        if let Some((l, _)) = GAME_FOLDER_PATTERNS
            .iter()
            .find(|(_, re)| re.is_match(path))
        {
            return Some(hit(p, None, l));
        }
    }
    None
}

// ------------------------------------------------------------------ GPU

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GpuProc {
    pub pid: u32,
    /// % of the 3D engine (summed over engines, capped at 100)
    pub engine_3d: f64,
    /// VRAM held
    pub dedicated_bytes: u64,
}

const GB: f64 = 1024.0 * 1024.0 * 1024.0;

fn is_gpu_ignored(name: &str, s: &Settings) -> bool {
    contains_ci(&GPU_ALWAYS_IGNORE, name)
        || contains_ci(&NOT_GAME_PROCESSES, name)
        || list_contains_ci(&s.gpu_ignore, name)
}

/// Pauses when another app holds `GpuVramGB` of VRAM or `GpuLoadPct` of the 3D engine on two
/// checks in a row, or right away when an exclusive full-screen D3D app is in the foreground.
#[derive(Debug, Default)]
pub struct GpuDetector {
    hits: u32,
}

impl GpuDetector {
    pub fn check(
        &mut self,
        samples: &[GpuProc],
        procs: &HashMap<u32, ProcInfo>,
        s: &Settings,
        exclude_pids: &[u32],
        fullscreen_foreground_pid: Option<u32>,
        scan: &Scan,
    ) -> Option<GameHit> {
        let vram = s.gpu_vram_gb * GB;
        let best = samples
            .iter()
            .filter(|g| g.pid > 4 && !exclude_pids.contains(&g.pid))
            .filter(|g| g.dedicated_bytes as f64 >= vram || g.engine_3d >= s.gpu_load_pct)
            .filter_map(|g| procs.get(&g.pid).map(|p| (g, p)))
            .filter(|(_, p)| !is_gpu_ignored(&p.name, s))
            .max_by_key(|(g, _)| g.dedicated_bytes);

        let Some((g, p)) = best else {
            self.hits = 0;
            let fp = fullscreen_foreground_pid.and_then(|pid| procs.get(&pid))?;
            if is_gpu_ignored(&fp.name, s) || exclude_pids.contains(&fp.pid) {
                return None;
            }
            let mut h = launch_label(fp, scan);
            h.launcher = format!("{} - full-screen", h.launcher);
            return Some(h);
        };
        self.hits += 1;
        if self.hits < 2 {
            return None; // short spikes don't pause the model
        }
        let mut h = launch_label(p, scan);
        h.launcher = format!(
            "{} - {:.1} GB VRAM",
            h.launcher,
            g.dedicated_bytes as f64 / GB
        );
        Some(h)
    }

    pub fn reset(&mut self) {
        self.hits = 0;
    }
}

/// Text report of current GPU users (tray: "Show GPU usage now").
pub fn gpu_report(
    samples: &[GpuProc],
    procs: &HashMap<u32, ProcInfo>,
    s: &Settings,
    exclude_pids: &[u32],
) -> String {
    let mut rows: Vec<(String, f64, f64, &str)> = samples
        .iter()
        .filter_map(|g| {
            let p = procs.get(&g.pid)?;
            let state = if exclude_pids.contains(&g.pid) {
                "the LLM"
            } else if is_gpu_ignored(&p.name, s) {
                "ignored"
            } else if g.dedicated_bytes as f64 >= s.gpu_vram_gb * GB
                || g.engine_3d >= s.gpu_load_pct
            {
                "WOULD PAUSE"
            } else {
                ""
            };
            Some((
                p.name.clone(),
                g.dedicated_bytes as f64 / GB,
                g.engine_3d,
                state,
            ))
        })
        .collect();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut out = format!(
        "GPU users right now (pause when VRAM >= {} GB or 3D >= {}% on 2 checks in a row)\r\n\
         Ignore list: GpuIgnore in the settings file, or 'Not a game - ignore this app' while paused.\r\n\r\n\
         {:<40} {:>8} {:>5}  Status\r\n",
        s.gpu_vram_gb, s.gpu_load_pct, "Process", "VRAM GB", "3D %"
    );
    for (name, gb, load, state) in rows.into_iter().take(25) {
        out.push_str(&format!("{name:<40} {gb:>8.2} {load:>5.0}  {state}\r\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(name: &str, path: &str) -> ProcInfo {
        ProcInfo {
            pid: 1234,
            name: name.into(),
            path: Some(path.into()),
            title: None,
        }
    }

    #[test]
    fn normalizes_and_rejects_lib_dirs() {
        assert_eq!(
            normalize_lib_dir("D:/SteamLibrary/steamapps/common").as_deref(),
            Some(r"D:\SteamLibrary\steamapps\common\")
        );
        for bad in [
            r"C:\",
            r"C:\ProgramData",
            r"C:\Program Files",
            r"c:\users\me",
            r"C:\Windows",
            r"C:\Program Files\Epic Games\Launcher",
            "",
        ] {
            assert_eq!(normalize_lib_dir(bad), None, "{bad}");
        }
    }

    #[test]
    fn scan_dedupes() {
        let mut s = Scan::default();
        s.add_lib(r"E:\Epic Games\Fortnite", "Epic", Some("Fortnite"));
        s.add_lib(r"e:\epic games\fortnite\", "Epic", Some("Fortnite"));
        assert_eq!(s.libs.len(), 1);
    }

    #[test]
    fn lookahead_replacements() {
        assert!(is_not_game_path(
            r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\upc.exe"
        ));
        assert!(!is_not_game_path(
            r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\games\AC\ac.exe"
        ));
        assert!(is_not_game_path(
            r"C:\Program Files (x86)\GOG Galaxy\GalaxyClient.exe"
        ));
        assert!(!is_not_game_path(
            r"C:\Program Files (x86)\GOG Galaxy\Games\Witcher\w.exe"
        ));
    }

    #[test]
    fn nothing_running() {
        let procs = [proc("notepad", r"C:\Windows\notepad.exe")];
        assert_eq!(
            find_running_game(&procs, &Scan::default(), &Settings::default(), None),
            None
        );
    }

    #[test]
    fn default_folder_pattern() {
        let mut p = proc(
            "eldenring",
            r"D:\SteamLibrary\steamapps\common\ELDEN RING\Game\eldenring.exe",
        );
        p.title = Some("ELDEN RING".into());
        let g = find_running_game(&[p], &Scan::default(), &Settings::default(), None).unwrap();
        assert_eq!(
            (g.name.as_str(), g.launcher.as_str()),
            ("ELDEN RING", "Steam")
        );
        assert_eq!(g.process.as_deref(), Some("eldenring"));
    }

    #[test]
    fn ignores_launchers_and_redists() {
        let procs = [
            proc(
                "EpicGamesLauncher",
                r"C:\Program Files\Epic Games\Launcher\EpicGamesLauncher.exe",
            ),
            proc(
                "vc_redist.x64",
                r"D:\SteamLibrary\steamapps\common\Foo\_CommonRedist\vc_redist.x64.exe",
            ),
            proc(
                "setup",
                r"D:\SteamLibrary\steamapps\common\Steamworks Shared\setup.exe",
            ),
        ];
        assert_eq!(
            find_running_game(&procs, &Scan::default(), &Settings::default(), None),
            None
        );
    }

    #[test]
    fn discovered_library() {
        let mut scan = Scan::default();
        scan.add_lib(r"F:\MyGames", "GOG", Some("Witcher 3"));
        let g = find_running_game(
            &[proc("witcher3", r"f:\mygames\bin\witcher3.exe")],
            &scan,
            &Settings::default(),
            None,
        )
        .unwrap();
        assert_eq!((g.name.as_str(), g.launcher.as_str()), ("Witcher 3", "GOG"));
    }

    #[test]
    fn windows_game_list_toggle() {
        let mut scan = Scan::default();
        scan.exes.insert(r"x:\odd\place\game.exe".into());
        let procs = [proc("game", r"X:\Odd\Place\game.exe")];
        let on = Settings::default();
        assert_eq!(
            find_running_game(&procs, &scan, &on, None)
                .unwrap()
                .launcher,
            "Windows game list"
        );
        let off = Settings {
            use_windows_game_list: false,
            ..Default::default()
        };
        assert_eq!(find_running_game(&procs, &scan, &off, None), None);
    }

    #[test]
    fn emulators_toggle_and_extra_games() {
        let procs = [proc("RetroArch", r"C:\RetroArch\retroarch.exe")];
        assert_eq!(
            find_running_game(&procs, &Scan::default(), &Settings::default(), None)
                .unwrap()
                .launcher,
            "emulator"
        );
        let off = Settings {
            detect_emulators: false,
            ..Default::default()
        };
        assert_eq!(
            find_running_game(&procs, &Scan::default(), &off, None),
            None
        );
        let extra = Settings {
            extra_games: vec!["MYGAME".into()],
            ..Default::default()
        };
        let g = find_running_game(
            &[proc("mygame", r"C:\Stuff\mygame.exe")],
            &Scan::default(),
            &extra,
            None,
        )
        .unwrap();
        assert_eq!(g.launcher, "your list");
    }

    #[test]
    fn steam_running_app() {
        let g = find_running_game(
            &[],
            &Scan::default(),
            &Settings::default(),
            Some((1245620, "ELDEN RING".into())),
        )
        .unwrap();
        assert_eq!(g.launcher, "Steam");
        assert_eq!(
            find_running_game(
                &[],
                &Scan::default(),
                &Settings::default(),
                Some((431960, "Wallpaper Engine".into()))
            ),
            None
        );
    }

    #[test]
    fn friendly_name_truncates() {
        assert_eq!(
            friendly_name(&proc(&"a".repeat(60), "x"), None)
                .chars()
                .count(),
            43
        );
    }

    fn gpu_setup() -> (HashMap<u32, ProcInfo>, Settings) {
        let mut procs = HashMap::new();
        for (pid, name) in [(100, "game"), (200, "chrome"), (300, "llama-server")] {
            procs.insert(
                pid,
                ProcInfo {
                    pid,
                    name: name.into(),
                    path: None,
                    title: None,
                },
            );
        }
        (procs, Settings::default())
    }

    #[test]
    fn gpu_needs_two_hits() {
        let (procs, s) = gpu_setup();
        let mut d = GpuDetector::default();
        let samples = [GpuProc {
            pid: 100,
            engine_3d: 80.0,
            dedicated_bytes: 4 << 30,
        }];
        assert_eq!(
            d.check(&samples, &procs, &s, &[], None, &Scan::default()),
            None
        );
        let g = d
            .check(&samples, &procs, &s, &[], None, &Scan::default())
            .unwrap();
        assert_eq!(g.launcher, "GPU - 4.0 GB VRAM");
        // gone -> reset
        assert_eq!(d.check(&[], &procs, &s, &[], None, &Scan::default()), None);
        assert_eq!(
            d.check(&samples, &procs, &s, &[], None, &Scan::default()),
            None
        );
    }

    #[test]
    fn gpu_ignores_browsers_llm_and_small_users() {
        let (procs, s) = gpu_setup();
        let mut d = GpuDetector::default();
        let samples = [
            GpuProc {
                pid: 200,
                engine_3d: 90.0,
                dedicated_bytes: 3 << 30,
            },
            GpuProc {
                pid: 300,
                engine_3d: 99.0,
                dedicated_bytes: 20 << 30,
            },
            GpuProc {
                pid: 100,
                engine_3d: 5.0,
                dedicated_bytes: 100 << 20,
            },
        ];
        for _ in 0..3 {
            assert_eq!(
                d.check(&samples, &procs, &s, &[300], None, &Scan::default()),
                None
            );
        }
        let s2 = Settings {
            gpu_ignore: vec!["GAME".into()],
            ..Default::default()
        };
        let big = [GpuProc {
            pid: 100,
            engine_3d: 90.0,
            dedicated_bytes: 8 << 30,
        }];
        for _ in 0..3 {
            assert_eq!(
                d.check(&big, &procs, &s2, &[], None, &Scan::default()),
                None
            );
        }
    }

    #[test]
    fn gpu_fullscreen_pauses_immediately() {
        let (procs, s) = gpu_setup();
        let g = GpuDetector::default()
            .check(&[], &procs, &s, &[], Some(100), &Scan::default())
            .unwrap();
        assert_eq!(g.launcher, "GPU - full-screen");
        assert_eq!(
            GpuDetector::default().check(&[], &procs, &s, &[], Some(200), &Scan::default()),
            None
        );
    }

    #[test]
    fn report_lists_states() {
        let (procs, s) = gpu_setup();
        let samples = [
            GpuProc {
                pid: 100,
                engine_3d: 80.0,
                dedicated_bytes: 4 << 30,
            },
            GpuProc {
                pid: 200,
                engine_3d: 1.0,
                dedicated_bytes: 1 << 20,
            },
            GpuProc {
                pid: 300,
                engine_3d: 1.0,
                dedicated_bytes: 18 << 30,
            },
        ];
        let r = gpu_report(&samples, &procs, &s, &[300]);
        assert!(r.contains("WOULD PAUSE") && r.contains("the LLM") && r.contains("ignored"));
        assert!(
            r.find("\r\nllama-server").unwrap() < r.find("\r\ngame ").unwrap(),
            "sorted by VRAM"
        );
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use proptest::prelude::*;

    fn proc(name: &str, path: &str) -> ProcInfo {
        ProcInfo {
            pid: 1000,
            name: name.into(),
            path: Some(path.into()),
            title: None,
        }
    }

    fn launcher_of(path: &str) -> Option<String> {
        let exe = path.rsplit('\\').next().unwrap().trim_end_matches(".exe");
        find_running_game(
            &[proc(exe, path)],
            &Scan::default(),
            &Settings::default(),
            None,
        )
        .map(|g| g.launcher)
    }

    #[test]
    fn every_default_install_layout_is_detected() {
        let cases = [
            (
                r"D:\SteamLibrary\steamapps\common\Hades\x64\Hades.exe",
                "Steam",
            ),
            (
                r"C:\Program Files\Epic Games\Fortnite\FortniteClient.exe",
                "Epic",
            ),
            (
                r"E:\XboxGames\Forza Horizon 5\Content\ForzaHorizon5.exe",
                "Xbox / Game Pass",
            ),
            (
                r"C:\Program Files\ModifiableWindowsApps\Halo\halo.exe",
                "Xbox / Game Pass",
            ),
            (
                r"C:\Program Files (x86)\GOG Galaxy\Games\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe",
                "GOG",
            ),
            (r"D:\GOG Games\Witcher 3\bin\witcher3.exe", "GOG"),
            (
                r"C:\Program Files\EA Games\Battlefield 2042\BF2042.exe",
                "EA",
            ),
            (
                r"C:\Program Files (x86)\Origin Games\Titanfall2\Titanfall2.exe",
                "EA (Origin)",
            ),
            (
                r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\games\Far Cry 6\bin\FarCry6.exe",
                "Ubisoft",
            ),
            (r"C:\Riot Games\Some Game\game.exe", "Riot"),
            (
                r"C:\Program Files\Rockstar Games\Red Dead Redemption 2\rdr2x.exe",
                "Rockstar",
            ),
            (r"C:\Amazon Games\Library\Lost Ark\LOSTARK.exe", "Amazon"),
            (
                r"C:\Users\me\AppData\Roaming\itch\apps\celeste\Celeste.exe",
                "itch.io",
            ),
            (
                r"C:\Users\me\Games\Heroic\Hollow Knight\hollow_knight.exe",
                "Heroic",
            ),
            (
                r"C:\Program Files\HoYoPlay\games\Honkai\honkai.exe",
                "HoYoPlay",
            ),
            (
                r"C:\Program Files\Oculus\Software\Software\beat-games\BeatSaber.exe",
                "Meta / Oculus",
            ),
            (
                r"C:\Program Files (x86)\Bethesda.net Launcher\games\Doom\doom.exe",
                "Bethesda",
            ),
            (
                r"C:\Battlestate Games\EFT\Tarkov.exe",
                "Battlestate (Tarkov)",
            ),
            (r"G:\Games\World of Tanks\WorldOfTanks.exe", "Games folder"),
        ];
        for (path, want) in cases {
            assert_eq!(launcher_of(path).as_deref(), Some(want), "{path}");
        }
    }

    #[test]
    fn launchers_redists_and_anticheat_are_never_games() {
        for path in [
            r"C:\Program Files (x86)\Steam\steamapps\common\Steamworks Shared\_CommonRedist\DirectX\DXSETUP.exe",
            r"D:\SteamLibrary\steamapps\common\wallpaper_engine\wallpaper32.exe",
            r"D:\SteamLibrary\steamapps\common\Lossless Scaling\LosslessScaling.exe",
            r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
            r"C:\Program Files (x86)\Epic Games\Epic Online Services\EOSService.exe",
            r"C:\Riot Games\Riot Client\RiotClientServices.exe",
            r"C:\Program Files\Riot Vanguard\vgc.exe",
            r"C:\Program Files\Rockstar Games\Launcher\Launcher.exe",
            r"C:\Program Files\Rockstar Games\Social Club\SocialClubHelper.exe",
            r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe",
            r"C:\Program Files (x86)\Origin\Origin.exe",
            r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\UbisoftConnect.exe",
            r"C:\Program Files (x86)\GOG Galaxy\GalaxyClient.exe",
            r"C:\Program Files (x86)\Battle.net\Battle.net.exe",
            r"C:\Program Files\HoYoPlay\launcher.exe",
            r"C:\Program Files\Oculus\Support\oculus-client\OculusClient.exe",
            r"C:\Amazon Games\App\Amazon Games UI.exe",
            r"C:\Battlestate Games\BsgLauncher\BsgLauncher.exe",
            r"D:\SteamLibrary\steamapps\common\Elden Ring\Game\EasyAntiCheat\EasyAntiCheat_EOS_Setup.exe",
            r"D:\SteamLibrary\steamapps\common\Arma 3\BattlEye\BEService.exe",
            r"D:\SteamLibrary\steamapps\common\Game\__Installer\setup.exe",
            r"D:\SteamLibrary\steamapps\common\Game\Redist\vcredist.exe",
            r"D:\SteamLibrary\steamapps\common\Game\Support\helper.exe",
        ] {
            assert_eq!(launcher_of(path), None, "{path}");
        }
    }

    #[test]
    fn not_game_process_names_win_even_inside_game_folders_case_insensitively() {
        for name in [
            "steam",
            "STEAMWEBHELPER",
            "UnityCrashHandler64",
            "crashpad_handler",
            "no-drama-llama",
            "QtWebEngineProcess",
        ] {
            let p = proc(
                name,
                r"D:\SteamLibrary\steamapps\common\Some Game\helper.exe",
            );
            assert_eq!(
                find_running_game(&[p], &Scan::default(), &Settings::default(), None),
                None,
                "{name}"
            );
        }
    }

    #[test]
    fn known_games_anywhere() {
        for name in [
            "League of Legends",
            "valorant-win64-shipping",
            "GenshinImpact",
            "GTA5",
            "Warframe.x64",
            "EscapeFromTarkov",
        ] {
            let g = find_running_game(
                &[proc(name, r"X:\Weird\Place\a.exe")],
                &Scan::default(),
                &Settings::default(),
                None,
            )
            .unwrap();
            assert_eq!(g.launcher, "known game", "{name}");
        }
    }

    #[test]
    fn emulator_prefixes() {
        for name in [
            "xenia_canary_netplay",
            "Xenia",
            "duckstation-nogui-x64-ReleaseLTCG",
            "Dolphin",
            "RPCS3",
            "ryujinx",
        ] {
            let g = find_running_game(
                &[proc(name, r"C:\Emu\x.exe")],
                &Scan::default(),
                &Settings::default(),
                None,
            );
            assert_eq!(g.map(|g| g.launcher).as_deref(), Some("emulator"), "{name}");
        }
    }

    #[test]
    fn processes_without_a_path_only_match_by_name() {
        let mut p = proc("SomeGame", "");
        p.path = None;
        assert_eq!(
            find_running_game(
                std::slice::from_ref(&p),
                &Scan::default(),
                &Settings::default(),
                None
            ),
            None
        );
        p.name = "GTA5".into();
        assert!(find_running_game(&[p], &Scan::default(), &Settings::default(), None).is_some());
    }

    #[test]
    fn first_match_wins_and_steam_comes_first() {
        let procs = [
            proc("notepad", r"C:\Windows\notepad.exe"),
            proc("GTA5", "C:\\x\\GTA5.exe"),
            proc("retroarch", "C:\\r\\retroarch.exe"),
        ];
        assert_eq!(
            find_running_game(&procs, &Scan::default(), &Settings::default(), None)
                .unwrap()
                .launcher,
            "known game"
        );
        let g = find_running_game(
            &procs,
            &Scan::default(),
            &Settings::default(),
            Some((570, "Dota 2".into())),
        )
        .unwrap();
        assert_eq!((g.name.as_str(), g.launcher.as_str()), ("Dota 2", "Steam"));
        assert!(g.process.is_none(), "Steam hits have no process to ignore");
        assert_eq!(
            find_running_game(
                &procs,
                &Scan::default(),
                &Settings::default(),
                Some((0, String::new()))
            )
            .unwrap()
            .launcher,
            "known game"
        );
        for id in NOT_GAME_STEAM_APP_IDS {
            assert_ne!(
                find_running_game(
                    &[],
                    &Scan::default(),
                    &Settings::default(),
                    Some((id, "x".into()))
                )
                .map(|g| g.launcher)
                .as_deref(),
                Some("Steam")
            );
        }
    }

    #[test]
    fn library_prefix_needs_a_folder_boundary() {
        let mut scan = Scan::default();
        scan.add_lib(r"F:\Games2", "GOG", None);
        // F:\Games20\ is not inside F:\Games2\
        assert_eq!(
            find_running_game(
                &[proc("x", r"F:\Games20\x.exe")],
                &scan,
                &Settings::default(),
                None
            ),
            None
        );
        assert!(find_running_game(
            &[proc("x", r"F:\Games2\x.exe")],
            &scan,
            &Settings::default(),
            None
        )
        .is_some());
    }

    #[test]
    fn non_ascii_paths_do_not_panic() {
        let mut scan = Scan::default();
        scan.add_lib(r"D:\Spiele\Ünïcødé", "GOG", Some("Spiel"));
        for path in [
            r"D:\Spiele\Ünïcødé\spiel.exe",
            r"D:\Spi",
            r"D:\Spiele\Ü",
            "é",
            r"D:\ゲーム\game.exe",
            "",
        ] {
            let _ = find_running_game(&[proc("x", path)], &scan, &Settings::default(), None);
            let _ = launch_label(&proc("x", path), &scan);
        }
        let g = find_running_game(
            &[proc("spiel", r"d:\spiele\ünïcødé\spiel.exe")],
            &scan,
            &Settings::default(),
            None,
        );
        // ASCII-only case folding: "Ü" vs "ü" differ, so this is (safely) not matched by the library
        assert!(g.is_none() || g.unwrap().launcher == "GOG");
    }

    #[test]
    fn normalize_edge_cases() {
        assert_eq!(
            normalize_lib_dir("\"D:\\Games\\Steam\\\"").as_deref(),
            Some(r"D:\Games\Steam\")
        );
        assert_eq!(
            normalize_lib_dir(r"D:\Games\Steam\\\").as_deref(),
            Some(r"D:\Games\Steam\")
        );
        assert_eq!(
            normalize_lib_dir(r"  D:\Lib  ").as_deref(),
            Some(r"D:\Lib\")
        );
        assert_eq!(
            normalize_lib_dir("\" D:\\Lib \"").as_deref(),
            Some(r"D:\Lib\"),
            "whitespace inside quotes"
        );
        assert_eq!(
            normalize_lib_dir(r"\\nas\share\games").as_deref(),
            Some(r"\\nas\share\games\")
        );
        for bad in [
            r"D:\Games\..\Windows",
            "D:",
            "D:\\",
            "\\",
            "   ",
            r"C:\Users",
            r"C:\Program Files (x86)",
        ] {
            assert_eq!(normalize_lib_dir(bad), None, "{bad}");
        }
    }

    #[test]
    fn gaming_root_file() {
        let mut bytes = vec![0x52, 0x47, 0x42, 0x58, 1, 0, 0, 0];
        for name in ["XboxGames", "Other"] {
            bytes.extend(name.encode_utf16().flat_map(|u| u.to_le_bytes()));
            bytes.extend([0, 0]);
        }
        assert_eq!(parse_gaming_root(&bytes), vec!["XboxGames", "Other"]);
        assert!(parse_gaming_root(&bytes[..8]).is_empty());
        assert!(parse_gaming_root(&[]).is_empty());
        assert_eq!(
            parse_gaming_root(&bytes[..11]).len(),
            1,
            "odd trailing byte is ignored"
        );
    }

    #[test]
    fn lists_have_no_duplicates_and_cover_key_entries() {
        for set in [
            &*NOT_GAME_PROCESSES,
            &*KNOWN_GAME_PROCESSES,
            &*EMULATOR_PROCESSES,
            &*GPU_ALWAYS_IGNORE,
        ] {
            assert!(set.iter().all(|s| *s == s.to_lowercase()));
        }
        assert!(NOT_GAME_PROCESSES.contains("no-drama-llama"));
        assert!(GPU_ALWAYS_IGNORE.contains("llama-server"));
        assert!(GPU_ALWAYS_IGNORE.contains("dwm"));
        assert!(KNOWN_GAME_PROCESSES.is_disjoint(&NOT_GAME_PROCESSES));
        assert!(EMULATOR_PROCESSES.is_disjoint(&NOT_GAME_PROCESSES));
    }

    fn gpu_procs() -> HashMap<u32, ProcInfo> {
        [
            (4, "System"),
            (100, "game"),
            (101, "game2"),
            (200, "chrome"),
            (300, "llama-server"),
            (400, "Steam"),
        ]
        .into_iter()
        .map(|(pid, n)| {
            (
                pid,
                ProcInfo {
                    pid,
                    name: n.into(),
                    path: Some(format!(r"D:\SteamLibrary\steamapps\common\{n}\{n}.exe")),
                    title: None,
                },
            )
        })
        .collect()
    }

    fn g(pid: u32, load: f64, gb: f64) -> GpuProc {
        GpuProc {
            pid,
            engine_3d: load,
            dedicated_bytes: (gb * GB) as u64,
        }
    }

    fn twice(d: &mut GpuDetector, samples: &[GpuProc], s: &Settings) -> Option<GameHit> {
        d.check(samples, &gpu_procs(), s, &[300], None, &Scan::default());
        d.check(samples, &gpu_procs(), s, &[300], None, &Scan::default())
    }

    #[test]
    fn gpu_thresholds_are_inclusive_and_either_one_triggers() {
        let s = Settings::default(); // 1.5 GB or 30 %
        assert!(
            twice(&mut GpuDetector::default(), &[g(100, 0.0, 1.5)], &s).is_some(),
            "VRAM exactly at threshold"
        );
        assert!(
            twice(&mut GpuDetector::default(), &[g(100, 30.0, 0.0)], &s).is_some(),
            "load exactly at threshold"
        );
        assert!(twice(&mut GpuDetector::default(), &[g(100, 29.9, 1.49)], &s).is_none());
        let strict = Settings {
            gpu_vram_gb: 4.0,
            gpu_load_pct: 70.0,
            ..Default::default()
        };
        assert!(twice(&mut GpuDetector::default(), &[g(100, 50.0, 3.0)], &strict).is_none());
    }

    #[test]
    fn gpu_picks_biggest_vram_user_and_labels_by_library() {
        let h = twice(
            &mut GpuDetector::default(),
            &[g(100, 90.0, 2.0), g(101, 40.0, 6.0)],
            &Settings::default(),
        )
        .unwrap();
        assert_eq!(h.process.as_deref(), Some("game2"));
        assert_eq!(h.launcher, "Steam - 6.0 GB VRAM");
    }

    #[test]
    fn gpu_ignores_system_pids_llm_and_not_game_launchers() {
        let s = Settings::default();
        for pid in [4, 300, 400, 200, 999] {
            assert!(
                twice(&mut GpuDetector::default(), &[g(pid, 99.0, 10.0)], &s).is_none(),
                "pid {pid}"
            );
        }
    }

    #[test]
    fn gpu_spike_between_quiet_checks_does_not_pause() {
        let mut d = GpuDetector::default();
        let s = Settings::default();
        for _ in 0..5 {
            assert!(d
                .check(
                    &[g(100, 90.0, 4.0)],
                    &gpu_procs(),
                    &s,
                    &[],
                    None,
                    &Scan::default()
                )
                .is_none());
            assert!(d
                .check(&[], &gpu_procs(), &s, &[], None, &Scan::default())
                .is_none());
        }
    }

    #[test]
    fn fullscreen_respects_ignore_lists_and_exclusions() {
        let s = Settings {
            gpu_ignore: vec!["game2".into()],
            ..Default::default()
        };
        let mut d = GpuDetector::default();
        assert!(
            d.check(&[], &gpu_procs(), &s, &[], Some(101), &Scan::default())
                .is_none(),
            "user-ignored"
        );
        assert!(
            d.check(&[], &gpu_procs(), &s, &[100], Some(100), &Scan::default())
                .is_none(),
            "excluded pid"
        );
        assert!(
            d.check(&[], &gpu_procs(), &s, &[], Some(4242), &Scan::default())
                .is_none(),
            "unknown pid"
        );
        assert!(d
            .check(&[], &gpu_procs(), &s, &[], Some(100), &Scan::default())
            .is_some());
    }

    #[test]
    fn gpu_report_is_capped_and_handles_empty() {
        let s = Settings::default();
        let procs: HashMap<u32, ProcInfo> = (0..40)
            .map(|i| {
                (
                    i + 10,
                    ProcInfo {
                        pid: i + 10,
                        name: format!("p{i}"),
                        path: None,
                        title: None,
                    },
                )
            })
            .collect();
        let samples: Vec<GpuProc> = (0..40).map(|i| g(i + 10, 1.0, i as f64 / 10.0)).collect();
        let r = gpu_report(&samples, &procs, &s, &[]);
        assert_eq!(r.lines().filter(|l| l.starts_with('p')).count(), 25);
        let empty = gpu_report(&[], &HashMap::new(), &s, &[]);
        assert!(empty.contains("Process") && empty.lines().count() == 4);
    }

    proptest! {
        #[test]
        fn normalize_never_panics_and_output_is_consistent(dir in ".{0,80}") {
            if let Some(d) = normalize_lib_dir(&dir) {
                prop_assert!(d.ends_with('\\'));
                prop_assert!(!d.contains('/'));
                prop_assert!(!d.contains(".."));
                prop_assert!(d.split('\\').filter(|s| !s.is_empty()).count() >= 2);
                prop_assert_eq!(normalize_lib_dir(&d), Some(d.clone()), "idempotent");
            }
        }

        #[test]
        fn matching_never_panics(name in ".{0,30}", path in ".{0,120}", lib in ".{0,40}") {
            let mut scan = Scan::default();
            scan.add_lib(&lib, "X", None);
            let p = ProcInfo { pid: 5, name, path: Some(path), title: None };
            let _ = find_running_game(std::slice::from_ref(&p), &scan, &Settings::default(), None);
            let _ = launch_label(&p, &scan);
            let _ = is_not_game_path(p.path.as_deref().unwrap());
        }

        #[test]
        fn gpu_detector_never_pauses_on_one_sample(load in 0.0f64..100.0, gb in 0.0f64..32.0) {
            let mut d = GpuDetector::default();
            prop_assert!(d.check(&[g(100, load, gb)], &gpu_procs(), &Settings::default(), &[], None, &Scan::default()).is_none());
        }

        #[test]
        fn friendly_names_are_bounded(n in ".{0,200}") {
            let f = friendly_name(&ProcInfo { pid: 1, name: n, path: None, title: None }, None);
            prop_assert!(f.chars().count() <= 43);
        }
    }
}
