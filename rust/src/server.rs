//! llama-server command line. Pure, so it can be unit-tested.

use crate::settings::{Reasoning, Settings, CONTEXT_AUTO};
use std::path::Path;

/// Qwen 3.8's chat template accepts reasoning_effort = low | medium | xhigh and raises an
/// error for anything else; "no thinking" is the separate switch enable_thinking = false.
pub fn chat_template_kwargs(r: Reasoning) -> String {
    match r {
        Reasoning::None => r#"{"enable_thinking":false}"#.into(),
        other => format!(r#"{{"reasoning_effort":"{}"}}"#, other.as_str()),
    }
}

/// What the installed llama.cpp build can do (from `llama-server --help`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerCaps {
    /// `--fit`: sizes GPU layers / context / MoE CPU offload to the free device memory.
    pub fit: bool,
}

impl Default for ServerCaps {
    fn default() -> Self {
        ServerCaps { fit: true }
    }
}

/// Context used with builds that can't fit automatically.
pub const FALLBACK_CONTEXT: u32 = 32768;

/// Qwen 3.8's recommended sampling and reasoning switches only make sense for Qwen models;
/// anything else you drop into the models folder runs on its own GGUF defaults.
pub fn is_qwen(model_file: &str) -> bool {
    model_file.to_ascii_lowercase().starts_with("qwen")
}

/// Arguments for llama-server (passed as separate argv entries; std quotes them).
pub fn server_args(s: &Settings, model: &Path, caps: ServerCaps) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "-m".into(),
        model.to_string_lossy().into_owned(),
        "--host".into(),
        s.listen_host.clone(),
        "--port".into(),
        s.port.to_string(),
    ];
    if caps.fit {
        // Leave -ngl unset so llama.cpp fits layers (and MoE experts) to this GPU's memory,
        // and -c unset for "auto" so it picks the largest context that fits.
        if s.context != CONTEXT_AUTO {
            a.extend(["-c".into(), s.context.to_string()]);
        }
    } else {
        let ctx = if s.context == CONTEXT_AUTO {
            FALLBACK_CONTEXT
        } else {
            s.context
        };
        a.extend(["-ngl".into(), "99".into(), "-c".into(), ctx.to_string()]);
    }
    a.extend(
        [
            "-fa",
            "on",
            "--cache-type-k",
            "q8_0",
            "--cache-type-v",
            "q8_0",
            "--parallel",
            "1",
            "--jinja",
        ]
        .iter()
        .map(|x| x.to_string()),
    );
    if !s.api_key.is_empty() {
        a.extend(["--api-key".into(), s.api_key.clone()]);
    }
    let file = model
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    if is_qwen(&file) {
        a.extend([
            "--chat-template-kwargs".into(),
            chat_template_kwargs(s.reasoning),
        ]);
        // Sampling recommended on the model card: non-thinking vs. thinking mode
        let sampling: &[&str] = if s.reasoning == Reasoning::None {
            &[
                "--temp",
                "0.7",
                "--top-p",
                "0.8",
                "--top-k",
                "20",
                "--min-p",
                "0",
                "--presence-penalty",
                "1.5",
            ]
        } else {
            &[
                "--temp", "1.0", "--top-p", "0.95", "--top-k", "20", "--min-p", "0",
            ]
        };
        a.extend(sampling.iter().map(|x| x.to_string()));
    }
    a
}

pub fn health_url(s: &Settings) -> String {
    format!("http://127.0.0.1:{}/health", s.port)
}

pub fn chat_url(s: &Settings) -> String {
    format!("http://127.0.0.1:{}", s.port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Reasoning;
    use proptest::prelude::*;

    const QWEN: &str = "C:/LLM/models/Qwen3.8-27B-UD-Q4_K_XL.gguf";
    const FIT: ServerCaps = ServerCaps { fit: true };
    const OLD: ServerCaps = ServerCaps { fit: false };

    fn arg<'a>(a: &'a [String], name: &str) -> Option<&'a str> {
        a.iter().position(|x| x == name).map(|i| a[i + 1].as_str())
    }

    fn args(s: &Settings) -> Vec<String> {
        server_args(s, Path::new(QWEN), FIT)
    }

    #[test]
    fn kwargs() {
        assert_eq!(
            chat_template_kwargs(Reasoning::None),
            r#"{"enable_thinking":false}"#
        );
        assert_eq!(
            chat_template_kwargs(Reasoning::Low),
            r#"{"reasoning_effort":"low"}"#
        );
        assert_eq!(
            chat_template_kwargs(Reasoning::XHigh),
            r#"{"reasoning_effort":"xhigh"}"#
        );
    }

    #[test]
    fn auto_context_lets_llama_cpp_fit_everything() {
        let a = args(&Settings::default());
        assert_eq!(arg(&a, "-c"), None, "context left to --fit");
        assert_eq!(
            arg(&a, "-ngl"),
            None,
            "GPU layers / MoE offload left to --fit"
        );
        assert_eq!(arg(&a, "--host"), Some("127.0.0.1"));
        assert_eq!(arg(&a, "--port"), Some("8080"));
    }

    #[test]
    fn fixed_context_is_passed_and_layers_still_fit() {
        let a = args(&Settings {
            context: 65536,
            ..Default::default()
        });
        assert_eq!(arg(&a, "-c"), Some("65536"));
        assert_eq!(arg(&a, "-ngl"), None);
    }

    #[test]
    fn builds_without_fit_get_the_old_explicit_settings() {
        let a = server_args(&Settings::default(), Path::new(QWEN), OLD);
        assert_eq!(arg(&a, "-ngl"), Some("99"));
        assert_eq!(arg(&a, "-c"), Some(&*FALLBACK_CONTEXT.to_string()));
        let a = server_args(
            &Settings {
                context: 8192,
                ..Default::default()
            },
            Path::new(QWEN),
            OLD,
        );
        assert_eq!(arg(&a, "-c"), Some("8192"));
    }

    #[test]
    fn fixed_performance_flags() {
        let a = args(&Settings::default());
        assert_eq!(arg(&a, "-fa"), Some("on"), "flash attention");
        assert_eq!(arg(&a, "--cache-type-k"), Some("q8_0"));
        assert_eq!(arg(&a, "--cache-type-v"), Some("q8_0"));
        assert_eq!(arg(&a, "--parallel"), Some("1"));
        assert!(a.contains(&"--jinja".to_string()));
    }

    #[test]
    fn qwen_gets_reasoning_and_model_card_sampling() {
        for r in [Reasoning::Low, Reasoning::Medium, Reasoning::XHigh] {
            let a = args(&Settings {
                reasoning: r,
                ..Default::default()
            });
            assert_eq!(
                (arg(&a, "--temp"), arg(&a, "--top-p")),
                (Some("1.0"), Some("0.95")),
                "{r:?}"
            );
            assert_eq!(arg(&a, "--presence-penalty"), None);
            let k: serde_json::Value =
                serde_json::from_str(arg(&a, "--chat-template-kwargs").unwrap()).unwrap();
            assert!(["low", "medium", "xhigh"].contains(&k["reasoning_effort"].as_str().unwrap()));
        }
        let a = args(&Settings {
            reasoning: Reasoning::None,
            ..Default::default()
        });
        assert_eq!(
            (
                arg(&a, "--temp"),
                arg(&a, "--top-p"),
                arg(&a, "--presence-penalty")
            ),
            (Some("0.7"), Some("0.8"), Some("1.5"))
        );
        assert_eq!(
            arg(&a, "--chat-template-kwargs"),
            Some(r#"{"enable_thinking":false}"#)
        );
    }

    #[test]
    fn other_models_use_their_own_defaults() {
        for f in [
            "C:/LLM/models/gemma-4-27b-Q4_K_M.gguf",
            "C:/LLM/models/Llama-4-Scout.gguf",
            "C:/LLM/models/my.gguf",
        ] {
            let a = server_args(&Settings::default(), Path::new(f), FIT);
            assert_eq!(arg(&a, "--temp"), None, "{f}");
            assert_eq!(arg(&a, "--chat-template-kwargs"), None, "{f}");
            assert!(a.contains(&"--jinja".to_string()));
        }
        assert!(is_qwen("qwen3.8-flash-next-UD-IQ1_S-00001-of-00003.gguf"));
    }

    #[test]
    fn api_key_only_when_set() {
        assert_eq!(arg(&args(&Settings::default()), "--api-key"), None);
        assert_eq!(
            arg(
                &args(&Settings {
                    api_key: "s3cret".into(),
                    ..Default::default()
                }),
                "--api-key"
            ),
            Some("s3cret")
        );
    }

    #[test]
    fn model_path_is_one_argument_even_with_spaces() {
        let a = server_args(
            &Settings::default(),
            Path::new(r"C:\LLM\models\My Model (v2).gguf"),
            FIT,
        );
        assert_eq!(arg(&a, "-m"), Some(r"C:\LLM\models\My Model (v2).gguf"));
    }

    #[test]
    fn no_flag_appears_twice() {
        for caps in [FIT, OLD] {
            let a = server_args(
                &Settings {
                    api_key: "k".into(),
                    context: 4096,
                    ..Default::default()
                },
                Path::new(QWEN),
                caps,
            );
            let flags: Vec<&String> = a
                .iter()
                .filter(|x| x.starts_with('-') && x.parse::<f64>().is_err())
                .collect();
            let mut dedup = flags.clone();
            dedup.sort();
            dedup.dedup();
            assert_eq!(flags.len(), dedup.len(), "{flags:?}");
        }
    }

    #[test]
    fn urls_always_use_loopback() {
        let s = Settings {
            listen_host: "0.0.0.0".into(),
            port: 9123,
            ..Default::default()
        };
        assert_eq!(health_url(&s), "http://127.0.0.1:9123/health");
        assert_eq!(chat_url(&s), "http://127.0.0.1:9123");
    }

    proptest! {
        #[test]
        fn args_are_well_formed(port in 1024u16.., ctx in prop_oneof![Just(0u32), 512u32..1_048_576], key in "[A-Za-z0-9._~-]{0,20}", fit: bool, qwen: bool) {
            let s = Settings { port, context: ctx, api_key: key.clone(), ..Default::default() };
            let model = if qwen { QWEN } else { "C:/m/other.gguf" };
            let a = server_args(&s, Path::new(model), ServerCaps { fit });
            prop_assert!(a.iter().all(|x| !x.is_empty()));
            prop_assert_eq!(a.len() % 2, 1, "flag/value pairs + the lone --jinja");
            prop_assert_eq!(arg(&a, "--port"), Some(&*port.to_string()));
            prop_assert_eq!(a.contains(&"--api-key".to_string()), !key.is_empty());
            prop_assert_eq!(arg(&a, "-ngl").is_some(), !fit);
            match (fit, ctx) {
                (true, 0) => prop_assert_eq!(arg(&a, "-c"), None),
                (false, 0) => prop_assert_eq!(arg(&a, "-c"), Some(&*FALLBACK_CONTEXT.to_string())),
                _ => prop_assert_eq!(arg(&a, "-c"), Some(&*ctx.to_string())),
            }
        }
    }
}
