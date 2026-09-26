//! The tray's side of the settings window: starts it, keeps it fed with state and collects
//! its requests. Called from the UI thread and never blocks it: one thread writes to the
//! window and another reads from it.

use super::worker::{Snapshot, UpdateState};
use crate::control::{self, DownloadView, Event, InstalledModel, Request, UpdateView, View};
use crate::log;
use crate::state::Status;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};

struct Window {
    child: Child,
    /// Lines for the writer thread; closing it closes the window's stdin, which ends it.
    tx: Sender<String>,
}

pub struct Host {
    window: Option<Window>,
    tx_req: Sender<Request>,
    rx_req: Receiver<Request>,
}

impl Default for Host {
    fn default() -> Host {
        let (tx_req, rx_req) = mpsc::channel();
        Host {
            window: None,
            tx_req,
            rx_req,
        }
    }
}

impl Host {
    /// Opens the window, or brings it to the front if it's already open.
    pub fn open(&mut self, snap: Option<&Snapshot>) {
        if self.send(&Event::Focus) {
            return;
        }
        match self.spawn() {
            Ok(w) => {
                self.window = Some(w);
                if let Some(s) = snap {
                    self.state(s);
                }
            }
            Err(e) => log!("couldn't open the settings window: {e}"),
        }
    }

    pub fn state(&mut self, snap: &Snapshot) {
        if self.window.is_some() {
            self.send(&Event::State(Box::new(view(snap))));
        }
    }

    pub fn saved(&mut self, warnings: Vec<String>) {
        self.send(&Event::Saved { warnings });
    }

    /// Requests the window sent since the last call.
    pub fn requests(&self) -> Vec<Request> {
        self.rx_req.try_iter().collect()
    }

    /// False (and forgets the window) if it isn't open.
    fn send(&mut self, e: &Event) -> bool {
        let Some(w) = &mut self.window else {
            return false;
        };
        if matches!(w.child.try_wait(), Ok(None)) && w.tx.send(e.to_line()).is_ok() {
            return true;
        }
        self.window = None;
        false
    }

    fn spawn(&self) -> std::io::Result<Window> {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("settings-window")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let (tx, rx) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            for line in rx {
                if stdin
                    .write_all(line.as_bytes())
                    .and_then(|_| stdin.flush())
                    .is_err()
                {
                    break;
                }
            }
        });
        let tx_req = self.tx_req.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                match Request::parse(&line) {
                    Some(r) => {
                        if tx_req.send(r).is_err() {
                            break;
                        }
                    }
                    None => log!("settings window: ignored {:.200}", line),
                }
            }
        });
        log!("settings window opened");
        Ok(Window { child, tx })
    }
}

fn view(s: &Snapshot) -> View {
    let models: Vec<InstalledModel> = s
        .models
        .iter()
        .map(|(name, size)| InstalledModel {
            name: name.clone(),
            size: *size,
        })
        .collect();
    View {
        version: env!("CARGO_PKG_VERSION").into(),
        tone: s.status.tone(),
        status_text: s.status_text.clone(),
        off: s.status == Status::Off,
        running: s.status == Status::Running,
        can_restart: !matches!(s.status, Status::Off | Status::Paused(_)),
        game_process: match &s.status {
            Status::Paused(_) => s.game_process.clone(),
            _ => None,
        },
        chat_url: crate::server::chat_url(&s.settings),
        gpu_name: s.gpu_name.clone(),
        backend: s.backend.map(|b| b.label().to_string()),
        n_ctx: s.n_ctx,
        update: match &s.update {
            UpdateState::None => UpdateView::Idle,
            UpdateState::Checking => UpdateView::Checking,
            UpdateState::Available(v) => UpdateView::Available(v.clone()),
            UpdateState::Installing(v) => UpdateView::Installing(v.clone()),
        },
        catalog: control::catalog_entries(
            &s.pc,
            &models,
            s.download.as_ref().map(|d| d.id.as_str()),
        ),
        models,
        download: s.download.as_ref().map(|d| DownloadView {
            id: d.id.clone(),
            label: d.label.clone(),
            done: d.done,
            total: d.total,
        }),
        settings: s.settings.clone(),
    }
}
