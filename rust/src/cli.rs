//! Command-line parsing (kept free of Windows APIs so it can be unit-tested).

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallArgs {
    pub skip_model: bool,
    pub skip_power: bool,
    pub skip_wol: bool,
    pub llama_tag: Option<String>,
    pub update_llama: bool,
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
      --skip-model            don't download the default model
      --skip-power-settings   leave the power plan alone
      --skip-wake-on-lan      leave network adapter wake settings alone
      --llama-cpp-tag <tag>   install this llama.cpp build (default: latest)
      --update-llama-cpp      download llama.cpp again
  uninstall    Remove everything and restore your settings
      --keep-models           move models to Downloads first
      --yes                   don't ask
  update       Check for a new version and install it
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
                    "--skip-power-settings" => a.skip_power = true,
                    "--skip-wake-on-lan" => a.skip_wol = true,
                    "--update-llama-cpp" => a.update_llama = true,
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
            },
        ];
        for a in cases {
            let s = install_args_string(&a);
            let parsed = parse(s.split(' ').map(str::to_owned)).unwrap();
            assert_eq!(parsed, Command::Install(a), "{s}");
        }
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
        fn install_args_always_round_trip(skip_model: bool, skip_power: bool, skip_wol: bool, update_llama: bool, tag in prop::option::of("[a-z0-9._-]{1,12}")) {
            let a = InstallArgs { skip_model, skip_power, skip_wol, update_llama, llama_tag: tag };
            let s = install_args_string(&a);
            prop_assert_eq!(parse(s.split(' ').map(str::to_owned)), Ok(Command::Install(a)));
        }
    }
}
