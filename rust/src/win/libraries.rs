//! Finds where games are installed by reading each launcher's own records (config files and
//! registry), on any drive, plus Windows' own list of known game exes.

use crate::detect::{self, Scan};
use regex::Regex;
use std::path::{Path, PathBuf};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

fn hklm() -> RegKey {
    RegKey::predef(HKEY_LOCAL_MACHINE)
}
fn hkcu() -> RegKey {
    RegKey::predef(HKEY_CURRENT_USER)
}

/// Subkeys of `path`, as (name, key).
fn subkeys(root: &RegKey, path: &str) -> Vec<(String, RegKey)> {
    let Ok(k) = root.open_subkey(path) else {
        return Vec::new();
    };
    k.enum_keys()
        .flatten()
        .filter_map(|n| k.open_subkey(&n).ok().map(|s| (n, s)))
        .collect()
}

fn val(k: &RegKey, name: &str) -> Option<String> {
    k.get_value::<String, _>(name)
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

fn first_capture(path: &Path, re: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    Regex::new(re)
        .ok()?
        .captures(&text)
        .map(|c| c[1].trim().to_string())
}

fn env_path(var: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(var).unwrap_or_default())
}

pub struct Scanner {
    steam: Option<steamlocate::SteamDir>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub fn new() -> Scanner {
        Scanner {
            steam: steamlocate::SteamDir::locate().ok(),
        }
    }

    /// Steam's RunningAppID with the game's name, if Steam says a game is running.
    pub fn steam_running(&self) -> Option<(u32, String)> {
        let id = hkcu()
            .open_subkey(r"Software\Valve\Steam")
            .ok()?
            .get_value::<u32, _>("RunningAppID")
            .ok()?;
        if id == 0 {
            return None;
        }
        let name = self
            .steam
            .as_ref()
            .and_then(|s| s.find_app(id).ok().flatten())
            .and_then(|(app, _)| app.name)
            .unwrap_or_else(|| format!("Steam app {id}"));
        Some((id, name))
    }

    pub fn scan(&mut self) -> Scan {
        let mut s = Scan::default();
        if self.steam.is_none() {
            self.steam = steamlocate::SteamDir::locate().ok();
        }

        // Steam: every library in libraryfolders.vdf
        if let Some(steam) = &self.steam {
            s.add_lib(
                &steam.path().join(r"steamapps\common").to_string_lossy(),
                "Steam",
                None,
            );
            if let Ok(libs) = steam.libraries() {
                for lib in libs.flatten() {
                    s.add_lib(
                        &lib.path().join(r"steamapps\common").to_string_lossy(),
                        "Steam",
                        None,
                    );
                }
            }
        }

        // Epic: launcher manifests (skip Unreal Engine installs)
        let manifests = env_path("ProgramData").join(r"Epic\EpicGamesLauncher\Data\Manifests");
        for e in std::fs::read_dir(manifests).into_iter().flatten().flatten() {
            if e.path().extension().is_some_and(|x| x == "item") {
                if let Some(m) = read_json(&e.path()) {
                    let app = m["AppName"].as_str().unwrap_or("");
                    if let (false, Some(dir)) =
                        (app.starts_with("UE_"), m["InstallLocation"].as_str())
                    {
                        s.add_lib(dir, "Epic", m["DisplayName"].as_str());
                    }
                }
            }
        }

        // GOG Galaxy
        for (_, k) in subkeys(&hklm(), r"SOFTWARE\WOW6432Node\GOG.com\Games") {
            if let Some(p) = val(&k, "path") {
                s.add_lib(&p, "GOG", val(&k, "gameName").as_deref());
            }
        }

        // EA app / Origin
        for root in [
            r"SOFTWARE\WOW6432Node\Electronic Arts",
            r"SOFTWARE\WOW6432Node\EA Games",
            r"SOFTWARE\WOW6432Node\Origin Games",
        ] {
            for (name, k) in subkeys(&hklm(), root) {
                if let Some(p) = val(&k, "Install Dir") {
                    s.add_lib(&p, "EA", Some(&name));
                }
            }
        }

        // Ubisoft Connect (+ custom default library folder)
        for (_, k) in subkeys(&hklm(), r"SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs") {
            if let Some(p) = val(&k, "InstallDir") {
                s.add_lib(&p, "Ubisoft", None);
            }
        }
        let yml = env_path("LOCALAPPDATA").join(r"Ubisoft Game Launcher\settings.yml");
        if let Some(p) = first_capture(&yml, r#"game_installation_path:\s*"?([^"\r\n]+)"#) {
            s.add_lib(&p, "Ubisoft", None);
        }

        // Rockstar
        for (name, k) in subkeys(&hklm(), r"SOFTWARE\WOW6432Node\Rockstar Games") {
            if let Some(p) = val(&k, "InstallFolder") {
                s.add_lib(&p, "Rockstar", Some(&name));
            }
        }

        // Riot: client installs json + per-product settings
        let riot = env_path("ProgramData").join("Riot Games");
        if let Some(j) = read_json(&riot.join("RiotClientInstalls.json")) {
            if let Some(obj) = j["associated_client"].as_object() {
                for dir in obj.keys() {
                    s.add_lib(dir, "Riot", None);
                }
            }
        }
        for product in std::fs::read_dir(riot.join("Metadata"))
            .into_iter()
            .flatten()
            .flatten()
        {
            for f in std::fs::read_dir(product.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                if f.file_name()
                    .to_string_lossy()
                    .ends_with(".product_settings.yaml")
                {
                    if let Some(p) =
                        first_capture(&f.path(), r#"product_install_full_path:\s*"?([^"\r\n]+)"#)
                    {
                        s.add_lib(&p, "Riot", None);
                    }
                }
            }
        }

        // Battle.net and other launchers that register games as installed programs
        let uninstall = [
            (
                hklm(),
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                hklm(),
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                hkcu(),
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
        ];
        let launcher_re: Vec<(Regex, &str)> = [
            (r"Blizzard Uninstaller|Battle\.net", "Battle.net"),
            ("RiotClientServices", "Riot"),
            ("Uplay|Ubisoft", "Ubisoft"),
            ("EAInstaller|EA Desktop|Origin", "EA"),
            ("Amazon Games", "Amazon"),
            ("Rockstar", "Rockstar"),
            ("GOG Galaxy|GalaxyClient", "GOG"),
        ]
        .into_iter()
        .map(|(r, l)| (Regex::new(&format!("(?i){r}")).unwrap(), l))
        .collect();
        for (root, path) in &uninstall {
            for (key_name, k) in subkeys(root, path) {
                let us = val(&k, "UninstallString").unwrap_or_default();
                let mut l = launcher_re
                    .iter()
                    .find(|(re, _)| re.is_match(&us))
                    .map(|(_, l)| *l);
                if l.is_none()
                    && val(&k, "Publisher").is_some_and(|p| p.contains("Battlestate Games"))
                {
                    l = Some("Battlestate (Tarkov)");
                }
                if l.is_none() && key_name.starts_with("Uplay Install ") {
                    l = Some("Ubisoft");
                }
                if let (Some(l), Some(dir)) = (l, val(&k, "InstallLocation")) {
                    s.add_lib(&dir, l, val(&k, "DisplayName").as_deref());
                }
            }
        }

        // Xbox / Game Pass: every drive with a .GamingRoot file names its games folder
        for letter in b'C'..=b'Z' {
            let root = format!("{}:\\", letter as char);
            if let Ok(bytes) = std::fs::read(format!("{root}.GamingRoot")) {
                for name in detect::parse_gaming_root(&bytes) {
                    s.add_lib(&format!("{root}{name}"), "Xbox / Game Pass", None);
                }
            }
        }

        // Meta / Oculus libraries
        for (_, k) in subkeys(&hkcu(), r"Software\Oculus VR, LLC\Oculus\Libraries") {
            if let Some(p) = val(&k, "OriginalPath") {
                s.add_lib(&format!(r"{p}\Software"), "Meta / Oculus", None);
            }
        }

        // Heroic / Legendary (Epic, GOG and Amazon games installed through Heroic)
        let appdata = env_path("APPDATA");
        for f in [
            appdata.join(r"heroic\legendaryConfig\legendary\installed.json"),
            env_path("USERPROFILE").join(r".config\legendary\installed.json"),
        ] {
            if let Some(serde_json::Value::Object(m)) = read_json(&f) {
                for g in m.values() {
                    if let Some(p) = g["install_path"].as_str() {
                        s.add_lib(p, "Heroic (Epic)", g["title"].as_str());
                    }
                }
            }
        }
        if let Some(j) = read_json(&appdata.join(r"heroic\gog_store\installed.json")) {
            for g in j["installed"].as_array().into_iter().flatten() {
                if let Some(p) = g["install_path"].as_str() {
                    s.add_lib(p, "Heroic (GOG)", None);
                }
            }
        }
        if let Some(serde_json::Value::Array(a)) =
            read_json(&appdata.join(r"heroic\nile_config\nile\installed.json"))
        {
            for g in a {
                if let Some(p) = g["path"].as_str() {
                    s.add_lib(p, "Heroic (Amazon)", None);
                }
            }
        }

        // Humble App
        if let Some(j) = read_json(&appdata.join(r"Humble App\config.json")) {
            for g in j["game-collection-4"].as_array().into_iter().flatten() {
                if let Some(p) = g["filePath"].as_str() {
                    s.add_lib(p, "Humble", g["gameName"].as_str());
                }
            }
        }

        // HoYoPlay
        for root in [r"Software\Cognosphere\HYP\1_0", r"Software\miHoYo\HYP\1_0"] {
            for (_, k) in subkeys(&hkcu(), root) {
                if let Some(p) = val(&k, "GameInstallPath") {
                    s.add_lib(&p, "HoYoPlay", None);
                }
            }
        }

        // Windows' own list of exes it recognises as games (Game Bar)
        for (_, k) in subkeys(&hkcu(), r"System\GameConfigStore\Children") {
            if let Some(p) = val(&k, "MatchedExeFullPath") {
                s.exes.insert(p.to_lowercase());
            }
        }
        s
    }
}

/// Text report of what was found (tray: "Rescan and show detected libraries").
pub fn report(scan: &Scan, extra_games: &[String]) -> String {
    let mut out = format!(
        "Scanned {}\r\n\r\nGame library folders (any process running from these = gaming):\r\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    );
    let mut libs = scan.libs.clone();
    libs.sort_by(|a, b| a.launcher.cmp(&b.launcher).then(a.dir.cmp(&b.dir)));
    for l in libs {
        out += &format!(
            "  [{}] {}{}\r\n",
            l.launcher,
            l.dir,
            l.name.map(|n| format!("  ({n})")).unwrap_or_default()
        );
    }
    out += "\r\nFallback folder patterns (any drive):\r\n";
    for (l, p) in crate::detect::game_folder_patterns() {
        out += &format!("  [{l}] {p}\r\n");
    }
    let mut exes: Vec<_> = scan.exes.iter().collect();
    exes.sort();
    out += &format!("\r\nWindows Game Bar list: {} exes\r\n", exes.len());
    for e in exes {
        out += &format!("  {e}\r\n");
    }
    out += &format!(
        "\r\nExtra games from settings: {}\r\n",
        extra_games.join(", ")
    );
    out
}
