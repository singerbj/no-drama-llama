//! Upgrading from the PowerShell edition keeps the user's settings.

use no_drama_llama::settings::{DetectionMode, PopupPosition, Reasoning, Settings};

#[test]
fn reads_a_settings_file_written_by_windows_powershell_5_1() {
    // UTF-8 BOM, CRLF, PowerShell's padded ConvertTo-Json layout, legacy "high"
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/ps51-settings.json"
        ),
        &path,
    )
    .unwrap();
    let (s, warnings) = Settings::load(&path);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(s.reasoning, Reasoning::XHigh);
    assert_eq!(s.context, 65536);
    assert_eq!(s.listen_host, "0.0.0.0");
    assert!(!s.detect_emulators);
    assert_eq!(s.popup_position, PopupPosition::BottomRight);
    assert_eq!(s.extra_games, vec!["mygame"]);
    assert_eq!(s.detection_mode, DetectionMode::Both);
    assert_eq!(s.gpu_vram_gb, 2.0);
    assert_eq!(s.resume_after_sec, 120);
    assert!(s.gpu_ignore.is_empty());
    assert_eq!(s.api_key, "s3cret");
    assert!(s.auto_update, "new key gets its default");

    // Saving keeps every key the old edition knew, so a downgrade still works
    s.save(&path).unwrap();
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for k in [
        "Model",
        "Reasoning",
        "Context",
        "ListenHost",
        "Port",
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
        "ApiKey",
    ] {
        assert!(saved.get(k).is_some(), "{k}");
    }
    assert_eq!(saved["Reasoning"], "xhigh");
    assert!(saved["GpuIgnore"].is_array() && saved["ExtraGames"].is_array());
}
