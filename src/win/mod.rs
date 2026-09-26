//! Windows side: tray app, probes, installer, updater.

pub mod gpu;
pub mod install;
pub mod laya;
pub mod libraries;
pub mod models;
pub mod net;
pub mod osd;
pub mod probe;
pub mod procs;
pub mod settings_app;
pub mod settings_host;
pub mod sys;
pub mod tray;
pub mod updater;
pub mod worker;

use crate::cli::{self, Command};
use crate::paths::{self, Paths};

fn console_result(r: anyhow::Result<()>) -> i32 {
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("\nError: {e:#}");
            1
        }
    }
}

/// Keeps a freshly opened console on screen until the user has read it.
fn pause_if_own_console(own: bool) {
    if own {
        println!("\nPress Enter to close.");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

fn running_installed_copy() -> bool {
    std::env::current_exe().is_ok_and(|e| {
        e.as_os_str()
            .eq_ignore_ascii_case(paths::installed_exe().as_os_str())
    })
}

pub fn main() -> i32 {
    // Before anything else (and before any thread starts): the elevated app must not pass the
    // user's environment on to the programs it runs.
    sys::scrub_environment();
    let cmd = match cli::parse(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            let _ = sys::console(false);
            eprintln!("{e}");
            return 2;
        }
    };
    match cmd {
        Command::Help => {
            let _ = sys::console(false);
            print!("{}", cli::HELP);
            0
        }
        Command::Models => {
            let _ = sys::console(false);
            let p = Paths::system();
            let (pc, gpu) = probe::machine(&p.server_exe);
            println!(
                "GPU: {} · RAM: {:.0} GB",
                gpu.as_deref()
                    .unwrap_or("unknown (install first to detect AMD/Intel GPUs)"),
                crate::hardware::gib(pc.ram)
            );
            let rec = crate::catalog::recommend(&pc).map(|m| m.id);
            for m in crate::catalog::catalog() {
                let star = if rec.as_deref() == Some(m.id.as_str()) {
                    "  <- recommended"
                } else {
                    ""
                };
                println!(
                    "  {:<34} {:>6.1} GB  {}{star}",
                    m.id,
                    m.size as f64 / 1e9,
                    crate::catalog::fit_note(&m, &pc)
                );
            }
            println!("\nInstall one with: no-drama-llama.exe install --model <id>  (or from the tray: Settings > Model)");
            0
        }
        Command::Version => {
            let _ = sys::console(false);
            println!("no-drama-llama {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Command::Run { after_update } => tray::run(Paths::system(), after_update),
        Command::SettingsWindow => settings_app::run(),
        Command::Install(a) => {
            if !sys::is_elevated() {
                return match sys::run_elevated(&cli::install_args_string(&a), true) {
                    Ok(code) => code as i32,
                    Err(e) => {
                        sys::message(&format!("Install needs administrator rights: {e}"), true);
                        1
                    }
                };
            }
            let own = sys::console(true);
            let opts = install::InstallOptions {
                skip_model: a.skip_model,
                skip_power: a.skip_power,
                skip_wol: a.skip_wol,
                llama_tag: a.llama_tag,
                update_llama: a.update_llama,
                laya: a.laya,
                model: a.model,
                backend: a
                    .backend
                    .as_deref()
                    .and_then(crate::hardware::Backend::parse),
            };
            let code = console_result(install::install(&opts));
            pause_if_own_console(own);
            code
        }
        Command::Uninstall { keep_models, yes } => {
            if !sys::is_elevated() {
                let args = format!(
                    "uninstall{}{}",
                    if keep_models { " --keep-models" } else { "" },
                    if yes { " --yes" } else { "" }
                );
                return sys::run_elevated(&args, true)
                    .map(|c| c as i32)
                    .unwrap_or(1);
            }
            let own = sys::console(true);
            let code = console_result(install::uninstall(keep_models, yes));
            pause_if_own_console(own && !yes);
            install::delete_install_dir_after_exit();
            code
        }
        Command::Update => {
            let own = sys::console(true);
            let r = (|| -> anyhow::Result<()> {
                match updater::check()? {
                    None => println!(
                        "No Drama Llama {} is up to date.",
                        env!("CARGO_PKG_VERSION")
                    ),
                    Some((release, v)) => {
                        println!("Version {v} is available: {}", release.html_url);
                        if !updater::can_self_update()
                            || !running_installed_copy()
                            || !sys::is_elevated()
                        {
                            println!("The tray app installs it automatically (Settings > Update automatically), or download it from the page above.");
                        } else {
                            install::stop_running(&Paths::system());
                            updater::apply(&release, &v)?;
                            install::run_task()?;
                            println!("Updated to {v}.");
                        }
                    }
                }
                Ok(())
            })();
            let code = console_result(r);
            pause_if_own_console(own);
            code
        }
        Command::Default => {
            if running_installed_copy() && install::task_exists() {
                if sys::is_elevated() {
                    return tray::run(Paths::system(), None);
                }
                // Started from the Start menu: run the elevated logon task (no UAC prompt).
                return match install::run_task() {
                    Ok(()) => 0,
                    Err(e) => {
                        sys::message(&format!("Couldn't start the tray app: {e}"), true);
                        1
                    }
                };
            }
            let installed = paths::installed_exe().exists();
            let q = if installed {
                format!(
                    "No Drama Llama is already installed.\n\nInstall this copy (version {}) over it?",
                    env!("CARGO_PKG_VERSION")
                )
            } else {
                "Install No Drama Llama?\n\nIt downloads llama.cpp and the Qwen 3.8 27B model (17.6 GB), sets up \
                 low-power always-on settings and Wake-on-LAN (undone by uninstall), and starts with Windows.\n\n\
                 Windows will ask for administrator rights."
                    .to_string()
            };
            if !sys::ask(&q) {
                return 0;
            }
            match sys::run_elevated("install", false) {
                Ok(_) => 0,
                Err(e) => {
                    sys::message(&format!("Install needs administrator rights: {e}"), true);
                    1
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
