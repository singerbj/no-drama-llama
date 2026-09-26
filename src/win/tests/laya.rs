//! Installing Ollaya from a (fake) GitHub release: checksums, unpacking, swapping the files
//! in, keeping the model store.

use super::super::laya as win_laya;
use super::long_tempdir;
use super::platform::make_zip;
use crate::laya;
use crate::paths::Paths;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// Serves `files` (path -> bytes) over HTTP; returns the base URL.
fn serve(files: Arc<Mutex<HashMap<String, Vec<u8>>>>) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            let files = files.clone();
            std::thread::spawn(move || {
                let mut s = s;
                let mut r = BufReader::new(s.try_clone().unwrap());
                let mut first = String::new();
                let _ = r.read_line(&mut first);
                loop {
                    let mut line = String::new();
                    if r.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                }
                let path = first.split_whitespace().nth(1).unwrap_or("/").to_string();
                let (status, body) = match files.lock().unwrap().get(&path) {
                    Some(b) => ("200 OK", b.clone()),
                    None => ("404 Not Found", b"no".to_vec()),
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
    base
}

fn release_json(base: &str, tag: &str, assets: &[(&str, usize)]) -> Vec<u8> {
    let assets: Vec<serde_json::Value> = assets
        .iter()
        .map(|(n, size)| {
            serde_json::json!({"name": n, "browser_download_url": format!("{base}/dl/{n}"), "size": size})
        })
        .collect();
    serde_json::json!({"tag_name": tag, "assets": assets})
        .to_string()
        .into_bytes()
}

/// A release with the Windows archive (and its checksum) at `tag`.
fn publish(
    files: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
    base: &str,
    dir: &std::path::Path,
    tag: &str,
    exe: &[u8],
    lie_about_checksum: bool,
) {
    let zip = dir.join(format!("{tag}.zip"));
    make_zip(
        &zip,
        &[
            ("bin/ollaya.exe", exe),
            ("bin/DirectML.dll", b"dml"),
            ("lib/ollaya/llama/llama.dll", b"llama"),
            ("share/doc/ollaya/LICENSE", b"Apache-2.0"),
        ],
    );
    let data = std::fs::read(&zip).unwrap();
    let sha = if lie_about_checksum {
        "0".repeat(64)
    } else {
        crate::update::sha256_hex(&data)
    };
    let mut f = files.lock().unwrap();
    f.insert(
        "/release.json".into(),
        release_json(base, tag, &[(laya::ARCHIVE, data.len()), (laya::SUMS, 100)]),
    );
    f.insert(
        format!("/dl/{}", laya::SUMS),
        format!("{sha}  {}\n", laya::ARCHIVE).into_bytes(),
    );
    f.insert(format!("/dl/{}", laya::ARCHIVE), data);
}

#[test]
fn installs_updates_and_keeps_the_models() {
    let (_t, root) = long_tempdir();
    let p = Paths::under(&root);
    let files = Arc::new(Mutex::new(HashMap::new()));
    let base = serve(files.clone());
    let api = format!("{base}/release.json");
    let no = AtomicBool::new(false);

    publish(&files, &base, &root, "v0.5.0", b"exe-0.5.0", false);
    let mut seen = Vec::new();
    let r = win_laya::install(&p, &api, false, |d, t| seen.push((d, t)), &no).unwrap();
    assert_eq!(
        r,
        laya::Installed {
            version: "0.5.0".into(),
            gpu_pack: false,
            gpu_wanted: false
        }
    );
    assert_eq!(std::fs::read(&p.ollaya_exe).unwrap(), b"exe-0.5.0");
    assert!(p.ollaya_dir.join(r"lib\ollaya\llama\llama.dll").exists());
    assert!(p.ollaya_dir.join(r"share\doc\ollaya\LICENSE").exists());
    assert_eq!(p.ollaya_record(), Some(r));
    assert!(!p.ollaya_dir.join(".stage").exists() && !p.ollaya_dir.join(".download").exists());
    let (done, total) = *seen.last().unwrap();
    assert!(total > 0 && done == total, "{seen:?}");

    // Pulled models survive an update; the old build's files don't.
    std::fs::create_dir_all(&p.ollaya_models).unwrap();
    std::fs::write(p.ollaya_models.join("blob"), b"model").unwrap();
    std::fs::write(p.ollaya_dir.join(r"bin\old-only.dll"), b"x").unwrap();
    publish(&files, &base, &root, "v0.6.0", b"exe-0.6.0", false);
    let r = win_laya::install(&p, &api, false, |_, _| {}, &no).unwrap();
    assert_eq!(r.version, "0.6.0");
    assert_eq!(std::fs::read(&p.ollaya_exe).unwrap(), b"exe-0.6.0");
    assert!(!p.ollaya_dir.join(r"bin\old-only.dll").exists());
    assert_eq!(
        std::fs::read(p.ollaya_models.join("blob")).unwrap(),
        b"model"
    );

    // A download that doesn't match sha256sum.txt is thrown away; the installed build stays.
    publish(&files, &base, &root, "v0.7.0", b"exe-0.7.0", true);
    let e = win_laya::install(&p, &api, false, |_, _| {}, &no).unwrap_err();
    assert!(format!("{e:#}").contains("checksum mismatch"), "{e:#}");
    assert_eq!(std::fs::read(&p.ollaya_exe).unwrap(), b"exe-0.6.0");
    assert_eq!(p.ollaya_record().unwrap().version, "0.6.0");

    // A release without a Windows build is an error, not a half install.
    files.lock().unwrap().insert(
        "/release.json".into(),
        release_json(&base, "v0.8.0", &[(laya::SUMS, 1)]),
    );
    assert!(win_laya::install(&p, &api, false, |_, _| {}, &no).is_err());
    assert_eq!(std::fs::read(&p.ollaya_exe).unwrap(), b"exe-0.6.0");
}

#[test]
fn finds_processes_under_a_folder() {
    let (_t, root) = long_tempdir();
    let p = Paths::under(&root);
    let runner = p
        .ollaya_dir
        .join(r"lib\ollaya\cuda_v13\ollaya-runner-0123.exe");
    super::copy_exe(&super::example_exe("fake_ollaya"), &runner);
    let mut child = std::process::Command::new(&runner).spawn().unwrap();
    let mut procs = super::super::procs::Procs::new();
    super::wait_until("the runner shows up", 10, || {
        procs.refresh();
        win_laya::pids(&procs, &p).contains(&child.id())
    });
    assert!(
        procs.pids_under(&root.join("ollay")).is_empty(),
        "a folder whose name is only a prefix"
    );
    assert!(procs.pids_under(&root).contains(&child.id()));
    let _ = child.kill();
    let _ = child.wait();
}
