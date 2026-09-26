//! Pure pieces of install/uninstall (task definition, settings backup/restore, powercfg
//! parsing), kept here so they are unit-tested on every OS.

use serde_json::{json, Value};
use std::path::Path;

/// Power-plan settings install changes (backed up first). Same aliases as the PowerShell
/// edition, so either edition can restore the other's backup.
pub const POWER_SETTINGS: [(&str, &str); 7] = [
    ("SUB_SLEEP", "STANDBYIDLE"),
    ("SUB_SLEEP", "HIBERNATEIDLE"),
    ("SUB_VIDEO", "VIDEOIDLE"),
    ("SUB_DISK", "DISKIDLE"),
    ("SUB_PCIEXPRESS", "ASPM"),
    ("SUB_PROCESSOR", "PROCTHROTTLEMIN"),
    (
        "2a737441-1930-4402-8d77-b2bebba308a3",
        "48e6b7a6-50f5-4782-a5d4-53bb8f07e226",
    ), // USB selective suspend
];

/// `powercfg` argument lists that set up low-power, always-on operation.
pub fn power_commands() -> Vec<Vec<String>> {
    let usb = POWER_SETTINGS[6];
    let cmds: [&[&str]; 10] = [
        &["/setactive", "SCHEME_BALANCED"],
        &["/change", "standby-timeout-ac", "0"], // never sleep
        &["/change", "hibernate-timeout-ac", "0"],
        &["/change", "monitor-timeout-ac", "10"], // screen off after 10 min
        &["/change", "disk-timeout-ac", "20"],
        &["/hibernate", "off"],
        &[
            "/setacvalueindex",
            "SCHEME_CURRENT",
            "SUB_PCIEXPRESS",
            "ASPM",
            "2",
        ], // PCIe link power: max savings
        &[
            "/setacvalueindex",
            "SCHEME_CURRENT",
            "SUB_PROCESSOR",
            "PROCTHROTTLEMIN",
            "5",
        ], // let the CPU clock down
        &["/setacvalueindex", "SCHEME_CURRENT", usb.0, usb.1, "1"], // USB selective suspend
        &["/setactive", "SCHEME_CURRENT"],
    ];
    cmds.iter()
        .map(|c| c.iter().map(|s| s.to_string()).collect())
        .collect()
}

/// AC value from `powercfg /q <scheme> <sub> <setting>` output. The last two hex numbers are
/// the AC and DC values, whatever the display language.
pub fn parse_ac_value(powercfg_q: &str) -> Option<u64> {
    let re = regex::Regex::new(r"0x([0-9a-fA-F]+)").unwrap();
    let hex: Vec<&str> = re
        .captures_iter(powercfg_q)
        .map(|c| c.get(1).unwrap().as_str())
        .collect();
    u64::from_str_radix(hex.get(hex.len().checked_sub(2)?)?, 16).ok()
}

/// Scheme GUID from `powercfg /getactivescheme`.
pub fn parse_active_scheme(text: &str) -> Option<String> {
    let re = regex::Regex::new(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
    )
    .unwrap();
    re.find(text).map(|m| m.as_str().to_string())
}

/// The backup file (same shape as the PowerShell edition's settings-backup.json).
pub fn backup_json(
    active_scheme: Option<String>,
    hibernate: Option<u32>,
    power_values: &[Option<u64>],
    nics: Value,
) -> Value {
    let power: Vec<Value> = POWER_SETTINGS
        .iter()
        .zip(power_values.iter().chain(std::iter::repeat(&None)))
        .map(|((sub, set), v)| json!({ "Sub": sub, "Set": set, "Value": v }))
        .collect();
    let nics = if nics.is_array() { nics } else { json!([]) };
    json!({ "ActiveScheme": active_scheme, "HibernateEnabled": hibernate, "Power": power, "Nics": nics })
}

fn is_safe_power_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `powercfg` argument lists that put a backup back. Anything malformed is skipped.
pub fn restore_commands(backup: &Value) -> Vec<Vec<String>> {
    let mut cmds = Vec::new();
    for s in backup["Power"].as_array().into_iter().flatten() {
        if let (Some(sub), Some(set), Some(v)) =
            (s["Sub"].as_str(), s["Set"].as_str(), s["Value"].as_u64())
        {
            if is_safe_power_token(sub) && is_safe_power_token(set) {
                cmds.push(vec![
                    "/setacvalueindex".into(),
                    "SCHEME_BALANCED".into(),
                    sub.into(),
                    set.into(),
                    v.to_string(),
                ]);
            }
        }
    }
    if backup["HibernateEnabled"].as_u64() == Some(1) {
        cmds.push(vec!["/hibernate".into(), "on".into()]);
    }
    let scheme = backup["ActiveScheme"]
        .as_str()
        .filter(|s| parse_active_scheme(s).as_deref() == Some(*s))
        .unwrap_or("SCHEME_BALANCED");
    cmds.push(vec!["/setactive".into(), scheme.into()]);
    cmds
}

pub fn ps_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// PowerShell that restores the network adapters listed in the backup file.
pub fn nic_restore_ps(backup: &Path) -> String {
    format!(
        r#"
$b = Get-Content -Raw {} | ConvertFrom-Json
foreach ($n in $b.Nics) {{
  if ($n.Magic) {{ Set-NetAdapterPowerManagement -Name $n.Name -WakeOnMagicPacket $n.Magic -ErrorAction SilentlyContinue }}
  foreach ($a in $n.Adv) {{ Set-NetAdapterAdvancedProperty -Name $n.Name -DisplayName $a.DisplayName -DisplayValue $a.DisplayValue -ErrorAction SilentlyContinue }}
}}
"#,
        ps_single_quote(&backup.to_string_lossy())
    )
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Scheduled task: starts the app elevated at logon without a UAC prompt, restarts it if it
/// crashes, never times out, and runs on battery too.
pub fn task_xml(exe: &Path, user_sid: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>No Drama Llama: local LLM server that pauses while you play.</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{sid}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{sid}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>5</Priority>
    <RestartOnFailure>
      <Interval>PT1M</Interval>
      <Count>99</Count>
    </RestartOnFailure>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{cmd}</Command>
    </Exec>
  </Actions>
</Task>
"#,
        sid = xml_escape(user_sid),
        cmd = xml_escape(&exe.to_string_lossy())
    )
}

/// `Get-ScheduledTask` State output -> whether the task will run at logon. Empty output (no
/// such task) and `Disabled` are both "no".
pub fn task_state_is_enabled(state: &str) -> bool {
    matches!(
        state.trim().to_ascii_lowercase().as_str(),
        "ready" | "running" | "queued"
    )
}

/// UTF-16LE with BOM, which is what `schtasks /create /xml` expects.
pub fn utf16le_with_bom(s: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(s.encode_utf16().flat_map(|u| u.to_le_bytes()));
    bytes
}

/// Files the PowerShell edition kept directly in C:\LLM (1.0) that now live in data\.
pub const LEGACY_DATA_FILES: [&str; 6] = [
    "settings.json",
    "off.flag",
    "tray.log",
    "server.log",
    "gpu-usage.txt",
    "game-libraries.txt",
];
/// The PowerShell edition's scripts (in C:\LLM or C:\LLM\app).
pub const LEGACY_SCRIPTS: [&str; 8] = [
    "config.ps1",
    "server.ps1",
    "llm-tray.ps1",
    "llm-toggle.ps1",
    "games.ps1",
    "gpu.ps1",
    "osd.ps1",
    "uninstall.ps1",
];
pub const LEGACY_SHORTCUTS: [&str; 2] = ["Toggle Local LLM.lnk", "Local LLM.lnk"];

/// Moves the PowerShell edition's files into the current layout. Returns what it did.
pub fn migrate_legacy_layout(p: &crate::paths::Paths) -> Vec<String> {
    let mut done = Vec::new();
    if std::fs::remove_dir_all(p.root.join("app")).is_ok() {
        done.push("removed app\\".to_string());
    }
    for f in LEGACY_SCRIPTS {
        if std::fs::remove_file(p.root.join(f)).is_ok() {
            done.push(format!("removed {f}"));
        }
    }
    let _ = std::fs::create_dir_all(&p.data_dir);
    for f in LEGACY_DATA_FILES {
        let old = p.root.join(f);
        if old.is_file() {
            let new = p.data_dir.join(f);
            if new.exists() {
                let _ = std::fs::remove_file(&old);
                done.push(format!("dropped old {f}"));
            } else if std::fs::rename(&old, &new).is_ok() {
                done.push(format!("moved {f}"));
            }
        }
    }
    done
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;

    const POWERCFG_EN: &str = "Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)
  Subgroup GUID: 238c9fa8-0aad-41ed-83f4-97be242c8f20  (Sleep)
    GUID Alias: SUB_SLEEP
    Power Setting GUID: 29f6c1db-86da-48c5-9fdb-f2b67b1f44da  (Sleep after)
      GUID Alias: STANDBYIDLE
      Minimum Possible Setting: 0x00000000
      Maximum Possible Setting: 0xffffffff
      Possible Settings increment: 0x00000001
      Possible Settings units: Seconds
    Current AC Power Setting Index: 0x00000708
    Current DC Power Setting Index: 0x00000384
";
    const POWERCFG_DE: &str = "GUID des Energieschemas: 381b4222-f694-41f0-9685-ff5bb260df2e  (Ausbalanciert)
  GUID der Untergruppe: 501a4d13-42af-4429-9fd1-a8218c268e20  (PCI Express)
    GUID-Alias: SUB_PCIEXPRESS
    GUID der Energieeinstellung: ee12f906-d277-404b-b6da-e5fa1a576df5  (Verbindungszustand-Energieverwaltung)
      GUID-Alias: ASPM
      Mögliche Einstellung: 000
      Index der aktuellen Wechselstromeinstellung: 0x00000001
      Index der aktuellen Gleichstromeinstellung: 0x00000002
";

    #[test]
    fn parses_ac_value_in_any_language() {
        assert_eq!(parse_ac_value(POWERCFG_EN), Some(0x708));
        assert_eq!(parse_ac_value(POWERCFG_DE), Some(1));
        assert_eq!(
            parse_ac_value("Invalid Parameters -- try \"/?\" for help"),
            None
        );
        assert_eq!(parse_ac_value("only one 0x1"), None);
        assert_eq!(parse_ac_value(""), None);
    }

    #[test]
    fn parses_active_scheme() {
        assert_eq!(
            parse_active_scheme(
                "Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)"
            )
            .as_deref(),
            Some("381b4222-f694-41f0-9685-ff5bb260df2e")
        );
        assert_eq!(parse_active_scheme("nothing here"), None);
    }

    #[test]
    fn power_commands_cover_every_backed_up_setting() {
        let cmds = power_commands();
        assert_eq!(cmds.first().unwrap(), &["/setactive", "SCHEME_BALANCED"]);
        assert_eq!(cmds.last().unwrap(), &["/setactive", "SCHEME_CURRENT"]);
        let flat: Vec<String> = cmds.concat();
        for alias in [
            "standby-timeout-ac",
            "hibernate-timeout-ac",
            "monitor-timeout-ac",
            "disk-timeout-ac",
            "ASPM",
            "PROCTHROTTLEMIN",
            POWER_SETTINGS[6].1,
        ] {
            assert!(flat.iter().any(|a| a == alias), "{alias}");
        }
        // every setting install touches is in the backup list
        for (sub, _) in POWER_SETTINGS {
            assert!(!sub.is_empty());
        }
        assert!(cmds.iter().all(|c| c.iter().all(|a| !a.is_empty())));
    }

    #[test]
    fn backup_round_trips_to_restore_commands() {
        let values: Vec<Option<u64>> = vec![
            Some(1800),
            Some(0),
            Some(600),
            None,
            Some(1),
            Some(5),
            Some(0),
        ];
        let nics = json!([{ "Name": "Ethernet", "Magic": "Enabled", "Adv": [] }]);
        let b = backup_json(
            Some("381b4222-f694-41f0-9685-ff5bb260df2e".into()),
            Some(1),
            &values,
            nics,
        );
        assert_eq!(b["Power"].as_array().unwrap().len(), POWER_SETTINGS.len());
        assert_eq!(b["Nics"][0]["Name"], "Ethernet");
        let text = serde_json::to_string_pretty(&b).unwrap();
        let back: Value = serde_json::from_str(&text).unwrap();
        let cmds = restore_commands(&back);
        // 6 values present (one None skipped) + hibernate on + setactive
        assert_eq!(cmds.len(), 6 + 2);
        assert_eq!(
            cmds[0],
            [
                "/setacvalueindex",
                "SCHEME_BALANCED",
                "SUB_SLEEP",
                "STANDBYIDLE",
                "1800"
            ]
        );
        assert!(cmds.contains(&vec!["/hibernate".to_string(), "on".to_string()]));
        assert_eq!(
            cmds.last().unwrap(),
            &["/setactive", "381b4222-f694-41f0-9685-ff5bb260df2e"]
        );
    }

    #[test]
    fn restores_a_powershell_edition_backup() {
        // As written by install.ps1 (ConvertTo-Json -Depth 5 on Windows PowerShell 5.1)
        let ps = "\u{feff}{\r\n    \"ActiveScheme\":  \"8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c\",\r\n    \"HibernateEnabled\":  0,\r\n    \"Power\":  [\r\n                  {\r\n                      \"Set\":  \"STANDBYIDLE\",\r\n                      \"Sub\":  \"SUB_SLEEP\",\r\n                      \"Value\":  900\r\n                  },\r\n                  {\r\n                      \"Set\":  \"ASPM\",\r\n                      \"Sub\":  \"SUB_PCIEXPRESS\",\r\n                      \"Value\":  null\r\n                  }\r\n              ],\r\n    \"Nics\":  [\r\n\r\n             ]\r\n}";
        let b: Value = serde_json::from_str(ps.trim_start_matches('\u{feff}')).unwrap();
        let cmds = restore_commands(&b);
        assert_eq!(
            cmds,
            vec![
                vec![
                    "/setacvalueindex",
                    "SCHEME_BALANCED",
                    "SUB_SLEEP",
                    "STANDBYIDLE",
                    "900"
                ],
                vec!["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"],
            ]
        );
    }

    #[test]
    fn restore_ignores_tampered_backup_values() {
        let b = json!({
            "ActiveScheme": "x & calc",
            "Power": [
                { "Sub": "SUB_SLEEP & calc", "Set": "STANDBYIDLE", "Value": 1 },
                { "Sub": "SUB_SLEEP", "Set": "STANDBYIDLE", "Value": -5 },
                { "Sub": "SUB_SLEEP", "Set": "STANDBYIDLE", "Value": "1" },
                "garbage"
            ],
            "HibernateEnabled": "yes"
        });
        assert_eq!(
            restore_commands(&b),
            vec![vec!["/setactive", "SCHEME_BALANCED"]]
        );
        assert_eq!(
            restore_commands(&json!(null)),
            vec![vec!["/setactive", "SCHEME_BALANCED"]]
        );
    }

    #[test]
    fn backup_json_tolerates_bad_nics_and_short_values() {
        let b = backup_json(None, None, &[Some(1)], json!("not an array"));
        assert_eq!(b["Nics"], json!([]));
        assert_eq!(b["Power"][0]["Value"], 1);
        assert_eq!(b["Power"][6]["Value"], Value::Null);
        assert_eq!(b["ActiveScheme"], Value::Null);
    }

    #[test]
    fn nic_restore_script_quotes_path() {
        let s = nic_restore_ps(Path::new(r"C:\LL'M\settings-backup.json"));
        assert!(
            s.contains(r"Get-Content -Raw 'C:\LL''M\settings-backup.json'"),
            "{s}"
        );
        assert_eq!(ps_single_quote("a'b"), "'a''b'");
    }

    #[test]
    fn task_xml_is_correct() {
        let x = task_xml(
            Path::new(r"C:\Program Files\No Drama Llama\no-drama-llama.exe"),
            "S-1-5-21-1-2-3-1001",
        );
        for needle in [
            "<RunLevel>HighestAvailable</RunLevel>",
            "<LogonType>InteractiveToken</LogonType>",
            "<UserId>S-1-5-21-1-2-3-1001</UserId>",
            "<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>",
            "<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>",
            "<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>",
            "<MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>",
            "<Command>C:\\Program Files\\No Drama Llama\\no-drama-llama.exe</Command>",
            "<Count>99</Count>",
        ] {
            assert!(x.contains(needle), "{needle}");
        }
        // two occurrences of the user (trigger + principal)
        assert_eq!(x.matches("S-1-5-21-1-2-3-1001").count(), 2);
        // balanced tags
        for tag in [
            "Task",
            "Triggers",
            "LogonTrigger",
            "Principals",
            "Principal",
            "Settings",
            "RestartOnFailure",
            "Actions",
            "Exec",
        ] {
            assert_eq!(
                x.matches(&format!("<{tag}>")).count() + x.matches(&format!("<{tag} ")).count(),
                x.matches(&format!("</{tag}>")).count(),
                "{tag}"
            );
        }
    }

    #[test]
    fn task_xml_escapes() {
        let x = task_xml(Path::new(r"C:\A&B <x>\app.exe"), "S-1'\"");
        assert!(x.contains(r"<Command>C:\A&amp;B &lt;x&gt;\app.exe</Command>"));
        assert!(x.contains("<UserId>S-1&apos;&quot;</UserId>"));
        assert_eq!(xml_escape("&<>\"'"), "&amp;&lt;&gt;&quot;&apos;");
    }

    #[test]
    fn task_state() {
        for on in ["Ready", "Running\r\n", "queued"] {
            assert!(task_state_is_enabled(on), "{on:?}");
        }
        for off in [
            "",
            "\r\n",
            "Disabled",
            "Unknown",
            "Get-ScheduledTask : No MSFT_ScheduledTask objects found",
        ] {
            assert!(!task_state_is_enabled(off), "{off:?}");
        }
    }

    #[test]
    fn utf16_bom_encoding() {
        let b = utf16le_with_bom("Aé");
        assert_eq!(b, vec![0xFF, 0xFE, b'A', 0, 0xE9, 0]);
    }

    #[test]
    fn migrates_version_1_0_layout() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths::under(dir.path());
        std::fs::create_dir_all(p.root.join("app")).unwrap();
        std::fs::write(p.root.join("app").join("llm-tray.ps1"), "x").unwrap();
        std::fs::write(p.root.join("llm-tray.ps1"), "x").unwrap();
        std::fs::write(p.root.join("settings.json"), "{\"Model\":\"a.gguf\"}").unwrap();
        std::fs::write(p.root.join("off.flag"), "").unwrap();
        std::fs::create_dir_all(&p.data_dir).unwrap();
        std::fs::write(p.data_dir.join("tray.log"), "new").unwrap();
        std::fs::write(p.root.join("tray.log"), "old").unwrap();
        let done = migrate_legacy_layout(&p);
        assert!(!p.root.join("app").exists());
        assert!(!p.root.join("llm-tray.ps1").exists());
        assert_eq!(
            std::fs::read_to_string(&p.settings).unwrap(),
            "{\"Model\":\"a.gguf\"}"
        );
        assert!(p.off_flag.exists());
        assert_eq!(
            std::fs::read_to_string(&p.log).unwrap(),
            "new",
            "never overwrites newer data"
        );
        assert!(!p.root.join("tray.log").exists());
        assert!(done.len() >= 5, "{done:?}");
        // idempotent
        assert!(migrate_legacy_layout(&p).is_empty());
    }

    #[test]
    fn legacy_lists_match_the_powershell_edition() {
        // Every script the PowerShell installer copied is cleaned up.
        let install_ps1 = include_str!("../../src/install.ps1");
        let line = install_ps1
            .lines()
            .find(|l| l.starts_with("$AppFiles"))
            .unwrap();
        for f in line.split('\'').filter(|s| s.ends_with(".ps1")) {
            assert!(LEGACY_SCRIPTS.contains(&f), "{f}");
        }
    }
}
