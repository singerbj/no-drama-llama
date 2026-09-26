//! A tiny fake Hugging Face: `/api/models/<repo>/tree/main[/<dir>]` listings with LFS
//! size + sha256, and `/<repo>/resolve/main/<path>` downloads with Range support.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub struct FakeFile {
    pub repo: String,
    pub path: String,
    pub data: Vec<u8>,
    /// What the listing claims (defaults to the real hash)
    pub sha: Option<String>,
}

pub fn file(repo: &str, path: &str, data: Vec<u8>) -> FakeFile {
    FakeFile {
        repo: repo.into(),
        path: path.into(),
        data,
        sha: None,
    }
}

pub struct FakeHf {
    pub base: String,
    pub downloads: Arc<Mutex<Vec<String>>>,
}

pub fn start(files: Vec<FakeFile>) -> FakeHf {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let files: Arc<HashMap<String, FakeFile>> = Arc::new(
        files
            .into_iter()
            .map(|f| (format!("{}/{}", f.repo, f.path), f))
            .collect(),
    );
    let downloads = Arc::new(Mutex::new(Vec::new()));
    let dl = downloads.clone();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            let files = files.clone();
            let dl = dl.clone();
            std::thread::spawn(move || {
                let mut s = s;
                let mut r = BufReader::new(s.try_clone().unwrap());
                let mut first = String::new();
                let _ = r.read_line(&mut first);
                let path = first.split_whitespace().nth(1).unwrap_or("/").to_string();
                let mut range: Option<usize> = None;
                loop {
                    let mut line = String::new();
                    if r.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    if let Some(v) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                        range = v.trim().trim_end_matches('-').parse().ok();
                    }
                }
                let (status, body): (&str, Vec<u8>) = if let Some(rest) =
                    path.strip_prefix("/api/models/")
                {
                    // <owner>/<name>/tree/main[/<dir>]
                    let parts: Vec<&str> = rest.splitn(5, '/').collect();
                    let repo = format!("{}/{}", parts[0], parts[1]);
                    let dir = parts.get(4).copied().unwrap_or("");
                    let list: Vec<serde_json::Value> = files
                        .values()
                        .filter(|f| f.repo == repo && f.path.rsplit_once('/').map(|(d, _)| d).unwrap_or("") == dir)
                        .map(|f| {
                            let sha = f.sha.clone().unwrap_or_else(|| crate::update::sha256_hex(&f.data));
                            serde_json::json!({"type": "file", "path": f.path, "size": 135, "lfs": {"oid": sha, "size": f.data.len()}})
                        })
                        .collect();
                    ("200 OK", serde_json::to_vec(&list).unwrap())
                } else if let Some((key, _)) = path
                    .trim_start_matches('/')
                    .split_once("/resolve/main/")
                    .map(|(repo, p)| (format!("{repo}/{p}"), ()))
                {
                    match files.get(&key) {
                        Some(f) => {
                            dl.lock().unwrap().push(key.clone());
                            match range {
                                Some(start) if start >= f.data.len() => {
                                    ("416 Range Not Satisfiable", Vec::new())
                                }
                                Some(start) => ("206 Partial Content", f.data[start..].to_vec()),
                                None => ("200 OK", f.data.clone()),
                            }
                        }
                        None => ("404 Not Found", b"no".to_vec()),
                    }
                } else {
                    ("404 Not Found", b"no".to_vec())
                };
                let _ = write!(
                    s,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(&body);
            });
        }
    });
    FakeHf { base, downloads }
}
