//! The setup wizard: a Tauri window (`no-drama-llama.exe setup`, elevated) that shows
//! `ui/setup.html`. The page asks for the survey, validates the user's choices here, then
//! starts the install (or, with `--uninstall`, the uninstall) and follows its progress events.
//! The install runs on its own thread in this process, through the same code as the
//! `install` command, reporting to the page instead of a console.

use super::{install, survey, sys};
use crate::installer::{
    self, Finished, Plan, Progress, Reporter, StepId, StepInfo, Survey, UninstallPlan,
    UninstallSurvey,
};
use crate::paths::{self, Paths, APP_NAME};
use crate::settings::Settings;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const LABEL: &str = "setup";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
enum Mode {
    Install,
    Uninstall,
}

struct Wizard {
    mode: Mode,
    survey: Mutex<Option<Survey>>,
    /// An install or uninstall is running
    busy: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
    /// The uninstall finished: remove the install folder once we exit
    uninstalled: AtomicBool,
    /// Where WebView2 keeps this window's profile (deleted after exit)
    webview_dir: PathBuf,
}

/// Sends progress to the page and keeps a copy for `setup.log`.
struct Page {
    app: AppHandle,
    cancel: Arc<AtomicBool>,
    last_bytes: Instant,
    log: Vec<String>,
}

impl Page {
    fn emit(&self, p: Progress) {
        let _ = self.app.emit_to(LABEL, "progress", p);
    }
}

impl Reporter for Page {
    fn step(&mut self, id: StepId) {
        self.log.push(format!("== {}", id.label()));
        self.emit(Progress::Step { id });
    }

    fn info(&mut self, text: &str) {
        self.log.push(format!("  {text}"));
        self.emit(Progress::Log { text: text.into() });
    }

    fn progress(&mut self, done: u64, total: u64) {
        // A few updates a second are plenty for a progress bar.
        if done < total && self.last_bytes.elapsed() < Duration::from_millis(150) {
            return;
        }
        self.last_bytes = Instant::now();
        self.emit(Progress::Bytes { done, total });
    }

    fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }
}

#[tauri::command]
fn mode(w: tauri::State<'_, Wizard>) -> Mode {
    w.mode
}

/// Probes the PC and the download sites (a few seconds). Called again by "Check again".
#[tauri::command(async)]
fn survey(w: tauri::State<'_, Wizard>) -> Survey {
    let s = survey::survey();
    *w.survey.lock().unwrap() = Some(s.clone());
    s
}

#[tauri::command(async)]
fn uninstall_survey() -> UninstallSurvey {
    survey::uninstall_survey()
}

/// Why the plan can't be installed (empty = fine), and the steps it would run.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Review {
    problems: Vec<String>,
    steps: Vec<StepInfo>,
    sizes: installer::Sizes,
}

#[tauri::command]
fn review(plan: Plan, w: tauri::State<'_, Wizard>) -> Result<Review, String> {
    let guard = w.survey.lock().unwrap();
    let s = guard.as_ref().ok_or("the system check hasn't run yet")?;
    Ok(Review {
        problems: installer::validate(&plan, s),
        steps: installer::planned_steps(&plan, s),
        sizes: installer::sizes(&plan, s),
    })
}

fn finish(page: &mut Page, result: anyhow::Result<Option<String>>) {
    let cancelled = page.cancel.load(Ordering::Relaxed);
    let (ok, error, chat_url) = match result {
        Ok(url) => (true, None, url),
        Err(_) if cancelled => (false, None, None),
        Err(e) => {
            page.log.push(format!("Error: {e:#}"));
            (false, Some(format!("{e:#}")), None)
        }
    };
    // Kept for support questions, where the tray's own log lives (admin-only by now).
    let p = Paths::system();
    let log_file = (p.data_dir.exists() && !sys::is_reparse_point(&p.data_dir))
        .then(|| p.data_dir.join("setup.log"))
        .filter(|f| std::fs::write(f, page.log.join("\r\n") + "\r\n").is_ok())
        .map(|f| f.display().to_string());
    page.emit(Progress::Finished(Finished {
        ok,
        cancelled: cancelled && !ok,
        error,
        chat_url,
        log_file,
    }));
}

/// Runs `job` on its own thread, reporting to the page. Refused if one is already running.
fn start_job(
    app: AppHandle,
    w: &Wizard,
    job: impl FnOnce(&mut Page) -> anyhow::Result<Option<String>> + Send + 'static,
) -> Result<(), String> {
    if w.busy.swap(true, Ordering::SeqCst) {
        return Err("already running".into());
    }
    w.cancel.store(false, Ordering::SeqCst);
    let busy = w.busy.clone();
    let mut page = Page {
        app,
        cancel: w.cancel.clone(),
        last_bytes: Instant::now(),
        log: vec![format!("{APP_NAME} {} setup", env!("CARGO_PKG_VERSION"))],
    };
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(&mut page)))
            .unwrap_or_else(|_| Err(anyhow::anyhow!("the installer crashed")));
        busy.store(false, Ordering::SeqCst);
        finish(&mut page, result);
    });
    Ok(())
}

#[tauri::command]
fn install(plan: Plan, app: AppHandle, w: tauri::State<'_, Wizard>) -> Result<(), String> {
    let problems = {
        let guard = w.survey.lock().unwrap();
        let s = guard.as_ref().ok_or("the system check hasn't run yet")?;
        installer::validate(&plan, s)
    };
    if !problems.is_empty() {
        return Err(problems.join(" "));
    }
    let ask_privacy = crate::posthog::available();
    start_job(app, &w, move |page| {
        let settings = Settings::load(&Paths::system().settings).0;
        let opts = survey::options(&plan, &settings, ask_privacy);
        install::install(&opts, page).map(|done| Some(done.chat_url))
    })
}

#[tauri::command]
fn uninstall(
    plan: UninstallPlan,
    app: AppHandle,
    w: tauri::State<'_, Wizard>,
) -> Result<(), String> {
    if w.mode != Mode::Uninstall {
        return Err("not the uninstall wizard".into());
    }
    let done = app.clone();
    start_job(app, &w, move |page| {
        install::uninstall(&plan, page)?;
        done.state::<Wizard>()
            .uninstalled
            .store(true, Ordering::SeqCst);
        Ok(None)
    })
}

/// Stops the install at the next step; a download stops right away (and resumes next time).
#[tauri::command]
fn cancel(w: tauri::State<'_, Wizard>) {
    w.cancel.store(true, Ordering::SeqCst);
}

/// Opens the chat page in the user's browser (not elevated).
#[tauri::command]
fn open_chat() {
    let port = Settings::load(&Paths::system().settings).0.port;
    sys::open_unelevated(std::path::Path::new(&format!("http://127.0.0.1:{port}")));
}

#[tauri::command]
fn open_log() {
    let f = Paths::system().data_dir.join("setup.log");
    if f.exists() {
        sys::open_in_notepad(&f);
    }
}

#[tauri::command]
fn close(app: AppHandle, w: tauri::State<'_, Wizard>) {
    if !w.busy.load(Ordering::SeqCst) {
        app.exit(0);
    }
}

/// Removes what only this window needed, once the process is gone.
fn clean_up(w: &Wizard) {
    let dir = paths::install_dir();
    if w.uninstalled.load(Ordering::SeqCst) {
        // The profile is outside the install folder under Administrator Protection.
        install::delete_after_exit(&[(&w.webview_dir, true), (&dir, true)]);
    } else {
        // Cancelled before anything was installed: don't leave an empty folder in Program Files.
        install::delete_after_exit(&[(&w.webview_dir, true), (&dir, false)]);
    }
}

pub fn run(uninstall_mode: bool) -> i32 {
    let _one = match sys::SingleInstance::acquire_named("NoDramaLlamaSetup", 0) {
        Ok(Some(i)) => i,
        Ok(None) => return 0, // a wizard is already open
        Err(e) => {
            sys::message(&format!("Couldn't start setup: {e}"), true);
            return 1;
        }
    };
    // WebView2's profile defaults to %LOCALAPPDATA%, which unelevated programs can write to (and
    // so plant scripts in this elevated window). Program Files is admin-only.
    let webview_dir =
        super::settings_app::webview_dir(paths::install_dir().join("setup-webview2"), "setup");
    if let Err(e) = std::fs::create_dir_all(&webview_dir) {
        sys::message(
            &format!("Couldn't create {}: {e}", webview_dir.display()),
            true,
        );
        return 1;
    }
    let wizard = Wizard {
        mode: if uninstall_mode {
            Mode::Uninstall
        } else {
            Mode::Install
        },
        survey: Mutex::default(),
        busy: Arc::default(),
        cancel: Arc::default(),
        uninstalled: AtomicBool::new(false),
        webview_dir: webview_dir.clone(),
    };
    let app = tauri::Builder::default()
        .manage(wizard)
        .invoke_handler(tauri::generate_handler![
            mode,
            survey,
            uninstall_survey,
            review,
            install,
            uninstall,
            cancel,
            open_chat,
            open_log,
            close
        ])
        .on_window_event(|window, event| {
            // Closing mid-install asks first (the page shows the question).
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.state::<Wizard>().busy.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.emit_to(LABEL, "close_requested", ());
                }
            }
        })
        .setup(move |app| {
            WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("setup.html".into()))
                .data_directory(webview_dir)
                .title(if uninstall_mode {
                    format!("Uninstall {APP_NAME}")
                } else {
                    format!("{APP_NAME} Setup")
                })
                .inner_size(920.0, 640.0)
                .min_inner_size(760.0, 560.0)
                .center()
                .build()?;
            Ok(())
        })
        .build(super::settings_app::context());
    match app {
        Ok(app) => {
            app.run(|app, event| {
                if let RunEvent::Exit = event {
                    clean_up(&app.state::<Wizard>());
                }
            });
            0
        }
        Err(e) => {
            sys::message(
                &format!(
                    "Couldn't open the setup window: {e}\n\n\
                     It needs the Microsoft Edge WebView2 Runtime, which Windows 10 and 11 \
                     normally include. To install without it, run from a terminal:\n\
                     no-drama-llama.exe install"
                ),
                true,
            );
            1
        }
    }
}
