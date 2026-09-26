//! Downloads, resume, checksums and the health probe against a local HTTP server.

use super::super::net;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    /// Ignores Range and always sends the whole body with 200.
    NoRange,
    /// Sends only half the body, then closes.
    Truncate,
    Status(u16),
    /// Sends the body in 50 small pieces, `gap` apart.
    Trickle(Duration),
    /// Sends half the body, then holds the connection open without sending anything for `hold`.
    /// Requests with a Range are served normally.
    Stall(Duration),
}

struct Server {
    url: String,
    hits: Arc<AtomicUsize>,
}

fn serve(body: Vec<u8>, mode: Mode) -> Server {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/file", l.local_addr().unwrap());
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    let body = Arc::new(body);
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            h.fetch_add(1, Ordering::SeqCst);
            let body = body.clone();
            std::thread::spawn(move || respond(s, &body, mode));
        }
    });
    Server { url, hits }
}

fn respond(mut s: TcpStream, body: &[u8], mode: Mode) {
    let mut reader = BufReader::new(s.try_clone().unwrap());
    let mut range_start: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("range: bytes=") {
            range_start = v.trim().trim_end_matches('-').parse().ok();
        }
    }
    let ranges = matches!(mode, Mode::Normal | Mode::Stall(_));
    let (status, part): (String, &[u8]) = match (mode, range_start) {
        (Mode::Status(c), _) => (format!("{c} Nope"), b"nope"),
        (_, Some(r)) if ranges && r >= body.len() => ("416 Range Not Satisfiable".into(), b""),
        (_, Some(r)) if ranges => ("206 Partial Content".into(), &body[r..]),
        _ => ("200 OK".into(), body),
    };
    let _ = write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        part.len()
    );
    match (mode, range_start) {
        (Mode::Truncate, _) => {
            let _ = s.write_all(&part[..part.len() / 2]);
        }
        (Mode::Trickle(gap), _) => {
            for piece in part.chunks(part.len().div_ceil(50).max(1)) {
                let _ = s.write_all(piece);
                let _ = s.flush();
                std::thread::sleep(gap);
            }
        }
        (Mode::Stall(hold), None) => {
            let _ = s.write_all(&part[..part.len() / 2]);
            let _ = s.flush();
            std::thread::sleep(hold);
        }
        _ => {
            let _ = s.write_all(part);
        }
    }
    let _ = s.flush();
}

fn body(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i * 31 % 251) as u8).collect()
}

#[test]
fn full_download() {
    let data = body(3_000_000);
    let srv = serve(data.clone(), Mode::Normal);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    let mut calls = 0;
    net::download(&srv.url, &dest, |done, total| {
        calls += 1;
        assert!(done <= total);
    })
    .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert!(calls >= 1, "progress reported");
}

#[test]
fn resumes_a_partial_file() {
    let data = body(1_000_000);
    let srv = serve(data.clone(), Mode::Normal);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    std::fs::write(&dest, &data[..400_000]).unwrap();
    net::download(&srv.url, &dest, |_, total| {
        assert_eq!(total, data.len() as u64)
    })
    .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
}

#[test]
fn complete_file_gets_416_and_is_left_alone() {
    let data = body(10_000);
    let srv = serve(data.clone(), Mode::Normal);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    std::fs::write(&dest, &data).unwrap();
    net::download(&srv.url, &dest, |_, _| {}).unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
}

#[test]
fn server_without_range_support_restarts_cleanly() {
    let data = body(50_000);
    let srv = serve(data.clone(), Mode::NoRange);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    std::fs::write(&dest, b"garbage that must not stay in front").unwrap();
    net::download(&srv.url, &dest, |_, _| {}).unwrap();
    assert_eq!(
        std::fs::read(&dest).unwrap(),
        data,
        "file truncated, not appended"
    );
}

#[test]
fn truncated_body_is_an_error_but_keeps_progress() {
    let data = body(200_000);
    let srv = serve(data.clone(), Mode::Truncate);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    let e = net::download(&srv.url, &dest, |_, _| {}).unwrap_err();
    let msg = format!("{e:#}");
    assert!(msg.contains("resume"), "{msg}");
    let kept = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    assert!(
        kept > 0 && kept < data.len() as u64,
        "partial file kept for resuming ({kept})"
    );
}

#[test]
fn slow_but_steady_download_is_not_cut_off() {
    // 50 pieces 60 ms apart: about 3 s in total, far past the 500 ms stall threshold,
    // but never more than 60 ms without data.
    let data = body(100_000);
    let srv = serve(data.clone(), Mode::Trickle(Duration::from_millis(60)));
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    let t = Instant::now();
    net::download_with_stall(&srv.url, &dest, |_, _| {}, None, Duration::from_millis(500)).unwrap();
    assert!(t.elapsed() > Duration::from_secs(2), "{:?}", t.elapsed());
    assert_eq!(std::fs::read(&dest).unwrap(), data);
}

#[test]
fn stalled_download_is_abandoned_then_resumes() {
    let data = body(100_000);
    let srv = serve(data.clone(), Mode::Stall(Duration::from_secs(30)));
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("f.part");
    let stall = Duration::from_millis(500);
    let t = Instant::now();
    let e = net::download_with_stall(&srv.url, &dest, |_, _| {}, None, stall).unwrap_err();
    assert!(t.elapsed() < Duration::from_secs(10), "{:?}", t.elapsed());
    let msg = format!("{e:#}");
    assert!(
        msg.contains("resume") && msg.contains("no data received"),
        "{msg}"
    );
    let kept = std::fs::metadata(&dest).unwrap().len();
    assert_eq!(
        kept,
        data.len() as u64 / 2,
        "partial file kept for resuming"
    );

    net::download_with_stall(&srv.url, &dest, |_, _| {}, None, stall).unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert_eq!(srv.hits.load(Ordering::SeqCst), 2);
}

#[test]
fn http_errors_fail() {
    for code in [404u16, 500, 403] {
        let srv = serve(Vec::new(), Mode::Status(code));
        let dir = tempfile::tempdir().unwrap();
        let e = net::download(&srv.url, &dir.path().join("f"), |_, _| {}).unwrap_err();
        assert!(e.to_string().contains(&code.to_string()), "{e}");
    }
}

#[test]
fn sha256_of_file() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("x");
    std::fs::write(&f, b"abc").unwrap();
    assert_eq!(
        net::file_sha256(&f).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let big = body(5_000_000);
    std::fs::write(&f, &big).unwrap();
    assert_eq!(
        net::file_sha256(&f).unwrap(),
        crate::update::sha256_hex(&big)
    );
    assert!(net::file_sha256(&dir.path().join("missing")).is_err());
}

#[test]
fn health_probe() {
    let ok = serve(b"{}".to_vec(), Mode::Normal);
    let loading = serve(Vec::new(), Mode::Status(503));
    let agent = net::health_agent();
    assert!(net::is_healthy(&agent, &ok.url));
    assert!(!net::is_healthy(&agent, &loading.url));
    let closed = format!("http://127.0.0.1:{}/health", super::free_port());
    let t = std::time::Instant::now();
    assert!(!net::is_healthy(&agent, &closed));
    assert!(
        t.elapsed() < std::time::Duration::from_secs(3),
        "fails fast"
    );
    assert!(ok.hits.load(Ordering::SeqCst) >= 1);
}

#[test]
fn get_text_and_bytes_limits() {
    let srv = serve(b"hello".to_vec(), Mode::Normal);
    let agent = net::agent();
    assert_eq!(net::get_text(&agent, &srv.url).unwrap(), "hello");
    let big = serve(body(10_000), Mode::Normal);
    assert!(
        net::get_bytes(&agent, &big.url, 100).is_err(),
        "limit enforced"
    );
    assert_eq!(
        net::get_bytes(&agent, &big.url, 1_000_000).unwrap().len(),
        10_000
    );
}
