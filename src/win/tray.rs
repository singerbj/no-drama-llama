//! The tray app's UI thread: tray icon + menu, Ctrl+Alt+L hotkey, on-screen popups. All
//! real work happens on the worker thread; this thread only renders snapshots and forwards
//! clicks, so the menu never freezes.

use super::settings_host::Host;
use super::worker::DownloadInfo;
use super::worker::{Cmd, Snapshot, UiMsg, UpdateState, Worker};
use super::{osd, sys};
use crate::catalog;
use crate::control::Request;
use crate::laya;
use crate::log;
use crate::paths::{self, Paths};
use crate::settings::{DetectionMode, PopupPosition, Reasoning, Settings};
use crate::state::{Status, Tone};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use tray_icon::menu::{
    CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
};
use tray_icon::{Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, SetTimer, TranslateMessage, MSG, WM_TIMER,
};

type Edit = Arc<dyn Fn(&mut Settings) + Send + Sync>;
type Pred = Box<dyn Fn(&Snapshot) -> bool>;

#[derive(Clone)]
enum Act {
    Toggle,
    OpenChat,
    Restart,
    Edit(Edit),
    IgnoreGame,
    GpuReport,
    Libraries,
    TestPopup,
    OpenSettings,
    CheckUpdates,
    EditSettings,
    OpenFolder,
    ViewLog,
    Download(String),
    CancelDownload,
    OpenModels,
    RestartLaya,
    UpdateLaya,
    ViewLayaLog,
    Exit,
}

fn edit(f: impl Fn(&mut Settings) + Send + Sync + 'static) -> Act {
    Act::Edit(Arc::new(f))
}

pub(crate) const ICON_SIZE: usize = 32;

/// The logo (icons/logo.svg) at 32x32, rendered by scripts/icons.ts: everything but its
/// `.status` parts (the circle and inner ears) as RGBA, and their coverage, which is filled
/// with the status color.
const LOGO: &[u8; ICON_SIZE * ICON_SIZE * 4] = include_bytes!("../../icons/tray-logo.rgba");
const DISC: &[u8; ICON_SIZE * ICON_SIZE] = include_bytes!("../../icons/tray-disc.a");

/// 32x32 tray icon (RGBA): the logo with its circle in the status color.
pub(crate) fn icon_rgba(t: Tone) -> Vec<u8> {
    let (r, g, b) = osd::tone_rgb(t);
    let mut rgba = Vec::with_capacity(LOGO.len());
    for (&disc, logo) in DISC.iter().zip(LOGO.as_chunks::<4>().0) {
        let mut px = [r, g, b, disc];
        over(&mut px, (logo[0], logo[1], logo[2]), logo[3] as f32 / 255.0);
        rgba.extend_from_slice(&px);
    }
    rgba
}

/// Paints `rgb` at coverage `a` over a straight-alpha RGBA pixel.
fn over(px: &mut [u8], (r, g, b): (u8, u8, u8), a: f32) {
    let da = px[3] as f32 / 255.0;
    let oa = a + da * (1.0 - a);
    if oa == 0.0 {
        return;
    }
    for (c, s) in px[..3].iter_mut().zip([r, g, b]) {
        *c = ((s as f32 * a + *c as f32 * da * (1.0 - a)) / oa).round() as u8;
    }
    px[3] = (oa * 255.0).round() as u8;
}

fn status_icon(t: Tone) -> Icon {
    Icon::from_rgba(icon_rgba(t), ICON_SIZE as u32, ICON_SIZE as u32).expect("valid icon")
}

struct Ui {
    menu: Menu,
    status: MenuItem,
    toggle: MenuItem,
    open: MenuItem,
    restart: MenuItem,
    models: Submenu,
    model_key: String,
    model_checks: Vec<(CheckMenuItem, String)>,
    model_items: Vec<MenuItem>,
    download_item: Option<MenuItem>,
    gpu_info: MenuItem,
    not_game: MenuItem,
    update: MenuItem,
    laya_status: MenuItem,
    laya_update: MenuItem,
    laya_only: Vec<Submenu>,
    laya_restart: MenuItem,
    gpu_only: Vec<Submenu>,
    checks: Vec<(CheckMenuItem, Pred)>,
    actions: HashMap<MenuId, Act>,
    snap: Option<Snapshot>,
    icons: HashMap<Tone, Icon>,
    tone: Option<Tone>,
    settings_window: Host,
}

impl Ui {
    fn item(&mut self, parent: &dyn Append, text: &str, act: Act) -> MenuItem {
        let i = MenuItem::new(text, true, None);
        self.actions.insert(i.id().clone(), act);
        parent.add(&i);
        i
    }

    fn check(
        &mut self,
        parent: &dyn Append,
        text: &str,
        act: Act,
        pred: impl Fn(&Snapshot) -> bool + 'static,
    ) {
        let i = CheckMenuItem::new(text, true, false, None);
        self.actions.insert(i.id().clone(), act);
        parent.add(&i);
        self.checks.push((i, Box::new(pred)));
    }
}

/// `Menu` and `Submenu` both take items.
trait Append {
    fn add(&self, item: &dyn IsMenuItem);
}
impl Append for Menu {
    fn add(&self, item: &dyn IsMenuItem) {
        let _ = self.append(item);
    }
}
impl Append for Submenu {
    fn add(&self, item: &dyn IsMenuItem) {
        let _ = self.append(item);
    }
}

fn sep(parent: &dyn Append) {
    parent.add(&PredefinedMenuItem::separator());
}

fn build() -> Ui {
    let menu = Menu::new();
    let mut ui = Ui {
        status: MenuItem::new("Starting...", false, None),
        toggle: MenuItem::new("Turn off", true, None),
        open: MenuItem::new("Open chat", true, None),
        restart: MenuItem::new("Restart server", true, None),
        models: Submenu::new("Model", true),
        model_key: String::new(),
        model_checks: Vec::new(),
        model_items: Vec::new(),
        download_item: None,
        gpu_info: MenuItem::new("GPU: detecting...", false, None),
        not_game: MenuItem::new("Not a game - ignore this app", false, None),
        update: MenuItem::new("Check for updates", true, None),
        laya_status: MenuItem::new("Laya: off", false, None),
        laya_update: MenuItem::new("Check for Ollaya updates", true, None),
        laya_only: Vec::new(),
        laya_restart: MenuItem::new("Restart Laya", true, None),
        gpu_only: Vec::new(),
        checks: Vec::new(),
        actions: HashMap::new(),
        snap: None,
        icons: [
            Tone::Running,
            Tone::Loading,
            Tone::Paused,
            Tone::Off,
            Tone::Error,
        ]
        .into_iter()
        .map(|t| (t, status_icon(t)))
        .collect(),
        tone: None,
        settings_window: Host::default(),
        menu: menu.clone(),
    };
    menu.add(&ui.status);
    sep(&menu);
    for (i, a) in [
        (ui.toggle.clone(), Act::Toggle),
        (ui.open.clone(), Act::OpenChat),
        (ui.restart.clone(), Act::Restart),
    ] {
        ui.actions.insert(i.id().clone(), a);
        menu.add(&i);
    }
    sep(&menu);
    ui.item(&menu, "Open settings window...", Act::OpenSettings);

    let settings = Submenu::new("Settings", true);
    menu.add(&settings);
    settings.add(&ui.gpu_info.clone());
    sep(&settings);
    settings.add(&ui.models);

    let reasoning = Submenu::new("Reasoning", true);
    settings.add(&reasoning);
    for r in Reasoning::ALL {
        ui.check(
            &reasoning,
            r.as_str(),
            edit(move |s| s.reasoning = r),
            move |n| n.settings.reasoning == r,
        );
    }
    let ctx = Submenu::new("Context length", true);
    settings.add(&ctx);
    ui.check(
        &ctx,
        "Auto (largest that fits your GPU)",
        edit(|s| s.context = crate::settings::CONTEXT_AUTO),
        |n| n.settings.context == crate::settings::CONTEXT_AUTO,
    );
    sep(&ctx);
    for c in [8192u32, 16384, 32768, 65536, 131072] {
        ui.check(
            &ctx,
            &format!("{}K tokens", c / 1024),
            edit(move |s| s.context = c),
            move |n| n.settings.context == c,
        );
    }
    let access = Submenu::new("Access", true);
    settings.add(&access);
    for (label, host) in [
        ("This PC only", "127.0.0.1"),
        ("Devices on my network", "0.0.0.0"),
    ] {
        ui.check(
            &access,
            label,
            edit(move |s| s.listen_host = host.into()),
            move |n| n.settings.listen_host == host,
        );
    }
    let lm = Submenu::new("Laya (decision model)", true);
    settings.add(&lm);
    lm.add(&ui.laya_status.clone());
    sep(&lm);
    ui.check(
        &lm,
        "Run Laya alongside the LLM",
        edit(|s| s.run_laya = !s.run_laya),
        |n| n.settings.run_laya,
    );
    let laya_model = Submenu::new("Model", true);
    lm.add(&laya_model);
    for (m, label) in laya::MODELS {
        ui.check(
            &laya_model,
            label,
            edit(move |s| s.laya_model = m.into()),
            move |n| n.settings.laya_model == m,
        );
    }
    let device = Submenu::new("Run on", true);
    lm.add(&device);
    for (d, label) in laya::Device::ALL {
        ui.check(&device, label, edit(move |s| s.laya_device = d), move |n| {
            n.settings.laya_device == d
        });
    }
    let keep = Submenu::new("Keep the model loaded", true);
    lm.add(&keep);
    for (v, label) in laya::KEEP_ALIVE_PRESETS {
        ui.check(
            &keep,
            label,
            edit(move |s| s.laya_keep_alive = v.into()),
            move |n| n.settings.laya_keep_alive == v,
        );
    }
    ui.laya_only = vec![laya_model, device, keep];
    sep(&lm);
    ui.actions
        .insert(ui.laya_restart.id().clone(), Act::RestartLaya);
    lm.add(&ui.laya_restart.clone());
    ui.actions
        .insert(ui.laya_update.id().clone(), Act::UpdateLaya);
    lm.add(&ui.laya_update.clone());
    ui.item(&lm, "View Laya log", Act::ViewLayaLog);
    sep(&settings);

    let games = Submenu::new("Game detection", true);
    settings.add(&games);
    ui.check(
        &games,
        "Pause while gaming",
        edit(|s| s.pause_while_gaming = !s.pause_while_gaming),
        |n| n.settings.pause_while_gaming,
    );
    sep(&games);
    let mode = Submenu::new("Detect games by", true);
    games.add(&mode);
    for (label, m) in [
        ("GPU usage + launchers (recommended)", DetectionMode::Both),
        ("GPU usage only", DetectionMode::Gpu),
        ("Launchers only", DetectionMode::Launchers),
    ] {
        ui.check(
            &mode,
            label,
            edit(move |s| s.detection_mode = m),
            move |n| n.settings.detection_mode == m,
        );
    }
    let vram = Submenu::new("GPU: VRAM threshold", true);
    games.add(&vram);
    for v in [1.0f64, 1.5, 2.0, 3.0, 4.0] {
        ui.check(
            &vram,
            &format!("Another app uses {v} GB+"),
            edit(move |s| s.gpu_vram_gb = v),
            move |n| n.settings.gpu_vram_gb == v,
        );
    }
    let load = Submenu::new("GPU: 3D load threshold", true);
    games.add(&load);
    for v in [20.0f64, 30.0, 50.0, 70.0] {
        ui.check(
            &load,
            &format!("Another app uses {v}%+"),
            edit(move |s| s.gpu_load_pct = v),
            move |n| n.settings.gpu_load_pct == v,
        );
    }
    ui.gpu_only = vec![vram, load];
    let resume = Submenu::new("Resume after game closes", true);
    games.add(&resume);
    for v in [15u32, 30, 60, 120, 300] {
        let label = if v < 60 {
            format!("{v} seconds")
        } else {
            format!("{} min", v / 60)
        };
        ui.check(
            &resume,
            &label,
            edit(move |s| s.resume_after_sec = v),
            move |n| n.settings.resume_after_sec == v,
        );
    }
    ui.actions.insert(ui.not_game.id().clone(), Act::IgnoreGame);
    games.add(&ui.not_game.clone());
    ui.item(&games, "Show GPU usage now", Act::GpuReport);
    sep(&games);
    ui.check(
        &games,
        "Count emulators as games",
        edit(|s| s.detect_emulators = !s.detect_emulators),
        |n| n.settings.detect_emulators,
    );
    ui.check(
        &games,
        "Use Windows' game list (Game Bar)",
        edit(|s| s.use_windows_game_list = !s.use_windows_game_list),
        |n| n.settings.use_windows_game_list,
    );
    sep(&games);
    ui.item(&games, "Rescan and show detected libraries", Act::Libraries);

    let popups = Submenu::new("On-screen popups", true);
    settings.add(&popups);
    ui.check(
        &popups,
        "Show popups",
        edit(|s| s.popups = !s.popups),
        |n| n.settings.popups,
    );
    sep(&popups);
    for (p, label) in PopupPosition::ALL {
        ui.check(
            &popups,
            label,
            edit(move |s| s.popup_position = p),
            move |n| n.settings.popup_position == p,
        );
    }
    sep(&popups);
    ui.item(&popups, "Test popup", Act::TestPopup);

    ui.check(
        &settings,
        "Start with Windows",
        edit(|s| s.start_with_windows = !s.start_with_windows),
        |n| n.settings.start_with_windows,
    );
    ui.check(
        &settings,
        "Update automatically",
        edit(|s| s.auto_update = !s.auto_update),
        |n| n.settings.auto_update,
    );
    sep(&settings);
    ui.item(&settings, "Edit settings file", Act::EditSettings);

    ui.actions.insert(ui.update.id().clone(), Act::CheckUpdates);
    menu.add(&ui.update.clone());
    ui.item(&menu, "Open folder", Act::OpenFolder);
    ui.item(&menu, "View log", Act::ViewLog);
    sep(&menu);
    ui.item(&menu, "Exit (stops the model)", Act::Exit);
    ui
}

impl Ui {
    fn render(&mut self, tray: &TrayIcon, snap: Snapshot) {
        let tone = snap.status.tone();
        if self.tone != Some(tone) {
            let _ = tray.set_icon(self.icons.get(&tone).cloned());
            self.tone = Some(tone);
        }
        let dl = snap
            .download
            .as_ref()
            .map(|d| format!(" · downloading {:.0}%", percent(d)))
            .unwrap_or_default();
        let laya_tip = if snap.settings.run_laya {
            format!(" · Laya: {}", snap.laya.status_text)
        } else {
            String::new()
        };
        let tip: String = format!("{}: {}{dl}{laya_tip}", paths::APP_NAME, snap.status_text)
            .chars()
            .take(127)
            .collect();
        let _ = tray.set_tooltip(Some(tip));
        self.status.set_text(&snap.status_text);
        self.toggle.set_text(if snap.status == Status::Off {
            "Turn on"
        } else {
            "Turn off"
        });
        self.open.set_enabled(snap.status == Status::Running);
        self.restart
            .set_enabled(!matches!(snap.status, Status::Off | Status::Paused(_)));
        match (&snap.status, &snap.game_process) {
            (Status::Paused(_), Some(p)) => {
                self.not_game.set_text(format!("Not a game - ignore {p}"));
                self.not_game.set_enabled(true);
            }
            _ => {
                self.not_game.set_text("Not a game - ignore this app");
                self.not_game.set_enabled(false);
            }
        }
        let (text, enabled) = match &snap.update {
            UpdateState::None => ("Check for updates".to_string(), true),
            UpdateState::Checking => ("Checking for updates...".to_string(), false),
            UpdateState::Available(v) => {
                (format!("Update available: {v} (open download page)"), true)
            }
            UpdateState::Installing(v) => (format!("Installing update {v}..."), false),
        };
        self.update.set_text(text);
        self.update.set_enabled(enabled);
        for g in &self.gpu_only {
            g.set_enabled(snap.settings.detection_mode.uses_gpu());
        }
        let l = &snap.laya;
        let job = l
            .job
            .as_ref()
            .filter(|d| d.total > 0)
            .map(|d| format!(" {:.0}%", percent(d)))
            .unwrap_or_default();
        self.laya_status
            .set_text(format!("Laya: {}{job}", l.status_text));
        for m in &self.laya_only {
            m.set_enabled(snap.settings.run_laya);
        }
        self.laya_restart.set_enabled(snap.settings.run_laya);
        let (text, enabled) = match (&l.update, l.checking, &l.version) {
            (_, true, _) => ("Checking for Ollaya updates...".to_string(), false),
            (Some(v), _, _) => (format!("Install Ollaya {v}"), true),
            (None, _, Some(v)) => (format!("Check for Ollaya updates (have {v})"), true),
            (None, _, None) => ("Check for Ollaya updates".to_string(), false),
        };
        self.laya_update.set_text(text);
        self.laya_update
            .set_enabled(enabled && snap.settings.run_laya);
        for (item, pred) in &self.checks {
            item.set_checked(pred(&snap));
        }
        let key = format!(
            "{:?}|{:?}|{:?}",
            snap.models,
            snap.pc,
            snap.download.as_ref().map(|d| &d.id)
        );
        if key != self.model_key {
            self.rebuild_models(&snap);
            self.model_key = key;
        }
        for (item, name) in &self.model_checks {
            item.set_checked(snap.settings.model == *name);
        }
        if let (Some(item), Some(d)) = (&self.download_item, &snap.download) {
            item.set_text(format!(
                "Downloading {}  {:.0}%  -  click to cancel",
                d.label,
                percent(d)
            ));
        }
        let gpu = snap
            .gpu_name
            .as_deref()
            .unwrap_or("not detected (runs on the CPU)");
        let backend = snap
            .backend
            .map(|b| format!("  ·  llama.cpp {}", b.label()))
            .unwrap_or_default();
        self.gpu_info.set_text(format!("GPU: {gpu}{backend}"));
        self.settings_window.state(&snap);
        self.snap = Some(snap);
    }

    fn rebuild_models(&mut self, snap: &Snapshot) {
        while self.models.remove_at(0).is_some() {}
        for (item, _) in self.model_checks.drain(..) {
            self.actions.remove(item.id());
        }
        for item in self.model_items.drain(..) {
            self.actions.remove(item.id());
        }
        self.download_item = None;
        for (name, size) in &snap.models {
            let n = name.clone();
            let i = CheckMenuItem::new(
                format!("{name}  ({:.1} GB)", *size as f64 / 1e9),
                true,
                false,
                None,
            );
            self.actions
                .insert(i.id().clone(), edit(move |s| s.model = n.clone()));
            let _ = self.models.append(&i);
            self.model_checks.push((i, name.clone()));
        }
        if snap.models.is_empty() {
            let _ = self.models.append(&MenuItem::new(
                "(no models yet - download one below)",
                false,
                None,
            ));
        }
        sep(&self.models);
        let dl = Submenu::new("Download a model", true);
        let rec = catalog::recommend(&snap.pc).map(|m| m.id);
        for m in catalog::catalog() {
            let installed = snap
                .models
                .iter()
                .any(|(n, _)| n.eq_ignore_ascii_case(&m.primary_file()));
            let fit = catalog::fit(&m, &snap.pc);
            let star = if rec.as_deref() == Some(m.id.as_str()) {
                "   ★ recommended"
            } else {
                ""
            };
            let mut text = format!(
                "{}  -  {}{star}",
                m.label(),
                catalog::fit_note(&m, &snap.pc)
            );
            let (act, enabled) = match &snap.download {
                Some(d) if d.id == m.id => {
                    text = format!(
                        "Downloading {}  {:.0}%  -  click to cancel",
                        d.label,
                        percent(d)
                    );
                    (Act::CancelDownload, true)
                }
                Some(_) => (Act::Download(m.id.clone()), false),
                None if installed => {
                    text += "   (installed)";
                    (Act::Download(m.id.clone()), false)
                }
                None => (Act::Download(m.id.clone()), fit != catalog::Fit::TooBig),
            };
            let item = MenuItem::new(&text, enabled, None);
            if matches!(act, Act::CancelDownload) {
                self.download_item = Some(item.clone());
            }
            self.actions.insert(item.id().clone(), act);
            let _ = dl.append(&item);
            self.model_items.push(item);
        }
        let _ = self.models.append(&dl);
        let open = MenuItem::new("Open models folder", true, None);
        self.actions.insert(open.id().clone(), Act::OpenModels);
        let _ = self.models.append(&open);
        self.model_items.push(open);
    }

    fn popup(&self, title: &str, sub: &str, tone: Tone) {
        let pos = self
            .snap
            .as_ref()
            .map(|s| s.settings.popup_position)
            .unwrap_or(PopupPosition::TopCenter);
        osd::show(title, sub, tone, pos);
    }

    fn chat_url(&self) -> Option<String> {
        self.snap
            .as_ref()
            .map(|s| crate::server::chat_url(&s.settings))
    }

    fn on_menu(&mut self, id: &MenuId, p: &Paths, tx: &Sender<Cmd>) {
        if let Some(act) = self.actions.get(id).cloned() {
            self.act(act, p, tx);
        }
    }

    /// A request from the settings window: the same actions as the menu.
    fn on_request(&mut self, r: Request, p: &Paths, tx: &Sender<Cmd>) {
        let act = match r {
            Request::Toggle => Act::Toggle,
            Request::Restart => Act::Restart,
            Request::OpenChat => Act::OpenChat,
            Request::IgnoreCurrentGame => Act::IgnoreGame,
            Request::GpuReport => Act::GpuReport,
            Request::Libraries => Act::Libraries,
            Request::TestPopup => Act::TestPopup,
            Request::CheckForUpdates => Act::CheckUpdates,
            Request::CancelDownload => Act::CancelDownload,
            Request::OpenFolder => Act::OpenFolder,
            Request::OpenModels => Act::OpenModels,
            Request::ViewLog => Act::ViewLog,
            Request::EditSettingsFile => Act::EditSettings,
            Request::Exit => Act::Exit,
            Request::RestartLaya => Act::RestartLaya,
            Request::UpdateLaya => Act::UpdateLaya,
            Request::ViewLayaLog => Act::ViewLayaLog,
            Request::Download { id } => {
                // The window has already asked the user.
                if catalog::find(&id).is_some() {
                    let _ = tx.send(Cmd::DownloadModel { id, switch: true });
                }
                return;
            }
            Request::Save { settings } => {
                // Which values are invalid doesn't depend on the current settings.
                let warnings = self
                    .snap
                    .as_ref()
                    .map(|s| s.settings.with_patch(&settings).1)
                    .unwrap_or_default();
                let _ = tx.send(Cmd::Edit(Box::new(move |s| {
                    *s = s.with_patch(&settings).0;
                })));
                self.settings_window.saved(warnings);
                return;
            }
        };
        self.act(act, p, tx);
    }

    fn act(&mut self, act: Act, p: &Paths, tx: &Sender<Cmd>) {
        let send = |c: Cmd| {
            let _ = tx.send(c);
        };
        match act {
            Act::Toggle => send(Cmd::Toggle),
            Act::Restart => send(Cmd::Restart),
            Act::Edit(f) => send(Cmd::Edit(Box::new(move |s| f(s)))),
            Act::IgnoreGame => send(Cmd::IgnoreCurrentGame),
            Act::GpuReport => send(Cmd::ShowGpuReport),
            Act::Libraries => send(Cmd::ShowLibraries),
            Act::OpenSettings => self.settings_window.open(self.snap.as_ref()),
            Act::CheckUpdates => match self.snap.as_ref().map(|s| &s.update) {
                Some(UpdateState::Available(_)) => sys::open_unelevated(Path::new(&format!(
                    "https://github.com/{}/releases/latest",
                    paths::REPO
                ))),
                _ => send(Cmd::CheckForUpdates { manual: true }),
            },
            Act::Exit => send(Cmd::Exit),
            Act::OpenChat => {
                if let Some(u) = self.chat_url() {
                    sys::open_unelevated(Path::new(&u));
                }
            }
            Act::TestPopup => osd::show(
                "LLM paused",
                "Example Game (Steam) detected  -  GPU freed",
                Tone::Paused,
                self.snap
                    .as_ref()
                    .map(|s| s.settings.popup_position)
                    .unwrap_or(PopupPosition::TopCenter),
            ),
            Act::EditSettings => sys::open_in_notepad(&p.settings),
            Act::OpenFolder => sys::open_unelevated(&p.root),
            Act::ViewLog => sys::open_in_notepad(&p.log),
            Act::OpenModels => sys::open_unelevated(&p.models_dir),
            Act::CancelDownload => send(Cmd::CancelDownload),
            Act::RestartLaya => send(Cmd::RestartLaya),
            Act::UpdateLaya => send(Cmd::CheckLayaUpdate { manual: true }),
            Act::ViewLayaLog => sys::open_in_notepad(&p.laya_log),
            Act::Download(id) => {
                if let Some(m) = catalog::find(&id) {
                    let q = format!(
                        "Download {}?\n\n{:.1} GB from Hugging Face. It keeps running in the background and switches over when it's done ({}).",
                        m.label(),
                        m.size as f64 / 1e9,
                        self.snap.as_ref().map(|s| catalog::fit_note(&m, &s.pc)).unwrap_or("")
                    );
                    if sys::ask(&q) {
                        send(Cmd::DownloadModel { id, switch: true });
                    }
                }
            }
        }
    }
}

fn percent(d: &DownloadInfo) -> f64 {
    if d.total == 0 {
        0.0
    } else {
        d.done as f64 * 100.0 / d.total as f64
    }
}

fn lock_if_auto_signed_in_at_boot() {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let auto = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon")
        .and_then(|k| k.get_value::<String, _>("AutoAdminLogon"))
        .is_ok_and(|v| v == "1");
    let uptime_ms = unsafe { windows::Win32::System::SystemInformation::GetTickCount64() };
    // After an automatic sign-in right after boot (power loss, update restart), lock the
    // screen. Only with auto sign-in: someone who just typed their password stays signed in.
    if auto && uptime_ms < 3 * 60 * 1000 {
        log!(
            "automatic sign-in at boot (uptime {}s) - locking screen",
            uptime_ms / 1000
        );
        unsafe {
            let _ = windows::Win32::System::Shutdown::LockWorkStation();
        }
    }
}

pub fn run(p: Paths, after_update: Option<String>) -> i32 {
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let wait = if after_update.is_some() { 20_000 } else { 0 };
    let _instance = match sys::SingleInstance::acquire(wait) {
        Ok(Some(i)) => i,
        Ok(None) => return 0, // already running
        Err(e) => {
            sys::message(&format!("Couldn't start: {e}"), true);
            return 1;
        }
    };
    let _ = std::fs::create_dir_all(&p.data_dir);
    log::init(&p.log);
    log!("tray started (v{})", env!("CARGO_PKG_VERSION"));
    super::updater::remove_leftovers();
    lock_if_auto_signed_in_at_boot();

    let (tx_cmd, rx_cmd) = mpsc::channel::<Cmd>();
    let (tx_ui, rx_ui) = mpsc::channel::<UiMsg>();
    {
        let (p, tx_self) = (p.clone(), tx_cmd.clone());
        std::thread::Builder::new()
            .name("worker".into())
            .spawn(move || Worker::new(p, tx_ui, tx_self).run(rx_cmd))
            .expect("spawn worker");
    }

    let mut ui = build();
    let tray = match TrayIconBuilder::new()
        .with_menu(Box::new(ui.menu.clone()))
        .with_tooltip(paths::APP_NAME)
        .with_icon(ui.icons[&Tone::Loading].clone())
        .build()
    {
        Ok(t) => t,
        Err(e) => {
            log!("couldn't create the tray icon: {e}");
            return 1;
        }
    };

    let hotkeys = GlobalHotKeyManager::new().ok();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL);
    if let Some(Err(e)) = hotkeys.as_ref().map(|m| m.register(hotkey)) {
        log!("couldn't register Ctrl+Alt+L: {e}");
    }

    if let Some(old) = &after_update {
        ui.popup(
            "No Drama Llama updated",
            &format!("v{old} -> v{}", env!("CARGO_PKG_VERSION")),
            Tone::Running,
        );
    }

    let mut relaunch: Option<String> = None;
    unsafe {
        SetTimer(None, 0, 100, None);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_TIMER && msg.hwnd.0.is_null() {
                if let Some(r) = pump(&mut ui, &tray, &p, &tx_cmd, &rx_ui, hotkey.id()) {
                    relaunch = r;
                    PostQuitMessage(0);
                }
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    drop(tray);
    log!("tray exited");
    if let Some(old) = relaunch {
        drop(_instance); // let the new version take over
        let _ = std::process::Command::new(paths::installed_exe())
            .args(["run", "--after-update", &old])
            .spawn();
    }
    0
}

/// Handles queued events. `Some(relaunch)` means quit.
fn pump(
    ui: &mut Ui,
    tray: &TrayIcon,
    p: &Paths,
    tx: &Sender<Cmd>,
    rx: &Receiver<UiMsg>,
    hotkey_id: u32,
) -> Option<Option<String>> {
    while let Ok(e) = MenuEvent::receiver().try_recv() {
        ui.on_menu(&e.id, p, tx);
    }
    for r in ui.settings_window.requests() {
        ui.on_request(r, p, tx);
    }
    while let Ok(e) = TrayIconEvent::receiver().try_recv() {
        if let TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } = e
        {
            if ui
                .snap
                .as_ref()
                .is_some_and(|s| s.status == Status::Running)
            {
                if let Some(u) = ui.chat_url() {
                    sys::open_unelevated(Path::new(&u));
                }
            }
        }
    }
    while let Ok(e) = GlobalHotKeyEvent::receiver().try_recv() {
        if e.id == hotkey_id && e.state == HotKeyState::Pressed {
            let _ = tx.send(Cmd::Toggle);
        }
    }
    while let Ok(m) = rx.try_recv() {
        match m {
            UiMsg::State(s) => ui.render(tray, *s),
            UiMsg::Popup(pp) => ui.popup(&pp.title, &pp.subtitle, pp.tone),
            UiMsg::Quit(r) => return Some(r),
        }
    }
    None
}
