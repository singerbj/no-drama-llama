//! Laya next to the LLM. [Ollaya](https://ollaya.dev) is a local runtime for *decision models*:
//! it pulls and serves them the way Ollama serves LLMs, from one `ollaya.exe` daemon on its own
//! port. Its flagship model, Laya, answers typed questions (choice, score, yes/no) about a text
//! in one forward pass, in milliseconds, instead of generating text.
//!
//! This module is the platform-independent part: settings values, the daemon's environment,
//! release assets and checksums, pull progress, and the state machine that runs Ollaya alongside
//! llama-server (same on/off switch, same pause while gaming). `win::laya` downloads, starts and
//! talks to the daemon.

use crate::hardware::NvidiaGpu;
use crate::settings::Settings;
use crate::state::Tone;
use crate::update::{Asset, Release};
use anyhow::{bail, Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

pub const REPO: &str = "ollaya-dev/ollaya";
/// The CPU build: `bin/ollaya.exe`, `lib/ollaya/llama/`, `share/`.
pub const ARCHIVE: &str = "ollaya-windows-amd64.zip";
/// The NVIDIA GPU pack (CUDA 13 libraries, about 1.1 GB): `lib/ollaya/cuda_v13/`.
pub const GPU_ARCHIVE: &str = "ollaya-windows-amd64-cuda.zip";
/// sha256 of every library in the GPU pack; installed as `lib/ollaya/cuda_v13/FILES.sha256`.
pub const GPU_FILES: &str = "ollaya-windows-amd64-cuda.sha256";
/// sha256 of every archive in the release.
pub const SUMS: &str = "sha256sum.txt";

pub const DEFAULT_MODEL: &str = "laya";
pub const DEFAULT_PORT: u16 = 11435;
/// Keep the model loaded until Laya stops (pause, off, exit): always ready, like the LLM.
pub const DEFAULT_KEEP_ALIVE: &str = "-1";

/// The Laya models offered in the tray and the settings window. Any other Ollaya model name
/// can go in settings.json.
pub const MODELS: [(&str, &str); 4] = [
    ("laya", "Laya (picks English or multilingual per request)"),
    ("laya:en", "Laya English (421M, the fastest)"),
    (
        "laya:multilingual",
        "Laya multilingual (322M, 100+ languages)",
    ),
    ("laya:typed-decisions", "Laya typed-decisions"),
];

/// How long the model stays loaded after a request, as Ollaya's `keep_alive`.
pub const KEEP_ALIVE_PRESETS: [(&str, &str); 5] = [
    ("-1", "Always (until paused or off)"),
    ("1h", "1 hour after the last request"),
    ("30m", "30 minutes after the last request"),
    ("5m", "5 minutes after the last request"),
    ("0", "Unload after each request"),
];

/// Where Ollaya runs its models (`OLLAYA_DEVICE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    /// The NVIDIA GPU when the GPU pack is installed, else the CPU
    Auto,
    /// Never the GPU: Laya keeps running while you play
    Cpu,
    /// The NVIDIA GPU, or fail
    Cuda,
}

impl Device {
    pub const ALL: [(Device, &'static str); 3] = [
        (Device::Auto, "Auto (NVIDIA GPU if there is one)"),
        (Device::Cpu, "CPU only (keeps running while gaming)"),
        (Device::Cuda, "NVIDIA GPU only"),
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Device::Auto => "auto",
            Device::Cpu => "cpu",
            Device::Cuda => "cuda",
        }
    }

    pub fn parse(s: &str) -> Option<Device> {
        match s {
            "auto" => Some(Device::Auto),
            "cpu" => Some(Device::Cpu),
            "cuda" => Some(Device::Cuda),
            _ => None,
        }
    }
}

/// One part of an Ollaya model name: `[A-Za-z0-9_][A-Za-z0-9_.-]*`, at most 80 characters.
fn is_name_part(p: &str) -> bool {
    p.len() <= 80
        && p.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        && p.chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))
}

/// `model`, `model:tag`, `namespace/model` or `namespace/model:tag`. No registry host: models
/// come from ollaya.dev only, so the namespace can't contain a dot (it would read as a host).
pub fn is_valid_model_name(s: &str) -> bool {
    let (path, tag) = match s.split_once(':') {
        Some((p, t)) => (p, Some(t)),
        None => (s, None),
    };
    if tag.is_some_and(|t| !is_name_part(t)) {
        return false;
    }
    match path.split_once('/') {
        Some((ns, model)) => !ns.contains('.') && is_name_part(ns) && is_name_part(model),
        None => is_name_part(path),
    }
}

/// The name Ollaya reports for a model: lower case, with the tag (`laya` -> `laya:latest`).
pub fn canonical_name(s: &str) -> String {
    let s = s.trim().to_ascii_lowercase();
    if s.contains(':') {
        s
    } else {
        format!("{s}:latest")
    }
}

/// Ollaya's `keep_alive`: a Go duration (`5m`, `1h30m`, `-1s`), or a number of seconds
/// (`300`, `-1`). Negative = keep loaded, `0` = unload right away.
pub fn is_valid_keep_alive(s: &str) -> bool {
    if s.is_empty() || s.len() > 24 {
        return false;
    }
    let body = s.strip_prefix('-').unwrap_or(s);
    let number = |t: &str| {
        !t.is_empty()
            && t.chars().all(|c| c.is_ascii_digit() || c == '.')
            && t.parse::<f64>().is_ok_and(f64::is_finite)
    };
    if number(body) {
        return true;
    }
    let mut rest = body;
    if rest.is_empty() {
        return false;
    }
    while !rest.is_empty() {
        let n = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        if !number(&rest[..n]) {
            return false;
        }
        rest = &rest[n..];
        // Longest first: "ms" before "m" and "s".
        let Some(unit) = ["ns", "us", "µs", "μs", "ms", "s", "m", "h"]
            .into_iter()
            .find(|u| rest.starts_with(u))
        else {
            return false;
        };
        rest = &rest[unit.len()..];
    }
    true
}

/// `keep_alive` of zero: the model unloads right after each request, so preloading is pointless.
pub fn unloads_right_away(ka: &str) -> bool {
    ka.split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter(|n| !n.is_empty())
        .all(|n| n.parse::<f64>() == Ok(0.0))
}

pub fn keep_alive_label(ka: &str) -> String {
    KEEP_ALIVE_PRESETS
        .iter()
        .find(|(v, _)| *v == ka)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| {
            if ka.starts_with('-') {
                "Always (until paused or off)".into()
            } else {
                format!("{ka} after the last request")
            }
        })
}

/// The daemon's configuration, as the environment variables `ollaya serve` reads. It listens
/// where the LLM does (`ListenHost`) and takes the same API key.
pub fn serve_env(s: &Settings, models_dir: &Path) -> Vec<(&'static str, String)> {
    let mut env = vec![
        ("OLLAYA_HOST", format!("{}:{}", s.listen_host, s.laya_port)),
        ("OLLAYA_MODELS", models_dir.to_string_lossy().into_owned()),
        ("OLLAYA_KEEP_ALIVE", s.laya_keep_alive.clone()),
        ("OLLAYA_DEVICE", s.laya_device.as_str().to_string()),
    ];
    if !s.api_key.is_empty() {
        env.push(("OLLAYA_API_KEY", s.api_key.clone()));
    }
    env
}

/// Where this PC reaches the daemon (always loopback, like the LLM's health check).
pub fn base_url(s: &Settings) -> String {
    format!("http://127.0.0.1:{}", s.laya_port)
}

/// `GET /` answers 200 whenever the daemon is up, even with an API key set.
pub fn health_url(s: &Settings) -> String {
    format!("{}/", base_url(s))
}

/// Settings that can't work together, reported as Laya's error.
pub fn settings_problem(s: &Settings) -> Option<String> {
    (s.laya_port == s.port).then(|| {
        format!(
            "LayaPort {} is also the LLM's port - pick another",
            s.laya_port
        )
    })
}

/// Settings that change how the daemon runs (a change restarts it).
pub fn laya_changed(a: &Settings, b: &Settings) -> bool {
    a.run_laya != b.run_laya
        || a.laya_model != b.laya_model
        || a.laya_port != b.laya_port
        || a.laya_device != b.laya_device
        || a.laya_keep_alive != b.laya_keep_alive
        || a.listen_host != b.listen_host
        || a.api_key != b.api_key
}

/// Ollaya's GPU pack needs CUDA 13: an NVIDIA driver from R580 on and a Turing or newer card.
pub fn wants_gpu_pack(nvidia: &[NvidiaGpu], device: Device) -> bool {
    device != Device::Cpu
        && nvidia
            .iter()
            .any(|g| g.driver_major >= 580 && g.compute_cap >= 7.5)
}

/// Whether Laya holds GPU memory, and so has to stop while a game runs.
pub fn uses_gpu(device: Device) -> bool {
    device != Device::Cpu
}

// ------------------------------------------------------------------ install

/// `C:\LLM\ollaya\install.json`: what the app installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installed {
    pub version: String,
    /// The NVIDIA GPU pack is installed
    pub gpu_pack: bool,
    /// The install asked for the GPU pack (it may have been missing from the release)
    pub gpu_wanted: bool,
}

impl Installed {
    pub fn parse(text: &str) -> Option<Installed> {
        serde_json::from_str(text).ok()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("record serializes")
    }
}

/// Ollaya has to be installed (or reinstalled to add the GPU pack).
pub fn needs_install(record: Option<&Installed>, exe_exists: bool, want_gpu: bool) -> bool {
    match record {
        Some(r) => !exe_exists || (want_gpu && !r.gpu_wanted),
        None => true,
    }
}

/// What to download from a release.
#[derive(Debug)]
pub struct Plan<'a> {
    pub version: Version,
    pub archive: &'a Asset,
    pub sums: &'a Asset,
    /// The GPU pack, when wanted and in the release
    pub gpu: Option<&'a Asset>,
    /// The GPU pack's file list, to keep an unchanged installed pack
    pub gpu_files: Option<&'a Asset>,
}

pub fn plan(release: &Release, want_gpu: bool) -> Result<Plan<'_>> {
    let version = release.version()?;
    let archive = release
        .asset(ARCHIVE)
        .with_context(|| format!("Ollaya {} has no Windows build", release.tag_name))?;
    let sums = release
        .asset(SUMS)
        .with_context(|| format!("Ollaya {} has no {SUMS}", release.tag_name))?;
    let gpu = want_gpu.then(|| release.asset(GPU_ARCHIVE)).flatten();
    Ok(Plan {
        version,
        archive,
        sums,
        gpu,
        gpu_files: gpu.and_then(|_| release.asset(GPU_FILES)),
    })
}

/// A newer Ollaya than the installed one, if the release is a proper one with a Windows build.
pub fn newer_release(release: &Release, installed: &str) -> Option<Version> {
    if release.draft || release.prerelease || release.asset(ARCHIVE).is_none() {
        return None;
    }
    let v = release.version().ok()?;
    let have = Version::parse(installed.trim_start_matches('v')).ok();
    (v.pre.is_empty() && have.is_none_or(|h| v > h)).then_some(v)
}

/// `sha256sum` output (`<hex>  <name>` or `<hex> *<name>`) -> name -> lower-case hex.
pub fn parse_sums(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let (hex, name) = l.trim().split_once(char::is_whitespace)?;
            let name = name.trim().trim_start_matches('*');
            (hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) && !name.is_empty())
                .then(|| (name.to_string(), hex.to_ascii_lowercase()))
        })
        .collect()
}

/// The expected sha256 of a release archive. `sha256sum.txt` must list it; GitHub's own digest,
/// when the asset has one, must agree.
pub fn expected_sha256(asset: &Asset, sums: &HashMap<String, String>) -> Result<String> {
    let Some(want) = sums.get(&asset.name) else {
        bail!("{} is not listed in {SUMS}", asset.name);
    };
    if let Some(d) = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
    {
        if !d.eq_ignore_ascii_case(want) {
            bail!("{SUMS} and GitHub disagree about {}", asset.name);
        }
    }
    Ok(want.clone())
}

// ------------------------------------------------------------------ API

/// Whether `GET /api/tags` lists `model`; `None` if the answer isn't a model list.
pub fn tags_have(json: &str, model: &str) -> Option<bool> {
    let v: Value = serde_json::from_str(json).ok()?;
    let want = canonical_name(model);
    Some(
        v.get("models")?
            .as_array()?
            .iter()
            .filter_map(|m| m.get("name").and_then(Value::as_str))
            .any(|n| canonical_name(n) == want),
    )
}

/// `POST /api/pull`'s body.
pub fn pull_body(model: &str) -> String {
    serde_json::json!({ "model": model, "stream": true }).to_string()
}

/// `POST /api/decide` without a state: loads the model and applies `keep_alive`.
pub fn load_body(model: &str, keep_alive: &str) -> String {
    serde_json::json!({ "model": model, "keep_alive": keep_alive }).to_string()
}

/// The message of an Ollaya error body (`{"error": "...", "code": "..."}`).
pub fn error_message(body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(body).ok()?;
    let msg = v.get("error")?.as_str()?;
    Some(match v.get("code").and_then(Value::as_str) {
        Some(code) => format!("{msg} ({code})"),
        None => msg.to_string(),
    })
}

/// Progress of a streamed `/api/pull`: one JSON object per line.
#[derive(Debug, Default)]
pub struct PullProgress {
    layers: BTreeMap<String, (u64, u64)>,
    pub status: String,
    pub success: bool,
    pub error: Option<String>,
}

impl PullProgress {
    pub fn feed(&mut self, line: &str) {
        let Ok(v) = serde_json::from_str::<Value>(line.trim()) else {
            return;
        };
        if v.get("error").is_some() {
            self.error = error_message(line.trim());
            return;
        }
        if let Some(s) = v.get("status").and_then(Value::as_str) {
            self.status = s.to_string();
            self.success |= s == "success";
        }
        if let (Some(d), Some(t)) = (
            v.get("digest").and_then(Value::as_str),
            v.get("total").and_then(Value::as_u64),
        ) {
            let c = v.get("completed").and_then(Value::as_u64).unwrap_or(0);
            self.layers.insert(d.to_string(), (c.min(t), t));
        }
    }

    /// (bytes done, bytes total) over every layer seen so far.
    pub fn bytes(&self) -> (u64, u64) {
        self.layers
            .values()
            .fold((0, 0), |(d, t), (c, n)| (d + c, t + n))
    }
}

// ------------------------------------------------------------------ state machine

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// `RunLaya` is off
    Disabled,
    Off,
    Paused,
    Installing,
    Starting,
    /// Pulling the model
    Downloading,
    /// Loading the model into memory
    Loading,
    Ready,
    Error(String),
}

impl Status {
    pub fn tone(&self) -> Tone {
        match self {
            Status::Disabled | Status::Off => Tone::Off,
            Status::Paused => Tone::Paused,
            Status::Installing | Status::Starting | Status::Downloading | Status::Loading => {
                Tone::Loading
            }
            Status::Ready => Tone::Running,
            Status::Error(_) => Tone::Error,
        }
    }

    pub fn text(&self, model: &str) -> String {
        match self {
            Status::Disabled => "Not running (turn it on in settings)".into(),
            Status::Off => "Off".into(),
            Status::Paused => "Paused while gaming".into(),
            Status::Installing => "Installing Ollaya...".into(),
            Status::Starting => "Starting...".into(),
            Status::Downloading => format!("Downloading {model}..."),
            Status::Loading => format!("Loading {model}..."),
            Status::Ready => format!("Ready - {model}"),
            Status::Error(e) => format!("Error - {e}"),
        }
    }
}

/// Background work the worker runs one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Job {
    Install,
    Pull,
    Load,
}

impl Job {
    pub fn what(self) -> &'static str {
        match self {
            Job::Install => "install",
            Job::Pull => "download",
            Job::Load => "load",
        }
    }
}

/// Why Laya must not run right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hold {
    None,
    /// The LLM is turned off (the switch covers both)
    Off,
    /// The LLM is paused for a game and Laya uses the GPU
    Game,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop(String),
    Run(Job),
}

#[derive(Debug, Clone)]
pub struct Inputs {
    pub now: Instant,
    pub enabled: bool,
    pub hold: Hold,
    /// Settings that can't work (see [`settings_problem`])
    pub problem: Option<String>,
    pub needs_install: bool,
    /// The background job in flight
    pub job: Option<Job>,
    /// Our ollaya.exe processes exist
    pub running: bool,
    /// The port answers as Ollaya
    pub healthy: bool,
    /// From `/api/tags`; `None` = couldn't tell
    pub model_present: Option<bool>,
    /// The model was loaded since the daemon started
    pub warmed: bool,
    pub model: String,
    pub port: u16,
}

#[derive(Debug, Default)]
pub struct Output {
    pub actions: Vec<Action>,
    pub popup: Option<crate::state::Popup>,
    pub logs: Vec<String>,
}

/// First retry after a failed job; doubles with each failure, up to an hour.
pub const RETRY_FIRST: Duration = Duration::from_secs(120);
const RETRY_MAX: Duration = Duration::from_secs(3600);

#[derive(Debug)]
struct Failure {
    message: String,
    count: u32,
    at: Instant,
}

#[derive(Debug)]
pub struct Machine {
    status: Status,
    failed: u32,
    pending: bool,
    failures: HashMap<Job, Failure>,
    /// Say "Laya ready" once it's up after an install or a download
    announce: bool,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            status: Status::Starting,
            failed: 0,
            pending: false,
            failures: HashMap::new(),
            announce: false,
        }
    }
}

impl Machine {
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Clear crash counting and failed jobs (Restart Laya, turn on, settings change).
    pub fn reset(&mut self) {
        self.failed = 0;
        self.pending = false;
        self.failures.clear();
    }

    /// The daemon could not be launched at all.
    pub fn start_failed(&mut self) {
        self.pending = false;
        self.failed = 3;
    }

    /// A background job failed: show the error, retry later.
    pub fn job_failed(&mut self, job: Job, message: String, now: Instant) {
        let count = self.failures.get(&job).map_or(1, |f| f.count + 1);
        self.failures.insert(
            job,
            Failure {
                message,
                count,
                at: now,
            },
        );
    }

    pub fn job_succeeded(&mut self, job: Job) {
        self.failures.remove(&job);
    }

    /// The error of a failed job that isn't due for a retry yet.
    fn blocked(&self, job: Job, now: Instant) -> Option<String> {
        let f = self.failures.get(&job)?;
        let wait = RETRY_FIRST
            .saturating_mul(1u32 << (f.count - 1).min(10))
            .min(RETRY_MAX);
        (now.duration_since(f.at) < wait).then(|| {
            format!(
                "{} failed: {} (retrying in {} min)",
                job.what(),
                f.message,
                (wait - now.duration_since(f.at)).as_secs().div_ceil(60)
            )
        })
    }

    /// `job` should run now, unless another job is busy or it failed recently.
    fn want(&mut self, job: Job, i: &Inputs, busy: Status, out: &mut Output) {
        if i.job == Some(job) {
            self.set(busy, i, out);
        } else if let Some(e) = self.blocked(job, i.now) {
            self.set(Status::Error(e), i, out);
        } else {
            if i.job.is_none() {
                out.actions.push(Action::Run(job));
            }
            self.set(busy, i, out);
        }
    }

    pub fn step(&mut self, i: &Inputs) -> Output {
        let mut out = Output::default();
        let stop = |out: &mut Output, why: &str| {
            if i.running {
                out.actions.push(Action::Stop(why.into()));
            }
        };
        if !i.enabled {
            stop(&mut out, "Laya turned off in settings");
            self.pending = false;
            self.set(Status::Disabled, i, &mut out);
            return out;
        }
        if let Some(p) = &i.problem {
            stop(&mut out, "settings problem");
            self.pending = false;
            self.set(Status::Error(p.clone()), i, &mut out);
            return out;
        }
        match i.hold {
            Hold::Off => {
                stop(&mut out, "turned off");
                self.pending = false;
                self.set(Status::Off, i, &mut out);
                return out;
            }
            Hold::Game => {
                stop(&mut out, "gaming");
                self.pending = false;
                self.set(Status::Paused, i, &mut out);
                return out;
            }
            Hold::None => {}
        }
        if i.job == Some(Job::Install) || i.needs_install {
            // The files are being replaced: the daemon must not run.
            stop(&mut out, "installing Ollaya");
            self.pending = false;
            self.announce = true;
            self.want(Job::Install, i, Status::Installing, &mut out);
            return out;
        }
        if !i.running {
            if i.healthy {
                self.set(
                    Status::Error(format!(
                        "port {} is used by another program (another Ollaya?)",
                        i.port
                    )),
                    i,
                    &mut out,
                );
                return out;
            }
            if self.pending {
                self.failed += 1;
                self.pending = false;
                out.logs
                    .push(format!("Ollaya exited before ready (fail {})", self.failed));
            }
            if self.failed >= 3 {
                self.set(
                    Status::Error("Ollaya keeps crashing (see laya.log)".into()),
                    i,
                    &mut out,
                );
                return out;
            }
            out.actions.push(Action::Start);
            self.pending = true;
            self.set(Status::Starting, i, &mut out);
            return out;
        }
        if !i.healthy {
            self.set(Status::Starting, i, &mut out);
            return out;
        }
        if self.pending {
            self.pending = false;
            out.logs.push("Ollaya up".into());
        }
        match i.model_present {
            None => {
                self.set(Status::Starting, i, &mut out);
                return out;
            }
            Some(false) => {
                self.announce = true;
                self.want(Job::Pull, i, Status::Downloading, &mut out);
                return out;
            }
            Some(true) => {}
        }
        if !i.warmed {
            self.want(Job::Load, i, Status::Loading, &mut out);
            return out;
        }
        self.failed = 0;
        self.set(Status::Ready, i, &mut out);
        out
    }

    fn set(&mut self, status: Status, i: &Inputs, out: &mut Output) {
        if self.status == status {
            return;
        }
        let tone = status.tone();
        out.popup = match &status {
            Status::Ready if self.announce => {
                self.announce = false;
                Some(crate::state::Popup {
                    title: "Laya ready".into(),
                    subtitle: format!("{}  -  port {}", i.model, i.port),
                    tone,
                })
            }
            // Only new errors (a retry countdown changes the text, not the problem)
            Status::Error(e) if !matches!(self.status, Status::Error(_)) => {
                Some(crate::state::Popup {
                    title: "Laya error".into(),
                    subtitle: e.clone(),
                    tone,
                })
            }
            _ => None,
        };
        self.status = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::Asset;

    fn asset(name: &str, digest: Option<&str>) -> Asset {
        Asset {
            name: name.into(),
            browser_download_url: format!("https://dl/{name}"),
            size: 10,
            digest: digest.map(str::to_owned),
        }
    }

    fn release(tag: &str, names: &[&str]) -> Release {
        Release {
            tag_name: tag.into(),
            html_url: String::new(),
            draft: false,
            prerelease: false,
            assets: names.iter().map(|n| asset(n, None)).collect(),
        }
    }

    #[test]
    fn model_names() {
        for good in [
            "laya",
            "laya:en",
            "laya:multilingual",
            "laya:en-fp32",
            "acme/triage",
            "acme/triage:v1.2",
            "decider:0.8b",
            "_x",
        ] {
            assert!(is_valid_model_name(good), "{good}");
        }
        for bad in [
            "",
            ":en",
            "laya:",
            "laya:en:x",
            "evil.com/laya",
            "ollaya.dev/library/laya",
            "http://127.0.0.1:8123/library/laya:en",
            "a/b/c",
            "-laya",
            "laya en",
            "laya;calc",
            "laya\n",
            "../laya",
            &"a".repeat(81),
        ] {
            assert!(!is_valid_model_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn canonical_names() {
        assert_eq!(canonical_name("laya"), "laya:latest");
        assert_eq!(canonical_name(" Laya:EN "), "laya:en");
        assert_eq!(canonical_name("acme/triage"), "acme/triage:latest");
    }

    #[test]
    fn keep_alive_values() {
        for good in [
            "-1", "0", "300", "1.5", "5m", "1h30m", "300ms", "1.5h", "-5m", "0s", "2h0m0s", "10us",
            "10µs",
        ] {
            assert!(is_valid_keep_alive(good), "{good}");
        }
        for bad in [
            "", "-", "m", "5x", "5 m", "5m ", "1.2.3", "forever", "5m-1s", "--1", "1e3", "NaN",
            "inf", "5M",
        ] {
            assert!(!is_valid_keep_alive(bad), "{bad:?}");
        }
        for (v, _) in KEEP_ALIVE_PRESETS {
            assert!(is_valid_keep_alive(v), "{v}");
        }
        for zero in ["0", "0s", "0m0s", "0.0", "-0"] {
            assert!(unloads_right_away(zero), "{zero}");
        }
        for not in ["-1", "5m", "0.5s", "1h0m", "300"] {
            assert!(!unloads_right_away(not), "{not}");
        }
        assert_eq!(keep_alive_label("-1"), "Always (until paused or off)");
        assert_eq!(keep_alive_label("-5m"), "Always (until paused or off)");
        assert_eq!(keep_alive_label("2h"), "2h after the last request");
    }

    #[test]
    fn presets_are_valid_names() {
        for (m, _) in MODELS {
            assert!(is_valid_model_name(m), "{m}");
        }
        assert!(is_valid_model_name(DEFAULT_MODEL));
        assert!(is_valid_keep_alive(DEFAULT_KEEP_ALIVE));
    }

    #[test]
    fn devices() {
        for (d, _) in Device::ALL {
            assert_eq!(Device::parse(d.as_str()), Some(d));
            assert_eq!(serde_json::to_value(d).unwrap(), d.as_str());
        }
        assert_eq!(Device::parse("CUDA"), None);
        assert_eq!(Device::parse("metal"), None);
        assert!(uses_gpu(Device::Auto) && uses_gpu(Device::Cuda) && !uses_gpu(Device::Cpu));
    }

    #[test]
    fn daemon_environment() {
        let s = Settings::default();
        let env: HashMap<_, _> = serve_env(&s, Path::new(r"C:\LLM\ollaya\models"))
            .into_iter()
            .collect();
        assert_eq!(env["OLLAYA_HOST"], "127.0.0.1:11435");
        assert_eq!(env["OLLAYA_MODELS"], r"C:\LLM\ollaya\models");
        assert_eq!(env["OLLAYA_KEEP_ALIVE"], "-1");
        assert_eq!(env["OLLAYA_DEVICE"], "auto");
        assert!(!env.contains_key("OLLAYA_API_KEY"));
        let s = Settings {
            listen_host: "0.0.0.0".into(),
            laya_port: 12000,
            api_key: "k".into(),
            laya_device: Device::Cpu,
            laya_keep_alive: "5m".into(),
            ..Default::default()
        };
        let env: HashMap<_, _> = serve_env(&s, Path::new("m")).into_iter().collect();
        assert_eq!(env["OLLAYA_HOST"], "0.0.0.0:12000");
        assert_eq!(env["OLLAYA_API_KEY"], "k");
        assert_eq!(env["OLLAYA_DEVICE"], "cpu");
        assert_eq!(env["OLLAYA_KEEP_ALIVE"], "5m");
        assert_eq!(health_url(&s), "http://127.0.0.1:12000/");
        assert_eq!(base_url(&s), "http://127.0.0.1:12000");
    }

    #[test]
    fn port_clash_is_a_problem() {
        assert_eq!(settings_problem(&Settings::default()), None);
        let s = Settings {
            laya_port: 8080,
            ..Default::default()
        };
        assert!(settings_problem(&s).unwrap().contains("8080"));
    }

    #[test]
    fn restart_worthy_changes() {
        let a = Settings::default();
        let edits: [fn(&mut Settings); 7] = [
            |s| s.run_laya = true,
            |s| s.laya_model = "laya:en".into(),
            |s| s.laya_port = 12000,
            |s| s.laya_device = Device::Cpu,
            |s| s.laya_keep_alive = "5m".into(),
            |s| s.listen_host = "0.0.0.0".into(),
            |s| s.api_key = "k".into(),
        ];
        for e in edits {
            let mut b = a.clone();
            e(&mut b);
            assert!(laya_changed(&a, &b));
        }
        let mut b = a.clone();
        b.model = "x.gguf".into();
        b.context = 4096;
        b.popups = false;
        assert!(!laya_changed(&a, &b));
    }

    fn gpu(driver: u32, cc: f32) -> NvidiaGpu {
        NvidiaGpu {
            name: "RTX".into(),
            driver_major: driver,
            compute_cap: cc,
            memory_mib: 8192,
        }
    }

    #[test]
    fn gpu_pack_needs_cuda_13() {
        assert!(wants_gpu_pack(&[gpu(580, 8.9)], Device::Auto));
        assert!(wants_gpu_pack(&[gpu(600, 7.5)], Device::Cuda));
        assert!(!wants_gpu_pack(&[gpu(580, 8.9)], Device::Cpu));
        assert!(
            !wants_gpu_pack(&[gpu(566, 8.9)], Device::Auto),
            "old driver"
        );
        assert!(!wants_gpu_pack(&[gpu(580, 6.1)], Device::Auto), "Pascal");
        assert!(!wants_gpu_pack(&[], Device::Auto), "AMD / Intel / none");
    }

    #[test]
    fn install_records() {
        let r = Installed {
            version: "0.5.0".into(),
            gpu_pack: false,
            gpu_wanted: false,
        };
        assert_eq!(Installed::parse(&r.to_json()), Some(r.clone()));
        assert_eq!(Installed::parse("garbage"), None);
        assert!(needs_install(None, true, false));
        assert!(needs_install(Some(&r), false, false), "exe deleted");
        assert!(!needs_install(Some(&r), true, false));
        assert!(needs_install(Some(&r), true, true), "GPU pack wanted now");
        let asked = Installed {
            gpu_wanted: true,
            ..r
        };
        assert!(
            !needs_install(Some(&asked), true, true),
            "asked before; the release had none"
        );
        assert!(
            !needs_install(Some(&asked), true, false),
            "a pack installed earlier stays"
        );
    }

    #[test]
    fn plans_the_download() {
        let all = [
            ARCHIVE,
            SUMS,
            GPU_ARCHIVE,
            GPU_FILES,
            "ollaya-linux-amd64.tar.zst",
        ];
        let r = release("v0.5.0", &all);
        let p = plan(&r, true).unwrap();
        assert_eq!(p.version, Version::new(0, 5, 0));
        assert_eq!(p.archive.name, ARCHIVE);
        assert_eq!(p.sums.name, SUMS);
        assert_eq!(p.gpu.unwrap().name, GPU_ARCHIVE);
        assert_eq!(p.gpu_files.unwrap().name, GPU_FILES);
        let p = plan(&r, false).unwrap();
        assert!(p.gpu.is_none() && p.gpu_files.is_none());
        let no_gpu = release("v0.5.0", &[ARCHIVE, SUMS]);
        assert!(plan(&no_gpu, true).unwrap().gpu.is_none());
        assert!(plan(&release("v0.5.0", &[SUMS]), false).is_err());
        assert!(
            plan(&release("v0.5.0", &[ARCHIVE]), false).is_err(),
            "unverifiable"
        );
        assert!(plan(&release("nightly", &[ARCHIVE, SUMS]), false).is_err());
    }

    #[test]
    fn newer_releases_only() {
        let r = release("v0.6.0", &[ARCHIVE, SUMS]);
        assert_eq!(newer_release(&r, "0.5.0"), Some(Version::new(0, 6, 0)));
        assert_eq!(newer_release(&r, "v0.5.0"), Some(Version::new(0, 6, 0)));
        assert_eq!(newer_release(&r, "0.6.0"), None);
        assert_eq!(newer_release(&r, "0.10.0"), None, "semver, not strings");
        assert_eq!(
            newer_release(&r, "unknown"),
            Some(Version::new(0, 6, 0)),
            "an unreadable record gets replaced"
        );
        let mut pre = r.clone();
        pre.prerelease = true;
        assert_eq!(newer_release(&pre, "0.5.0"), None);
        assert_eq!(
            newer_release(&release("v0.6.0", &[SUMS]), "0.5.0"),
            None,
            "no Windows build"
        );
        assert_eq!(
            newer_release(&release("v0.7.0-rc.1", &[ARCHIVE]), "0.5.0"),
            None
        );
    }

    #[test]
    fn checksums() {
        let a = "a".repeat(64);
        let b = "B".repeat(64);
        let text =
            format!("{a}  {ARCHIVE}\n{b} *{GPU_ARCHIVE}\nnot a line\n{a}\n1234  short.zip\n");
        let sums = parse_sums(&text);
        assert_eq!(sums.len(), 2);
        assert_eq!(sums[ARCHIVE], a);
        assert_eq!(sums[GPU_ARCHIVE], "b".repeat(64));
        assert_eq!(expected_sha256(&asset(ARCHIVE, None), &sums).unwrap(), a);
        assert!(expected_sha256(&asset(ARCHIVE, Some(&format!("sha256:{a}"))), &sums).is_ok());
        assert!(expected_sha256(
            &asset(ARCHIVE, Some(&format!("sha256:{}", "c".repeat(64)))),
            &sums
        )
        .is_err());
        assert!(expected_sha256(&asset("other.zip", None), &sums).is_err());
    }

    #[test]
    fn tags() {
        let j = r#"{"models":[{"name":"laya:latest","model":"laya:latest"},{"name":"laya:en"}]}"#;
        assert_eq!(tags_have(j, "laya"), Some(true));
        assert_eq!(tags_have(j, "Laya:EN"), Some(true));
        assert_eq!(tags_have(j, "laya:multilingual"), Some(false));
        assert_eq!(tags_have(r#"{"models":[]}"#, "laya"), Some(false));
        assert_eq!(tags_have("Ollaya is running", "laya"), None);
        assert_eq!(tags_have(r#"{"error":"x"}"#, "laya"), None);
    }

    #[test]
    fn request_bodies() {
        let v: Value = serde_json::from_str(&pull_body("laya:en")).unwrap();
        assert_eq!(
            (v["model"].as_str(), v["stream"].as_bool()),
            (Some("laya:en"), Some(true))
        );
        let v: Value = serde_json::from_str(&load_body("laya", "-1")).unwrap();
        assert_eq!(v["keep_alive"], "-1");
        assert!(v.get("state").is_none(), "no state = load only");
        assert_eq!(
            error_message(r#"{"error":"model \"x:latest\" not found","code":"MODEL_NOT_FOUND"}"#)
                .unwrap(),
            r#"model "x:latest" not found (MODEL_NOT_FOUND)"#
        );
        assert_eq!(error_message("nope"), None);
    }

    #[test]
    fn pull_progress_adds_up_layers() {
        let mut p = PullProgress::default();
        for l in [
            r#"{"status":"pulling manifest"}"#,
            r#"{"status":"pulling 2409","digest":"sha256:a","total":318,"completed":318}"#,
            r#"{"status":"pulling 8d32","digest":"sha256:b","total":1000,"completed":0}"#,
            r#"{"status":"pulling 8d32","digest":"sha256:b","total":1000,"completed":400}"#,
            "",
            "garbage",
        ] {
            p.feed(l);
        }
        assert_eq!(p.bytes(), (718, 1318));
        assert!(!p.success && p.error.is_none());
        p.feed(r#"{"status":"pulling 8d32","digest":"sha256:b","total":1000,"completed":1000}"#);
        p.feed(r#"{"status":"verifying sha256 digest"}"#);
        p.feed(r#"{"status":"success"}"#);
        assert_eq!(p.bytes(), (1318, 1318));
        assert!(p.success);
        let mut e = PullProgress::default();
        e.feed(r#"{"error":"blob does not match","code":"DIGEST_MISMATCH"}"#);
        assert_eq!(
            e.error.as_deref(),
            Some("blob does not match (DIGEST_MISMATCH)")
        );
    }

    // ---------------------------------------------------------------- machine

    struct Sim {
        m: Machine,
        i: Inputs,
        present: bool,
    }

    impl Sim {
        fn new() -> Sim {
            Sim {
                m: Machine::default(),
                i: Inputs {
                    now: Instant::now(),
                    enabled: true,
                    hold: Hold::None,
                    problem: None,
                    needs_install: false,
                    job: None,
                    running: false,
                    healthy: false,
                    model_present: None,
                    warmed: false,
                    model: "laya".into(),
                    port: 11435,
                },
                present: true,
            }
        }

        /// Steps and carries out the actions: the daemon comes up healthy right away, jobs
        /// finish (successfully) on the next tick.
        fn tick(&mut self, secs: u64) -> Output {
            self.i.now += Duration::from_secs(secs);
            if let Some(j) = self.i.job.take() {
                match j {
                    Job::Install => self.i.needs_install = false,
                    Job::Pull => self.present = true,
                    Job::Load => self.i.warmed = true,
                }
                self.m.job_succeeded(j);
            }
            self.i.model_present = self.i.healthy.then_some(self.present);
            let out = self.m.step(&self.i);
            for a in &out.actions {
                match a {
                    Action::Start => {
                        self.i.running = true;
                        self.i.healthy = true;
                        self.i.warmed = false;
                    }
                    Action::Stop(_) => {
                        self.i.running = false;
                        self.i.healthy = false;
                        self.i.warmed = false;
                    }
                    Action::Run(j) => self.i.job = Some(*j),
                }
            }
            out
        }

        fn until_ready(&mut self) {
            for _ in 0..10 {
                self.tick(5);
                if *self.m.status() == Status::Ready {
                    return;
                }
            }
            panic!("never ready: {:?}", self.m.status());
        }
    }

    #[test]
    fn disabled_does_nothing() {
        let mut s = Sim::new();
        s.i.enabled = false;
        let out = s.tick(0);
        assert!(out.actions.is_empty());
        assert_eq!(*s.m.status(), Status::Disabled);
        assert_eq!(out.popup, None);
    }

    #[test]
    fn installs_downloads_loads_and_announces() {
        let mut s = Sim::new();
        s.i.needs_install = true;
        s.present = false;
        let out = s.tick(0);
        assert_eq!(out.actions, vec![Action::Run(Job::Install)]);
        assert_eq!(*s.m.status(), Status::Installing);
        let out = s.tick(5);
        assert_eq!(out.actions, vec![Action::Start]);
        assert_eq!(*s.m.status(), Status::Starting);
        let out = s.tick(5);
        assert_eq!(out.actions, vec![Action::Run(Job::Pull)]);
        assert_eq!(*s.m.status(), Status::Downloading);
        let out = s.tick(5);
        assert_eq!(out.actions, vec![Action::Run(Job::Load)]);
        assert_eq!(*s.m.status(), Status::Loading);
        let out = s.tick(5);
        assert_eq!(*s.m.status(), Status::Ready);
        assert_eq!(out.popup.unwrap().title, "Laya ready");
        let out = s.tick(5);
        assert!(out.actions.is_empty() && out.popup.is_none(), "steady");
    }

    #[test]
    fn a_ready_daemon_is_adopted_quietly() {
        // After an app update the daemon is still running with the model loaded.
        let mut s = Sim::new();
        s.i.running = true;
        s.i.healthy = true;
        let out = s.tick(0);
        assert_eq!(
            out.actions,
            vec![Action::Run(Job::Load)],
            "loading is idempotent"
        );
        let out = s.tick(1);
        assert_eq!(*s.m.status(), Status::Ready);
        assert_eq!(out.popup, None, "nothing new to announce");
    }

    #[test]
    fn stops_for_games_and_off_unless_on_the_cpu() {
        let mut s = Sim::new();
        s.until_ready();
        s.i.hold = Hold::Game;
        let out = s.tick(5);
        assert!(matches!(out.actions[..], [Action::Stop(_)]));
        assert_eq!(*s.m.status(), Status::Paused);
        assert_eq!(out.popup, None, "the LLM's popup says it");
        s.i.hold = Hold::None;
        s.until_ready();
        s.i.hold = Hold::Off;
        s.tick(5);
        assert_eq!(*s.m.status(), Status::Off);
        assert!(!s.i.running);
        s.i.hold = Hold::None;
        s.until_ready();
    }

    #[test]
    fn keeps_crashing_then_restart_clears_it() {
        let mut s = Sim::new();
        s.tick(0); // Start
        for _ in 0..3 {
            s.i.running = false; // died
            s.i.healthy = false;
            s.tick(5);
        }
        assert_eq!(
            *s.m.status(),
            Status::Error("Ollaya keeps crashing (see laya.log)".into())
        );
        let out = s.tick(5);
        assert!(out.actions.is_empty(), "no more tries");
        s.m.reset();
        s.until_ready();
    }

    #[test]
    fn failed_jobs_retry_later_with_backoff() {
        let mut s = Sim::new();
        s.present = false;
        s.tick(0); // Start
        let out = s.tick(5);
        assert_eq!(out.actions, vec![Action::Run(Job::Pull)]);
        // The download fails
        s.i.job = None;
        s.m.job_failed(Job::Pull, "offline".into(), s.i.now);
        let out = s.tick(5);
        assert!(out.actions.is_empty());
        assert!(
            matches!(s.m.status(), Status::Error(e) if e.contains("download failed: offline") && e.contains("2 min"))
        );
        assert_eq!(out.popup.unwrap().title, "Laya error");
        let out = s.tick(60);
        assert!(out.actions.is_empty() && out.popup.is_none(), "same error");
        let out = s.tick(60);
        assert_eq!(out.actions, vec![Action::Run(Job::Pull)], "retried");
        s.i.job = None;
        s.m.job_failed(Job::Pull, "offline".into(), s.i.now);
        s.tick(200);
        assert!(
            matches!(s.m.status(), Status::Error(_)),
            "second failure waits 4 min"
        );
        let out = s.tick(41);
        assert_eq!(out.actions, vec![Action::Run(Job::Pull)]);
    }

    #[test]
    fn one_job_at_a_time() {
        let mut s = Sim::new();
        s.i.running = true;
        s.i.healthy = true;
        s.i.model_present = Some(false);
        s.i.job = Some(Job::Load); // something else is busy
        let out = s.m.step(&Inputs {
            model_present: Some(false),
            ..s.i.clone()
        });
        assert!(out.actions.is_empty());
        assert_eq!(*s.m.status(), Status::Downloading);
    }

    #[test]
    fn install_stops_the_daemon_first() {
        let mut s = Sim::new();
        s.until_ready();
        s.i.needs_install = true; // an update
        let out = s.tick(5);
        assert_eq!(
            out.actions,
            vec![
                Action::Stop("installing Ollaya".into()),
                Action::Run(Job::Install)
            ]
        );
        s.until_ready();
    }

    #[test]
    fn someone_else_on_the_port() {
        let mut s = Sim::new();
        s.i.healthy = true; // answers, but it isn't ours
        let out = s.m.step(&s.i);
        assert!(out.actions.is_empty());
        assert!(matches!(s.m.status(), Status::Error(e) if e.contains("port 11435")));
    }

    #[test]
    fn settings_problem_wins() {
        let mut s = Sim::new();
        s.until_ready();
        s.i.problem = Some("LayaPort 8080 is also the LLM's port".into());
        let out = s.tick(5);
        assert!(matches!(out.actions[..], [Action::Stop(_)]));
        assert!(matches!(s.m.status(), Status::Error(_)));
    }

    #[test]
    fn statuses_have_tones_and_text() {
        assert_eq!(Status::Ready.tone(), Tone::Running);
        assert_eq!(Status::Downloading.tone(), Tone::Loading);
        assert_eq!(Status::Disabled.tone(), Tone::Off);
        assert_eq!(Status::Paused.tone(), Tone::Paused);
        assert_eq!(Status::Error("x".into()).tone(), Tone::Error);
        assert_eq!(Status::Ready.text("laya:en"), "Ready - laya:en");
        assert_eq!(Status::Downloading.text("laya"), "Downloading laya...");
    }

    mod props {
        use super::*;
        use proptest::prelude::*;

        #[derive(Debug, Clone)]
        enum Ev {
            Tick(u64),
            Enable(bool),
            Hold(u8),
            Crash,
            JobFails,
            JobDone,
            Reset,
            Install,
            Unpull,
        }

        fn ev() -> impl Strategy<Value = Ev> {
            prop_oneof![
                (0u64..400).prop_map(Ev::Tick),
                any::<bool>().prop_map(Ev::Enable),
                (0u8..3).prop_map(Ev::Hold),
                Just(Ev::Crash),
                Just(Ev::JobFails),
                Just(Ev::JobDone),
                Just(Ev::Reset),
                Just(Ev::Install),
                Just(Ev::Unpull),
            ]
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(500))]

            /// Whatever happens: never runs while disabled, off or held for a game, never
            /// starts a second daemon, never starts one while installing, and says Ready only
            /// when the daemon is up with the model loaded.
            #[test]
            fn safety(events in prop::collection::vec(ev(), 1..80)) {
                let mut s = Sim::new();
                for e in events {
                    match e {
                        Ev::Tick(n) => { s.i.now += Duration::from_secs(n); }
                        Ev::Enable(b) => s.i.enabled = b,
                        Ev::Hold(h) => s.i.hold = [Hold::None, Hold::Off, Hold::Game][h as usize],
                        Ev::Crash => { s.i.running = false; s.i.healthy = false; s.i.warmed = false; }
                        Ev::JobFails => if let Some(j) = s.i.job.take() { s.m.job_failed(j, "x".into(), s.i.now); },
                        Ev::JobDone => {} // the next tick finishes it
                        Ev::Reset => s.m.reset(),
                        Ev::Install => s.i.needs_install = true,
                        Ev::Unpull => s.present = false,
                    }
                    let was_running = s.i.running;
                    s.i.now += Duration::from_secs(1);
                    if let Some(j) = s.i.job.take() {
                        match j {
                            Job::Install => s.i.needs_install = false,
                            Job::Pull => s.present = true,
                            Job::Load => s.i.warmed = true,
                        }
                        s.m.job_succeeded(j);
                    }
                    s.i.model_present = s.i.healthy.then_some(s.present);
                    let out = s.m.step(&s.i);
                    let starts = out.actions.iter().filter(|a| **a == Action::Start).count();
                    let jobs = out.actions.iter().filter(|a| matches!(a, Action::Run(_))).count();
                    prop_assert!(starts <= 1);
                    prop_assert!(jobs <= 1);
                    if starts == 1 {
                        prop_assert!(!was_running, "second daemon");
                        prop_assert!(s.i.enabled && s.i.hold == Hold::None && !s.i.needs_install);
                    }
                    for a in &out.actions {
                        match a {
                            Action::Start => { s.i.running = true; s.i.healthy = true; s.i.warmed = false; }
                            Action::Stop(_) => { s.i.running = false; s.i.healthy = false; s.i.warmed = false; }
                            Action::Run(j) => s.i.job = Some(*j),
                        }
                    }
                    let must_stop = !s.i.enabled || s.i.hold != Hold::None || s.i.needs_install
                        || s.i.job == Some(Job::Install);
                    if must_stop {
                        prop_assert!(!s.i.running, "running while it must not: {:?}", s.m.status());
                    }
                    if *s.m.status() == Status::Ready {
                        prop_assert!(s.i.running && s.i.healthy && s.i.warmed && s.present);
                    }
                }
            }
        }
    }
}
