//! What the setup wizard needs to know before installing: this PC's hardware, what's already
//! installed, which llama.cpp release and models are available, and the system checks.
//! The decisions are in [`crate::installer`]; this module only gathers the facts.

use super::{install, models, probe, sys};
use crate::catalog;
use crate::control::{self, InstalledModel};
use crate::hardware;
use crate::installer::{self as inst, Existing, ModelChoice, Plan, Survey, UninstallSurvey};
use crate::paths::{self, Paths};
use crate::settings::Settings;
use std::net::TcpListener;

/// The app itself plus unpacked llama.cpp: the least an install needs on disk.
const MIN_INSTALL: u64 = 1_500_000_000;

fn installed_models(p: &Paths) -> Vec<InstalledModel> {
    p.list_models()
        .into_iter()
        .map(|(name, size)| InstalledModel { name, size })
        .collect()
}

fn registered_version() -> Option<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NoDramaLlama")
        .and_then(|k| k.get_value("DisplayVersion"))
        .ok()
}

fn existing(p: &Paths, s: &Settings) -> Option<Existing> {
    let version = registered_version();
    if version.is_none() && !paths::installed_exe().exists() && !p.settings.exists() {
        return None;
    }
    let model = (p.model(&s.model).exists()
        && paths::missing_parts(&p.models_dir, &s.model).is_empty())
    .then(|| s.model.clone());
    Some(Existing {
        version,
        model,
        models: installed_models(p),
    })
}

fn drive_of(p: &Paths) -> String {
    p.root
        .components()
        .next()
        .map(|c| {
            c.as_os_str()
                .to_string_lossy()
                .trim_end_matches('\\')
                .to_string()
        })
        .unwrap_or_else(|| "C:".into())
}

fn folder_owned_by_someone_else(p: &Paths) -> bool {
    p.root.exists()
        && sys::owner_sid(&p.root).is_ok_and(|o| !sys::TRUSTED_OWNER_SIDS.contains(&o.as_str()))
}

pub fn survey() -> Survey {
    let p = Paths::system();
    let settings = Settings::load(&p.settings).0;
    let existing = existing(&p, &settings);

    // The slow parts (nvidia-smi, llama-server --list-devices, two web APIs) run side by side.
    let (nvidia, (pc, gpu), releases, huggingface) = std::thread::scope(|s| {
        let releases = s.spawn(install::llama_releases);
        let hf = s.spawn(|| {
            let m = catalog::catalog()
                .into_iter()
                .next()
                .expect("catalog isn't empty");
            models::hf_file_meta(models::HF, m.repo, &m.files[0])
                .map(|_| ())
                .map_err(|e| format!("{e:#}"))
        });
        let nvidia = probe::nvidia_gpus();
        let machine = probe::machine_with(&p.server_exe, &nvidia);
        (
            nvidia,
            machine,
            releases
                .join()
                .unwrap_or_else(|_| Err(anyhow::anyhow!("crashed"))),
            hf.join().unwrap_or_else(|_| Err("crashed".into())),
        )
    });

    let recommended_backend = hardware::choose_backend(&nvidia);
    let llama_installed = p
        .server_exe
        .exists()
        .then(|| install::installed_backend(&p))
        .flatten();
    let (github, releases) = match releases {
        Ok(r) => (Ok(()), r),
        Err(e) => (Err(format!("{e:#}")), Vec::new()),
    };
    let llama_tag =
        install::pick_llama_release(releases.clone(), recommended_backend).map(|r| r.tag_name);
    let backends = inst::backend_options(&nvidia, |b| {
        let rel = install::pick_llama_release(releases.clone(), b)?;
        install::llama_download_size(&rel, b)
    });
    let models = installed_models(&p);
    let catalog = control::catalog_entries(&pc, &models, None);

    let drive = drive_of(&p);
    let disk_free = sys::disk_free(&p.root);
    let laptop = sys::has_battery();
    let port_free = existing.is_some() || TcpListener::bind(("127.0.0.1", settings.port)).is_ok();
    let checks = vec![
        inst::windows_check(sys::windows_build()),
        inst::gpu_check(gpu.as_deref(), pc.vram, &nvidia),
        inst::ram_check(pc.ram),
        inst::disk_check(&drive, disk_free, MIN_INSTALL),
        inst::network_check(github, huggingface, llama_installed.is_some()),
        inst::folder_check(
            &p.root.display().to_string(),
            p.root.exists(),
            sys::is_reparse_point(&p.root),
            folder_owned_by_someone_else(&p),
        ),
        inst::port_check(settings.port, port_free),
        inst::power_check(laptop),
    ];

    let keep = existing.as_ref().and_then(|e| e.model.as_ref()).is_some();
    let recommended = catalog.iter().find(|c| c.recommended).map(|c| c.id.clone());
    let defaults = Plan {
        model: match (keep, recommended) {
            (true, _) => ModelChoice::Keep,
            (false, Some(id)) => ModelChoice::Download(id),
            (false, None) => ModelChoice::None,
        },
        backend: recommended_backend,
        power: !laptop,
        wake_on_lan: !laptop,
        start_with_windows: settings.start_with_windows,
        laya: settings.run_laya,
        crash_reports: settings.send_crash_reports,
        usage_stats: settings.share_usage_stats,
    };

    Survey {
        version: env!("CARGO_PKG_VERSION").into(),
        existing,
        gpu,
        vram: pc.vram,
        ram: pc.ram,
        laptop,
        backends,
        recommended_backend,
        llama_tag,
        llama_installed,
        laya_installed: p.ollaya_exe.exists(),
        catalog,
        drive,
        disk_free,
        checks,
        defaults,
        ask_privacy: crate::posthog::available(),
        install_dir: paths::install_dir().display().to_string(),
        data_dir: p.root.display().to_string(),
    }
}

pub fn uninstall_survey() -> UninstallSurvey {
    let p = Paths::system();
    UninstallSurvey {
        version: registered_version(),
        models: installed_models(&p),
        auto_sign_in: install::auto_sign_in(),
        data_dir: p.root.display().to_string(),
        downloads_dir: install::downloads_dir().display().to_string(),
    }
}

/// The wizard's plan as installer options.
/// `ask_privacy`: the wizard showed the crash report / usage stats choices.
pub fn options(plan: &Plan, settings: &Settings, ask_privacy: bool) -> install::InstallOptions {
    let (skip_model, model) = match &plan.model {
        ModelChoice::Download(id) => (false, Some(id.clone())),
        ModelChoice::Keep | ModelChoice::None => (true, None),
    };
    install::InstallOptions {
        skip_model,
        model,
        backend: Some(plan.backend),
        skip_power: !plan.power,
        skip_wol: !plan.wake_on_lan,
        llama_tag: None,
        update_llama: false,
        // Only a change: turning Laya on when it's already on would download Ollaya again.
        laya: (plan.laya != settings.run_laya).then_some(plan.laya),
        start_with_windows: Some(plan.start_with_windows),
        privacy: ask_privacy.then_some((plan.crash_reports, plan.usage_stats)),
    }
}
