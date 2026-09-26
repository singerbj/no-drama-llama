//! llama-server command line. Pure, so it can be unit-tested.

use crate::settings::{Reasoning, Settings};
use std::path::Path;

/// Qwen 3.8's chat template accepts reasoning_effort = low | medium | xhigh and raises an
/// error for anything else; "no thinking" is the separate switch enable_thinking = false.
pub fn chat_template_kwargs(r: Reasoning) -> String {
    match r {
        Reasoning::None => r#"{"enable_thinking":false}"#.into(),
        other => format!(r#"{{"reasoning_effort":"{}"}}"#, other.as_str()),
    }
}

/// Arguments for llama-server (passed as separate argv entries; std quotes them).
pub fn server_args(s: &Settings, model: &Path) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "-m".into(),
        model.to_string_lossy().into_owned(),
        "--host".into(),
        s.listen_host.clone(),
        "--port".into(),
        s.port.to_string(),
        "-ngl".into(),
        "99".into(),
        "-c".into(),
        s.context.to_string(),
        "-fa".into(),
        "on".into(),
        "--cache-type-k".into(),
        "q8_0".into(),
        "--cache-type-v".into(),
        "q8_0".into(),
        "--parallel".into(),
        "1".into(),
        "--jinja".into(),
        "--chat-template-kwargs".into(),
        chat_template_kwargs(s.reasoning),
    ];
    if !s.api_key.is_empty() {
        a.extend(["--api-key".into(), s.api_key.clone()]);
    }
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

    fn arg<'a>(a: &'a [String], name: &str) -> &'a str {
        let i = a
            .iter()
            .position(|x| x == name)
            .unwrap_or_else(|| panic!("{name} missing"));
        &a[i + 1]
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
    fn args_from_settings() {
        let s = Settings {
            listen_host: "0.0.0.0".into(),
            port: 9000,
            context: 65536,
            ..Default::default()
        };
        let a = server_args(&s, Path::new("C:/LLM/models/My Model.gguf"));
        assert_eq!(arg(&a, "-m"), "C:/LLM/models/My Model.gguf");
        assert_eq!(arg(&a, "--host"), "0.0.0.0");
        assert_eq!(arg(&a, "--port"), "9000");
        assert_eq!(arg(&a, "-c"), "65536");
        assert_eq!(
            arg(&a, "--chat-template-kwargs"),
            r#"{"reasoning_effort":"low"}"#
        );
        assert!(!a.contains(&"--api-key".to_string()));
    }

    #[test]
    fn sampling_and_api_key() {
        let none = Settings {
            reasoning: Reasoning::None,
            api_key: "s3cret".into(),
            ..Default::default()
        };
        let a = server_args(&none, Path::new("m.gguf"));
        assert_eq!(arg(&a, "--temp"), "0.7");
        assert_eq!(arg(&a, "--api-key"), "s3cret");
        assert_eq!(
            arg(
                &server_args(&Settings::default(), Path::new("m.gguf")),
                "--temp"
            ),
            "1.0"
        );
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use crate::settings::Reasoning;
    use proptest::prelude::*;

    fn flag_values(a: &[String]) -> std::collections::HashMap<&str, &str> {
        let mut m = std::collections::HashMap::new();
        let mut i = 0;
        while i < a.len() {
            if a[i].starts_with('-') && i + 1 < a.len() && !a[i + 1].starts_with("--") {
                m.insert(a[i].as_str(), a[i + 1].as_str());
                i += 2;
            } else {
                m.insert(a[i].as_str(), "");
                i += 1;
            }
        }
        m
    }

    #[test]
    fn fixed_performance_flags() {
        let a = server_args(&Settings::default(), Path::new("m.gguf"));
        let f = flag_values(&a);
        assert_eq!(f["-ngl"], "99", "all layers on the GPU");
        assert_eq!(f["-fa"], "on", "flash attention");
        assert_eq!(f["--cache-type-k"], "q8_0");
        assert_eq!(f["--cache-type-v"], "q8_0");
        assert_eq!(f["--parallel"], "1");
        assert!(
            f.contains_key("--jinja"),
            "chat template kwargs need --jinja"
        );
        assert_eq!(f["--top-k"], "20");
        assert_eq!(f["--min-p"], "0");
    }

    #[test]
    fn every_reasoning_level_gives_valid_template_json() {
        for r in Reasoning::ALL {
            let k = chat_template_kwargs(r);
            let v: serde_json::Value = serde_json::from_str(&k).unwrap();
            match r {
                Reasoning::None => assert_eq!(v["enable_thinking"], false),
                // Qwen 3.8's template only accepts these three
                _ => {
                    assert!(["low", "medium", "xhigh"]
                        .contains(&v["reasoning_effort"].as_str().unwrap()))
                }
            }
        }
    }

    #[test]
    fn thinking_modes_use_thinking_sampling() {
        for r in [Reasoning::Low, Reasoning::Medium, Reasoning::XHigh] {
            let a = server_args(
                &Settings {
                    reasoning: r,
                    ..Default::default()
                },
                Path::new("m.gguf"),
            );
            let f = flag_values(&a);
            assert_eq!((f["--temp"], f["--top-p"]), ("1.0", "0.95"), "{r:?}");
            assert!(!f.contains_key("--presence-penalty"));
        }
        let a = server_args(
            &Settings {
                reasoning: Reasoning::None,
                ..Default::default()
            },
            Path::new("m.gguf"),
        );
        let f = flag_values(&a);
        assert_eq!(
            (f["--temp"], f["--top-p"], f["--presence-penalty"]),
            ("0.7", "0.8", "1.5")
        );
    }

    #[test]
    fn no_flag_appears_twice() {
        let s = Settings {
            api_key: "k".into(),
            ..Default::default()
        };
        let a = server_args(&s, Path::new("m.gguf"));
        let flags: Vec<&String> = a
            .iter()
            .filter(|x| x.starts_with('-') && x.parse::<f64>().is_err())
            .collect();
        let mut dedup = flags.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(flags.len(), dedup.len(), "{flags:?}");
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

    #[test]
    fn model_path_is_one_argument_even_with_spaces() {
        let a = server_args(
            &Settings::default(),
            Path::new(r"C:\LLM\models\My Model (v2).gguf"),
        );
        let i = a.iter().position(|x| x == "-m").unwrap();
        assert_eq!(a[i + 1], r"C:\LLM\models\My Model (v2).gguf");
    }

    proptest! {
        /// Whatever validated settings contain, every value lands in exactly one argument.
        #[test]
        fn args_are_well_formed(port in 1024u16.., ctx in 512u32..1_048_576, key in "[A-Za-z0-9._~-]{0,20}") {
            let s = Settings { port, context: ctx, api_key: key.clone(), ..Default::default() };
            let a = server_args(&s, Path::new("m.gguf"));
            prop_assert!(a.iter().all(|x| !x.is_empty()));
            prop_assert_eq!(a.len() % 2, 1, "flag/value pairs + the lone --jinja");
            prop_assert_eq!(a.contains(&"--api-key".to_string()), !key.is_empty());
            let i = a.iter().position(|x| x == "--port").unwrap();
            prop_assert_eq!(&a[i + 1], &port.to_string());
            let i = a.iter().position(|x| x == "-c").unwrap();
            prop_assert_eq!(&a[i + 1], &ctx.to_string());
        }
    }
}
