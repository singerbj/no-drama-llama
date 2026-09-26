//! Install layout (shared with the PowerShell edition, so models and settings carry over).
//!
//! ```text
//! %ProgramFiles%\No Drama Llama\no-drama-llama.exe   the app (admin-only, Windows default)
//! C:\LLM\                admin-only (the elevated app runs llama-server.exe from here)
//!   llama\               llama.cpp binaries
//!   data\                settings.json, off.flag, logs
//!   models\              *.gguf (you can add your own)
//!   settings-backup.json your original Windows settings, restored on uninstall
//! ```

use std::path::PathBuf;

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
    pub data_dir: PathBuf,
    pub settings: PathBuf,
    pub off_flag: PathBuf,
    pub log: PathBuf,
    pub server_log: PathBuf,
    pub backup: PathBuf,
}

impl Paths {
    pub fn under(root: impl Into<PathBuf>) -> Paths {
        let root = root.into();
        let data_dir = root.join("data");
        Paths {
            llama_dir: root.join("llama"),
            server_exe: root.join("llama").join("llama-server.exe"),
            models_dir: root.join("models"),
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

    pub fn model(&self, name: &str) -> PathBuf {
        self.models_dir.join(name)
    }

    /// `.gguf` models in the models folder (vision projectors and imatrix files excluded), sorted.
    pub fn list_models(&self) -> Vec<(String, u64)> {
        let mut v: Vec<(String, u64)> = std::fs::read_dir(&self.models_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let lower = name.to_ascii_lowercase();
                let is_model = lower.ends_with(".gguf")
                    && !lower.starts_with("mmproj")
                    && !lower.starts_with("imatrix");
                is_model.then(|| (name, e.metadata().map(|m| m.len()).unwrap_or(0)))
            })
            .collect();
        v.sort();
        v
    }
}

/// Where the installed app lives: `%ProgramFiles%\No Drama Llama`.
pub fn install_dir() -> PathBuf {
    let pf = std::env::var_os("ProgramW6432")
        .or_else(|| std::env::var_os("ProgramFiles"))
        .unwrap_or_else(|| r"C:\Program Files".into());
    PathBuf::from(pf).join(APP_NAME)
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
    fn layout_matches_the_powershell_edition() {
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
        // and the PowerShell config agrees
        let ps = include_str!("../../src/config.ps1");
        for needle in [
            "$Root         = 'C:\\LLM'",
            "\"$Root\\llama\\llama-server.exe\"",
            "\"$DataDir\\settings.json\"",
            "\"$Root\\settings-backup.json\"",
        ] {
            assert!(ps.contains(needle), "{needle}");
        }
    }

    #[test]
    fn user_writable_state_stays_out_of_llama_and_root() {
        let p = Paths::under("/x");
        for f in [&p.settings, &p.off_flag, &p.log, &p.server_log] {
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
