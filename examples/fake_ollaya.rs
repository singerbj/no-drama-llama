//! Test double for `ollaya.exe` (used by the Windows end-to-end tests).
//!
//! `ollaya serve` writes its `OLLAYA_*` environment to `env.txt` next to itself and listens on
//! `OLLAYA_HOST`. It answers `GET /`, `GET /api/tags` (the models in `pulled.txt` in
//! `OLLAYA_MODELS`), `POST /api/pull` (streams progress, then adds the model to `pulled.txt`) and
//! `POST /api/decide` (appends the body to `loaded.txt`). With `OLLAYA_API_KEY` set, everything
//! but `GET /` needs the bearer token. If a file named `crash` is next to it, it exits right away.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap().to_path_buf();
    if std::env::args().nth(1).as_deref() != Some("serve") {
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
    if dir.join("crash").exists() {
        eprintln!("fake ollaya: crashing on purpose");
        std::process::exit(1);
    }
    let mut env: Vec<String> = std::env::vars()
        .filter(|(k, _)| k.starts_with("OLLAYA_"))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    env.sort();
    std::fs::write(dir.join("env.txt"), env.join("\n")).unwrap();
    let host = std::env::var("OLLAYA_HOST").unwrap_or_else(|_| "127.0.0.1:11435".into());
    let models = PathBuf::from(std::env::var("OLLAYA_MODELS").unwrap());
    std::fs::create_dir_all(&models).unwrap();
    let key = std::env::var("OLLAYA_API_KEY").ok();
    eprintln!("fake ollaya listening on {host}");
    let listener = TcpListener::bind(&host).expect("bind");
    for stream in listener.incoming().flatten() {
        let (models, key) = (models.clone(), key.clone());
        std::thread::spawn(move || handle(stream, &models, key.as_deref()));
    }
}

fn respond(s: &mut TcpStream, status: &str, body: &str) {
    let _ = write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

fn canonical(m: &str) -> String {
    let m = m.to_ascii_lowercase();
    if m.contains(':') {
        m
    } else {
        format!("{m}:latest")
    }
}

fn model_of(body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    v["model"].as_str().unwrap_or_default().to_string()
}

fn handle(mut s: TcpStream, models: &Path, key: Option<&str>) {
    let mut r = BufReader::new(s.try_clone().unwrap());
    let mut first = String::new();
    let _ = r.read_line(&mut first);
    let mut parts = first.split_whitespace();
    let (method, path) = (
        parts.next().unwrap_or("").to_string(),
        parts.next().unwrap_or("/").to_string(),
    );
    let (mut len, mut auth) = (0usize, String::new());
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            len = v.trim().parse().unwrap_or(0);
        }
        if lower.starts_with("authorization:") {
            auth = line["authorization:".len()..].trim().to_string();
        }
    }
    let mut body = vec![0u8; len];
    let _ = r.read_exact(&mut body);
    let body = String::from_utf8_lossy(&body).to_string();

    if (method.as_str(), path.as_str()) == ("GET", "/") {
        return respond(&mut s, "200 OK", "Ollaya is running");
    }
    if let Some(k) = key {
        if auth != format!("Bearer {k}") {
            return respond(
                &mut s,
                "401 Unauthorized",
                r#"{"error":"missing or wrong API key","code":"UNAUTHORIZED"}"#,
            );
        }
    }
    let pulled = models.join("pulled.txt");
    match (method.as_str(), path.as_str()) {
        ("GET", "/api/tags") => {
            let list: Vec<serde_json::Value> = std::fs::read_to_string(&pulled)
                .unwrap_or_default()
                .lines()
                .map(|m| serde_json::json!({ "name": m, "model": m }))
                .collect();
            respond(
                &mut s,
                "200 OK",
                &serde_json::json!({ "models": list }).to_string(),
            );
        }
        ("POST", "/api/pull") => {
            let m = model_of(&body);
            if m == "missing" {
                return respond(
                    &mut s,
                    "404 Not Found",
                    r#"{"error":"model \"missing:latest\" not found in registry ollaya.dev","code":"MODEL_NOT_FOUND"}"#,
                );
            }
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nConnection: close\r\n\r\n"
            );
            let _ = writeln!(s, r#"{{"status":"pulling manifest"}}"#);
            for done in [0, 500_000, 1_000_000] {
                std::thread::sleep(Duration::from_millis(150));
                let _ = writeln!(
                    s,
                    r#"{{"status":"pulling 8d32a80bb199","digest":"sha256:8d32","total":1000000,"completed":{done}}}"#
                );
            }
            let mut all = std::fs::read_to_string(&pulled).unwrap_or_default();
            all.push_str(&canonical(&m));
            all.push('\n');
            std::fs::write(&pulled, all).unwrap();
            let _ = writeln!(s, r#"{{"status":"writing manifest"}}"#);
            let _ = writeln!(s, r#"{{"status":"success"}}"#);
        }
        ("POST", "/api/decide") => {
            let mut all = std::fs::read_to_string(models.join("loaded.txt")).unwrap_or_default();
            all.push_str(&body);
            all.push('\n');
            std::fs::write(models.join("loaded.txt"), all).unwrap();
            respond(
                &mut s,
                "200 OK",
                &serde_json::json!({ "model": model_of(&body), "answers": {}, "done_reason": "load" })
                    .to_string(),
            );
        }
        _ => respond(
            &mut s,
            "404 Not Found",
            r#"{"error":"no such endpoint","code":"NOT_FOUND"}"#,
        ),
    }
}
