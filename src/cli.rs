//! Command-line parsing (kept free of Windows APIs so it can be unit-tested).

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallArgs {
    pub skip_model: bool,
    pub skip_power: bool,
    pub skip_wol: bool,
    pub llama_tag: Option<String>,
    pub update_llama: bool,
    /// Catalog id (`qwen3.8-27b:UD-Q4_K_XL`), or None = pick for this PC
    pub model: Option<String>,
    /// vulkan | cuda12 | cuda13, or None = pick for this PC
    pub backend: Option<String>,
    /// Turn on Laya and install Ollaya
    pub laya: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// No arguments: install if needed, otherwise start the tray app.
    Default,
    Run {
        after_update: Option<String>,
    },
    Install(InstallArgs),
    Uninstall {
        keep_models: bool,
        yes: bool,
    },
    Update,
    /// The settings window (started by the tray, which talks to it over stdin/stdout).
    SettingsWindow,
    Models,
    Version,
    Help,
}

pub const HELP: &str = "\
No Drama Llama - local LLM server that pauses while you play

Usage: no-drama-llama.exe [command]

Commands:
  (none)       Install if needed, otherwise start the tray app
  run          Start the tray app
  install      Install or upgrade (needs admin; asks for it)
      --model <id|auto|none>  model to download (default: auto = best for your GPU;
                              ids: `no-drama-llama.exe models`)
      --skip-model            same as --model none
      --backend <name>        llama.cpp build: auto (default), vulkan, cuda12, cuda13
      --skip-power-settings   leave the power plan alone
      --skip-wake-on-lan      leave network adapter wake settings alone
      --llama-cpp-tag <tag>   install this llama.cpp build (default: latest)
      --update-llama-cpp      download llama.cpp again
      --laya                  also run Laya, a decision model (installs Ollaya from ollaya.dev)
  uninstall    Remove everything and restore your settings
      --keep-models           move models to Downloads first
      --yes                   don't ask
  update       Check for a new version and install it
  models       List downloadable models and what fits this PC
  version      Print the version
";

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Command, String> {
    let mut it = args.into_iter();
    let Some(cmd) = it.next() else {
        return Ok(Command::Default);
    };
    let rest: Vec<String> = it.collect();
    let mut flags = rest.iter().map(String::as_str).peekable();
    let unknown = |f: &str| Err(format!("unknown option '{f}' (see --help)"));
    match cmd.as_str() {
        "run" => {
            let mut after_update = None;
            while let Some(f) = flags.next() {
                match f {
                    "--after-update" => {
                        after_update = Some(
                            flags
                                .next()
                                .ok_or("--after-update needs a version")?
                                .to_string(),
                        )
                    }
                    _ => return unknown(f),
                }
            }
            Ok(Command::Run { after_update })
        }
        "install" => {
            let mut a = InstallArgs::default();
            while let Some(f) = flags.next() {
                match f {
                    "--skip-model" => a.skip_model = true,
                    "--model" => {
                        let m = flags
                            .next()
                            .ok_or("--model needs a model id, auto or none")?;
                        match m.to_ascii_lowercase().as_str() {
                            "none" => a.skip_model = true,
                            "auto" => a.model = None,
                            _ if crate::catalog::find(m).is_some() => a.model = Some(m.to_string()),
                            _ => {
                                return Err(format!(
                                    "unknown model '{m}' (see `no-drama-llama.exe models`)"
                                ))
                            }
                        }
                    }
                    "--backend" => {
                        let b = flags
                            .next()
                            .ok_or("--backend needs auto, vulkan, cuda12 or cuda13")?;
                        if b.eq_ignore_ascii_case("auto") {
                            a.backend = None;
                        } else if crate::hardware::Backend::parse(b).is_some() {
                            a.backend = Some(b.to_ascii_lowercase());
                        } else {
                            return Err(format!(
                                "unknown backend '{b}' (auto, vulkan, cuda12, cuda13)"
                            ));
                        }
                    }
                    "--skip-power-settings" => a.skip_power = true,
                    "--skip-wake-on-lan" => a.skip_wol = true,
                    "--update-llama-cpp" => a.update_llama = true,
                    "--laya" => a.laya = true,
                    "--llama-cpp-tag" => {
                        let t = flags
                            .next()
                            .ok_or("--llama-cpp-tag needs a tag, e.g. b6500")?;
                        if t.is_empty()
                            || !t
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
                        {
                            return Err(format!("invalid llama.cpp tag '{t}'"));
                        }
                        a.llama_tag = Some(t.to_string());
                    }
                    _ => return unknown(f),
                }
            }
            Ok(Command::Install(a))
        }
        "uninstall" => {
            let (mut keep_models, mut yes) = (false, false);
            for f in flags {
                match f {
                    "--keep-models" => keep_models = true,
                    "--yes" | "-y" => yes = true,
                    _ => return unknown(f),
                }
            }
            Ok(Command::Uninstall { keep_models, yes })
        }
        "update" if rest.is_empty() => Ok(Command::Update),
        "settings-window" if rest.is_empty() => Ok(Command::SettingsWindow),
        "models" if rest.is_empty() => Ok(Command::Models),
        "version" | "--version" | "-V" if rest.is_empty() => Ok(Command::Version),
        "help" | "--help" | "-h" | "/?" => Ok(Command::Help),
        other => Err(format!("unknown command '{other}' (see --help)")),
    }
}

/// Arguments to pass when re-launching elevated for an install.
pub fn install_args_string(a: &InstallArgs) -> String {
    let mut s = String::from("install");
    if a.skip_model {
        s += " --skip-model";
    }
    if a.skip_power {
        s += " --skip-power-settings";
    }
    if a.skip_wol {
        s += " --skip-wake-on-lan";
    }
    if a.update_llama {
        s += " --update-llama-cpp";
    }
    if let Some(t) = &a.llama_tag {
        s += &format!(" --llama-cpp-tag {t}");
    }
    if let Some(m) = &a.model {
        s += &format!(" --model {m}");
    }
    if let Some(b) = &a.backend {
        s += &format!(" --backend {b}");
    }
    if a.laya {
        s += " --laya";
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Result<Command, String> {
        parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_args_is_default() {
        assert_eq!(p(&[]), Ok(Command::Default));
    }

    #[test]
    fn run_and_after_update() {
        assert_eq!(p(&["run"]), Ok(Command::Run { after_update: None }));
        assert_eq!(
            p(&["run", "--after-update", "2.0.0"]),
            Ok(Command::Run {
                after_update: Some("2.0.0".into())
            })
        );
        assert!(p(&["run", "--after-update"]).is_err());
        assert!(p(&["run", "--bogus"]).is_err());
    }

    #[test]
    fn install_flags() {
        let a = match p(&[
            "install",
            "--skip-model",
            "--skip-power-settings",
            "--skip-wake-on-lan",
            "--update-llama-cpp",
            "--llama-cpp-tag",
            "b6500",
        ]) {
            Ok(Command::Install(a)) => a,
            other => panic!("{other:?}"),
        };
        assert!(a.skip_model && a.skip_power && a.skip_wol && a.update_llama);
        assert!(!a.laya);
        assert!(matches!(p(&["install", "--laya"]), Ok(Command::Install(a)) if a.laya));
        assert_eq!(a.llama_tag.as_deref(), Some("b6500"));
        assert_eq!(
            p(&["install"]),
            Ok(Command::Install(InstallArgs::default()))
        );
    }

    #[test]
    fn install_rejects_bad_input() {
        assert!(p(&["install", "--llama-cpp-tag"]).is_err());
        assert!(p(&["install", "--llama-cpp-tag", "../../evil"]).is_err());
        assert!(p(&["install", "--llama-cpp-tag", "b1 && calc"]).is_err());
        assert!(p(&["install", "--skip-everything"]).is_err());
    }

    #[test]
    fn install_args_round_trip() {
        let cases = [
            InstallArgs::default(),
            InstallArgs {
                skip_model: true,
                ..Default::default()
            },
            InstallArgs {
                skip_power: true,
                skip_wol: true,
                llama_tag: Some("b6500".into()),
                update_llama: true,
                skip_model: true,
                model: None,
                backend: Some("cuda13".into()),
                laya: true,
            },
            InstallArgs {
                model: Some("qwen3.8-27b:UD-IQ3_XXS".into()),
                backend: Some("vulkan".into()),
                ..Default::default()
            },
        ];
        for a in cases {
            let s = install_args_string(&a);
            let parsed = parse(s.split(' ').map(str::to_owned)).unwrap();
            assert_eq!(parsed, Command::Install(a), "{s}");
        }
    }

    #[test]
    fn model_and_backend_options() {
        let a = |args: &[&str]| match p(args) {
            Ok(Command::Install(a)) => a,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            a(&["install", "--model", "qwen3.8-27b:UD-Q4_K_XL"])
                .model
                .as_deref(),
            Some("qwen3.8-27b:UD-Q4_K_XL")
        );
        assert!(a(&["install", "--model", "none"]).skip_model);
        assert_eq!(a(&["install", "--model", "auto"]).model, None);
        assert_eq!(
            a(&["install", "--backend", "CUDA12"]).backend.as_deref(),
            Some("cuda12")
        );
        assert_eq!(a(&["install", "--backend", "auto"]).backend, None);
        assert!(p(&["install", "--model", "gpt-9"])
            .unwrap_err()
            .contains("unknown model"));
        assert!(p(&["install", "--model"]).is_err());
        assert!(p(&["install", "--backend", "rocm"])
            .unwrap_err()
            .contains("unknown backend"));
        assert_eq!(p(&["models"]), Ok(Command::Models));
    }

    #[test]
    fn uninstall_flags() {
        assert_eq!(
            p(&["uninstall"]),
            Ok(Command::Uninstall {
                keep_models: false,
                yes: false
            })
        );
        assert_eq!(
            p(&["uninstall", "--keep-models", "-y"]),
            Ok(Command::Uninstall {
                keep_models: true,
                yes: true
            })
        );
        assert!(p(&["uninstall", "--keep"]).is_err());
    }

    #[test]
    fn misc_commands() {
        assert_eq!(p(&["update"]), Ok(Command::Update));
        assert!(p(&["update", "now"]).is_err());
        assert_eq!(p(&["settings-window"]), Ok(Command::SettingsWindow));
        assert!(p(&["settings-window", "x"]).is_err());
        for v in ["version", "--version", "-V"] {
            assert_eq!(p(&[v]), Ok(Command::Version));
        }
        for h in ["help", "--help", "-h", "/?"] {
            assert_eq!(p(&[h]), Ok(Command::Help));
        }
        assert!(p(&["explode"]).unwrap_err().contains("unknown command"));
    }

    #[test]
    fn help_mentions_every_command_and_flag() {
        for w in [
            "models",
            "--model",
            "--backend",
            "run",
            "install",
            "uninstall",
            "update",
            "version",
            "--skip-model",
            "--skip-power-settings",
            "--skip-wake-on-lan",
            "--llama-cpp-tag",
            "--update-llama-cpp",
            "--keep-models",
            "--yes",
            "--laya",
        ] {
            assert!(HELP.contains(w), "{w}");
        }
    }
}

#[cfg(test)]
mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parsing_never_panics(args in prop::collection::vec(".{0,20}", 0..6)) {
            let _ = parse(args);
        }

        #[test]
        fn install_args_always_round_trip(
            skip_model: bool, skip_power: bool, skip_wol: bool, update_llama: bool, laya: bool,
            tag in prop::option::of("[a-z0-9._-]{1,12}"),
            model in prop::option::of(prop::sample::select(crate::catalog::catalog().into_iter().map(|m| m.id).collect::<Vec<_>>())),
            backend in prop::option::of(prop::sample::select(vec!["vulkan".to_string(), "cuda12".to_string(), "cuda13".to_string()])),
        ) {
            let a = InstallArgs { skip_model, skip_power, skip_wol, update_llama, llama_tag: tag, model, backend, laya };
            let s = install_args_string(&a);
            prop_assert_eq!(parse(s.split(' ').map(str::to_owned)), Ok(Command::Install(a)));
        }
    }
}
