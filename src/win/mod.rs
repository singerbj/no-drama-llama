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
pub mod setup_app;
pub mod survey;
pub mod sys;
pub mod tray;
pub mod updater;
pub mod worker;

use crate::cli::{self, Command};
use crate::installer::UninstallPlan;
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
        Command::Setup { uninstall } => open_setup(uninstall),
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
                laya: a.laya.then_some(true),
                model: a.model,
                backend: a
                    .backend
                    .as_deref()
                    .and_then(crate::hardware::Backend::parse),
                ..Default::default()
            };
            let r = install::install(&opts, &mut install::Console::default()).map(|done| {
                println!(
                    "\nDone. Look for the dot in the system tray (click ^ if hidden, drag it onto the taskbar to pin).\n\
                     Chat + OpenAI-compatible API: {}   Hotkey: Ctrl+Alt+L   Log: {}\n\
                     Uninstall from Settings > Apps, or: \"{}\" uninstall",
                    done.chat_url,
                    done.log.display(),
                    done.exe.display()
                );
                if let Some(url) = done.laya_url {
                    println!("Laya (Ollaya API): {url}");
                }
            });
            let code = console_result(r);
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
            let r = (|| -> anyhow::Result<()> {
                if !yes && !install::prompt(
                    "Remove No Drama Llama, llama.cpp, your settings and (unless --keep-models) the models?",
                    false,
                ) {
                    println!("Cancelled.");
                    return Ok(());
                }
                let plan = UninstallPlan {
                    keep_models,
                    disable_auto_sign_in: install::auto_sign_in()
                        && (yes || install::prompt("Automatic sign-in is on. Turn it off?", true)),
                };
                install::uninstall(&plan, &mut install::Console::default())?;
                println!("\nDone. {}", install::BIOS_NOTE);
                Ok(())
            })();
            let code = console_result(r);
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
            open_setup(false)
        }
    }
}

/// The setup (or uninstall) wizard, elevated: it asks for administrator rights first.
fn open_setup(uninstall: bool) -> i32 {
    if sys::is_elevated() {
        return setup_app::run(uninstall);
    }
    let args = if uninstall {
        "setup --uninstall"
    } else {
        "setup"
    };
    // Settings > Apps waits for an uninstall to finish before it refreshes the list.
    match sys::run_elevated(args, uninstall) {
        Ok(code) => code as i32,
        Err(e) => {
            sys::message(
                &format!("Setting up No Drama Llama needs administrator rights: {e}"),
                true,
            );
            1
        }
    }
}

#[cfg(test)]
mod tests;
