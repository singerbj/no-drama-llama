//! The setup wizard's decisions: what each system check concludes, which llama.cpp builds
//! this PC can run, how much disk a plan needs, which steps it runs, and the messages between
//! the wizard's page (`ui/src/setup/`) and the installer. The Windows side gathers the facts
//! (`win::survey`) and carries the plan out (`win::install`, driven by `win::setup_app`).

use crate::control::{CatalogEntry, FitView, InstalledModel};
use crate::hardware::{self, Backend, NvidiaGpu};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub const GB: u64 = 1_000_000_000;
const GIB: u64 = 1024 * 1024 * 1024;
/// Left free on the drive after everything is in place (logs, updates, Windows itself).
pub const DISK_MARGIN: u64 = 2 * GB;
/// A llama.cpp download when the release couldn't be read (a CUDA build with its runtime).
pub const LLAMA_ESTIMATE: u64 = 700_000_000;
/// Ollaya with its NVIDIA GPU pack.
pub const LAYA_ESTIMATE: u64 = 1_300_000_000;
/// Windows 10 1809: the oldest with the WebView2 runtime and the NetAdapter cmdlets we use.
pub const MIN_WINDOWS_BUILD: u32 = 17763;
/// Windows 10 22H2, the last supported Windows 10.
pub const SUPPORTED_WINDOWS_BUILD: u32 = 19045;
/// Below this, even the smallest model has to share RAM with Windows and the games.
pub const LOW_RAM: u64 = 16 * GIB;

// ------------------------------------------------------------------ checks

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    /// Installs, but something isn't ideal: the detail says what to do about it.
    Warn,
    /// Can't install until it's fixed.
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    pub id: &'static str,
    pub title: String,
    pub status: Status,
    pub detail: String,
}

fn check(
    id: &'static str,
    title: impl Into<String>,
    status: Status,
    detail: impl Into<String>,
) -> Check {
    Check {
        id,
        title: title.into(),
        status,
        detail: detail.into(),
    }
}

pub fn gb(bytes: u64) -> String {
    let g = bytes as f64 / GB as f64;
    if g >= 10.0 {
        format!("{g:.0} GB")
    } else {
        format!("{g:.1} GB")
    }
}

/// `build`: CurrentBuildNumber from the registry.
pub fn windows_check(build: Option<u32>) -> Check {
    let Some(b) = build else {
        return check(
            "windows",
            "Windows",
            Status::Warn,
            "Couldn't read the Windows version. Windows 10 22H2 or Windows 11 is recommended.",
        );
    };
    let name = if b >= 22000 {
        "Windows 11"
    } else {
        "Windows 10"
    };
    let title = format!("{name} (build {b})");
    if b < MIN_WINDOWS_BUILD {
        check(
            "windows",
            title,
            Status::Fail,
            "This Windows is too old. Update to Windows 10 22H2 or Windows 11.",
        )
    } else if b < SUPPORTED_WINDOWS_BUILD {
        check(
            "windows",
            title,
            Status::Warn,
            "Works, but this Windows no longer gets updates. Windows 10 22H2 or Windows 11 is recommended.",
        )
    } else {
        check("windows", title, Status::Pass, "Supported.")
    }
}

pub fn ram_check(ram: u64) -> Check {
    let title = format!("{:.0} GB of memory", ram as f64 / GIB as f64);
    if ram < LOW_RAM {
        check(
            "ram",
            title,
            Status::Warn,
            "Less than 16 GB. Pick a model that fits your GPU, or it will be slow.",
        )
    } else {
        check(
            "ram",
            title,
            Status::Pass,
            "Enough for Windows, your games and the model's overflow.",
        )
    }
}

/// `gpu`: the primary GPU's name as the probes found it; `vram`: 0 when there's no usable GPU.
pub fn gpu_check(gpu: Option<&str>, vram: u64, nvidia: &[NvidiaGpu]) -> Check {
    let Some(name) = gpu.filter(|_| vram > 0) else {
        return check(
            "gpu",
            "No graphics card found",
            Status::Warn,
            "The model will run on the CPU, slowly. Update your graphics driver if you have a dedicated GPU.",
        );
    };
    let title = name.to_string();
    let best = hardware::choose_backend(nvidia);
    if let Some(g) = nvidia.iter().max_by_key(|g| g.memory_mib) {
        if best == Backend::Vulkan {
            return check(
                "gpu",
                title,
                Status::Warn,
                format!(
                    "NVIDIA driver {} is too old for the CUDA build, so it uses Vulkan (slower). Driver 580 or newer is recommended.",
                    g.driver_major
                ),
            );
        }
        if best == Backend::Cuda12 && g.compute_cap >= 7.5 {
            return check(
                "gpu",
                title,
                Status::Warn,
                format!(
                    "Uses the CUDA 12 build. NVIDIA driver 580 or newer (you have {}) enables the newer CUDA 13 build.",
                    g.driver_major
                ),
            );
        }
    }
    if vram < 8 * GIB {
        return check(
            "gpu",
            title,
            Status::Warn,
            "Under 8 GB of video memory: only the smallest models fit, and they may spill into RAM.",
        );
    }
    check(
        "gpu",
        title,
        Status::Pass,
        format!(
            "{} build · {:.0} GB of video memory.",
            best.label(),
            vram as f64 / GIB as f64
        ),
    )
}

/// `needed`: the smallest install (the app and llama.cpp, no model).
pub fn disk_check(drive: &str, free: Option<u64>, needed: u64) -> Check {
    let Some(free) = free else {
        return check(
            "disk",
            format!("Space on {drive}"),
            Status::Warn,
            "Couldn't read the free space. Models need 8 - 110 GB.",
        );
    };
    let title = format!("{} free on {drive}", gb(free));
    if free < needed + DISK_MARGIN {
        check(
            "disk",
            title,
            Status::Fail,
            format!(
                "Needs at least {} free. Free up some space and check again.",
                gb(needed + DISK_MARGIN)
            ),
        )
    } else if free < 12 * GB + DISK_MARGIN {
        check(
            "disk",
            title,
            Status::Warn,
            "Room for the app, but only the smallest models fit.",
        )
    } else {
        check("disk", title, Status::Pass, "Room for the app and a model.")
    }
}

/// `github`/`huggingface`: whether their APIs answered. Without GitHub there's no llama.cpp,
/// unless it's already installed.
pub fn network_check(
    github: Result<(), String>,
    huggingface: Result<(), String>,
    llama_installed: bool,
) -> Check {
    match (github, huggingface) {
        (Ok(()), Ok(())) => check(
            "network",
            "Internet connection",
            Status::Pass,
            "GitHub (llama.cpp) and Hugging Face (models) are reachable.",
        ),
        (Err(e), _) if !llama_installed => check(
            "network",
            "Can't reach GitHub",
            Status::Fail,
            format!("llama.cpp downloads from GitHub. Check your connection, VPN or firewall. ({e})"),
        ),
        (Err(e), _) => check(
            "network",
            "Can't reach GitHub",
            Status::Warn,
            format!("Keeps the llama.cpp you have; updates need GitHub. ({e})"),
        ),
        (Ok(()), Err(e)) => check(
            "network",
            "Can't reach Hugging Face",
            Status::Warn,
            format!("Models download from Hugging Face. Install without one, or check your connection. ({e})"),
        ),
    }
}

/// C:\LLM as it is now.
pub fn folder_check(root: &str, exists: bool, is_link: bool, foreign_owner: bool) -> Check {
    if is_link {
        check(
            "folder",
            root.to_string(),
            Status::Fail,
            format!("{root} is a link to another folder. Delete it (not what it points to) and check again."),
        )
    } else if foreign_owner {
        check(
            "folder",
            root.to_string(),
            Status::Warn,
            "Created by another account: its programs are downloaded again, and it's made admin-only.",
        )
    } else if exists {
        check(
            "folder",
            root.to_string(),
            Status::Pass,
            "Found. Your models and settings are kept.",
        )
    } else {
        check(
            "folder",
            root.to_string(),
            Status::Pass,
            "Will hold llama.cpp, your models and settings.",
        )
    }
}

pub fn port_check(port: u16, free: bool) -> Check {
    if free {
        check(
            "port",
            format!("Port {port}"),
            Status::Pass,
            "Free for the chat page and the API.",
        )
    } else {
        check(
            "port",
            format!("Port {port} is in use"),
            Status::Warn,
            "Another program uses it. After installing, pick another port in Settings → LLM.",
        )
    }
}

pub fn power_check(laptop: bool) -> Check {
    if laptop {
        check(
            "power",
            "Laptop",
            Status::Warn,
            "No Drama Llama is made for desktops that stay on. The always-on power settings are off by default.",
        )
    } else {
        check(
            "power",
            "Desktop PC",
            Status::Pass,
            "Can stay on as a local AI server.",
        )
    }
}

/// True when any check blocks the install.
pub fn blocked(checks: &[Check]) -> bool {
    checks.iter().any(|c| c.status == Status::Fail)
}

// ------------------------------------------------------------------ survey and plan

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendOption {
    pub id: Backend,
    pub label: &'static str,
    pub available: bool,
    /// Download size (the zip and, for CUDA, its runtime), if the release was read
    pub download: Option<u64>,
    pub note: &'static str,
}

/// The llama.cpp builds, with whether this PC can run each.
pub fn backend_options(
    nvidia: &[NvidiaGpu],
    download: impl Fn(Backend) -> Option<u64>,
) -> Vec<BackendOption> {
    [Backend::Vulkan, Backend::Cuda12, Backend::Cuda13]
        .into_iter()
        .map(|b| BackendOption {
            id: b,
            label: b.label(),
            available: hardware::backend_supported(b, nvidia),
            download: download(b),
            note: match b {
                Backend::Vulkan => "Any GPU: AMD, Intel or NVIDIA",
                Backend::Cuda12 => "NVIDIA GTX 10 series or newer, driver 528+",
                Backend::Cuda13 => "NVIDIA RTX 20 series or newer, driver 580+",
            },
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Existing {
    /// From Apps & features
    pub version: Option<String>,
    /// The configured model, if it's on disk
    pub model: Option<String>,
    pub models: Vec<InstalledModel>,
}

/// Everything the wizard shows before installing.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Survey {
    pub version: String,
    pub existing: Option<Existing>,
    pub gpu: Option<String>,
    pub vram: u64,
    pub ram: u64,
    pub laptop: bool,
    pub backends: Vec<BackendOption>,
    pub recommended_backend: Backend,
    /// The llama.cpp release that would be installed
    pub llama_tag: Option<String>,
    /// The llama.cpp build installed now
    pub llama_installed: Option<Backend>,
    pub laya_installed: bool,
    pub catalog: Vec<CatalogEntry>,
    pub drive: String,
    pub disk_free: Option<u64>,
    pub checks: Vec<Check>,
    pub defaults: Plan,
    /// Offer the crash report / usage stats choices (only builds with PostHog configured)
    pub ask_privacy: bool,
    pub install_dir: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum ModelChoice {
    /// The configured model, already on disk
    Keep,
    /// No model now (drop a .gguf in later, or download from the tray)
    None,
    /// A catalog id
    Download(String),
}

/// What the wizard's user picked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub model: ModelChoice,
    pub backend: Backend,
    pub power: bool,
    pub wake_on_lan: bool,
    pub start_with_windows: bool,
    pub laya: bool,
    pub crash_reports: bool,
    pub usage_stats: bool,
}

/// What the plan downloads and how much disk it takes once unpacked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sizes {
    pub download: u64,
    pub disk: u64,
}

pub fn sizes(plan: &Plan, s: &Survey) -> Sizes {
    let model = match &plan.model {
        ModelChoice::Download(id) => s
            .catalog
            .iter()
            .find(|c| &c.id == id && !c.installed)
            .map_or(0, |c| c.size),
        _ => 0,
    };
    let llama = if s.llama_installed == Some(plan.backend) {
        0
    } else {
        s.backends
            .iter()
            .find(|b| b.id == plan.backend)
            .and_then(|b| b.download)
            .unwrap_or(LLAMA_ESTIMATE)
    };
    let laya = if plan.laya && !s.laya_installed {
        LAYA_ESTIMATE
    } else {
        0
    };
    Sizes {
        download: model + llama + laya,
        // Zips take about twice their size unpacked, and sit next to what they unpack to.
        disk: model + 3 * llama + 2 * laya,
    }
}

/// Why the plan can't be installed as it is (empty = fine).
pub fn validate(plan: &Plan, s: &Survey) -> Vec<String> {
    let mut problems = Vec::new();
    if blocked(&s.checks) {
        problems.push("Fix the failed system checks first.".to_string());
    }
    match &plan.model {
        ModelChoice::Keep if s.existing.as_ref().and_then(|e| e.model.as_ref()).is_none() => {
            problems.push("There's no current model to keep. Pick one to download.".into());
        }
        ModelChoice::Download(id) => match s.catalog.iter().find(|c| &c.id == id) {
            None => problems.push(format!("Unknown model {id}.")),
            Some(c) if c.fit == FitView::TooBig => {
                problems.push(format!(
                    "{} is too big for this PC. Pick a smaller one.",
                    c.label
                ));
            }
            Some(_) => {}
        },
        _ => {}
    }
    if let Some(b) = s.backends.iter().find(|b| b.id == plan.backend) {
        if !b.available {
            problems.push(format!(
                "This PC can't run the {} build ({}).",
                b.label, b.note
            ));
        }
    }
    if let Some(free) = s.disk_free {
        let need = sizes(plan, s).disk + DISK_MARGIN;
        if need > free {
            problems.push(format!(
                "Needs {} free on {}, and there's {}. Free up space or pick a smaller model.",
                gb(need),
                s.drive,
                gb(free)
            ));
        }
    }
    problems
}

// ------------------------------------------------------------------ steps and progress

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StepId {
    Stop,
    App,
    Folders,
    Hardware,
    LlamaCpp,
    Model,
    Laya,
    Settings,
    Register,
    Start,
    // uninstall
    Restore,
    Files,
}

impl StepId {
    pub fn label(self) -> &'static str {
        match self {
            StepId::Stop => "Stopping any running copy",
            StepId::App => "Installing the app",
            StepId::Folders => r"Setting up C:\LLM",
            StepId::Hardware => "Checking your hardware",
            StepId::LlamaCpp => "Downloading llama.cpp",
            StepId::Model => "Downloading the model",
            StepId::Laya => "Installing Laya (Ollaya)",
            StepId::Settings => "Power and network settings",
            StepId::Register => "Start menu, Apps & features, start at sign-in",
            StepId::Start => "Starting No Drama Llama",
            StepId::Restore => "Restoring your power and network settings",
            StepId::Files => "Removing files",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StepInfo {
    pub id: StepId,
    pub label: &'static str,
}

fn infos(ids: &[StepId]) -> Vec<StepInfo> {
    ids.iter()
        .map(|&id| StepInfo {
            id,
            label: id.label(),
        })
        .collect()
}

/// The steps the wizard lists for a plan, in order.
pub fn planned_steps(plan: &Plan, s: &Survey) -> Vec<StepInfo> {
    let mut v = vec![StepId::Stop, StepId::App, StepId::Folders, StepId::Hardware];
    if s.llama_installed != Some(plan.backend) {
        v.push(StepId::LlamaCpp);
    }
    if matches!(plan.model, ModelChoice::Download(_)) {
        v.push(StepId::Model);
    }
    if plan.laya {
        v.push(StepId::Laya);
    }
    v.extend([StepId::Settings, StepId::Register, StepId::Start]);
    infos(&v)
}

pub fn uninstall_steps() -> Vec<StepInfo> {
    infos(&[StepId::Stop, StepId::Restore, StepId::Files])
}

/// How an install or uninstall reports what it's doing: printed in a console, or sent to the
/// wizard's page.
pub trait Reporter {
    fn step(&mut self, id: StepId);
    fn info(&mut self, text: &str);
    fn progress(&mut self, done: u64, total: u64);
    /// Set to stop at the next step (downloads stop right away and resume next time).
    fn cancel_flag(&self) -> Arc<AtomicBool>;
}

pub const CANCELLED: &str = "Cancelled";

/// Wizard page <- installer.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Progress {
    Step { id: StepId },
    Log { text: String },
    Bytes { done: u64, total: u64 },
    Finished(Finished),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    pub ok: bool,
    pub cancelled: bool,
    pub error: Option<String>,
    pub chat_url: Option<String>,
    pub log_file: Option<String>,
}

/// What uninstalling asks first.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallSurvey {
    pub version: Option<String>,
    pub models: Vec<InstalledModel>,
    pub auto_sign_in: bool,
    pub data_dir: String,
    pub downloads_dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallPlan {
    pub keep_models: bool,
    pub disable_auto_sign_in: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{catalog, Machine};
    use crate::control::catalog_entries;

    fn nv(driver: u32, cc: f32, mib: u64) -> NvidiaGpu {
        NvidiaGpu {
            name: "NVIDIA GeForce RTX 4090".into(),
            driver_major: driver,
            compute_cap: cc,
            memory_mib: mib,
        }
    }

    fn survey(vram_gib: u64, free: Option<u64>) -> Survey {
        let pc = Machine {
            vram: vram_gib * GIB,
            ram: 32 * GIB,
        };
        let nvidia = [nv(581, 8.9, vram_gib * 1024)];
        Survey {
            version: "1.0.0".into(),
            existing: None,
            gpu: Some("RTX".into()),
            vram: pc.vram,
            ram: pc.ram,
            laptop: false,
            backends: backend_options(&nvidia, |b| match b {
                Backend::Vulkan => Some(50_000_000),
                _ => Some(600_000_000),
            }),
            recommended_backend: Backend::Cuda13,
            llama_tag: Some("b11222".into()),
            llama_installed: None,
            laya_installed: false,
            catalog: catalog_entries(&pc, &[], None),
            drive: "C:".into(),
            disk_free: free,
            checks: vec![],
            defaults: plan(ModelChoice::None),
            ask_privacy: false,
            install_dir: String::new(),
            data_dir: String::new(),
        }
    }

    fn plan(model: ModelChoice) -> Plan {
        Plan {
            model,
            backend: Backend::Cuda13,
            power: true,
            wake_on_lan: true,
            start_with_windows: true,
            laya: false,
            crash_reports: false,
            usage_stats: false,
        }
    }

    const Q4: &str = "qwen3.8-27b:UD-Q4_K_XL";

    #[test]
    fn windows_versions() {
        assert_eq!(windows_check(Some(26100)).status, Status::Pass);
        assert!(windows_check(Some(26100)).title.starts_with("Windows 11"));
        assert_eq!(windows_check(Some(19045)).status, Status::Pass);
        assert_eq!(windows_check(Some(19041)).status, Status::Warn);
        assert_eq!(windows_check(Some(17134)).status, Status::Fail);
        assert_eq!(windows_check(None).status, Status::Warn);
    }

    #[test]
    fn gpu_advice() {
        let g = Some("RTX 4090");
        assert_eq!(
            gpu_check(g, 24 * GIB, &[nv(581, 8.9, 24564)]).status,
            Status::Pass
        );
        let old_driver = gpu_check(g, 24 * GIB, &[nv(470, 8.9, 24564)]);
        assert_eq!(old_driver.status, Status::Warn);
        assert!(old_driver.detail.contains("470"), "{old_driver:?}");
        // a Turing card on a 5xx driver could have CUDA 13 with a newer driver
        assert_eq!(
            gpu_check(g, 8 * GIB, &[nv(560, 7.5, 8192)]).status,
            Status::Warn
        );
        // Pascal can't run CUDA 13 whatever the driver: nothing to advise
        assert_eq!(
            gpu_check(g, 8 * GIB, &[nv(581, 6.1, 8192)]).status,
            Status::Pass
        );
        assert_eq!(
            gpu_check(Some("Radeon RX 7900 XTX"), 24 * GIB, &[]).status,
            Status::Pass
        );
        assert_eq!(
            gpu_check(Some("Radeon RX 6500"), 4 * GIB, &[]).status,
            Status::Warn
        );
        assert_eq!(gpu_check(None, 0, &[]).status, Status::Warn);
        assert_eq!(gpu_check(Some("Intel UHD"), 0, &[]).status, Status::Warn);
    }

    #[test]
    fn disk_and_network() {
        assert_eq!(disk_check("C:", Some(GB), GB).status, Status::Fail);
        assert_eq!(disk_check("C:", Some(5 * GB), GB).status, Status::Warn);
        assert_eq!(disk_check("C:", Some(500 * GB), GB).status, Status::Pass);
        assert_eq!(disk_check("C:", None, GB).status, Status::Warn);
        let down = || Err("timed out".to_string());
        assert_eq!(network_check(Ok(()), Ok(()), false).status, Status::Pass);
        assert_eq!(network_check(down(), Ok(()), false).status, Status::Fail);
        assert_eq!(network_check(down(), Ok(()), true).status, Status::Warn);
        assert_eq!(network_check(Ok(()), down(), false).status, Status::Warn);
    }

    #[test]
    fn folder_port_power() {
        assert_eq!(
            folder_check(r"C:\LLM", true, true, false).status,
            Status::Fail
        );
        assert_eq!(
            folder_check(r"C:\LLM", true, false, true).status,
            Status::Warn
        );
        assert_eq!(
            folder_check(r"C:\LLM", false, false, false).status,
            Status::Pass
        );
        assert_eq!(port_check(8080, false).status, Status::Warn);
        assert_eq!(power_check(true).status, Status::Warn);
        assert!(!blocked(&[power_check(true), port_check(1, true)]));
        assert!(blocked(&[folder_check("x", true, true, false)]));
    }

    #[test]
    fn backend_availability() {
        let opts = backend_options(&[nv(560, 8.6, 12288)], |_| None);
        let avail: Vec<bool> = opts.iter().map(|o| o.available).collect();
        assert_eq!(avail, [true, true, false]);
        let amd = backend_options(&[], |_| None);
        assert_eq!(amd.iter().filter(|o| o.available).count(), 1);
    }

    #[test]
    fn sizes_count_only_what_is_missing() {
        let mut s = survey(24, Some(500 * GB));
        let q4 = catalog().into_iter().find(|m| m.id == Q4).unwrap();
        let p = plan(ModelChoice::Download(Q4.into()));
        assert_eq!(sizes(&p, &s).download, q4.size + 600_000_000);
        s.llama_installed = Some(Backend::Cuda13);
        assert_eq!(sizes(&p, &s).download, q4.size);
        // switching builds downloads again
        s.llama_installed = Some(Backend::Vulkan);
        assert_eq!(sizes(&p, &s).download, q4.size + 600_000_000);
        let keep = Plan {
            laya: true,
            ..plan(ModelChoice::Keep)
        };
        assert_eq!(sizes(&keep, &s).download, 600_000_000 + LAYA_ESTIMATE);
        s.catalog.iter_mut().find(|c| c.id == Q4).unwrap().installed = true;
        s.llama_installed = Some(Backend::Cuda13);
        s.laya_installed = true;
        assert_eq!(sizes(&Plan { laya: true, ..p }, &s).download, 0);
    }

    #[test]
    fn validation() {
        let s = survey(24, Some(500 * GB));
        assert!(validate(&plan(ModelChoice::Download(Q4.into())), &s).is_empty());
        assert!(validate(&plan(ModelChoice::None), &s).is_empty());
        assert_eq!(
            validate(&plan(ModelChoice::Keep), &s).len(),
            1,
            "nothing to keep"
        );
        assert_eq!(
            validate(&plan(ModelChoice::Download("nope".into())), &s).len(),
            1
        );
        let big = "qwen3.8-flash-next:UD-Q4_K_XL";
        assert!(validate(&plan(ModelChoice::Download(big.into())), &s)[0].contains("too big"));

        let full = survey(24, Some(10 * GB));
        let p = validate(&plan(ModelChoice::Download(Q4.into())), &full);
        assert!(p[0].contains("Free up space"), "{p:?}");

        let mut amd = survey(24, Some(500 * GB));
        amd.backends = backend_options(&[], |_| None);
        assert!(validate(&plan(ModelChoice::None), &amd)[0].contains("CUDA 13"));

        let mut failed = survey(24, Some(500 * GB));
        failed.checks = vec![folder_check("x", true, true, false)];
        assert_eq!(validate(&plan(ModelChoice::None), &failed).len(), 1);
    }

    #[test]
    fn steps_follow_the_plan() {
        let mut s = survey(24, None);
        let ids =
            |p: &Plan, s: &Survey| planned_steps(p, s).iter().map(|i| i.id).collect::<Vec<_>>();
        let full = Plan {
            laya: true,
            ..plan(ModelChoice::Download(Q4.into()))
        };
        assert_eq!(
            ids(&full, &s),
            [
                StepId::Stop,
                StepId::App,
                StepId::Folders,
                StepId::Hardware,
                StepId::LlamaCpp,
                StepId::Model,
                StepId::Laya,
                StepId::Settings,
                StepId::Register,
                StepId::Start
            ]
        );
        s.llama_installed = Some(Backend::Cuda13);
        let keep = ids(&plan(ModelChoice::Keep), &s);
        assert!(!keep.contains(&StepId::LlamaCpp) && !keep.contains(&StepId::Model));
    }

    #[test]
    fn messages_match_the_page() {
        let p: Plan = serde_json::from_str(
            r#"{"model":{"kind":"download","id":"x"},"backend":"cuda12","power":false,"wakeOnLan":true,
                "startWithWindows":true,"laya":false,"crashReports":false,"usageStats":true}"#,
        )
        .unwrap();
        assert_eq!(p.model, ModelChoice::Download("x".into()));
        assert_eq!(p.backend, Backend::Cuda12);
        let keep: ModelChoice = serde_json::from_str(r#"{"kind":"keep"}"#).unwrap();
        assert_eq!(keep, ModelChoice::Keep);
        let v = serde_json::to_value(Progress::Step {
            id: StepId::LlamaCpp,
        })
        .unwrap();
        assert_eq!(v, serde_json::json!({"type": "step", "id": "llamaCpp"}));
        let v = serde_json::to_value(Progress::Finished(Finished {
            ok: true,
            cancelled: false,
            error: None,
            chat_url: Some("http://127.0.0.1:8080".into()),
            log_file: None,
        }))
        .unwrap();
        assert_eq!(v["type"], "finished");
        assert_eq!(v["chatUrl"], "http://127.0.0.1:8080");
        assert_eq!(serde_json::to_value(Status::Warn).unwrap(), "warn");
        assert_eq!(serde_json::to_value(Backend::Cuda13).unwrap(), "cuda13");
    }
}
