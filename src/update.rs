//! Self-update from GitHub Releases (platform-independent part).
//!
//! A release carries `no-drama-llama.exe` and `no-drama-llama.exe.minisig`: a minisign
//! signature whose trusted comment is `no-drama-llama <version>`. The app only installs an
//! update that is newer than itself, signed by the key baked in at build time
//! (`NDL_UPDATE_PUBKEY`), and whose signed version matches the release. Builds without a key
//! only report that an update exists.

use anyhow::{anyhow, bail, Context, Result};
use semver::Version;
use serde::Deserialize;

pub const EXE_ASSET: &str = "no-drama-llama.exe";
pub const SIG_ASSET: &str = "no-drama-llama.exe.minisig";

/// minisign public key (base64) for verifying updates, baked in from the build environment.
/// CI passes an empty string when the repository variable isn't set: that means "no key".
pub fn update_pubkey() -> Option<&'static str> {
    option_env!("NDL_UPDATE_PUBKEY")
        .map(str::trim)
        .filter(|k| !k.is_empty())
}

pub fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("crate version is semver")
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
    /// `sha256:<hex>` (GitHub adds this for new uploads)
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn parse(json: &str) -> Result<Release> {
        serde_json::from_str(json).context("unexpected GitHub release JSON")
    }

    pub fn version(&self) -> Result<Version> {
        let v = self.tag_name.trim_start_matches('v');
        Version::parse(v).with_context(|| format!("release tag {} is not a version", self.tag_name))
    }

    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == name)
    }
}

/// The release to update to, if it is a proper newer release with the app assets.
pub fn update_candidate(release: &Release, current: &Version) -> Option<Version> {
    if release.draft || release.prerelease {
        return None;
    }
    let v = release.version().ok()?;
    // pre-release versions (2.1.0-beta) only come through as explicit GitHub pre-releases, which we skip
    (v.pre.is_empty()
        && v > *current
        && release.asset(EXE_ASSET).is_some()
        && release.asset(SIG_ASSET).is_some())
    .then_some(v)
}

pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Checks GitHub's digest (transport integrity) if the asset has one.
pub fn check_digest(asset: &Asset, data: &[u8]) -> Result<()> {
    if let Some(d) = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
    {
        let actual = sha256_hex(data);
        if !actual.eq_ignore_ascii_case(d) {
            bail!(
                "{} checksum mismatch (expected {d}, got {actual})",
                asset.name
            );
        }
    }
    Ok(())
}

/// Verifies the minisign signature and that it was made for exactly `version`.
pub fn verify_signature(
    data: &[u8],
    sig_text: &str,
    pubkey_b64: &str,
    version: &Version,
) -> Result<()> {
    use minisign_verify::{PublicKey, Signature};
    let pk = PublicKey::from_base64(pubkey_b64.trim())
        .map_err(|e| anyhow!("bad update public key: {e}"))?;
    let sig = Signature::decode(sig_text).map_err(|e| anyhow!("bad signature file: {e}"))?;
    pk.verify(data, &sig, false)
        .map_err(|e| anyhow!("signature check failed: {e}"))?;
    let expected = format!("no-drama-llama {version}");
    if sig.trusted_comment().trim() != expected {
        bail!(
            "signature is for '{}', expected '{expected}'",
            sig.trusted_comment().trim()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUB: &str = include_str!("../tests/fixtures/test-key.pub");
    const PAYLOAD: &[u8] = include_bytes!("../tests/fixtures/payload.bin");
    const SIG: &str = include_str!("../tests/fixtures/payload.bin.minisig");

    fn release(tag: &str, assets: &[&str]) -> Release {
        Release {
            tag_name: tag.into(),
            html_url: String::new(),
            draft: false,
            prerelease: false,
            assets: assets
                .iter()
                .map(|n| Asset {
                    name: n.to_string(),
                    browser_download_url: format!("https://x/{n}"),
                    size: 1,
                    digest: None,
                })
                .collect(),
        }
    }

    #[test]
    fn parses_github_json() {
        let r = Release::parse(
            r#"{"tag_name":"v2.1.0","html_url":"https://github.com/x","draft":false,"prerelease":false,
               "assets":[{"name":"no-drama-llama.exe","browser_download_url":"https://dl/exe","size":5,
                          "digest":"sha256:abc","extra":1}],"body":"notes"}"#,
        )
        .unwrap();
        assert_eq!(r.version().unwrap(), Version::new(2, 1, 0));
        assert_eq!(
            r.asset(EXE_ASSET).unwrap().digest.as_deref(),
            Some("sha256:abc")
        );
    }

    #[test]
    fn only_newer_complete_releases() {
        let cur = Version::new(2, 0, 0);
        let both = [EXE_ASSET, SIG_ASSET];
        assert_eq!(
            update_candidate(&release("v2.1.0", &both), &cur),
            Some(Version::new(2, 1, 0))
        );
        assert_eq!(update_candidate(&release("v2.0.0", &both), &cur), None);
        assert_eq!(update_candidate(&release("v1.9.9", &both), &cur), None);
        assert_eq!(
            update_candidate(&release("v2.1.0", &[EXE_ASSET]), &cur),
            None,
            "unsigned"
        );
        assert_eq!(update_candidate(&release("nightly", &both), &cur), None);
        let mut pre = release("v3.0.0", &both);
        pre.prerelease = true;
        assert_eq!(update_candidate(&pre, &cur), None);
    }

    #[test]
    fn signature_ok() {
        verify_signature(PAYLOAD, SIG, PUB, &Version::new(2, 1, 0)).unwrap();
    }

    #[test]
    fn signature_rejects_tampering_and_version_swap() {
        let mut bad = PAYLOAD.to_vec();
        bad[0] ^= 1;
        assert!(verify_signature(&bad, SIG, PUB, &Version::new(2, 1, 0)).is_err());
        // a genuinely signed older build re-uploaded under a newer tag
        let e = verify_signature(PAYLOAD, SIG, PUB, &Version::new(9, 0, 0)).unwrap_err();
        assert!(
            e.to_string().contains("expected 'no-drama-llama 9.0.0'"),
            "{e}"
        );
        // someone else's key
        let other = "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
        assert!(verify_signature(PAYLOAD, SIG, other, &Version::new(2, 1, 0)).is_err());
    }

    #[test]
    fn digest_check() {
        let mut a = release("v2.1.0", &[EXE_ASSET]).assets.remove(0);
        a.digest = Some(format!("sha256:{}", sha256_hex(b"abc")));
        check_digest(&a, b"abc").unwrap();
        assert!(check_digest(&a, b"abd").is_err());
        a.digest = None;
        check_digest(&a, b"anything").unwrap();
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use proptest::prelude::*;

    const PUB: &str = include_str!("../tests/fixtures/test-key.pub");
    const PAYLOAD: &[u8] = include_bytes!("../tests/fixtures/payload.bin");
    const SIG: &str = include_str!("../tests/fixtures/payload.bin.minisig");

    fn rel(tag: &str) -> Release {
        Release::parse(&format!(
            r#"{{"tag_name":"{tag}","assets":[{{"name":"{EXE_ASSET}","browser_download_url":"u"}},{{"name":"{SIG_ASSET}","browser_download_url":"u"}}]}}"#
        ))
        .unwrap()
    }

    #[test]
    fn version_parsing() {
        assert_eq!(rel("v2.1.0").version().unwrap(), Version::new(2, 1, 0));
        assert_eq!(rel("2.1.0").version().unwrap(), Version::new(2, 1, 0));
        assert!(rel("v2.1").version().is_err());
        assert!(rel("latest").version().is_err());
    }

    #[test]
    fn semver_ordering_not_string_ordering() {
        let cur = Version::new(2, 9, 0);
        assert_eq!(
            update_candidate(&rel("v2.10.0"), &cur),
            Some(Version::new(2, 10, 0))
        );
        assert_eq!(
            update_candidate(&rel("v10.0.0"), &Version::new(9, 0, 0)),
            Some(Version::new(10, 0, 0))
        );
    }

    #[test]
    fn prerelease_versions_and_drafts_are_skipped() {
        let cur = Version::new(2, 0, 0);
        assert_eq!(update_candidate(&rel("v2.1.0-beta.1"), &cur), None);
        let mut d = rel("v3.0.0");
        d.draft = true;
        assert_eq!(update_candidate(&d, &cur), None);
    }

    #[test]
    fn current_version_is_the_crate_version() {
        assert_eq!(current_version().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn bad_json() {
        assert!(Release::parse("").is_err());
        assert!(Release::parse("[]").is_err());
        assert!(Release::parse(r#"{"message":"API rate limit exceeded"}"#).is_err());
        let r = Release::parse(r#"{"tag_name":"v1.0.0"}"#).unwrap();
        assert!(r.assets.is_empty());
        assert_eq!(update_candidate(&r, &Version::new(0, 1, 0)), None);
    }

    #[test]
    fn signature_edge_cases() {
        let v = Version::new(2, 1, 0);
        assert!(
            verify_signature(PAYLOAD, SIG, &format!("  {PUB}  \n"), &v).is_ok(),
            "whitespace around key is fine"
        );
        assert!(verify_signature(b"", SIG, PUB, &v).is_err());
        assert!(verify_signature(PAYLOAD, "", PUB, &v).is_err());
        assert!(
            verify_signature(PAYLOAD, &SIG[..SIG.len() / 2], PUB, &v).is_err(),
            "truncated signature"
        );
        assert!(verify_signature(PAYLOAD, SIG, "not base64!", &v).is_err());
        assert!(verify_signature(PAYLOAD, SIG, "", &v).is_err());
        // swap the trusted comment: the global signature no longer matches
        let forged = SIG.replace("no-drama-llama 2.1.0", "no-drama-llama 9.9.9");
        assert!(verify_signature(PAYLOAD, &forged, PUB, &Version::new(9, 9, 9)).is_err());
        // appended data
        let mut longer = PAYLOAD.to_vec();
        longer.push(0);
        assert!(verify_signature(&longer, SIG, PUB, &v).is_err());
    }

    #[test]
    fn digest_is_case_insensitive_and_ignores_other_algorithms() {
        let hex = sha256_hex(b"abc");
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let mut a = Asset {
            name: "x".into(),
            browser_download_url: "u".into(),
            size: 3,
            digest: Some(format!("sha256:{}", hex.to_uppercase())),
        };
        check_digest(&a, b"abc").unwrap();
        a.digest = Some("sha512:zzz".into());
        check_digest(&a, b"anything").unwrap();
    }

    proptest! {
        #[test]
        fn verification_never_panics(data in prop::collection::vec(any::<u8>(), 0..64), sig in ".{0,300}", key in ".{0,80}") {
            let _ = verify_signature(&data, &sig, &key, &Version::new(1, 0, 0));
        }

        #[test]
        fn release_parsing_never_panics(json in ".{0,300}") {
            if let Ok(r) = Release::parse(&json) {
                let _ = update_candidate(&r, &Version::new(1, 0, 0));
            }
        }

        #[test]
        fn random_payloads_never_verify(data in prop::collection::vec(any::<u8>(), 0..256)) {
            prop_assume!(data != PAYLOAD);
            prop_assert!(verify_signature(&data, SIG, PUB, &Version::new(2, 1, 0)).is_err());
        }
    }
}
