//! Self-update (platform-independent part), and the GitHub release types the llama.cpp and
//! Ollaya installers share.
//!
//! Every release carries `latest.json`, the updater manifest in Tauri's format (the same one
//! rekt-clipz and tunedup read), listing `no-drama-llama.exe` and its minisign signature. The
//! app reads it from `releases/latest/download/latest.json`, a plain download (no GitHub API
//! rate limit; pre-releases are never "latest"). It only installs an update that is newer than
//! itself, signed by the key baked in at build time (`NDL_UPDATE_PUBKEY`), and whose signature
//! names that version (the `version:` field of the trusted comment, Tauri's format), so a signed
//! older build can't be passed off as a newer one. Builds without a key only report that an
//! update exists.

use anyhow::{anyhow, bail, Context, Result};
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

/// The `latest.json` platform this build installs.
pub const TARGET: &str = "windows-x86_64";

/// minisign public key for verifying updates, baked in from the build environment. Any encoding
/// works (a `.pub` file, its base64 as `tauri signer` writes it, or the bare `RW…` line). CI
/// passes an empty string when the repository variable isn't set: that means "no key".
pub fn update_pubkey() -> Option<&'static str> {
    option_env!("NDL_UPDATE_PUBKEY")
        .map(str::trim)
        .filter(|k| !k.is_empty())
}

pub fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("crate version is semver")
}

/// Where the latest release's manifest is.
pub fn manifest_url() -> String {
    format!(
        "https://github.com/{}/releases/latest/download/latest.json",
        crate::paths::REPO
    )
}

#[derive(Debug, Clone, Deserialize)]
struct PlatformEntry {
    url: String,
    signature: String,
}

/// `latest.json`: Tauri's static form (`platforms`), or its single-platform form (`url` and
/// `signature` at the top, what a dynamic update server sends).
#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    platforms: HashMap<String, PlatformEntry>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    signature: Option<String>,
}

/// The update the manifest offers this platform.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    pub version: Version,
    /// Download of the new exe (https).
    pub url: String,
    /// minisign signature of the exe: the text, or its base64 (Tauri's encoding).
    pub signature: String,
    pub notes: String,
}

impl Offer {
    pub fn parse(json: &str) -> Result<Offer> {
        let m: Manifest = serde_json::from_str(json).context("unexpected latest.json")?;
        let v = m.version.trim().trim_start_matches('v');
        let version = Version::parse(v)
            .with_context(|| format!("latest.json version {} is not a version", m.version))?;
        let (url, signature) = match m.platforms.get(TARGET) {
            Some(p) => (p.url.clone(), p.signature.clone()),
            None => (
                m.url
                    .with_context(|| format!("latest.json has no {TARGET} build"))?,
                m.signature.context("latest.json has no signature")?,
            ),
        };
        if !url.starts_with("https://") {
            bail!("latest.json has a non-https download URL: {url}");
        }
        if signature.trim().is_empty() {
            bail!("latest.json has no signature");
        }
        Ok(Offer {
            version,
            url,
            signature,
            notes: m.notes.unwrap_or_default(),
        })
    }

    /// The release's page, for people to download it themselves.
    pub fn page_url(&self) -> String {
        format!(
            "https://github.com/{}/releases/tag/v{}",
            crate::paths::REPO,
            self.version
        )
    }
}

/// The newer version this offer brings, if it is one (pre-releases are never offered).
pub fn update_candidate(offer: &Offer, current: &Version) -> Option<Version> {
    (offer.version.pre.is_empty() && offer.version > *current).then(|| offer.version.clone())
}

/// minisign text of `value`: itself if it already is, else its base64 decoding.
fn minisign_text(value: &str) -> Result<String> {
    use base64::Engine;
    let value = value.trim();
    if value.contains("untrusted comment:") {
        return Ok(value.to_string());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|e| anyhow!("not minisign text or its base64: {e}"))?;
    String::from_utf8(bytes).map_err(|_| anyhow!("not minisign text or its base64"))
}

fn decode_public_key(value: &str) -> Result<minisign_verify::PublicKey> {
    let value = value.trim();
    let key = if value.starts_with("RW") && !value.contains('\n') {
        minisign_verify::PublicKey::from_base64(value)
    } else {
        minisign_verify::PublicKey::decode(&minisign_text(value)?)
    };
    key.map_err(|e| anyhow!("bad update public key: {e}"))
}

/// The `version:` field of a trusted comment in Tauri's format
/// (`timestamp:…\tfile:…\tversion:…`).
pub fn signed_version(trusted_comment: &str) -> Option<&str> {
    trusted_comment
        .split('\t')
        .find_map(|field| field.trim().strip_prefix("version:"))
}

/// Verifies the minisign signature and that it was made for exactly `version`.
pub fn verify_signature(
    data: &[u8],
    signature: &str,
    pubkey: &str,
    version: &Version,
) -> Result<()> {
    use minisign_verify::Signature;
    let pk = decode_public_key(pubkey)?;
    let sig = Signature::decode(&minisign_text(signature).context("bad signature")?)
        .map_err(|e| anyhow!("bad signature: {e}"))?;
    pk.verify(data, &sig, false)
        .map_err(|e| anyhow!("signature check failed: {e}"))?;
    let signed = signed_version(sig.trusted_comment());
    if signed.and_then(|v| Version::parse(v.trim_start_matches('v')).ok()) != Some(version.clone())
    {
        bail!(
            "signature is for version {}, expected {version}",
            signed.unwrap_or("(none)")
        );
    }
    Ok(())
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

    /// Parses a `/releases` list (newest first).
    pub fn parse_list(json: &str) -> Result<Vec<Release>> {
        serde_json::from_str(json).context("unexpected GitHub releases JSON")
    }

    pub fn version(&self) -> Result<Version> {
        let v = self.tag_name.trim_start_matches('v');
        Version::parse(v).with_context(|| format!("release tag {} is not a version", self.tag_name))
    }

    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == name)
    }
}

pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use proptest::prelude::*;

    const PUB: &str = include_str!("../tests/fixtures/test.key.pub");
    const PAYLOAD: &[u8] = include_bytes!("../tests/fixtures/payload.bin");
    /// Signed as version 2.1.0 (scripts/release/minisign.ts, Tauri's trusted comment).
    const SIG: &str = include_str!("../tests/fixtures/payload.bin.minisig");

    fn b64(text: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(text)
    }

    fn manifest(version: &str) -> String {
        format!(
            r#"{{"version":"{version}","notes":"Fixes","pub_date":"2026-09-30T00:00:00Z",
               "platforms":{{"windows-x86_64":{{"url":"https://github.com/x/no-drama-llama.exe","signature":"{}"}}}}}}"#,
            b64(SIG)
        )
    }

    #[test]
    fn parses_latest_json() {
        let o = Offer::parse(&manifest("2.1.0")).unwrap();
        assert_eq!(o.version, Version::new(2, 1, 0));
        assert_eq!(o.url, "https://github.com/x/no-drama-llama.exe");
        assert_eq!(o.notes, "Fixes");
        assert_eq!(minisign_text(&o.signature).unwrap().trim(), SIG.trim());
        assert!(
            o.page_url().ends_with("/releases/tag/v2.1.0"),
            "{}",
            o.page_url()
        );
        // Tauri's single-platform form, and a leading v.
        let flat = Offer::parse(r#"{"version":"v2.2.0","url":"https://h/exe","signature":"c2ln"}"#)
            .unwrap();
        assert_eq!(
            (flat.version, flat.url.as_str()),
            (Version::new(2, 2, 0), "https://h/exe")
        );
    }

    #[test]
    fn rejects_bad_manifests() {
        for json in [
            "",
            "[]",
            r#"{"message":"Not Found"}"#,
            r#"{"version":"latest","url":"https://h/x","signature":"s"}"#,
            r#"{"version":"2.0.0","platforms":{"linux-x86_64":{"url":"https://h/x","signature":"s"}}}"#,
            r#"{"version":"2.0.0","url":"http://h/x","signature":"s"}"#,
            r#"{"version":"2.0.0","url":"https://h/x","signature":" "}"#,
        ] {
            assert!(Offer::parse(json).is_err(), "{json}");
        }
    }

    #[test]
    fn only_newer_full_releases() {
        let cur = Version::new(2, 0, 0);
        let offer = |v: &str| Offer::parse(&manifest(v)).unwrap();
        assert_eq!(
            update_candidate(&offer("2.1.0"), &cur),
            Some(Version::new(2, 1, 0))
        );
        assert_eq!(update_candidate(&offer("2.0.0"), &cur), None);
        assert_eq!(update_candidate(&offer("1.9.9"), &cur), None);
        assert_eq!(update_candidate(&offer("2.1.0-beta.1"), &cur), None);
        // semver, not string, ordering
        assert_eq!(
            update_candidate(&offer("2.10.0"), &Version::new(2, 9, 0)),
            Some(Version::new(2, 10, 0))
        );
    }

    #[test]
    fn signature_ok_in_every_encoding() {
        let v = Version::new(2, 1, 0);
        verify_signature(PAYLOAD, SIG, PUB, &v).unwrap();
        verify_signature(PAYLOAD, &b64(SIG), &b64(PUB), &v).unwrap();
        verify_signature(PAYLOAD, SIG, PUB.lines().nth(1).unwrap(), &v).unwrap();
        verify_signature(PAYLOAD, SIG, &format!("  {PUB}  \n"), &v).unwrap();
    }

    #[test]
    fn signature_rejects_tampering_and_version_swap() {
        let v = Version::new(2, 1, 0);
        let mut bad = PAYLOAD.to_vec();
        bad[0] ^= 1;
        assert!(verify_signature(&bad, SIG, PUB, &v).is_err());
        let mut longer = PAYLOAD.to_vec();
        longer.push(0);
        assert!(verify_signature(&longer, SIG, PUB, &v).is_err());
        // a genuinely signed older build offered as a newer version
        let e = verify_signature(PAYLOAD, SIG, PUB, &Version::new(9, 0, 0)).unwrap_err();
        assert!(
            e.to_string().contains("for version 2.1.0, expected 9.0.0"),
            "{e}"
        );
        // rewriting the trusted comment breaks the global signature
        let forged = SIG.replace("version:2.1.0", "version:9.0.0");
        assert!(verify_signature(PAYLOAD, &forged, PUB, &Version::new(9, 0, 0)).is_err());
        // someone else's key
        let other = "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
        assert!(verify_signature(PAYLOAD, SIG, other, &v).is_err());
        assert!(verify_signature(PAYLOAD, "", PUB, &v).is_err());
        assert!(verify_signature(PAYLOAD, &SIG[..SIG.len() / 2], PUB, &v).is_err());
        assert!(verify_signature(PAYLOAD, SIG, "not base64!", &v).is_err());
        assert!(verify_signature(PAYLOAD, SIG, "", &v).is_err());
    }

    #[test]
    fn reads_the_signed_version() {
        assert_eq!(
            signed_version("timestamp:1\tfile:no-drama-llama.exe\tversion:2.1.0"),
            Some("2.1.0")
        );
        assert_eq!(signed_version("no-drama-llama 2.1.0"), None);
    }

    #[test]
    fn parses_github_release_json() {
        let r = Release::parse(
            r#"{"tag_name":"v2.1.0","html_url":"https://github.com/x","draft":false,"prerelease":false,
               "assets":[{"name":"llama.zip","browser_download_url":"https://dl/zip","size":5,
                          "digest":"sha256:abc","extra":1}],"body":"notes"}"#,
        )
        .unwrap();
        assert_eq!(r.version().unwrap(), Version::new(2, 1, 0));
        assert_eq!(
            r.asset("llama.zip").unwrap().digest.as_deref(),
            Some("sha256:abc")
        );
        assert!(Release::parse(r#"{"message":"API rate limit exceeded"}"#).is_err());
        assert!(Release::parse(r#"{"tag_name":"latest"}"#)
            .unwrap()
            .version()
            .is_err());
    }

    #[test]
    fn current_version_is_the_crate_version() {
        assert_eq!(current_version().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn sha256() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    proptest! {
        #[test]
        fn verification_never_panics(data in prop::collection::vec(any::<u8>(), 0..64), sig in ".{0,300}", key in ".{0,80}") {
            let _ = verify_signature(&data, &sig, &key, &Version::new(1, 0, 0));
        }

        #[test]
        fn manifest_parsing_never_panics(json in ".{0,300}") {
            if let Ok(o) = Offer::parse(&json) {
                let _ = update_candidate(&o, &Version::new(1, 0, 0));
            }
        }

        #[test]
        fn random_payloads_never_verify(data in prop::collection::vec(any::<u8>(), 0..256)) {
            prop_assume!(data != PAYLOAD);
            prop_assert!(verify_signature(&data, SIG, PUB, &Version::new(2, 1, 0)).is_err());
        }
    }
}
