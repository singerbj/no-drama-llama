//! Test double for llama-server.exe (used by the Windows end-to-end tests).
//!
//! * Named `llama-server(.exe)`: records its arguments to `argv.txt` next to itself, listens
//!   on `--port`, answers `/health` with 503 for `FAKE_READY_MS` (default 400 ms) then 200.
//!   If a file named `crash` exists next to it, exits with code 1 right away.
//! * Any other name (a "game"): just sleeps until killed.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

fn main() {
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap().to_path_buf();
    let stem = exe.file_stem().unwrap().to_string_lossy().to_lowercase();
    if stem != "llama-server" {
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
    if dir.join("crash").exists() {
        eprintln!("fake llama-server: crashing on purpose");
        std::process::exit(1);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::fs::write(dir.join("argv.txt"), args.join("\n")).unwrap();
    let port = args
        .iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let ready_ms: u64 = std::env::var("FAKE_READY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400);
    let started = Instant::now();
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind");
    for stream in listener.incoming().flatten() {
        let mut s = stream;
        let mut line = String::new();
        let _ = BufReader::new(&s).read_line(&mut line);
        let ready = started.elapsed() >= Duration::from_millis(ready_ms);
        let (code, body) = if !line.starts_with("GET /health") {
            ("404 Not Found", "{}")
        } else if ready {
            ("200 OK", r#"{"status":"ok"}"#)
        } else {
            (
                "503 Service Unavailable",
                r#"{"error":{"message":"Loading model"}}"#,
            )
        };
        let _ = write!(s, "HTTP/1.1 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    }
}
