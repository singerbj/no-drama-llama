//! Downloading catalog models from Hugging Face: resumable, every file checked against
//! Hugging Face's size and SHA-256, split models downloaded part by part.

use super::{net, sys};
use crate::catalog::CatalogModel;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub const HF: &str = "https://huggingface.co";

/// (size, sha256) of a file in a Hugging Face repo, from its LFS metadata.
pub fn hf_file_meta(base: &str, repo: &str, path: &str) -> Result<(u64, Option<String>)> {
    let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let url = if dir.is_empty() {
        format!("{base}/api/models/{repo}/tree/main")
    } else {
        format!("{base}/api/models/{repo}/tree/main/{dir}")
    };
    let list: Value = serde_json::from_str(&net::get_text(&net::agent(), &url)?)
        .context("unexpected Hugging Face response")?;
    let e = list
        .as_array()
        .into_iter()
        .flatten()
        .find(|e| e["path"] == path)
        .with_context(|| format!("{path} not found in {repo}"))?;
    let size = e["lfs"]["size"]
        .as_u64()
        .or_else(|| e["size"].as_u64())
        .unwrap_or(0);
    Ok((size, e["lfs"]["oid"].as_str().map(str::to_owned)))
}

/// Downloads `m` into `models_dir`. `progress(done, total)` covers all parts. Files already
/// there with the right size are kept.
///
/// Partial files live in `staging_dir`, which must be admin-only: `models_dir` is writable by
/// the signed-in user, who could turn it into a link, and the elevated app would then create,
/// append to or delete whatever file the link points at. Only a verified file is moved over.
pub fn download_model(
    base: &str,
    m: &CatalogModel,
    models_dir: &Path,
    staging_dir: &Path,
    mut progress: impl FnMut(u64, u64),
    cancel: Option<&AtomicBool>,
) -> Result<()> {
    std::fs::create_dir_all(models_dir)?;
    std::fs::create_dir_all(staging_dir)?;
    let not_a_link = || {
        if sys::is_reparse_point(models_dir) {
            bail!(
                "{} is a link to another folder - make it a normal folder again",
                models_dir.display()
            );
        }
        Ok(())
    };
    not_a_link()?;
    let metas: Vec<(u64, Option<String>)> = m
        .files
        .iter()
        .map(|f| hf_file_meta(base, m.repo, f))
        .collect::<Result<_>>()?;
    let total: u64 = metas.iter().map(|(s, _)| s).sum();
    let mut before = 0u64;
    for (repo_path, (size, sha)) in m.files.iter().zip(&metas) {
        let name = repo_path.rsplit('/').next().unwrap();
        let dest = models_dir.join(name);
        if std::fs::metadata(&dest).is_ok_and(|md| md.len() == *size) {
            before += size;
            progress(before, total);
            continue;
        }
        let part = staging_dir.join(format!("{name}.part"));
        if std::fs::metadata(&part).is_ok_and(|md| md.len() > *size) {
            let _ = std::fs::remove_file(&part); // corrupt: bigger than the real file
        }
        let url = format!("{base}/{}/resolve/main/{repo_path}", m.repo);
        net::download_cancellable(
            &url,
            &part,
            |done, _| progress(before + done, total),
            cancel,
        )?;
        let got = std::fs::metadata(&part)?.len();
        if got != *size {
            bail!("{name}: incomplete ({got} of {size} bytes) - try again to resume");
        }
        if let Some(sha) = sha {
            let actual = net::file_sha256(&part)?;
            if !actual.eq_ignore_ascii_case(sha) {
                let _ = std::fs::remove_file(&part);
                bail!("{name}: checksum mismatch - the download was deleted, try again");
            }
        }
        not_a_link()?;
        std::fs::rename(&part, &dest)?;
        before += size;
        progress(before, total);
    }
    Ok(())
}
