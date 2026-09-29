//! Win32 helpers, process table, GPU counters, launcher scan, popup, icons, zip handling.

use super::super::{gpu, install, libraries, osd, procs::Procs, sys, tray, updater};
use super::{copy_exe, example_exe, long_tempdir, wait_until};
use crate::state::Tone;
use std::io::Write;

#[test]
fn wide_strings_are_nul_terminated() {
    assert_eq!(sys::wide("ab"), vec![b'a' as u16, b'b' as u16, 0]);
    assert_eq!(sys::wide(""), vec![0]);
}

#[test]
fn user_sid_looks_like_a_sid() {
    let sid = sys::current_user_sid().unwrap();
    assert!(sid.starts_with("S-1-5-"), "{sid}");
}

#[test]
fn signed_in_user_lookups() {
    let me = sys::current_user_sid().unwrap();
    assert!(sys::profile_local_app_data(&me).is_some_and(|p| p.is_dir()));
    assert!(sys::account_sid(r".\no-such-user-xyz").is_err());
    if let Some(other) = sys::other_signed_in_user() {
        assert_eq!(other.sid, me);
    }
    // None in session 0 (a service), `DOMAIN\user` in a desktop session.
    if let Some(name) = sys::session_user_name() {
        let sid = sys::account_sid(&name).unwrap();
        assert!(sid.starts_with("S-1-5-"), "{name}: {sid}");
    }
}

#[test]
fn elevation_check_does_not_fail() {
    let _ = sys::is_elevated();
}

#[test]
fn run_reports_success_output_and_failure() {
    let out = sys::run("cmd.exe", &["/c", "echo", "hello"]).unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("hello"));
    let e = sys::run("cmd.exe", &["/c", "exit", "3"]).unwrap_err();
    assert!(e.to_string().contains("failed"), "{e}");
    assert!(sys::run("definitely-not-a-program-xyz.exe", &[]).is_err());
}

#[test]
fn powershell_snippets() {
    assert_eq!(sys::powershell("'hi'").unwrap().trim(), "hi");
    assert!(sys::powershell("throw 'boom'").is_err());
}

#[test]
fn single_instance_mutex() {
    let name = format!(r"Local\NoDramaLlamaTest{}", std::process::id());
    let (tx, rx) = std::sync::mpsc::channel();
    let (tx_done, rx_done) = std::sync::mpsc::channel::<()>();
    let n = name.clone();
    let holder = std::thread::spawn(move || {
        let g = sys::SingleInstance::acquire_named(&n, 0).unwrap();
        tx.send(g.is_some()).unwrap();
        rx_done.recv().unwrap();
        drop(g);
    });
    assert!(rx.recv().unwrap(), "first instance gets it");
    assert!(
        sys::SingleInstance::acquire_named(&name, 0)
            .unwrap()
            .is_none(),
        "second instance is refused"
    );
    tx_done.send(()).unwrap();
    holder.join().unwrap();
    assert!(
        sys::SingleInstance::acquire_named(&name, 2000)
            .unwrap()
            .is_some(),
        "free again once released"
    );
}

#[test]
fn single_instance_waits_for_the_previous_one() {
    let name = format!(r"Local\NoDramaLlamaWait{}", std::process::id());
    let n = name.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let t = std::thread::spawn(move || {
        let g = sys::SingleInstance::acquire_named(&n, 0).unwrap();
        tx.send(()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        drop(g); // like the old version exiting after an update
    });
    rx.recv().unwrap();
    let start = std::time::Instant::now();
    assert!(sys::SingleInstance::acquire_named(&name, 10_000)
        .unwrap()
        .is_some());
    assert!(start.elapsed() >= std::time::Duration::from_millis(300));
    t.join().unwrap();
}

#[test]
fn process_table_sees_itself() {
    let p = Procs::new();
    let me = std::process::id();
    let exe = std::env::current_exe().unwrap();
    let list = p.list();
    let mine = list
        .iter()
        .find(|x| x.pid == me)
        .expect("own process listed");
    assert!(
        !mine.name.to_lowercase().ends_with(".exe"),
        "name has no .exe: {}",
        mine.name
    );
    assert!(mine
        .path
        .as_deref()
        .is_some_and(|x| x.eq_ignore_ascii_case(&exe.to_string_lossy())));
    assert!(p.pids_of(&exe).contains(&me));
    assert!(p.by_pid().contains_key(&me));
}

#[test]
fn process_table_finds_and_kills_by_path() {
    let (_t, dir) = long_tempdir();
    let exe = dir.join("ndl-test-sleeper.exe");
    copy_exe(&example_exe("fake_llama_server"), &exe);
    let mut child = std::process::Command::new(&exe).spawn().unwrap();
    let mut p = Procs::new();
    wait_until("sleeper in process list", 10, || {
        p.refresh();
        p.pids_of(&exe).contains(&child.id())
    });
    assert_eq!(
        p.parent_of(child.id()),
        Some(std::process::id()),
        "installer can spare the process that launched it"
    );
    let upper = std::path::PathBuf::from(exe.to_string_lossy().to_uppercase());
    assert!(
        p.pids_of(&upper).contains(&child.id()),
        "path match is case-insensitive"
    );
    p.kill(&[child.id()]);
    p.refresh();
    assert!(p.pids_of(&exe).is_empty());
    let _ = child.wait();
}

#[test]
fn process_table_matches_command_lines() {
    let marker = format!("ndl-marker-{}", std::process::id());
    // A single process (no children left behind): PowerShell sleeping, marker in a comment.
    let mut child = sys::hidden("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Start-Sleep 30 # {marker}"),
        ])
        .spawn()
        .unwrap();
    let mut p = Procs::new();
    let mut found = Vec::new();
    wait_until("marked cmd.exe", 10, || {
        found = p.pids_with_cmdline("powershell.exe", &marker.to_uppercase());
        !found.is_empty()
    });
    assert!(found.contains(&child.id()));
    p.kill(&found);
    let _ = child.wait();
}

#[test]
fn gpu_counters_do_not_crash() {
    // CI runners usually have no GPU: the sampler may be unavailable, but must not panic.
    if let Some(mut g) = gpu::GpuSampler::new() {
        for _ in 0..3 {
            for s in g.sample() {
                assert!(s.engine_3d >= 0.0 && s.engine_3d <= 100.0);
            }
        }
    }
    let _ = gpu::fullscreen_foreground_pid();
}

#[test]
fn launcher_scan_runs_and_reports() {
    let mut scanner = libraries::Scanner::new();
    let scan = scanner.scan();
    for l in &scan.libs {
        assert!(l.dir.ends_with('\\'), "{}", l.dir);
        assert!(crate::detect::normalize_lib_dir(&l.dir).is_some());
    }
    let _ = scanner.steam_running();
    let r = libraries::report(&scan, &["mygame".into()]);
    for section in [
        "Game library folders",
        "Fallback folder patterns",
        "Windows Game Bar list",
        "Extra games from settings: mygame",
    ] {
        assert!(r.contains(section), "{section}");
    }
}

#[test]
fn colors_and_icons() {
    assert_eq!(osd::rgb(0x12, 0x34, 0x56).0, 0x0056_3412);
    let tones = [
        Tone::Running,
        Tone::Loading,
        Tone::Paused,
        Tone::Off,
        Tone::Error,
    ];
    let colors: std::collections::HashSet<_> = tones.iter().map(|t| osd::tone_rgb(*t)).collect();
    assert_eq!(colors.len(), tones.len(), "every status has its own color");
    for t in tones {
        let px = tray::icon_rgba(t);
        let n = tray::ICON_SIZE;
        assert_eq!(px.len(), n * n * 4);
        assert_eq!(px[3], 0, "corner is transparent");
        let at = |x: usize, y: usize| &px[(y * n + x) * 4..][..4];
        assert_eq!(at(16, 8), [255, 255, 255, 255], "the llama's head");
        let (r, g, b) = osd::tone_rgb(t);
        assert_eq!(at(17, 3), [r, g, b, 255], "the circle is the status color");
    }
}

#[test]
fn popup_uses_the_design_systems_font() {
    use windows::Win32::Graphics::Gdi::{DeleteObject, HGDIOBJ};
    let face = |weight: i32, text: &str| {
        let f = osd::font(18, weight, &text.encode_utf16().collect::<Vec<_>>());
        let name = osd::face_name(f);
        let _ = unsafe { DeleteObject(HGDIOBJ(f.0)) };
        name
    };
    assert_eq!(face(600, "LLM paused"), "Google Sans Code SemiBold");
    assert_eq!(
        face(400, "Example Game (Steam) detected · GPU freed"),
        "Google Sans Code"
    );
    assert_eq!(face(400, ""), "Google Sans Code");
    // Characters it has no glyph for (here Japanese) fall back to Segoe UI and its font links.
    assert_eq!(face(600, "原神 detected"), "Segoe UI");
}

#[test]
fn popup_window_can_be_shown_and_replaced() {
    for pos in crate::settings::PopupPosition::ALL.map(|(p, _)| p) {
        osd::show(
            "LLM paused",
            "Example Game (Steam) detected",
            Tone::Paused,
            pos,
        );
    }
    osd::show(
        "",
        "",
        Tone::Error,
        crate::settings::PopupPosition::TopCenter,
    );
    osd::close_current();
    osd::close_current(); // twice is fine
}

pub(super) fn make_zip(path: &std::path::Path, files: &[(&str, &[u8])]) {
    let mut w = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in files {
        w.start_file(*name, opts).unwrap();
        w.write_all(data).unwrap();
    }
    w.finish().unwrap();
}

#[test]
fn llama_zip_is_extracted_flat() {
    let (_t, dir) = long_tempdir();
    let target = dir.join("llama");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("stale.dll"), b"old").unwrap();

    let flat = dir.join("flat.zip");
    make_zip(
        &flat,
        &[("llama-server.exe", b"exe"), ("ggml-vulkan.dll", b"dll")],
    );
    install::extract_llama_zip(&flat, &target).unwrap();
    assert_eq!(
        std::fs::read(target.join("llama-server.exe")).unwrap(),
        b"exe"
    );
    assert!(!target.join("stale.dll").exists(), "old build removed");

    let nested = dir.join("nested.zip");
    make_zip(
        &nested,
        &[
            ("llama-b6500-bin-win-vulkan-x64/llama-server.exe", b"exe2"),
            ("llama-b6500-bin-win-vulkan-x64/ggml.dll", b"d"),
        ],
    );
    install::extract_llama_zip(&nested, &target).unwrap();
    assert_eq!(
        std::fs::read(target.join("llama-server.exe")).unwrap(),
        b"exe2"
    );
    assert!(target.join("ggml.dll").exists());

    let bad = dir.join("bad.zip");
    make_zip(&bad, &[("readme.txt", b"no server here")]);
    assert!(install::extract_llama_zip(&bad, &target).is_err());

    let evil = dir.join("evil.zip");
    make_zip(
        &evil,
        &[("../escaped.exe", b"x"), ("llama-server.exe", b"ok")],
    );
    let _ = install::extract_llama_zip(&evil, &target);
    assert!(!dir.join("escaped.exe").exists(), "zip-slip blocked");
}

#[test]
fn find_file_is_recursive_and_case_insensitive() {
    let (_t, dir) = long_tempdir();
    std::fs::create_dir_all(dir.join("a").join("b")).unwrap();
    std::fs::write(dir.join("a").join("b").join("LLAMA-SERVER.EXE"), b"x").unwrap();
    assert!(install::find_file(&dir, "llama-server.exe").is_some());
    assert!(install::find_file(&dir, "nope.exe").is_none());
}

#[test]
fn updater_never_touches_a_dev_build() {
    let r = crate::update::Release::parse(
        r#"{"tag_name":"v99.0.0","assets":[{"name":"no-drama-llama.exe","browser_download_url":"http://127.0.0.1:1/x"},
                                            {"name":"no-drama-llama.exe.minisig","browser_download_url":"http://127.0.0.1:1/y"}]}"#,
    )
    .unwrap();
    let before = std::fs::read(std::env::current_exe().unwrap())
        .unwrap()
        .len();
    let e = updater::apply(&r, &semver::Version::new(99, 0, 0))
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("signing key") || e.contains("installed copy"),
        "{e}"
    );
    assert_eq!(
        std::fs::read(std::env::current_exe().unwrap())
            .unwrap()
            .len(),
        before
    );
    assert_eq!(
        updater::can_self_update(),
        crate::update::update_pubkey().is_some()
    );
}

#[test]
fn task_queries_do_not_fail_when_not_installed() {
    // On CI the app isn't installed; these must just say "no".
    if !install::task_exists() {
        assert_eq!(install::logon_start(), None);
    }
}
