//! User settings (`C:\LLM\data\settings.json`).
//!
//! The file uses the same keys as the PowerShell edition, so existing installs keep their
//! settings. It is read by an elevated process and turned into a command line, so every
//! value is validated; anything invalid falls back to its default and is reported.

use crate::laya::{self, Device as LayaDevice};
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::Path;

pub const DEFAULT_MODEL: &str = "Qwen3.8-27B-UD-Q4_K_XL.gguf";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Reasoning {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "xhigh")]
    XHigh,
}

impl Reasoning {
    pub const ALL: [Reasoning; 4] = [
        Reasoning::None,
        Reasoning::Low,
        Reasoning::Medium,
        Reasoning::XHigh,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Reasoning::None => "none",
            Reasoning::Low => "low",
            Reasoning::Medium => "medium",
            Reasoning::XHigh => "xhigh",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Reasoning::None),
            "low" => Some(Reasoning::Low),
            "medium" => Some(Reasoning::Medium),
            // Qwen 3.8's template treats "high" as "xhigh"; accept it from older settings files.
            "high" | "xhigh" => Some(Reasoning::XHigh),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DetectionMode {
    Both,
    Gpu,
    Launchers,
}

impl DetectionMode {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "Both" => Some(DetectionMode::Both),
            "Gpu" => Some(DetectionMode::Gpu),
            "Launchers" => Some(DetectionMode::Launchers),
            _ => None,
        }
    }
    pub fn uses_gpu(self) -> bool {
        self != DetectionMode::Launchers
    }
    pub fn uses_launchers(self) -> bool {
        self != DetectionMode::Gpu
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PopupPosition {
    TopCenter,
    TopRight,
    BottomRight,
    BottomCenter,
}

impl PopupPosition {
    pub const ALL: [(PopupPosition, &'static str); 4] = [
        (PopupPosition::TopCenter, "Top center"),
        (PopupPosition::TopRight, "Top right"),
        (PopupPosition::BottomRight, "Bottom right"),
        (PopupPosition::BottomCenter, "Bottom center"),
    ];

    fn parse(s: &str) -> Option<Self> {
        match s {
            "TopCenter" => Some(PopupPosition::TopCenter),
            "TopRight" => Some(PopupPosition::TopRight),
            "BottomRight" => Some(PopupPosition::BottomRight),
            "BottomCenter" => Some(PopupPosition::BottomCenter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
    pub model: String,
    pub reasoning: Reasoning,
    /// Tokens of context; [`CONTEXT_AUTO`] (0) = let llama.cpp fit the largest that fits.
    #[serde(serialize_with = "ser_context")]
    pub context: u32,
    pub listen_host: String,
    pub port: u16,
    pub api_key: String,
    pub pause_while_gaming: bool,
    pub detect_emulators: bool,
    pub use_windows_game_list: bool,
    pub popups: bool,
    pub popup_position: PopupPosition,
    pub extra_games: Vec<String>,
    pub detection_mode: DetectionMode,
    #[serde(rename = "GpuVramGB")]
    pub gpu_vram_gb: f64,
    pub gpu_load_pct: f64,
    pub resume_after_sec: u32,
    pub gpu_ignore: Vec<String>,
    pub auto_update: bool,
    /// Start the app when you sign in to Windows (the logon task's trigger).
    pub start_with_windows: bool,
    /// Run Laya (a decision model, served by Ollaya) alongside the LLM.
    pub run_laya: bool,
    /// Ollaya model name, e.g. `laya` or `laya:en`.
    pub laya_model: String,
    pub laya_port: u16,
    pub laya_device: LayaDevice,
    /// Ollaya's `keep_alive` (`-1` = keep loaded, `5m`, `0` = unload after each request).
    pub laya_keep_alive: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model: DEFAULT_MODEL.into(),
            reasoning: Reasoning::Low,
            context: CONTEXT_AUTO,
            listen_host: "127.0.0.1".into(),
            port: 8080,
            api_key: String::new(),
            pause_while_gaming: true,
            detect_emulators: true,
            use_windows_game_list: true,
            popups: true,
            popup_position: PopupPosition::TopCenter,
            extra_games: Vec::new(),
            detection_mode: DetectionMode::Both,
            gpu_vram_gb: 1.5,
            gpu_load_pct: 30.0,
            resume_after_sec: 60,
            gpu_ignore: Vec::new(),
            auto_update: true,
            start_with_windows: true,
            run_laya: false,
            laya_model: laya::DEFAULT_MODEL.into(),
            laya_port: laya::DEFAULT_PORT,
            laya_device: LayaDevice::Auto,
            laya_keep_alive: laya::DEFAULT_KEEP_ALIVE.into(),
        }
    }
}

/// Settings that change llama-server's command line (a change needs a server restart).
pub fn server_changed(a: &Settings, b: &Settings) -> bool {
    a.model != b.model
        || a.reasoning != b.reasoning
        || a.context != b.context
        || a.listen_host != b.listen_host
        || a.port != b.port
        || a.api_key != b.api_key
}

/// Every key in settings.json, as written.
pub const KEYS: [&str; 24] = [
    "Model",
    "Reasoning",
    "Context",
    "ListenHost",
    "Port",
    "ApiKey",
    "PauseWhileGaming",
    "DetectEmulators",
    "UseWindowsGameList",
    "Popups",
    "PopupPosition",
    "ExtraGames",
    "DetectionMode",
    "GpuVramGB",
    "GpuLoadPct",
    "ResumeAfterSec",
    "GpuIgnore",
    "AutoUpdate",
    "StartWithWindows",
    "RunLaya",
    "LayaModel",
    "LayaPort",
    "LayaDevice",
    "LayaKeepAlive",
];

/// `Context` value meaning "as much as fits" (llama.cpp's --fit picks it).
pub const CONTEXT_AUTO: u32 = 0;

fn ser_context<S: serde::Serializer>(c: &u32, ser: S) -> Result<S::Ok, S::Error> {
    if *c == CONTEXT_AUTO {
        ser.serialize_str("auto")
    } else {
        ser.serialize_u32(*c)
    }
}

pub fn is_valid_model_name(s: &str) -> bool {
    let ok_chars = s
        .chars()
        .all(|c| c.is_alphanumeric() || "._-+ ()[]".contains(c));
    ok_chars
        && s.len() <= 200
        && s.to_ascii_lowercase().ends_with(".gguf")
        && s.len() > 5
        && !s.contains("..")
}

pub fn is_valid_api_key(s: &str) -> bool {
    s.len() <= 128
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-~".contains(c))
}

fn as_str_list(v: &Value) -> Option<Vec<String>> {
    match v {
        Value::Null => Some(Vec::new()),
        Value::String(s) => Some(vec![s.clone()]),
        Value::Array(a) => a
            .iter()
            .filter(|x| !x.is_null())
            .map(|x| x.as_str().map(str::to_owned))
            .collect(),
        _ => None,
    }
}

fn as_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64().or_else(|| {
            n.as_f64()
                .filter(|f| f.fract() == 0.0 && *f >= 0.0)
                .map(|f| f as u64)
        }),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

impl Settings {
    /// Parses settings JSON. Returns the settings plus a warning per ignored value.
    pub fn from_json(text: &str) -> (Settings, Vec<String>) {
        let mut s = Settings::default();
        let mut warnings = Vec::new();
        // PowerShell 5.1 writes UTF-8 with a BOM
        let text = text.trim_start_matches('\u{feff}');
        let map: Map<String, Value> = match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(m)) => m,
            Ok(_) => {
                warnings.push("settings.json is not a JSON object - using defaults".into());
                return (s, warnings);
            }
            Err(e) => {
                warnings.push(format!("settings.json unreadable ({e}) - using defaults"));
                return (s, warnings);
            }
        };
        for (key, v) in &map {
            // PowerShell reads property names case-insensitively, so the old edition accepted "model".
            let canonical = KEYS
                .iter()
                .find(|k| k.eq_ignore_ascii_case(key))
                .copied()
                .unwrap_or("");
            let ok = match canonical {
                "Model" => v
                    .as_str()
                    .filter(|m| is_valid_model_name(m))
                    .map(|m| s.model = m.into())
                    .is_some(),
                "Reasoning" => v
                    .as_str()
                    .and_then(Reasoning::parse)
                    .map(|r| s.reasoning = r)
                    .is_some(),
                "Context" => (if v
                    .as_str()
                    .is_some_and(|t| t.trim().eq_ignore_ascii_case("auto"))
                {
                    Some(0)
                } else {
                    as_u64(v)
                })
                .filter(|c| *c == 0 || (512..=1_048_576).contains(c))
                .map(|c| s.context = c as u32)
                .is_some(),
                "ListenHost" => v
                    .as_str()
                    .filter(|h| *h == "127.0.0.1" || *h == "0.0.0.0")
                    .map(|h| s.listen_host = h.into())
                    .is_some(),
                "Port" => as_u64(v)
                    .filter(|p| (1024..=65535).contains(p))
                    .map(|p| s.port = p as u16)
                    .is_some(),
                "ApiKey" => v
                    .as_str()
                    .filter(|k| is_valid_api_key(k))
                    .map(|k| s.api_key = k.into())
                    .is_some(),
                "PauseWhileGaming" => v.as_bool().map(|b| s.pause_while_gaming = b).is_some(),
                "DetectEmulators" => v.as_bool().map(|b| s.detect_emulators = b).is_some(),
                "UseWindowsGameList" => v.as_bool().map(|b| s.use_windows_game_list = b).is_some(),
                "Popups" => v.as_bool().map(|b| s.popups = b).is_some(),
                "AutoUpdate" => v.as_bool().map(|b| s.auto_update = b).is_some(),
                "StartWithWindows" => v.as_bool().map(|b| s.start_with_windows = b).is_some(),
                "RunLaya" => v.as_bool().map(|b| s.run_laya = b).is_some(),
                "LayaModel" => v
                    .as_str()
                    .filter(|m| laya::is_valid_model_name(m))
                    .map(|m| s.laya_model = m.into())
                    .is_some(),
                "LayaPort" => as_u64(v)
                    .filter(|p| (1024..=65535).contains(p))
                    .map(|p| s.laya_port = p as u16)
                    .is_some(),
                "LayaDevice" => v
                    .as_str()
                    .and_then(LayaDevice::parse)
                    .map(|d| s.laya_device = d)
                    .is_some(),
                "LayaKeepAlive" => (match v {
                    Value::String(t) => Some(t.clone()),
                    // A number of seconds, as Ollaya takes it
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                })
                .filter(|k| laya::is_valid_keep_alive(k))
                .map(|k| s.laya_keep_alive = k)
                .is_some(),
                "PopupPosition" => v
                    .as_str()
                    .and_then(PopupPosition::parse)
                    .map(|p| s.popup_position = p)
                    .is_some(),
                "DetectionMode" => v
                    .as_str()
                    .and_then(DetectionMode::parse)
                    .map(|m| s.detection_mode = m)
                    .is_some(),
                "GpuVramGB" => as_f64(v)
                    .filter(|g| (0.1..=256.0).contains(g))
                    .map(|g| s.gpu_vram_gb = g)
                    .is_some(),
                "GpuLoadPct" => as_f64(v)
                    .filter(|g| (1.0..=100.0).contains(g))
                    .map(|g| s.gpu_load_pct = g)
                    .is_some(),
                "ResumeAfterSec" => as_u64(v)
                    .filter(|r| *r <= 3600)
                    .map(|r| s.resume_after_sec = r as u32)
                    .is_some(),
                "ExtraGames" => as_str_list(v).map(|l| s.extra_games = l).is_some(),
                "GpuIgnore" => as_str_list(v).map(|l| s.gpu_ignore = l).is_some(),
                _ => true, // unknown key: ignore
            };
            if !ok {
                warnings.push(format!(
                    "invalid {key} {v} in settings.json - using default"
                ));
            }
        }
        (s, warnings)
    }

    /// These settings with `patch` (settings.json keys -> values) applied, as the settings
    /// window sends it. An invalid or unknown value leaves that setting as it was and is
    /// reported.
    pub fn with_patch(&self, patch: &Map<String, Value>) -> (Settings, Vec<String>) {
        let Ok(Value::Object(mut merged)) = serde_json::to_value(self) else {
            unreachable!("settings serialize to an object")
        };
        let mut warnings = Vec::new();
        for (key, v) in patch {
            let Some(canonical) = KEYS.iter().find(|k| k.eq_ignore_ascii_case(key)) else {
                warnings.push(format!("unknown setting {key}"));
                continue;
            };
            let one = Value::Object(Map::from_iter([(canonical.to_string(), v.clone())]));
            if Settings::from_json(&one.to_string()).1.is_empty() {
                merged.insert(canonical.to_string(), v.clone());
            } else {
                warnings.push(format!("invalid {canonical} {v} - not changed"));
            }
        }
        (
            Settings::from_json(&Value::Object(merged).to_string()).0,
            warnings,
        )
    }

    /// Whether settings JSON sets `key` at all (case-insensitively, like the parser).
    pub fn json_has_key(text: &str, key: &str) -> bool {
        matches!(
            serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')),
            Ok(Value::Object(m)) if m.keys().any(|k| k.eq_ignore_ascii_case(key))
        )
    }

    pub fn load(path: &Path) -> (Settings, Vec<String>) {
        match std::fs::read_to_string(path) {
            Ok(text) => Settings::from_json(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Settings::default(), Vec::new()),
            Err(e) => (
                Settings::default(),
                vec![format!("settings.json unreadable ({e}) - using defaults")],
            ),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("settings serialize")
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, self.to_json())?;
        std::fs::rename(tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object_gives_defaults() {
        let (s, w) = Settings::from_json("{}");
        assert_eq!(s, Settings::default());
        assert!(w.is_empty());
    }

    #[test]
    fn valid_values_apply() {
        let (s, w) = Settings::from_json(
            r#"{"Model":"My Model-Q4_K_M.gguf","Reasoning":"none","Context":65536,"Port":"9000",
                "ListenHost":"0.0.0.0","Popups":false,"GpuVramGB":2.5,"ExtraGames":["foo","bar"],
                "AutoUpdate":false}"#,
        );
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(s.model, "My Model-Q4_K_M.gguf");
        assert_eq!(s.reasoning, Reasoning::None);
        assert_eq!(s.context, 65536);
        assert_eq!(s.port, 9000);
        assert_eq!(s.listen_host, "0.0.0.0");
        assert!(!s.popups);
        assert!(!s.auto_update);
        assert_eq!(s.gpu_vram_gb, 2.5);
        assert_eq!(s.extra_games, vec!["foo", "bar"]);
    }

    #[test]
    fn rejects_bad_values() {
        let cases = [
            r#"{"Model":"..\\..\\Windows\\evil.gguf"}"#,
            r#"{"Model":"x.gguf\" --log-file C:\\Windows\\x \""}"#,
            r#"{"Model":"notamodel.exe"}"#,
            r#"{"Reasoning":"extreme"}"#,
            r#"{"Context":"lots"}"#,
            r#"{"Context":100}"#,
            r#"{"ListenHost":"0.0.0.0 --api-key x"}"#,
            r#"{"Port":80}"#,
            r#"{"Popups":"yes"}"#,
            r#"{"ApiKey":"abc def"}"#,
            r#"{"GpuLoadPct":150}"#,
            r#"{"ExtraGames":[1,2]}"#,
        ];
        for c in cases {
            let (s, w) = Settings::from_json(c);
            assert_eq!(s, Settings::default(), "{c}");
            assert_eq!(w.len(), 1, "{c}");
        }
    }

    #[test]
    fn legacy_high_maps_to_xhigh() {
        assert_eq!(
            Settings::from_json(r#"{"Reasoning":"high"}"#).0.reasoning,
            Reasoning::XHigh
        );
    }

    #[test]
    fn single_string_list_and_bom() {
        let (s, w) = Settings::from_json("\u{feff}{\"ExtraGames\":\"mygame\",\"Unknown\":1}");
        assert!(w.is_empty());
        assert_eq!(s.extra_games, vec!["mygame"]);
    }

    #[test]
    fn corrupt_file_is_reported() {
        let (s, w) = Settings::from_json("{ not json");
        assert_eq!(s, Settings::default());
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data").join("settings.json");
        let s = Settings {
            reasoning: Reasoning::Medium,
            gpu_ignore: vec!["MyApp".into()],
            popup_position: PopupPosition::BottomRight,
            ..Default::default()
        };
        s.save(&path).unwrap();
        let (r, w) = Settings::load(&path);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(r, s);
    }

    #[test]
    fn missing_file_is_silent() {
        let (s, w) = Settings::load(Path::new("/definitely/not/here.json"));
        assert_eq!(s, Settings::default());
        assert!(w.is_empty());
    }

    #[test]
    fn patch_applies_valid_values_and_keeps_the_rest() {
        let base = Settings {
            port: 9000,
            gpu_ignore: vec!["a".into()],
            ..Default::default()
        };
        let patch = serde_json::json!({
            "StartWithWindows": false,
            "context": 16384,
            "GpuIgnore": ["a", "b"],
            "Port": 80,
            "Model": "..\\evil.gguf",
            "Bogus": 1
        });
        let (s, w) = base.with_patch(patch.as_object().unwrap());
        assert!(!s.start_with_windows);
        assert_eq!(s.context, 16384);
        assert_eq!(s.gpu_ignore, vec!["a", "b"]);
        assert_eq!(
            s.port, 9000,
            "invalid value keeps the current one, not the default"
        );
        assert_eq!(s.model, DEFAULT_MODEL);
        assert_eq!(w.len(), 3, "{w:?}");
        assert!(w.iter().any(|x| x.contains("Port")));
        assert!(w.iter().any(|x| x.contains("Model")));
        assert!(w.iter().any(|x| x.contains("unknown setting Bogus")));
    }

    #[test]
    fn empty_patch_changes_nothing() {
        let base = Settings {
            context: 4096,
            listen_host: "0.0.0.0".into(),
            ..Default::default()
        };
        let (s, w) = base.with_patch(&Map::new());
        assert_eq!(s, base);
        assert!(w.is_empty());
    }

    #[test]
    fn detects_keys_in_the_file() {
        assert!(Settings::json_has_key(
            r#"{"StartWithWindows":true}"#,
            "StartWithWindows"
        ));
        assert!(Settings::json_has_key(
            "\u{feff}{\"startwithwindows\":1}",
            "StartWithWindows"
        ));
        for t in ["{}", r#"{"Model":"x.gguf"}"#, "not json", "[]", ""] {
            assert!(!Settings::json_has_key(t, "StartWithWindows"), "{t}");
        }
    }

    #[test]
    fn server_change_detection() {
        let a = Settings::default();
        let mut b = a.clone();
        b.popups = false;
        assert!(!server_changed(&a, &b));
        b.context = 8192;
        assert!(server_changed(&a, &b));
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;

    fn one(key: &str, json_value: &str) -> (Settings, Vec<String>) {
        Settings::from_json(&format!("{{\"{key}\": {json_value}}}"))
    }

    fn accepted(key: &str, json_value: &str) -> Settings {
        let (s, w) = one(key, json_value);
        assert!(
            w.is_empty(),
            "{key}={json_value} should be accepted, got {w:?}"
        );
        s
    }

    fn rejected(key: &str, json_value: &str) {
        let (s, w) = one(key, json_value);
        assert_eq!(w.len(), 1, "{key}={json_value} should be rejected");
        assert_eq!(
            s,
            Settings::default(),
            "{key}={json_value} must fall back to defaults"
        );
        assert!(w[0].contains(key), "warning names the key: {}", w[0]);
    }

    #[test]
    fn context_bounds() {
        assert_eq!(accepted("Context", "512").context, 512);
        assert_eq!(accepted("Context", "1048576").context, 1_048_576);
        assert_eq!(accepted("Context", "32768.0").context, 32768);
        assert_eq!(accepted("Context", "\" 8192 \"").context, 8192);
        assert_eq!(accepted("Context", "\"auto\"").context, CONTEXT_AUTO);
        assert_eq!(accepted("Context", "\"AUTO\"").context, CONTEXT_AUTO);
        assert_eq!(accepted("Context", "0").context, CONTEXT_AUTO);
        for bad in [
            "511",
            "1048577",
            "-1",
            "1.5",
            "true",
            "null",
            "[]",
            "{}",
            "\"\"",
            "99999999999999999999",
        ] {
            rejected("Context", bad);
        }
    }

    #[test]
    fn port_bounds() {
        assert_eq!(accepted("Port", "1024").port, 1024);
        assert_eq!(accepted("Port", "65535").port, 65535);
        for bad in ["1023", "65536", "0", "-8080", "\"80a\"", "false"] {
            rejected("Port", bad);
        }
    }

    #[test]
    fn gpu_thresholds() {
        assert_eq!(accepted("GpuVramGB", "0.25").gpu_vram_gb, 0.25);
        assert_eq!(accepted("GpuVramGB", "256").gpu_vram_gb, 256.0);
        assert_eq!(accepted("GpuVramGB", "\"2\"").gpu_vram_gb, 2.0);
        for bad in ["0", "0.05", "7.4e-25", "-1", "256.1", "\"lots\"", "null"] {
            rejected("GpuVramGB", bad);
        }
        assert_eq!(accepted("GpuLoadPct", "100").gpu_load_pct, 100.0);
        assert_eq!(accepted("GpuLoadPct", "1").gpu_load_pct, 1.0);
        for bad in ["0", "0.5", "100.01", "-5"] {
            rejected("GpuLoadPct", bad);
        }
    }

    #[test]
    fn resume_bounds() {
        assert_eq!(accepted("ResumeAfterSec", "0").resume_after_sec, 0);
        assert_eq!(accepted("ResumeAfterSec", "3600").resume_after_sec, 3600);
        for bad in ["3601", "-1", "1.5"] {
            rejected("ResumeAfterSec", bad);
        }
    }

    #[test]
    fn enums() {
        for (v, want) in [
            ("none", Reasoning::None),
            ("low", Reasoning::Low),
            ("medium", Reasoning::Medium),
            ("xhigh", Reasoning::XHigh),
            ("high", Reasoning::XHigh),
        ] {
            assert_eq!(accepted("Reasoning", &format!("\"{v}\"")).reasoning, want);
        }
        for bad in ["\"LOW\"", "\"\"", "1", "null", "\"extreme\""] {
            rejected("Reasoning", bad);
        }
        for (v, want) in [
            ("Both", DetectionMode::Both),
            ("Gpu", DetectionMode::Gpu),
            ("Launchers", DetectionMode::Launchers),
        ] {
            assert_eq!(
                accepted("DetectionMode", &format!("\"{v}\"")).detection_mode,
                want
            );
        }
        rejected("DetectionMode", "\"gpu\"");
        for (p, _) in PopupPosition::ALL {
            let name = serde_json::to_string(&p).unwrap();
            assert_eq!(accepted("PopupPosition", &name).popup_position, p);
        }
        rejected("PopupPosition", "\"Middle\"");
    }

    #[test]
    fn listen_host_is_an_allow_list() {
        assert_eq!(accepted("ListenHost", "\"0.0.0.0\"").listen_host, "0.0.0.0");
        assert_eq!(
            accepted("ListenHost", "\"127.0.0.1\"").listen_host,
            "127.0.0.1"
        );
        for bad in [
            "\"192.168.1.5\"",
            "\"localhost\"",
            "\"::\"",
            "\"0.0.0.0 \"",
            "\"\"",
        ] {
            rejected("ListenHost", bad);
        }
    }

    #[test]
    fn booleans_are_strict() {
        for key in [
            "PauseWhileGaming",
            "DetectEmulators",
            "UseWindowsGameList",
            "Popups",
            "AutoUpdate",
        ] {
            let s = accepted(key, "false");
            assert_ne!(s, Settings::default(), "{key}");
            for bad in ["\"false\"", "0", "1", "null", "[]"] {
                rejected(key, bad);
            }
        }
    }

    #[test]
    fn lists() {
        assert_eq!(
            accepted("GpuIgnore", "[\"a\", null, \"b\"]").gpu_ignore,
            vec!["a", "b"]
        );
        assert!(accepted("ExtraGames", "null").extra_games.is_empty());
        assert!(accepted("ExtraGames", "[]").extra_games.is_empty());
        for bad in ["[1]", "[[\"a\"]]", "{}", "true", "5"] {
            rejected("ExtraGames", bad);
            rejected("GpuIgnore", bad);
        }
    }

    #[test]
    fn model_names() {
        for good in [
            "Qwen3.8-27B-UD-Q4_K_XL.gguf",
            "a.gguf",
            "My Model (v2) [Q4].GGUF",
            "llama+tuned_v1.gguf",
            "Ünïcødé-模型.gguf",
        ] {
            assert!(is_valid_model_name(good), "{good}");
        }
        for bad in [
            "",
            ".gguf",
            "model.bin",
            "..\\model.gguf",
            "../model.gguf",
            "C:\\model.gguf",
            "sub/model.gguf",
            "a\"b.gguf",
            "a;b.gguf",
            "a&b.gguf",
            "a|b.gguf",
            "a%PATH%.gguf",
            "x.gguf\n",
            "a..b.gguf",
            "a.gguf.exe",
        ] {
            assert!(!is_valid_model_name(bad), "{bad:?}");
        }
        assert!(!is_valid_model_name(&format!("{}.gguf", "a".repeat(200))));
    }

    #[test]
    fn api_keys() {
        for good in ["", "abc", "sk-local_1.2~x", &"k".repeat(128)] {
            assert!(is_valid_api_key(good), "{good}");
        }
        for bad in ["a b", "a\"b", "a;b", "é", &"k".repeat(129), "--flag x"] {
            assert!(!is_valid_api_key(bad), "{bad}");
        }
    }

    #[test]
    fn context_serializes_as_auto_or_a_number() {
        let v: serde_json::Value = serde_json::from_str(&Settings::default().to_json()).unwrap();
        assert_eq!(v["Context"], "auto");
        let fixed = Settings {
            context: 65536,
            ..Default::default()
        };
        let v: serde_json::Value = serde_json::from_str(&fixed.to_json()).unwrap();
        assert_eq!(v["Context"], 65536);
        assert_eq!(Settings::from_json(&fixed.to_json()).0.context, 65536);
    }

    #[test]
    fn keys_are_case_insensitive_like_powershell() {
        let (s, w) =
            Settings::from_json(r#"{"model":"x.gguf","CONTEXT":8192,"listenhost":"0.0.0.0"}"#);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(
            (s.model.as_str(), s.context, s.listen_host.as_str()),
            ("x.gguf", 8192, "0.0.0.0")
        );
    }

    #[test]
    fn one_bad_value_does_not_spoil_the_rest() {
        let (s, w) =
            Settings::from_json(r#"{"Model":"ok.gguf","Port":1,"Context":8192,"Popups":"x"}"#);
        assert_eq!(w.len(), 2);
        assert_eq!(s.model, "ok.gguf");
        assert_eq!(s.context, 8192);
        assert_eq!(s.port, 8080);
        assert!(s.popups);
    }

    #[test]
    fn non_object_json() {
        for t in ["[]", "1", "\"x\"", "null", "true", ""] {
            let (s, w) = Settings::from_json(t);
            assert_eq!(s, Settings::default(), "{t}");
            assert_eq!(w.len(), 1, "{t}");
        }
    }

    #[test]
    fn serialized_keys_are_exactly_the_documented_keys() {
        let v: serde_json::Value = serde_json::from_str(&Settings::default().to_json()).unwrap();
        let mut got: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        let mut want = KEYS.to_vec();
        got.sort();
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn every_key_is_parsed() {
        // A key that serializes but isn't parsed would silently reset on every load.
        let custom = Settings {
            model: "m.gguf".into(),
            reasoning: Reasoning::XHigh,
            context: 4096,
            listen_host: "0.0.0.0".into(),
            port: 9999,
            api_key: "k".into(),
            pause_while_gaming: false,
            detect_emulators: false,
            use_windows_game_list: false,
            popups: false,
            popup_position: PopupPosition::BottomCenter,
            extra_games: vec!["g".into()],
            detection_mode: DetectionMode::Gpu,
            gpu_vram_gb: 3.0,
            gpu_load_pct: 70.0,
            resume_after_sec: 5,
            gpu_ignore: vec!["i".into()],
            auto_update: false,
            start_with_windows: false,
            run_laya: true,
            laya_model: "laya:multilingual".into(),
            laya_port: 12345,
            laya_device: LayaDevice::Cpu,
            laya_keep_alive: "30m".into(),
        };
        let (back, w) = Settings::from_json(&custom.to_json());
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(back, custom);
    }

    #[test]
    fn defaults() {
        let d = Settings::default();
        assert_eq!(d.model, DEFAULT_MODEL);
        assert_eq!(d.reasoning, Reasoning::Low);
        assert_eq!(d.context, CONTEXT_AUTO);
        assert_eq!((d.listen_host.as_str(), d.port), ("127.0.0.1", 8080));
        assert_eq!(d.detection_mode, DetectionMode::Both);
        assert_eq!(
            (d.gpu_vram_gb, d.gpu_load_pct, d.resume_after_sec),
            (1.5, 30.0, 60)
        );
        assert_eq!(d.popup_position, PopupPosition::TopCenter);
        assert!(
            d.pause_while_gaming
                && d.detect_emulators
                && d.use_windows_game_list
                && d.popups
                && d.auto_update
                && d.start_with_windows
        );
        assert!(d.api_key.is_empty() && d.extra_games.is_empty() && d.gpu_ignore.is_empty());
        assert_eq!(serde_json::to_value(d.reasoning).unwrap(), "low");
        assert!(!d.run_laya, "Laya is opt-in");
        assert_eq!(
            (d.laya_model.as_str(), d.laya_port, d.laya_device),
            ("laya", 11435, LayaDevice::Auto)
        );
        assert_eq!(d.laya_keep_alive, "-1");
    }

    #[test]
    fn laya_settings() {
        assert!(accepted("RunLaya", "true").run_laya);
        rejected("RunLaya", "\"yes\"");
        assert_eq!(accepted("LayaModel", "\"laya:en\"").laya_model, "laya:en");
        for bad in ["\"\"", "\"evil.com/laya\"", "\"laya --x\"", "1", "null"] {
            rejected("LayaModel", bad);
        }
        assert_eq!(accepted("LayaPort", "12000").laya_port, 12000);
        for bad in ["80", "65536", "\"x\""] {
            rejected("LayaPort", bad);
        }
        for d in ["auto", "cpu", "cuda"] {
            let s = accepted("LayaDevice", &format!("\"{d}\""));
            assert_eq!(s.laya_device.as_str(), d);
        }
        rejected("LayaDevice", "\"cuda:0\"");
        assert_eq!(accepted("LayaKeepAlive", "\"5m\"").laya_keep_alive, "5m");
        assert_eq!(accepted("LayaKeepAlive", "300").laya_keep_alive, "300");
        assert_eq!(accepted("LayaKeepAlive", "-1").laya_keep_alive, "-1");
        for bad in ["\"forever\"", "\"\"", "true", "[]"] {
            rejected("LayaKeepAlive", bad);
        }
    }

    #[test]
    fn save_is_atomic_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "old").unwrap();
        Settings::default().save(&path).unwrap();
        assert!(!dir.path().join("settings.json.tmp").exists());
        assert_eq!(Settings::load(&path).0, Settings::default());
    }

    #[test]
    fn load_reports_unreadable_file() {
        let dir = tempfile::tempdir().unwrap();
        // a directory where the file should be
        let (s, w) = Settings::load(dir.path());
        assert_eq!(s, Settings::default());
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn server_changed_covers_every_server_setting() {
        let a = Settings::default();
        let edits: [fn(&mut Settings); 6] = [
            |s| s.model = "x.gguf".into(),
            |s| s.reasoning = Reasoning::None,
            |s| s.context = 1024,
            |s| s.listen_host = "0.0.0.0".into(),
            |s| s.port = 9000,
            |s| s.api_key = "k".into(),
        ];
        for e in edits {
            let mut b = a.clone();
            e(&mut b);
            assert!(server_changed(&a, &b));
        }
        let harmless: [fn(&mut Settings); 8] = [
            |s| s.popups = false,
            |s| s.pause_while_gaming = false,
            |s| s.gpu_vram_gb = 4.0,
            |s| s.extra_games = vec!["x".into()],
            |s| s.detection_mode = DetectionMode::Gpu,
            |s| s.resume_after_sec = 0,
            |s| s.auto_update = false,
            |s| s.popup_position = PopupPosition::TopRight,
        ];
        for e in harmless {
            let mut b = a.clone();
            e(&mut b);
            assert!(!server_changed(&a, &b));
        }
    }

    mod props {
        use super::super::*;
        use proptest::prelude::*;

        fn json_value() -> impl Strategy<Value = serde_json::Value> {
            let leaf = prop_oneof![
                Just(serde_json::Value::Null),
                any::<bool>().prop_map(serde_json::Value::from),
                any::<i64>().prop_map(serde_json::Value::from),
                any::<f64>()
                    .prop_filter("finite", |f| f.is_finite())
                    .prop_map(serde_json::Value::from),
                ".{0,40}".prop_map(serde_json::Value::from),
            ];
            leaf.prop_recursive(3, 16, 4, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..4).prop_map(serde_json::Value::from),
                ]
            })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(2000))]

            /// Whatever is in settings.json, loading never panics and yields settings that
            /// re-validate cleanly (so nothing unsafe ever reaches the command line).
            #[test]
            fn any_settings_file_yields_valid_settings(
                entries in prop::collection::vec((prop::sample::select(KEYS.to_vec()), json_value()), 0..12)
            ) {
                let map: serde_json::Map<String, serde_json::Value> =
                    entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
                let (s, _) = Settings::from_json(&serde_json::Value::Object(map).to_string());
                prop_assert!(is_valid_model_name(&s.model));
                prop_assert!(is_valid_api_key(&s.api_key));
                prop_assert!(s.listen_host == "127.0.0.1" || s.listen_host == "0.0.0.0");
                prop_assert!(s.port >= 1024);
                prop_assert!(s.context == CONTEXT_AUTO || (512..=1_048_576).contains(&s.context));
                prop_assert!(laya::is_valid_model_name(&s.laya_model));
                prop_assert!(laya::is_valid_keep_alive(&s.laya_keep_alive));
                prop_assert!(s.laya_port >= 1024);
                let (again, w) = Settings::from_json(&s.to_json());
                prop_assert!(w.is_empty(), "{:?}", w);
                prop_assert_eq!(again, s);
            }

            /// A patch from the settings window can only ever produce valid settings, and it
            /// never touches a setting it doesn't name.
            #[test]
            fn any_patch_yields_valid_settings(
                entries in prop::collection::vec((prop::sample::select(KEYS.to_vec()), json_value()), 0..6)
            ) {
                let base = Settings { port: 9123, context: 4096, ..Default::default() };
                let patch: serde_json::Map<String, serde_json::Value> =
                    entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
                let (s, _) = base.with_patch(&patch);
                let (again, w) = Settings::from_json(&s.to_json());
                prop_assert!(w.is_empty(), "{:?}", w);
                prop_assert_eq!(&again, &s);
                if !patch.contains_key("Port") {
                    prop_assert_eq!(s.port, 9123);
                }
                if !patch.contains_key("Context") {
                    prop_assert_eq!(s.context, 4096);
                }
            }

            #[test]
            fn arbitrary_text_never_panics(text in ".{0,200}") {
                let _ = Settings::from_json(&text);
            }

            #[test]
            fn model_names_that_pass_are_plain_file_names(name in ".{0,60}") {
                if is_valid_model_name(&name) {
                    prop_assert!(!name.contains(['\\', '/', ':', '"', '\n', '\r', '\0']));
                    prop_assert!(!name.contains(".."));
                    prop_assert!(name.to_ascii_lowercase().ends_with(".gguf"));
                }
            }
        }
    }
}
