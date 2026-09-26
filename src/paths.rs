//! Install layout (shared with the PowerShell edition, so models and settings carry over).
//!
//! ```text
//! %ProgramFiles%\No Drama Llama\no-drama-llama.exe   the app (admin-only, Windows default)
//! C:\LLM\                admin-only (the elevated app runs llama-server.exe from here)
//!   llama\               llama.cpp binaries
//!   data\                settings.json, off.flag, logs
//!     model-downloads\   models while they download (moved to models\ once verified)
//!   models\              *.gguf (you can add your own)
//!   ollaya\              Ollaya, which runs Laya (bin\ollaya.exe, lib\, its models\)
//!   settings-backup.json your original Windows settings, restored on uninstall
//! ```

use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "No Drama Llama";
pub const TASK_NAME: &str = "No Drama Llama";
pub const EXE_NAME: &str = "no-drama-llama.exe";
pub const REPO: &str = "singerbj/no-drama-llama";

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub llama_dir: PathBuf,
    pub server_exe: PathBuf,
    pub models_dir: PathBuf,
    /// Partial model downloads. Admin-only, unlike `models_dir`: the elevated app writes,
    /// appends to and deletes these files, which a link in a user-writable folder could
    /// redirect to any file on the PC.
    pub model_downloads: PathBuf,
    pub data_dir: PathBuf,
    pub settings: PathBuf,
    pub off_flag: PathBuf,
    pub log: PathBuf,
    pub server_log: PathBuf,
    pub backup: PathBuf,
    /// Ollaya's install: `bin\ollaya.exe`, `lib\ollaya\`, `share\`, `install.json`
    pub ollaya_dir: PathBuf,
    pub ollaya_exe: PathBuf,
    /// Ollaya's model store (`OLLAYA_MODELS`)
    pub ollaya_models: PathBuf,
    pub laya_log: PathBuf,
}

impl Paths {
    pub fn under(root: impl Into<PathBuf>) -> Paths {
        let root = root.into();
        let data_dir = root.join("data");
        let ollaya_dir = root.join("ollaya");
        Paths {
            ollaya_exe: ollaya_dir.join("bin").join("ollaya.exe"),
            ollaya_models: ollaya_dir.join("models"),
            laya_log: data_dir.join("laya.log"),
            ollaya_dir,
            llama_dir: root.join("llama"),
            server_exe: root.join("llama").join("llama-server.exe"),
            models_dir: root.join("models"),
            model_downloads: data_dir.join("model-downloads"),
            settings: data_dir.join("settings.json"),
            off_flag: data_dir.join("off.flag"),
            log: data_dir.join("tray.log"),
            server_log: data_dir.join("server.log"),
            backup: root.join("settings-backup.json"),
            data_dir,
            root,
        }
    }

    pub fn system() -> Paths {
        Paths::under(r"C:\LLM")
    }

    /// What the app installed of Ollaya (`None` = nothing, or an older unreadable record).
    pub fn ollaya_record(&self) -> Option<crate::laya::Installed> {
        std::fs::read_to_string(self.ollaya_dir.join("install.json"))
            .ok()
            .and_then(|t| crate::laya::Installed::parse(&t))
    }

    pub fn model(&self, name: &str) -> PathBuf {
        self.models_dir.join(name)
    }

    /// `.gguf` models in the models folder (vision projectors and imatrix files excluded),
    /// sorted. A split model shows once, as its first part, with the size of all parts.
    pub fn list_models(&self) -> Vec<(String, u64)> {
        let files: Vec<(String, u64)> = std::fs::read_dir(&self.models_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.metadata().map(|m| m.len()).unwrap_or(0),
                )
            })
            .collect();
        group_models(files)
    }
}

/// `name-00002-of-00003.gguf` -> (`name`, 2, 3)
pub fn split_part(file: &str) -> Option<(&str, u32, u32)> {
    let stem = file
        .strip_suffix(".gguf")
        .or_else(|| file.strip_suffix(".GGUF"))?;
    let (rest, total) = stem.rsplit_once("-of-")?;
    let (base, index) = rest.rsplit_once('-')?;
    if index.len() != 5 || total.len() != 5 {
        return None;
    }
    Some((base, index.parse().ok()?, total.parse().ok()?))
}

/// Model files -> loadable models with their total sizes.
pub fn group_models(files: Vec<(String, u64)>) -> Vec<(String, u64)> {
    let is_model = |n: &str| {
        let l = n.to_ascii_lowercase();
        l.ends_with(".gguf") && !l.starts_with("mmproj") && !l.starts_with("imatrix")
    };
    let mut v: Vec<(String, u64)> = files
        .iter()
        .filter(|(n, _)| is_model(n))
        .filter(|(n, _)| split_part(n).is_none_or(|(_, i, _)| i == 1))
        .map(|(n, size)| match split_part(n) {
            Some((base, _, total)) => {
                let all: u64 = files
                    .iter()
                    .filter(|(m, _)| split_part(m).is_some_and(|(b, _, t)| b == base && t == total))
                    .map(|(_, s)| s)
                    .sum();
                (n.clone(), all)
            }
            None => (n.clone(), *size),
        })
        .collect();
    v.sort();
    v
}

/// Parts of a split model that aren't on disk yet (empty for single files).
pub fn missing_parts(models_dir: &Path, first_part: &str) -> Vec<String> {
    let Some((base, _, total)) = split_part(first_part) else {
        return Vec::new();
    };
    (1..=total)
        .map(|i| format!("{base}-{i:05}-of-{total:05}.gguf"))
        .filter(|f| !models_dir.join(f).exists())
        .collect()
}

/// Where the installed app lives: `%ProgramFiles%\No Drama Llama`.
/// Program Files comes from the known-folder API, not %ProgramFiles%: the elevated app
/// inherits the signed-in user's environment, which unelevated programs can change.
pub fn install_dir() -> PathBuf {
    #[cfg(windows)]
    let pf = crate::win::sys::program_files();
    #[cfg(not(windows))]
    let pf = PathBuf::from(r"C:\Program Files");
    pf.join(APP_NAME)
}

pub fn installed_exe() -> PathBuf {
    install_dir().join(EXE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_only_models() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths::under(dir.path());
        std::fs::create_dir_all(&p.models_dir).unwrap();
        for f in [
            "b.gguf",
            "a.GGUF",
            "mmproj-F16.gguf",
            "imatrix_unsloth.gguf",
            "notes.txt",
            "x.gguf.part",
        ] {
            std::fs::write(p.models_dir.join(f), b"x").unwrap();
        }
        let names: Vec<_> = p.list_models().into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["a.GGUF", "b.gguf"]);
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn layout_is_c_llm() {
        // Same layout as the PowerShell edition, so its installs migrate in place.
        let p = Paths::system();
        assert_eq!(p.root, PathBuf::from(r"C:\LLM"));
        let s = |x: &PathBuf| x.to_string_lossy().replace('/', "\\");
        assert_eq!(s(&p.server_exe), r"C:\LLM\llama\llama-server.exe");
        assert_eq!(s(&p.models_dir), r"C:\LLM\models");
        assert_eq!(s(&p.settings), r"C:\LLM\data\settings.json");
        assert_eq!(s(&p.off_flag), r"C:\LLM\data\off.flag");
        assert_eq!(s(&p.log), r"C:\LLM\data\tray.log");
        assert_eq!(s(&p.server_log), r"C:\LLM\data\server.log");
        assert_eq!(s(&p.backup), r"C:\LLM\settings-backup.json");
        assert_eq!(s(&p.ollaya_exe), r"C:\LLM\ollaya\bin\ollaya.exe");
        assert_eq!(s(&p.ollaya_models), r"C:\LLM\ollaya\models");
        assert_eq!(s(&p.laya_log), r"C:\LLM\data\laya.log");
    }

    #[test]
    fn ollaya_stays_out_of_the_user_writable_models_folder() {
        // The elevated app runs ollaya.exe and its runners: admin-only, like llama\.
        let p = Paths::under("/x");
        assert!(!p.ollaya_dir.starts_with(&p.models_dir));
        assert!(p.ollaya_exe.starts_with(&p.ollaya_dir));
        assert!(p.ollaya_models.starts_with(&p.ollaya_dir));
    }

    #[test]
    fn partial_downloads_stay_out_of_the_user_writable_models_folder() {
        let p = Paths::under("/x");
        assert!(!p.model_downloads.starts_with(&p.models_dir));
        assert!(p.model_downloads.starts_with(&p.data_dir));
    }

    #[test]
    fn ollaya_record_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths::under(dir.path());
        assert_eq!(p.ollaya_record(), None);
        std::fs::create_dir_all(&p.ollaya_dir).unwrap();
        let r = crate::laya::Installed {
            version: "0.5.0".into(),
            gpu_pack: true,
            gpu_wanted: true,
        };
        std::fs::write(p.ollaya_dir.join("install.json"), r.to_json()).unwrap();
        assert_eq!(p.ollaya_record(), Some(r));
    }

    #[test]
    fn user_writable_state_stays_out_of_llama_and_root() {
        let p = Paths::under("/x");
        for f in [&p.settings, &p.off_flag, &p.log, &p.server_log, &p.laya_log] {
            assert!(f.starts_with(&p.data_dir), "{}", f.display());
        }
        assert!(
            !p.backup.starts_with(&p.data_dir),
            "backup is read by the elevated uninstaller"
        );
        assert!(p.server_exe.starts_with(&p.llama_dir));
    }

    #[test]
    fn missing_models_dir_lists_nothing() {
        assert!(Paths::under("/definitely/not/here")
            .list_models()
            .is_empty());
    }

    #[test]
    fn model_path_joins_name() {
        let p = Paths::under("/x");
        assert_eq!(p.model("a.gguf"), p.models_dir.join("a.gguf"));
    }

    #[test]
    fn installed_exe_is_under_program_files() {
        let e = installed_exe();
        assert!(e.ends_with(PathBuf::from(APP_NAME).join(EXE_NAME)));
    }

    #[test]
    fn split_models_show_once_with_their_total_size() {
        let files = vec![
            (
                "Qwen3.8-Flash-Next-UD-IQ1_S-00001-of-00003.gguf".to_string(),
                10,
            ),
            (
                "Qwen3.8-Flash-Next-UD-IQ1_S-00002-of-00003.gguf".to_string(),
                20,
            ),
            (
                "Qwen3.8-Flash-Next-UD-IQ1_S-00003-of-00003.gguf".to_string(),
                30,
            ),
            ("small.gguf".to_string(), 5),
            ("orphan-00002-of-00002.gguf".to_string(), 7),
        ];
        assert_eq!(
            group_models(files),
            vec![
                (
                    "Qwen3.8-Flash-Next-UD-IQ1_S-00001-of-00003.gguf".to_string(),
                    60
                ),
                ("small.gguf".to_string(), 5)
            ]
        );
    }

    #[test]
    fn split_part_names() {
        assert_eq!(split_part("a-b-00002-of-00004.gguf"), Some(("a-b", 2, 4)));
        assert_eq!(split_part("model.gguf"), None);
        assert_eq!(split_part("x-2-of-4.gguf"), None, "needs 5-digit parts");
        assert_eq!(split_part("x-00001-of-00002.bin"), None);
    }

    #[test]
    fn missing_parts_of_a_split_model() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("m-00001-of-00003.gguf"), b"").unwrap();
        std::fs::write(dir.path().join("m-00003-of-00003.gguf"), b"").unwrap();
        assert_eq!(
            missing_parts(dir.path(), "m-00001-of-00003.gguf"),
            vec!["m-00002-of-00003.gguf"]
        );
        assert!(missing_parts(dir.path(), "single.gguf").is_empty());
    }

    #[test]
    fn model_sizes_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths::under(dir.path());
        std::fs::create_dir_all(&p.models_dir).unwrap();
        std::fs::write(p.models_dir.join("m.gguf"), vec![0u8; 1234]).unwrap();
        std::fs::create_dir_all(p.models_dir.join("folder.gguf")).unwrap();
        let list = p.list_models();
        assert!(list.contains(&("m.gguf".into(), 1234)));
    }
}
