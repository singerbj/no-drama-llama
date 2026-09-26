//! The worker thread: polls every few seconds (or right after a command), detects games,
//! drives the state machine, starts/stops llama-server, checks for updates. Never touches UI
//! objects; it sends [`UiMsg`]s to the tray thread instead.

use super::gpu::{self, GpuSampler};
use super::libraries::{self, Scanner};
use super::procs::Procs;
use super::{install, net, sys, updater};
use crate::detect::{self, GameHit, GpuDetector, Scan};
use crate::paths::Paths;
use crate::settings::{self, Settings};
use crate::state::{Action, Inputs, Machine, Popup, Status};
use crate::{log, server};
use std::os::windows::process::CommandExt;
use std::process::Stdio;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime};

const POLL: Duration = Duration::from_secs(5);
const RESCAN: Duration = Duration::from_secs(600);
const UPDATE_EVERY: Duration = Duration::from_secs(24 * 3600);

pub type SettingsEdit = Box<dyn FnOnce(&mut Settings) + Send>;

pub enum Cmd {
    Toggle,
    Restart,
    Edit(SettingsEdit),
    IgnoreCurrentGame,
    ShowGpuReport,
    ShowLibraries,
    SetStartWithWindows(bool),
    CheckForUpdates {
        manual: bool,
    },
    UpdateChecked {
        manual: bool,
        result: Result<Option<(crate::update::Release, semver::Version)>, String>,
    },
    UpdateApplied(Result<semver::Version, String>),
    Exit,
}

#[derive(Clone)]
pub struct Snapshot {
    pub status: Status,
    pub status_text: String,
    pub settings: Settings,
    pub models: Vec<(String, u64)>,
    pub game_process: Option<String>,
    pub start_with_windows: bool,
    pub update: UpdateState,
}

#[derive(Clone, PartialEq)]
pub enum UpdateState {
    None,
    Checking,
    Available(String),
    Installing(String),
}

pub enum UiMsg {
    State(Box<Snapshot>),
    Popup(Popup),
    /// Exit the tray; `Some(old_version)` = relaunch the freshly updated exe.
    Quit(Option<String>),
}

pub struct Worker {
    p: Paths,
    s: Settings,
    settings_mtime: Option<SystemTime>,
    machine: Machine,
    gpu: Option<GpuSampler>,
    gpu_detector: GpuDetector,
    procs: Procs,
    scanner: Scanner,
    scan: Scan,
    scan_at: Instant,
    health: ureq::Agent,
    start_with_windows: bool,
    update: UpdateState,
    last_update_check: Option<Instant>,
    tx_ui: Sender<UiMsg>,
    tx_self: Sender<Cmd>,
}

fn mtime(p: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

impl Worker {
    pub fn new(p: Paths, tx_ui: Sender<UiMsg>, tx_self: Sender<Cmd>) -> Worker {
        let (s, warnings) = Settings::load(&p.settings);
        for w in warnings {
            log!("{w}");
        }
        if !p.settings.exists() {
            let _ = s.save(&p.settings);
        }
        let mut scanner = Scanner::new();
        let scan = scanner.scan();
        log!(
            "game scan: {} library folders, {} Windows-listed game exes",
            scan.libs.len(),
            scan.exes.len()
        );
        let gpu = GpuSampler::new();
        if gpu.is_none() {
            log!("GPU counters unavailable - GPU-based detection off");
        }
        Worker {
            settings_mtime: mtime(&p.settings),
            p,
            s,
            machine: Machine::default(),
            gpu,
            gpu_detector: GpuDetector::default(),
            procs: Procs::new(),
            scanner,
            scan,
            scan_at: Instant::now(),
            health: net::health_agent(),
            start_with_windows: install::task_enabled(),
            update: UpdateState::None,
            last_update_check: None,
            tx_ui,
            tx_self,
        }
    }

    pub fn run(mut self, rx: Receiver<Cmd>) {
        let mut next = Instant::now();
        loop {
            let timeout = next.saturating_duration_since(Instant::now());
            match rx.recv_timeout(timeout) {
                Ok(Cmd::Exit) => {
                    self.stop_server("tray exited");
                    let _ = self.tx_ui.send(UiMsg::Quit(None));
                    return;
                }
                Ok(cmd) => {
                    if self.handle(cmd) {
                        return;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            self.tick();
            next = Instant::now() + POLL;
        }
    }

    fn popup(&self, title: &str, sub: impl Into<String>, tone: crate::state::Tone) {
        let _ = self.tx_ui.send(UiMsg::Popup(Popup {
            title: title.into(),
            subtitle: sub.into(),
            tone,
        }));
    }

    fn save_settings(&mut self) {
        if let Err(e) = self.s.save(&self.p.settings) {
            log!("couldn't save settings: {e}");
        }
        self.settings_mtime = mtime(&self.p.settings);
    }

    /// Returns true if the worker should stop (update installed).
    pub(crate) fn handle(&mut self, cmd: Cmd) -> bool {
        match cmd {
            Cmd::Toggle => {
                if self.p.off_flag.exists() {
                    let _ = std::fs::remove_file(&self.p.off_flag);
                    self.machine.reset_failures();
                } else {
                    let _ = std::fs::write(&self.p.off_flag, b"");
                }
            }
            Cmd::Restart => {
                self.stop_server("restart");
                self.machine.reset_failures();
            }
            Cmd::Edit(f) => {
                let old = self.s.clone();
                f(&mut self.s);
                self.save_settings();
                log!("settings changed");
                if settings::server_changed(&old, &self.s) {
                    self.stop_server("settings changed");
                    self.machine.reset_failures();
                }
            }
            Cmd::IgnoreCurrentGame => {
                if let Some(proc) = self.machine.game().and_then(|g| g.process.clone()) {
                    if !self
                        .s
                        .gpu_ignore
                        .iter()
                        .any(|x| x.eq_ignore_ascii_case(&proc))
                    {
                        self.s.gpu_ignore.push(proc.clone());
                    }
                    self.save_settings();
                    self.machine.forget_game();
                    self.gpu_detector.reset();
                    log!("added {proc} to GpuIgnore");
                }
            }
            Cmd::ShowGpuReport => {
                let samples = self.gpu.as_mut().map(|g| g.sample()).unwrap_or_default();
                self.procs.refresh();
                let exclude = self.procs.pids_of(&self.p.server_exe);
                let text = detect::gpu_report(&samples, &self.procs.by_pid(), &self.s, &exclude);
                self.show_text("gpu-usage.txt", &text);
            }
            Cmd::ShowLibraries => {
                self.scan = self.scanner.scan();
                self.scan_at = Instant::now();
                let text = libraries::report(&self.scan, &self.s.extra_games);
                self.show_text("game-libraries.txt", &text);
            }
            Cmd::SetStartWithWindows(on) => {
                if let Err(e) = install::set_task_enabled(on) {
                    log!("couldn't change the logon task: {e}");
                }
                self.start_with_windows = install::task_enabled();
            }
            Cmd::CheckForUpdates { manual } => self.check_updates(manual),
            Cmd::UpdateChecked { manual, result } => match result {
                Ok(Some((release, version))) => {
                    log!("update available: {version}");
                    if updater::can_self_update()
                        && std::env::current_exe().is_ok_and(|e| {
                            e.as_os_str()
                                .eq_ignore_ascii_case(crate::paths::installed_exe().as_os_str())
                        })
                    {
                        self.update = UpdateState::Installing(version.to_string());
                        let tx = self.tx_self.clone();
                        std::thread::spawn(move || {
                            let r = updater::apply(&release, &version)
                                .map(|_| version)
                                .map_err(|e| format!("{e:#}"));
                            let _ = tx.send(Cmd::UpdateApplied(r));
                        });
                    } else {
                        self.update = UpdateState::Available(version.to_string());
                        if manual {
                            self.popup(
                                "Update available",
                                format!("Version {version} - download it from GitHub"),
                                crate::state::Tone::Loading,
                            );
                        }
                    }
                }
                Ok(None) => {
                    self.update = UpdateState::None;
                    if manual {
                        self.popup(
                            "Up to date",
                            format!("No Drama Llama {}", env!("CARGO_PKG_VERSION")),
                            crate::state::Tone::Running,
                        );
                    }
                }
                Err(e) => {
                    self.update = UpdateState::None;
                    log!("update check failed: {e}");
                    if manual {
                        self.popup("Update check failed", e, crate::state::Tone::Error);
                    }
                }
            },
            Cmd::UpdateApplied(r) => match r {
                Ok(v) => {
                    log!("updated to {v} - restarting");
                    let _ = self
                        .tx_ui
                        .send(UiMsg::Quit(Some(env!("CARGO_PKG_VERSION").to_string())));
                    return true; // llama-server keeps running; the new version adopts it
                }
                Err(e) => {
                    log!("update failed: {e}");
                    self.update = UpdateState::None;
                    self.popup("Update failed", e, crate::state::Tone::Error);
                }
            },
            Cmd::Exit => {}
        }
        false
    }

    fn check_updates(&mut self, manual: bool) {
        if matches!(
            self.update,
            UpdateState::Checking | UpdateState::Installing(_)
        ) {
            return;
        }
        self.update = UpdateState::Checking;
        self.last_update_check = Some(Instant::now());
        let tx = self.tx_self.clone();
        std::thread::spawn(move || {
            let result = updater::check().map_err(|e| format!("{e:#}"));
            let _ = tx.send(Cmd::UpdateChecked { manual, result });
        });
    }

    fn show_text(&self, name: &str, text: &str) {
        let f = self.p.data_dir.join(name);
        if std::fs::write(&f, text).is_ok() {
            sys::open_in_notepad(&f);
        }
    }

    pub(crate) fn stop_server(&mut self, why: &str) {
        self.procs.refresh();
        let pids = self.procs.pids_of(&self.p.server_exe);
        if !pids.is_empty() {
            self.procs.kill(&pids);
            log!("stopped server ({why})");
        }
    }

    fn start_server(&mut self) -> Result<(), String> {
        let model = self.p.model(&self.s.model);
        let log_file = std::fs::File::create(&self.p.server_log)
            .map_err(|e| format!("can't write server.log: {e}"))?;
        sys::hidden(&self.p.server_exe)
            .args(server::server_args(&self.s, &model))
            .current_dir(&self.p.llama_dir)
            .stdout(Stdio::null())
            .stderr(log_file)
            .creation_flags(sys::CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("can't start llama-server: {e}"))?;
        log!(
            "started server ({}, reasoning={}, ctx={}, host={})",
            self.s.model,
            self.s.reasoning.as_str(),
            self.s.context,
            self.s.listen_host
        );
        Ok(())
    }

    /// Reload settings.json if it was edited by hand.
    fn sync_settings_file(&mut self) {
        let m = mtime(&self.p.settings);
        if m.is_none() || m == self.settings_mtime {
            return;
        }
        self.settings_mtime = m;
        let (new, warnings) = Settings::load(&self.p.settings);
        log!("settings.json changed - reloaded");
        for w in warnings {
            log!("{w}");
        }
        if settings::server_changed(&self.s, &new) {
            self.s = new;
            self.stop_server("settings changed");
            self.machine.reset_failures();
        } else {
            self.s = new;
        }
    }

    fn detect_game(&mut self, server_pids: &[u32]) -> Option<GameHit> {
        if !self.s.pause_while_gaming {
            return None;
        }
        if self.scan_at.elapsed() >= RESCAN {
            self.scan = self.scanner.scan(); // pick up new installs
            self.scan_at = Instant::now();
        }
        let mode = self.s.detection_mode;
        let procs = self.procs.list();
        let mut game = None;
        if mode.uses_launchers() {
            game = detect::find_running_game(
                &procs,
                &self.scan,
                &self.s,
                self.scanner.steam_running(),
            );
        }
        if mode.uses_gpu() {
            // sample every tick so the utilization rate stays current
            let samples = self.gpu.as_mut().map(|g| g.sample()).unwrap_or_default();
            let by_pid = procs.into_iter().map(|p| (p.pid, p)).collect();
            let g = self.gpu_detector.check(
                &samples,
                &by_pid,
                &self.s,
                server_pids,
                gpu::fullscreen_foreground_pid(),
                &self.scan,
            );
            game = game.or(g);
        }
        game
    }

    pub(crate) fn tick(&mut self) {
        self.sync_settings_file();
        self.procs.refresh();
        let server_pids = self.procs.pids_of(&self.p.server_exe);
        let game = self.detect_game(&server_pids);
        let running = !server_pids.is_empty();
        let inputs = Inputs {
            now: Instant::now(),
            off: self.p.off_flag.exists(),
            game,
            resume_after: Duration::from_secs(self.s.resume_after_sec.into()),
            server_running: running,
            server_ready: running && net::is_healthy(&self.health, &server::health_url(&self.s)),
            model_exists: self.p.model(&self.s.model).exists(),
            server_exe_exists: self.p.server_exe.exists(),
            model_name: self.s.model.clone(),
        };
        let mut out = self.machine.step(&inputs);
        for a in std::mem::take(&mut out.actions) {
            match a {
                Action::StopServer(why) => self.stop_server(&why),
                Action::StartServer => {
                    if let Err(e) = self.start_server() {
                        log!("{e}");
                        self.machine.start_failed(e);
                        let retry = self.machine.step(&inputs);
                        out.popup = retry.popup.or(out.popup);
                        out.logs.extend(retry.logs);
                    }
                }
            }
        }
        for l in &out.logs {
            log!("{l}");
        }
        if let (Some(p), true) = (out.popup, self.s.popups) {
            let _ = self.tx_ui.send(UiMsg::Popup(p));
        }

        if self.s.auto_update
            && self
                .last_update_check
                .is_none_or(|t| t.elapsed() >= UPDATE_EVERY)
        {
            self.check_updates(false);
        }
        self.send_snapshot();
    }

    fn send_snapshot(&self) {
        let status = self.machine.status().clone();
        let snap = Snapshot {
            status_text: status.text(&crate::state::model_short(&self.s.model)),
            status,
            settings: self.s.clone(),
            models: self.p.list_models(),
            game_process: self.machine.game().and_then(|g| g.process.clone()),
            start_with_windows: self.start_with_windows,
            update: self.update.clone(),
        };
        let _ = self.tx_ui.send(UiMsg::State(Box::new(snap)));
    }
}
