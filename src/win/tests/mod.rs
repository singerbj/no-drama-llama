//! Windows-only tests: real Win32 calls, real processes, a local HTTP server, and an
//! end-to-end run of the worker against a fake llama-server. Run on Windows CI.

mod e2e;
mod hf;
mod http;
mod laya;
mod models;
mod platform;

use std::path::{Path, PathBuf};

/// `target/<profile>/examples/<name>.exe`, built by `cargo test` / `cargo build --examples`.
pub fn example_exe(name: &str) -> PathBuf {
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let p = deps
        .parent()
        .unwrap()
        .join("examples")
        .join(format!("{name}.exe"));
    assert!(
        p.exists(),
        "{} missing - run `cargo build --examples` first",
        p.display()
    );
    p
}

/// A temp dir with its long, non-verbatim path (sysinfo reports long paths; GitHub's runner
/// temp dir is an 8.3 short path).
pub fn long_tempdir() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let canon = std::fs::canonicalize(t.path()).unwrap();
    let s = canon.to_string_lossy();
    let p = PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s));
    (t, p)
}

pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

pub fn wait_until(what: &str, secs: u64, mut f: impl FnMut() -> bool) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    while std::time::Instant::now() < end {
        if f() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    panic!("timed out waiting for {what}");
}

pub fn copy_exe(from: &Path, to: &Path) {
    std::fs::create_dir_all(to.parent().unwrap()).unwrap();
    std::fs::copy(from, to).unwrap();
}
