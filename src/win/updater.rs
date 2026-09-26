//! Self-update: check GitHub Releases, download, verify the minisign signature, and swap the
//! running exe (see `crate::update` for the rules).

use super::net;
use crate::paths;
use crate::update::{self, Release, EXE_ASSET, SIG_ASSET};
use anyhow::{anyhow, bail, Context, Result};
use semver::Version;

pub fn latest_release() -> Result<Release> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        paths::REPO
    );
    Release::parse(&net::get_text(&net::agent(), &url)?)
}

/// A newer release, if there is one.
pub fn check() -> Result<Option<(Release, Version)>> {
    let r = latest_release()?;
    Ok(update::update_candidate(&r, &update::current_version()).map(|v| (r, v)))
}

pub fn can_self_update() -> bool {
    update::update_pubkey().is_some()
}

/// Downloads and verifies the new exe, then replaces the running one. Only for the
/// installed copy (never a dev build run from elsewhere).
pub fn apply(release: &Release, version: &Version) -> Result<()> {
    let key = update::update_pubkey().ok_or_else(|| {
        anyhow!("this build has no update signing key - download the new version manually")
    })?;
    let current = std::env::current_exe()?;
    if !current
        .as_os_str()
        .eq_ignore_ascii_case(paths::installed_exe().as_os_str())
    {
        bail!(
            "only the installed copy ({}) updates itself",
            paths::installed_exe().display()
        );
    }
    let exe = release.asset(EXE_ASSET).context("release has no exe")?;
    let sig = release
        .asset(SIG_ASSET)
        .context("release has no signature")?;
    let agent = net::agent();
    let sig_text = String::from_utf8(net::get_bytes(
        &agent,
        &sig.browser_download_url,
        64 * 1024,
    )?)?;
    let data = net::get_bytes(&agent, &exe.browser_download_url, 256 * 1024 * 1024)?;
    update::check_digest(exe, &data)?;
    update::verify_signature(&data, &sig_text, key, version)?;

    // Swap the files entirely inside the admin-only install folder. Windows lets a running exe
    // be renamed (not overwritten) within its volume, so: running -> .old, new -> exe. Never
    // stage or run anything from %TEMP%, which unelevated code can write to.
    let dir = paths::install_dir();
    remove_leftovers();
    let staged = dir.join("no-drama-llama.exe.new");
    std::fs::write(&staged, &data).context("couldn't stage the update")?;
    let old = dir.join(format!("no-drama-llama.{}{OLD_SUFFIX}", std::process::id()));
    if let Err(e) = std::fs::rename(&current, &old) {
        let _ = std::fs::remove_file(&staged);
        return Err(e).context("couldn't move the running exe aside");
    }
    if let Err(e) = std::fs::rename(&staged, &current) {
        let _ = std::fs::rename(&old, &current);
        let _ = std::fs::remove_file(&staged);
        return Err(e).context("couldn't replace the running exe");
    }
    super::install::set_registered_version(version);
    Ok(())
}

/// Suffix of the exes an update moved aside; they're deleted once no longer running.
const OLD_SUFFIX: &str = ".exe.old";

/// Deletes exes left by earlier updates (skipping any still running).
pub fn remove_leftovers() {
    let Ok(entries) = std::fs::read_dir(paths::install_dir()) else {
        return;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_ascii_lowercase();
        if name.starts_with("no-drama-llama.") && name.ends_with(OLD_SUFFIX) {
            let _ = std::fs::remove_file(e.path());
        }
    }
}
