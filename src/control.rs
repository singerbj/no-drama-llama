//! What the settings window and the tray say to each other: one JSON object per line.
//! The tray starts the window (`no-drama-llama.exe settings-window`) as a child process; the
//! window writes [`Request`]s to its stdout and reads [`Event`]s from its stdin. Nothing
//! listens on a port or pipe name, so only the tray can drive it.

use crate::catalog::{self, Fit, Machine};
use crate::settings::Settings;
use crate::state::Tone;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Settings window -> tray.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Toggle,
    Restart,
    OpenChat,
    /// Changed settings only, as settings.json keys -> values.
    Save {
        settings: Map<String, Value>,
    },
    IgnoreCurrentGame,
    GpuReport,
    Libraries,
    TestPopup,
    /// Check now, or open the download page when an update is waiting.
    CheckForUpdates,
    Download {
        id: String,
    },
    CancelDownload,
    OpenFolder,
    OpenModels,
    ViewLog,
    EditSettingsFile,
    Exit,
}

impl Request {
    /// `None` for anything that isn't a well-formed request.
    pub fn parse(line: &str) -> Option<Request> {
        serde_json::from_str(line.trim()).ok()
    }
}

/// Tray -> settings window.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Event {
    State(Box<View>),
    /// Reply to [`Request::Save`]: values that were rejected (the rest were applied).
    Saved {
        warnings: Vec<String>,
    },
    /// The tray menu asked for the window again: bring it to the front.
    Focus,
}

impl Event {
    pub fn to_line(&self) -> String {
        let mut s = serde_json::to_string(self).expect("event serializes");
        s.push('\n');
        s
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub version: String,
    pub tone: Tone,
    pub status_text: String,
    pub off: bool,
    pub running: bool,
    pub can_restart: bool,
    /// Process of the game the LLM is paused for, if any (offered as "not a game").
    pub game_process: Option<String>,
    /// As in settings.json (PascalCase keys); the window sends changes back the same way.
    pub settings: Settings,
    pub chat_url: String,
    pub gpu_name: Option<String>,
    pub backend: Option<String>,
    pub n_ctx: Option<u32>,
    pub update: UpdateView,
    pub models: Vec<InstalledModel>,
    pub catalog: Vec<CatalogEntry>,
    pub download: Option<DownloadView>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", content = "version", rename_all = "camelCase")]
pub enum UpdateView {
    Idle,
    Checking,
    Available(String),
    Installing(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InstalledModel {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadView {
    pub id: String,
    pub label: String,
    pub done: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FitView {
    Gpu,
    GpuAndRam,
    TooBig,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub label: String,
    pub file: String,
    pub size: u64,
    pub fit: FitView,
    pub note: String,
    pub recommended: bool,
    pub installed: bool,
    /// Download button enabled (the tray menu's rules: fits, not installed, nothing else
    /// downloading).
    pub can_download: bool,
}

/// The downloadable models as the tray's *Download a model* menu shows them.
pub fn catalog_entries(
    pc: &Machine,
    installed: &[InstalledModel],
    downloading: Option<&str>,
) -> Vec<CatalogEntry> {
    let rec = catalog::recommend(pc).map(|m| m.id);
    catalog::catalog()
        .into_iter()
        .map(|m| {
            let file = m.primary_file();
            let is_installed = installed.iter().any(|i| i.name.eq_ignore_ascii_case(&file));
            let fit = match catalog::fit(&m, pc) {
                Fit::Gpu => FitView::Gpu,
                Fit::GpuAndRam => FitView::GpuAndRam,
                Fit::TooBig => FitView::TooBig,
            };
            CatalogEntry {
                label: m.label(),
                note: catalog::fit_note(&m, pc).into(),
                recommended: rec.as_deref() == Some(m.id.as_str()),
                can_download: !is_installed && downloading.is_none() && fit != FitView::TooBig,
                installed: is_installed,
                size: m.size,
                fit,
                file,
                id: m.id,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::catalog;

    const GIB: u64 = 1024 * 1024 * 1024;

    #[test]
    fn parses_requests() {
        assert_eq!(Request::parse(r#"{"cmd":"toggle"}"#), Some(Request::Toggle));
        assert_eq!(
            Request::parse("{\"cmd\":\"download\",\"id\":\"x\"}\r\n"),
            Some(Request::Download { id: "x".into() })
        );
        let save = Request::parse(r#"{"cmd":"save","settings":{"Port":9000}}"#).unwrap();
        let Request::Save { settings } = save else {
            panic!("{save:?}")
        };
        assert_eq!(settings["Port"], 9000);
    }

    #[test]
    fn rejects_malformed_requests() {
        for bad in [
            "",
            "not json",
            "[]",
            r#"{"cmd":"format_c"}"#,
            r#"{"cmd":"download"}"#,
            r#"{"cmd":"save","settings":[]}"#,
            r#"{"type":"state"}"#,
        ] {
            assert_eq!(Request::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn events_are_single_json_lines() {
        let l = Event::Saved {
            warnings: vec!["a\nb".into()],
        }
        .to_line();
        assert!(l.ends_with('\n') && l.matches('\n').count() == 1, "{l:?}");
        let v: Value = serde_json::from_str(&l).unwrap();
        assert_eq!(v["type"], "saved");
        assert_eq!(v["data"]["warnings"][0], "a\nb");
        assert_eq!(Event::Focus.to_line(), "{\"type\":\"focus\"}\n");
    }

    fn view() -> View {
        View {
            version: "2.1.0".into(),
            tone: Tone::Running,
            status_text: "Running".into(),
            off: false,
            running: true,
            can_restart: true,
            game_process: None,
            settings: Settings::default(),
            chat_url: "http://127.0.0.1:8080".into(),
            gpu_name: Some("GPU".into()),
            backend: None,
            n_ctx: Some(65536),
            update: UpdateView::Available("2.2.0".into()),
            models: vec![],
            catalog: vec![],
            download: None,
        }
    }

    #[test]
    fn state_uses_settings_json_keys() {
        let v: Value = serde_json::from_str(&Event::State(Box::new(view())).to_line()).unwrap();
        let d = &v["data"];
        assert_eq!(d["tone"], "running");
        assert_eq!(d["statusText"], "Running");
        assert_eq!(d["nCtx"], 65536);
        assert_eq!(d["update"]["state"], "available");
        assert_eq!(d["update"]["version"], "2.2.0");
        assert_eq!(d["settings"]["StartWithWindows"], true);
        assert_eq!(d["settings"]["Context"], "auto");
        // What the window sends back parses as the same settings.
        let (s, w) = Settings::from_json(&d["settings"].to_string());
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn settings_window_covers_every_setting() {
        // A setting added to settings.rs must also get a field in the window and a type.
        let html = include_str!("../ui/index.html");
        let types = include_str!("../ui/src/types.ts");
        for k in crate::settings::KEYS {
            assert_eq!(
                html.matches(&format!("data-key=\"{k}\"")).count(),
                1,
                "ui/index.html needs one field with data-key=\"{k}\""
            );
            assert!(
                types.contains(&format!("\n  {k}: ")),
                "ui/src/types.ts: Settings needs {k}"
            );
        }
    }

    #[test]
    fn catalog_follows_the_menu_rules() {
        let pc = Machine {
            vram: 24 * GIB,
            ram: 32 * GIB,
        };
        let all = catalog();
        let first = all[0].primary_file();
        let installed = [InstalledModel {
            name: first.to_uppercase(),
            size: 1,
        }];
        let e = catalog_entries(&pc, &installed, None);
        assert_eq!(e.len(), all.len());
        assert_eq!(e.iter().filter(|x| x.recommended).count(), 1);
        assert!(e[0].installed && !e[0].can_download);
        for x in &e[1..] {
            assert!(!x.installed);
            assert_eq!(x.can_download, x.fit != FitView::TooBig, "{}", x.id);
        }
        let busy = catalog_entries(&pc, &installed, Some(&all[1].id));
        assert!(busy.iter().all(|x| !x.can_download));
    }

    #[test]
    fn nothing_fits_a_tiny_pc() {
        let e = catalog_entries(&Machine { vram: 0, ram: GIB }, &[], None);
        assert!(e
            .iter()
            .all(|x| x.fit == FitView::TooBig && !x.can_download));
        assert!(e.iter().all(|x| !x.recommended));
    }
}
