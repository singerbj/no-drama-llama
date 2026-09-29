//! The settings window: a small Tauri app that the tray starts as a child process
//! (`no-drama-llama.exe settings-window`). It only relays messages. Requests from the page go
//! to stdout, and the tray's events arrive on stdin (see [`crate::control`]). Closing the
//! window, or the tray going away, ends it.

use super::sys;
use crate::paths::APP_NAME;
use serde_json::Value;
use std::io::{BufRead, Write};
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "settings";

/// The tray's latest state, for a page that (re)loads after it arrived.
#[derive(Default)]
struct Latest(Mutex<Option<Value>>);

/// Page -> tray. The tray checks it (`control::Request::parse`) before acting on it.
#[tauri::command]
fn send(request: Value) -> Result<(), String> {
    if !request.is_object() {
        return Err("not a request".into());
    }
    let mut out = std::io::stdout().lock();
    writeln!(out, "{request}")
        .and_then(|_| out.flush())
        .map_err(|e| format!("the tray app isn't running ({e})"))
}

#[tauri::command]
fn state(latest: tauri::State<'_, Latest>) -> Option<Value> {
    latest.0.lock().unwrap().clone()
}

/// Tray -> page, until the tray closes our stdin.
fn relay_events(app: AppHandle) {
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let Ok(event) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match event["type"].as_str() {
            Some("state") => {
                *app.state::<Latest>().0.lock().unwrap() = Some(event["data"].clone());
                let _ = app.emit_to(LABEL, "state", &event["data"]);
            }
            Some("saved") => {
                let _ = app.emit_to(LABEL, "saved", &event["data"]);
            }
            Some("focus") => {
                if let Some(w) = app.get_webview_window(LABEL) {
                    let _ = w.unminimize();
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            _ => {}
        }
    }
    app.exit(0);
}

/// The app's Tauri config and bundled pages (`ui/dist`), shared with the setup wizard so
/// they're embedded once.
pub(crate) fn context() -> tauri::Context<tauri::Wry> {
    tauri::generate_context!()
}

/// Creates `dir` for an elevated window's WebView2 profile, writable only by SYSTEM,
/// administrators and the account WebView2 will run as. Its default, %LOCALAPPDATA%, would be
/// the elevated account's, and any unelevated program can write it. But WebView2 won't run
/// elevated: it relaunches itself through Explorer, as the signed-in user's unelevated token
/// (another account than ours under Administrator Protection), which can't write an admin-only
/// folder ("Microsoft Edge can't read and write to its data directory"). So that account gets
/// write access too: the shell's user, and the signed-in user in case the shell's token can't
/// be read.
pub(crate) fn webview_dir(dir: &Path) -> anyhow::Result<()> {
    use anyhow::Context;
    std::fs::create_dir_all(dir).with_context(|| format!("couldn't create {}", dir.display()))?;
    let mut users: Vec<String> = sys::shell_user_sid()
        .into_iter()
        .chain(sys::current_user_sid().ok())
        .collect();
    users.sort();
    users.dedup();
    let mut sddl = String::from("D:PAI(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
    for sid in &users {
        sddl.push_str(&format!("(A;OICI;FA;;;{sid})"));
    }
    sys::set_dacl(dir, &sddl)
}

pub fn run() -> i32 {
    let result = tauri::Builder::default()
        .manage(Latest::default())
        .invoke_handler(tauri::generate_handler![send, state])
        .setup(|app| {
            let mut builder =
                WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()));
            if sys::is_elevated() {
                let dir = crate::paths::Paths::system().data_dir.join("webview2");
                webview_dir(&dir)?;
                builder = builder.data_directory(dir);
            }
            builder
                .title(APP_NAME)
                .inner_size(860.0, 780.0)
                .min_inner_size(600.0, 480.0)
                .center()
                .build()?;
            let handle = app.handle().clone();
            std::thread::spawn(move || relay_events(handle));
            Ok(())
        })
        .run(context());
    match result {
        Ok(()) => 0,
        Err(e) => {
            sys::message(
                &format!(
                    "Couldn't open the settings window: {e}\n\n\
                     It needs the Microsoft Edge WebView2 Runtime, which Windows 10 and 11 \
                     normally include. All settings are also in the tray icon's menu."
                ),
                true,
            );
            1
        }
    }
}
