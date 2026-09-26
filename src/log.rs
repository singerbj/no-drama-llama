//! Minimal append-only log file with one 5 MB rotation.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static LOG: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(path: &Path) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(path)
        .map(|m| m.len() > 5 * 1024 * 1024)
        .unwrap_or(false)
    {
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
    *LOG.lock().unwrap() = Some(path.to_path_buf());
}

pub fn line(msg: &str) {
    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
    let text = format!("{ts}  {msg}\r\n");
    if let Some(p) = LOG.lock().unwrap().as_ref() {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
        {
            let _ = f.write_all(text.as_bytes());
        }
    }
    #[cfg(debug_assertions)]
    eprint!("{text}");
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { $crate::log::line(&format!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    // One test: the logger is process-global.
    #[test]
    fn writes_lines_and_rotates_big_logs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("tray.log");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, vec![b'x'; 5 * 1024 * 1024 + 1]).unwrap();
        init(&path);
        assert!(path.with_extension("log.1").exists(), "rotated");
        crate::log!("hello {}", 42);
        line("second");
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("  hello 42"));
        assert!(text.contains("\r\n"), "CRLF for Notepad");
        // timestamp shape: 2026-09-26T12:34:56
        assert_eq!(lines[1].as_bytes()[4], b'-');
        assert_eq!(lines[1].as_bytes()[10], b'T');
        // a small log is not rotated again
        std::fs::remove_file(path.with_extension("log.1")).unwrap();
        init(&path);
        assert!(!path.with_extension("log.1").exists());
        // unwritable path doesn't panic
        init(dir.path());
        line("ignored");
    }
}
