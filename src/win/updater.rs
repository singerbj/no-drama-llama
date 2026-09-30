//! Self-update: read the latest release's `latest.json`, download, verify the minisign
//! signature, and swap the running exe (see `crate::update` for the rules).

use super::net;
use crate::paths;
use crate::update::{self, Offer};
use anyhow::{anyhow, bail, Context, Result};
use semver::Version;

/// The latest release's offer; `None` when it has no `latest.json` (releases before 0.0.8).
pub fn latest_offer() -> Result<Option<Offer>> {
    let url = update::manifest_url();
    match net::get_text(&net::agent(), &url) {
        Ok(json) => Offer::parse(&json).map(Some),
        Err(e) if is_not_found(&e) => Ok(None),
        Err(e) => Err(e),
    }
}

fn is_not_found(e: &anyhow::Error) -> bool {
    e.chain().any(|c| {
        matches!(
            c.downcast_ref::<ureq::Error>(),
            Some(ureq::Error::StatusCode(404))
        )
    })
}

/// A newer release, if there is one.
pub fn check() -> Result<Option<(Offer, Version)>> {
    let Some(offer) = latest_offer()? else {
        return Ok(None);
    };
    Ok(update::update_candidate(&offer, &update::current_version()).map(|v| (offer, v)))
}

pub fn can_self_update() -> bool {
    update::update_pubkey().is_some()
}

/// Downloads and verifies the new exe, then replaces the running one. Only for the
/// installed copy (never a dev build run from elsewhere).
pub fn apply(offer: &Offer, version: &Version) -> Result<()> {
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
    let data = net::get_bytes(&net::agent(), &offer.url, 256 * 1024 * 1024)?;
    update::verify_signature(&data, &offer.signature, key, version)?;

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
