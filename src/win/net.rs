//! HTTP: GitHub/Hugging Face API calls, resumable verified downloads, the /health check.
//! TLS goes through Windows' own stack (SChannel), so no bundled certificates.

use anyhow::{bail, Context, Result};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use ureq::tls::{TlsConfig, TlsProvider};

fn agent_with(global: Option<Duration>) -> ureq::Agent {
    let cfg = ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .build(),
        )
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_global(global)
        .user_agent(format!("no-drama-llama/{}", env!("CARGO_PKG_VERSION")))
        .build();
    ureq::Agent::new_with_config(cfg)
}

/// For API calls and small files.
pub fn agent() -> ureq::Agent {
    agent_with(Some(Duration::from_secs(60)))
}

/// For the 700 ms readiness probe against the local server.
pub fn health_agent() -> ureq::Agent {
    agent_with(Some(Duration::from_millis(700)))
}

pub fn get_text(agent: &ureq::Agent, url: &str) -> Result<String> {
    let mut resp = agent
        .get(url)
        .header("Accept", "application/json")
        .call()
        .with_context(|| format!("GET {url}"))?;
    Ok(resp
        .body_mut()
        .with_config()
        .limit(16 * 1024 * 1024)
        .read_to_string()?)
}

pub fn get_bytes(agent: &ureq::Agent, url: &str, limit: u64) -> Result<Vec<u8>> {
    let mut resp = agent
        .get(url)
        .call()
        .with_context(|| format!("GET {url}"))?;
    Ok(resp.body_mut().with_config().limit(limit).read_to_vec()?)
}

pub fn is_healthy(agent: &ureq::Agent, url: &str) -> bool {
    agent.get(url).call().is_ok()
}

/// Downloads `url` to `dest`, resuming a partial file. `progress(done, total)` is called
/// about once a second.
pub fn download(url: &str, dest: &Path, progress: impl FnMut(u64, u64)) -> Result<()> {
    download_cancellable(url, dest, progress, None)
}

pub const CANCELLED: &str = "download cancelled";

/// Like [`download`], stopping (and keeping the partial file for resuming) when `cancel` is set.
pub fn download_cancellable(
    url: &str,
    dest: &Path,
    mut progress: impl FnMut(u64, u64),
    cancel: Option<&AtomicBool>,
) -> Result<()> {
    // No global timeout: a 17 GB model takes a while. Stalls are caught by the body timeout.
    let cfg = ureq::Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .build(),
        )
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_recv_body(Some(Duration::from_secs(120)))
        .http_status_as_error(false)
        .user_agent(format!("no-drama-llama/{}", env!("CARGO_PKG_VERSION")))
        .build();
    let agent = ureq::Agent::new_with_config(cfg);
    let have = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let mut req = agent.get(url);
    if have > 0 {
        req = req.header("Range", format!("bytes={have}-"));
    }
    let resp = req.call().with_context(|| format!("GET {url}"))?;
    let status = resp.status().as_u16();
    let (mut done, append) = match status {
        206 => (have, true),
        200 => (0, false),
        416 => return Ok(()), // already complete
        s => bail!("download failed: HTTP {s} for {url}"),
    };
    let len: u64 = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok()?.parse().ok())
        .unwrap_or(0);
    let total = if len > 0 { done + len } else { 0 };
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(dest)?;
    let mut reader = resp.into_body().into_reader();
    let mut buf = vec![0u8; 1 << 20];
    let mut last = Instant::now();
    loop {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            file.flush()?;
            bail!(CANCELLED);
        }
        let n = reader
            .read(&mut buf)
            .context("download interrupted - run it again to resume")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        done += n as u64;
        if last.elapsed() >= Duration::from_secs(1) {
            progress(done, total);
            last = Instant::now();
        }
    }
    file.flush()?;
    progress(done, total);
    if total > 0 && done != total {
        bail!("download incomplete ({done} of {total} bytes) - run it again to resume");
    }
    Ok(())
}

pub fn file_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
