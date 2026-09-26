//! Ollaya on Windows: installs it from its GitHub releases (every archive checked against the
//! release's `sha256sum.txt`), starts `ollaya serve` with the settings as its environment, and
//! uses its API to pull and load the Laya model. `crate::laya` has the rules.

use super::{net, procs::Procs, sys};
use crate::laya::{self, Installed};
use crate::paths::Paths;
use crate::settings::Settings;
use crate::update::{Asset, Release};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub fn release_api() -> String {
    format!(
        "https://api.github.com/repos/{}/releases/latest",
        laya::REPO
    )
}

pub fn latest_release(api: &str) -> Result<Release> {
    Release::parse(&net::get_text(&net::agent(), api)?)
}

/// The daemon and its model runners: everything running from the Ollaya folder.
pub fn pids(procs: &Procs, p: &Paths) -> Vec<u32> {
    procs.pids_under(&p.ollaya_dir)
}

/// Starts `ollaya serve` in the background, logging to `laya.log`.
pub fn start(p: &Paths, s: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(&p.ollaya_models)
        .map_err(|e| format!("can't create {}: {e}", p.ollaya_models.display()))?;
    let log =
        std::fs::File::create(&p.laya_log).map_err(|e| format!("can't write laya.log: {e}"))?;
    let out = log
        .try_clone()
        .map_err(|e| format!("can't write laya.log: {e}"))?;
    let mut cmd = sys::hidden(&p.ollaya_exe);
    // Only our configuration, never an OLLAYA_* set for the user's own Ollaya.
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("OLLAYA_")
        {
            cmd.env_remove(&k);
        }
    }
    cmd.envs(laya::serve_env(s, &p.ollaya_models))
        .arg("serve")
        .current_dir(&p.ollaya_dir)
        .stdout(out)
        .stderr(log)
        .creation_flags(sys::CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("can't start ollaya.exe: {e}"))?;
    Ok(())
}

// ------------------------------------------------------------------ API

fn api_agent(timeout: Option<Duration>) -> ureq::Agent {
    let cfg = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(5)))
        .timeout_global(timeout)
        .http_status_as_error(false)
        .user_agent(format!("no-drama-llama/{}", env!("CARGO_PKG_VERSION")))
        .build();
    ureq::Agent::new_with_config(cfg)
}

fn auth<B>(r: ureq::RequestBuilder<B>, key: &str) -> ureq::RequestBuilder<B> {
    if key.is_empty() {
        r
    } else {
        r.header("Authorization", format!("Bearer {key}"))
    }
}

/// Whether the configured model is in Ollaya's store; `None` = couldn't ask.
pub fn has_model(agent: &ureq::Agent, s: &Settings) -> Option<bool> {
    let url = format!("{}/api/tags", laya::base_url(s));
    let mut resp = auth(agent.get(&url), &s.api_key).call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let text = resp.body_mut().read_to_string().ok()?;
    laya::tags_have(&text, &s.laya_model)
}

fn post(
    agent: &ureq::Agent,
    s: &Settings,
    path: &str,
    body: String,
) -> Result<ureq::http::Response<ureq::Body>> {
    auth(
        agent.post(format!("{}{path}", laya::base_url(s))),
        &s.api_key,
    )
    .header("Content-Type", "application/json")
    .send(body)
    .context("couldn't reach Ollaya")
}

fn error_of(status: u16, body: &mut ureq::Body) -> anyhow::Error {
    let text = body
        .with_config()
        .limit(1 << 20)
        .read_to_string()
        .unwrap_or_default();
    anyhow::anyhow!(laya::error_message(&text).unwrap_or_else(|| format!("HTTP {status}")))
}

/// Pulls the configured model (a router pulls its targets too). `progress(done, total)` in bytes.
pub fn pull(s: &Settings, mut progress: impl FnMut(u64, u64), cancel: &AtomicBool) -> Result<()> {
    let resp = post(
        &api_agent(None),
        s,
        "/api/pull",
        laya::pull_body(&s.laya_model),
    )?;
    let status = resp.status().as_u16();
    let mut body = resp.into_body();
    if status != 200 {
        return Err(error_of(status, &mut body));
    }
    let mut p = laya::PullProgress::default();
    for line in BufReader::new(body.into_reader()).lines() {
        if cancel.load(Ordering::Relaxed) {
            bail!(net::CANCELLED);
        }
        p.feed(&line.context("the download stopped")?);
        if let Some(e) = &p.error {
            bail!("{e}");
        }
        let (done, total) = p.bytes();
        progress(done, total);
    }
    if !p.success {
        bail!("the download ended early");
    }
    Ok(())
}

/// Loads the configured model now and applies `LayaKeepAlive`, so the first request is fast.
pub fn load(s: &Settings) -> Result<()> {
    // Ollaya's own load deadline is 5 minutes.
    let agent = api_agent(Some(Duration::from_secs(330)));
    let resp = post(
        &agent,
        s,
        "/api/decide",
        laya::load_body(&s.laya_model, &s.laya_keep_alive),
    )?;
    let status = resp.status().as_u16();
    if status != 200 {
        return Err(error_of(status, &mut resp.into_body()));
    }
    Ok(())
}

// ------------------------------------------------------------------ install

/// Downloads `url` to `dest`, resuming as long as each try gets further (a slow link can hit
/// the download's time budget more than once on a 1 GB file).
fn download_resuming(
    url: &str,
    dest: &Path,
    mut progress: impl FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<()> {
    let size = |f: &Path| std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
    let mut have = size(dest);
    for _ in 0..50 {
        match net::download_cancellable(url, dest, &mut progress, Some(cancel)) {
            Ok(()) => return Ok(()),
            Err(e) if cancel.load(Ordering::Relaxed) => return Err(e),
            Err(e) => {
                let now = size(dest);
                if now <= have {
                    return Err(e);
                }
                have = now;
            }
        }
    }
    bail!("download of {url} keeps stopping")
}

/// Whether the installed GPU pack is byte for byte the one this release ships, so the ~1 GB
/// download can be skipped (Ollaya's own installer does the same).
fn gpu_pack_unchanged(
    agent: &ureq::Agent,
    files: &Asset,
    sums: &HashMap<String, String>,
    cuda_dir: &Path,
) -> bool {
    let Ok(want) = laya::expected_sha256(files, sums) else {
        return false;
    };
    let Ok(list) = net::get_bytes(agent, &files.browser_download_url, 1 << 20) else {
        return false;
    };
    if crate::update::sha256_hex(&list) != want
        || std::fs::read(cuda_dir.join("FILES.sha256")).ok().as_deref() != Some(&list[..])
    {
        return false;
    }
    let entries = laya::parse_sums(&String::from_utf8_lossy(&list));
    !entries.is_empty()
        && entries.iter().all(|(name, hex)| {
            !name.contains(['/', '\\', ':'])
                && !name.contains("..")
                && net::file_sha256(&cuda_dir.join(name)).is_ok_and(|h| h.eq_ignore_ascii_case(hex))
        })
}

fn unzip(zip: &Path, into: &Path) -> Result<()> {
    zip::ZipArchive::new(std::fs::File::open(zip)?)?
        .extract(into)
        .with_context(|| format!("couldn't unpack {}", zip.display()))
}

fn remove(path: &Path) -> Result<()> {
    let r = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match r {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| format!("couldn't remove {}", path.display()))
        }
        _ => Ok(()),
    }
}

/// Installs (or updates) Ollaya from the latest release that `api` describes. The daemon must
/// be stopped. The model store (`models\`) is kept.
pub fn install(
    p: &Paths,
    api: &str,
    want_gpu: bool,
    mut progress: impl FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<Installed> {
    let release = latest_release(api)?;
    let plan = laya::plan(&release, want_gpu)?;
    let agent = net::agent();
    let sums = laya::parse_sums(&String::from_utf8_lossy(&net::get_bytes(
        &agent,
        &plan.sums.browser_download_url,
        1 << 20,
    )?));
    let cuda_dir = p.ollaya_dir.join(r"lib\ollaya\cuda_v13");
    let keep_gpu = plan
        .gpu_files
        .is_some_and(|f| gpu_pack_unchanged(&agent, f, &sums, &cuda_dir));
    let mut downloads = vec![plan.archive];
    if let (Some(g), false) = (plan.gpu, keep_gpu) {
        downloads.push(g);
    }

    // Partial downloads resume; one folder per version so an old partial file is never mixed
    // into a new one.
    let dl_root = p.ollaya_dir.join(".download");
    let dl = dl_root.join(plan.version.to_string());
    for e in std::fs::read_dir(&dl_root).into_iter().flatten().flatten() {
        if e.path() != dl {
            let _ = remove(&e.path());
        }
    }
    std::fs::create_dir_all(&dl)?;
    let total: u64 = downloads.iter().map(|a| a.size).sum();
    let mut before = 0u64;
    for a in &downloads {
        let want = laya::expected_sha256(a, &sums)?;
        let dest = dl.join(&a.name);
        download_resuming(
            &a.browser_download_url,
            &dest,
            |d, _| progress(before + d, total),
            cancel,
        )?;
        let got = net::file_sha256(&dest)?;
        if !got.eq_ignore_ascii_case(&want) {
            let _ = std::fs::remove_file(&dest);
            bail!("{} checksum mismatch - the download was deleted", a.name);
        }
        before += a.size;
        progress(before, total);
    }

    let stage = p.ollaya_dir.join(".stage");
    remove(&stage)?;
    unzip(&dl.join(laya::ARCHIVE), &stage)?;
    let gpu_downloaded = downloads.len() > 1;
    if gpu_downloaded {
        unzip(&dl.join(laya::GPU_ARCHIVE), &stage)?;
        if !stage
            .join(r"lib\ollaya\cuda_v13\onnxruntime_providers_cuda.dll")
            .exists()
        {
            bail!("{} has no GPU pack in it", laya::GPU_ARCHIVE);
        }
    }
    if !stage.join(r"bin\ollaya.exe").exists() {
        bail!("{} has no bin\\ollaya.exe in it", laya::ARCHIVE);
    }

    // Swap the new files in. The GPU pack never outlives the build it came with, unless it's
    // byte for byte the one this release ships.
    let lib = p.ollaya_dir.join(r"lib\ollaya");
    if keep_gpu {
        for e in std::fs::read_dir(&cuda_dir).into_iter().flatten().flatten() {
            if e.file_name()
                .to_string_lossy()
                .starts_with("ollaya-runner-")
            {
                let _ = std::fs::remove_file(e.path());
            }
        }
        let notices = p.ollaya_dir.join(r"share\doc\ollaya\cuda_v13");
        let staged = stage.join(r"share\doc\ollaya\cuda_v13");
        if notices.exists() && !staged.exists() {
            std::fs::create_dir_all(staged.parent().unwrap())?;
            std::fs::rename(&notices, &staged)?;
        }
    } else {
        remove(&lib)?;
    }
    for part in ["bin", "share"] {
        remove(&p.ollaya_dir.join(part))?;
        if stage.join(part).exists() {
            std::fs::rename(stage.join(part), p.ollaya_dir.join(part))?;
        }
    }
    if stage.join(r"lib\ollaya").exists() {
        std::fs::create_dir_all(&lib)?;
        for e in std::fs::read_dir(stage.join(r"lib\ollaya"))?.flatten() {
            let target = lib.join(e.file_name());
            remove(&target)?;
            std::fs::rename(e.path(), target)?;
        }
    }
    let _ = remove(&stage);
    let _ = remove(&dl_root);

    let record = Installed {
        version: plan.version.to_string(),
        gpu_pack: keep_gpu || gpu_downloaded,
        gpu_wanted: want_gpu,
    };
    std::fs::write(p.ollaya_dir.join("install.json"), record.to_json())?;
    Ok(record)
}
