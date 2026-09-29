//! The worker thread end to end: a real (fake) llama-server process is started, health-checked,
//! paused for a "game", resumed, turned off/on, restarted on a settings edit, and reported as an
//! error after repeated crashes.

use super::super::worker::{Cmd, Snapshot, UiMsg, Worker};
use super::{copy_exe, example_exe, free_port, long_tempdir};
use crate::paths::Paths;
use crate::settings::{DetectionMode, Settings, DEFAULT_MODEL};
use crate::state::Status;
use std::sync::mpsc::{self, Receiver};

struct Harness {
    _tmp: tempfile::TempDir,
    p: Paths,
    w: Worker,
    rx: Receiver<UiMsg>,
    last: Option<Snapshot>,
    popups: Vec<String>,
    self_rx: Receiver<Cmd>,
}

impl Harness {
    fn new(port: u16) -> Harness {
        let (tmp, root) = long_tempdir();
        let p = Paths::under(&root);
        copy_exe(&example_exe("fake_llama_server"), &p.server_exe);
        std::fs::create_dir_all(&p.models_dir).unwrap();
        std::fs::write(p.model(DEFAULT_MODEL), b"not really a model").unwrap();
        let s = Settings {
            port,
            detection_mode: DetectionMode::Launchers,
            extra_games: vec!["ndl-fake-game".into()],
            resume_after_sec: 0,
            auto_update: false,
            ..Default::default()
        };
        s.save(&p.settings).unwrap();
        let (tx_ui, rx) = mpsc::channel();
        let (tx_cmd, self_rx) = mpsc::channel();
        let w = Worker::new(p.clone(), tx_ui, tx_cmd);
        Harness {
            _tmp: tmp,
            p,
            w,
            rx,
            last: None,
            popups: Vec::new(),
            self_rx,
        }
    }

    fn tick(&mut self) -> Status {
        self.w.tick();
        while let Ok(m) = self.rx.try_recv() {
            match m {
                UiMsg::State(s) => self.last = Some(*s),
                UiMsg::Popup(p) => self.popups.push(p.title),
                UiMsg::Quit(_) => panic!("unexpected quit"),
            }
        }
        self.last.as_ref().unwrap().status.clone()
    }

    fn cmd(&mut self, c: Cmd) {
        assert!(!self.w.handle(c));
    }

    fn until(&mut self, what: &str, f: impl Fn(&Status) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let s = self.tick();
            if f(&s) {
                return;
            }
            assert!(
                std::time::Instant::now() < end,
                "timed out waiting for {what}; last status {s:?}; popups {:?}",
                self.popups
            );
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    fn server_pids(&self) -> Vec<u32> {
        let mut p = super::super::procs::Procs::new();
        p.refresh();
        p.pids_of(&self.p.server_exe)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.w.stop_server("test done");
        self.w.stop_laya("test done");
    }
}

#[test]
fn worker_end_to_end() {
    let port = free_port();
    let mut h = Harness::new(port);

    // 1. Starts llama-server with our arguments and reports Running once /health says so
    assert_eq!(h.tick(), Status::Loading);
    h.until("running", |s| *s == Status::Running);
    assert_eq!(h.server_pids().len(), 1);
    let argv = std::fs::read_to_string(h.p.llama_dir.join("argv.txt")).unwrap();
    let args: Vec<&str> = argv.lines().collect();
    assert_eq!(
        args[args.iter().position(|a| *a == "--port").unwrap() + 1],
        port.to_string()
    );
    assert!(args.contains(&"--jinja"));
    assert!(
        argv.contains(r#"{"reasoning_effort":"low"}"#),
        "kwargs arrive intact: {argv}"
    );
    assert!(args[args.iter().position(|a| *a == "-m").unwrap() + 1].ends_with(DEFAULT_MODEL));
    // Auto sizing: llama.cpp fits GPU layers and context itself; we report what it chose
    assert!(!args.contains(&"-ngl"), "GPU layers left to --fit: {argv}");
    assert!(!args.contains(&"-c"), "context left to --fit: {argv}");
    let snap = h.last.clone().unwrap();
    assert!(snap.status_text.starts_with("Running - "));
    assert!(
        snap.status_text.ends_with(" · 64K context"),
        "{}",
        snap.status_text
    );
    assert_eq!(snap.n_ctx, Some(65536));
    assert_eq!(snap.models.len(), 1);
    assert_eq!(
        snap.pc.vram,
        16384 * 1024 * 1024,
        "GPU memory from llama-server --list-devices"
    );
    assert_eq!(snap.gpu_name.as_deref(), Some("Fake GPU 9000 (16 GB)"));

    // 2. A game starts: server stopped, status Paused naming the game
    let game_exe = h.p.root.join("games").join("ndl-fake-game.exe");
    copy_exe(&example_exe("fake_llama_server"), &game_exe);
    let mut game = std::process::Command::new(&game_exe).spawn().unwrap();
    h.until(
        "paused",
        |s| matches!(s, Status::Paused(g) if g.launcher == "your list"),
    );
    assert!(h.server_pids().is_empty(), "server freed for the game");
    assert_eq!(
        h.last.as_ref().unwrap().game_process.as_deref(),
        Some("ndl-fake-game")
    );
    assert!(h.popups.contains(&"LLM paused".to_string()));

    // 3. Game quits: resumes (ResumeAfterSec = 0)
    game.kill().unwrap();
    let _ = game.wait();
    h.until("running again", |s| *s == Status::Running);

    // 4. Turn off / on (hotkey or menu)
    h.cmd(Cmd::Toggle);
    assert_eq!(h.tick(), Status::Off);
    assert!(h.p.off_flag.exists(), "off survives a restart of the app");
    assert!(h.server_pids().is_empty());
    h.cmd(Cmd::Toggle);
    assert!(!h.p.off_flag.exists());
    h.until("running after turning on", |s| *s == Status::Running);

    // 5. Menu settings change that affects the server: restarted on the new port
    let new_port = free_port();
    h.cmd(Cmd::Edit(Box::new(move |s| s.port = new_port)));
    h.until("running on new port", |s| *s == Status::Running);
    let argv = std::fs::read_to_string(h.p.llama_dir.join("argv.txt")).unwrap();
    assert!(argv.contains(&new_port.to_string()));

    // 5b. A fixed context size is passed through, and auto goes back to fitting
    let argv_file = h.p.llama_dir.join("argv.txt");
    h.cmd(Cmd::Edit(Box::new(|s| s.context = 16384)));
    h.until("running with fixed context", |s| {
        *s == Status::Running
            && std::fs::read_to_string(&argv_file).is_ok_and(|a| a.contains("-c\n16384"))
    });
    h.cmd(Cmd::Edit(Box::new(|s| {
        s.context = crate::settings::CONTEXT_AUTO
    })));
    h.until("back to auto", |s| {
        *s == Status::Running
            && std::fs::read_to_string(&argv_file).is_ok_and(|a| !a.lines().any(|l| l == "-c"))
    });
    assert_eq!(Settings::load(&h.p.settings).0.port, new_port, "saved");

    // 6. Hand edit of settings.json is picked up
    let mut s = Settings::load(&h.p.settings).0;
    s.reasoning = crate::settings::Reasoning::None;
    std::thread::sleep(std::time::Duration::from_millis(1100)); // distinct mtime
    s.save(&h.p.settings).unwrap();
    h.until("restarted with reasoning none", |st| {
        *st == Status::Running
            && std::fs::read_to_string(&argv_file)
                .is_ok_and(|a| a.contains(r#"{"enable_thinking":false}"#))
    });

    // 7. "Not a game - ignore this app" adds the process to GpuIgnore
    let mut game = std::process::Command::new(&game_exe).spawn().unwrap();
    h.until("paused again", |s| matches!(s, Status::Paused(_)));
    h.cmd(Cmd::IgnoreCurrentGame);
    assert!(Settings::load(&h.p.settings)
        .0
        .gpu_ignore
        .contains(&"ndl-fake-game".to_string()));
    game.kill().unwrap();
    let _ = game.wait();
    h.until("running after ignore", |s| *s == Status::Running);

    // 8. Model deleted: error without starting; restored: recovers by itself
    h.w.stop_server("test");
    std::fs::rename(h.p.model(DEFAULT_MODEL), h.p.root.join("moved.gguf")).unwrap();
    h.until(
        "model missing error",
        |s| matches!(s, Status::Error(e) if e.contains("Model missing")),
    );
    assert!(h.server_pids().is_empty());
    std::fs::rename(h.p.root.join("moved.gguf"), h.p.model(DEFAULT_MODEL)).unwrap();
    h.until("recovered", |s| *s == Status::Running);

    // 9. A server that keeps crashing ends in Error after 3 tries, Restart clears it
    std::fs::write(h.p.llama_dir.join("crash"), b"").unwrap();
    h.w.stop_server("test");
    h.until(
        "crash error",
        |s| matches!(s, Status::Error(e) if e.contains("keeps crashing")),
    );
    std::fs::remove_file(h.p.llama_dir.join("crash")).unwrap();
    h.cmd(Cmd::Restart);
    h.until("running after restart", |s| *s == Status::Running);
}

#[test]
fn model_download_from_the_tray_switches_to_it() {
    let m = crate::catalog::find("qwen3.8-27b:UD-IQ2_XXS").unwrap();
    let data: Vec<u8> = (0..2_000_000u32).map(|i| (i % 251) as u8).collect();
    let fake = super::hf::start(vec![super::hf::file(m.repo, &m.files[0], data.clone())]);
    let mut h = Harness::new(free_port());
    h.w.hf_base = fake.base.clone();
    h.until("running", |s| *s == Status::Running);

    h.cmd(Cmd::DownloadModel {
        id: m.id.clone(),
        switch: true,
    });
    let file = m.primary_file();
    let argv_file = h.p.llama_dir.join("argv.txt");
    let end = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        // the download thread reports back through the worker's own channel; deliver it
        while let Ok(c) = h.self_rx.try_recv() {
            h.cmd(c);
        }
        let st = h.tick();
        let snap = h.last.clone().unwrap();
        if snap.download.is_none()
            && snap.settings.model == file
            && st == Status::Running
            && std::fs::read_to_string(&argv_file).is_ok_and(|a| a.contains(&file))
        {
            break;
        }
        assert!(
            std::time::Instant::now() < end,
            "download never finished; popups {:?}",
            h.popups
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    assert_eq!(std::fs::read(h.p.model(&file)).unwrap(), data);
    assert!(h.popups.contains(&"Model downloaded".to_string()));
    assert_eq!(Settings::load(&h.p.settings).0.model, file, "saved");
    assert_eq!(h.last.as_ref().unwrap().models.len(), 2);
}

#[test]
fn cancelled_download_reports_and_keeps_the_old_model() {
    let m = crate::catalog::find("qwen3.8-27b:UD-Q2_K_XL").unwrap();
    let data: Vec<u8> = vec![7u8; 64 * 1024 * 1024];
    let fake = super::hf::start(vec![super::hf::file(m.repo, &m.files[0], data)]);
    let mut h = Harness::new(free_port());
    h.w.hf_base = fake.base.clone();
    let before = Settings::load(&h.p.settings).0.model;
    h.cmd(Cmd::DownloadModel {
        id: m.id.clone(),
        switch: true,
    });
    h.cmd(Cmd::CancelDownload);
    let end = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !h
        .popups
        .iter()
        .any(|p| p == "Download cancelled" || p == "Model downloaded")
    {
        while let Ok(c) = h.self_rx.try_recv() {
            h.cmd(c);
        }
        h.tick();
        assert!(
            std::time::Instant::now() < end,
            "no result; popups {:?}",
            h.popups
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(
        h.popups.contains(&"Download cancelled".to_string()),
        "{:?}",
        h.popups
    );
    assert_eq!(Settings::load(&h.p.settings).0.model, before);
    assert!(h.last.as_ref().unwrap().download.is_none());
    assert!(!h.p.model(&m.primary_file()).exists());
}

impl Harness {
    /// Ticks (delivering the worker's own messages: finished jobs) until Laya's status matches.
    fn until_laya(&mut self, what: &str, f: impl Fn(&crate::laya::Status) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(60);
        loop {
            while let Ok(c) = self.self_rx.try_recv() {
                self.cmd(c);
            }
            self.tick();
            let l = &self.last.as_ref().unwrap().laya;
            if f(&l.status) {
                return;
            }
            assert!(
                std::time::Instant::now() < end,
                "timed out waiting for Laya {what}; last {:?}; popups {:?}",
                l.status,
                self.popups
            );
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    fn laya_pids(&self) -> Vec<u32> {
        let mut p = super::super::procs::Procs::new();
        p.refresh();
        super::super::laya::pids(&p, &self.p)
    }
}

#[test]
fn laya_runs_alongside_the_llm() {
    use crate::laya::{Device, Installed, Status as L};
    let mut h = Harness::new(free_port());
    // Ollaya already installed (the install itself is tested in tests/laya.rs)
    copy_exe(&example_exe("fake_ollaya"), &h.p.ollaya_exe);
    let record = Installed {
        version: "0.5.0".into(),
        gpu_pack: false,
        gpu_wanted: false,
    };
    std::fs::write(h.p.ollaya_dir.join("install.json"), record.to_json()).unwrap();
    h.w.laya_record = Some(record);
    h.w.nvidia = Some(Vec::new()); // no GPU pack wanted, whatever this machine has
    h.w.laya_release_api = "http://127.0.0.1:9/nothing".into();
    h.until("running", |s| *s == Status::Running);
    assert_eq!(h.last.as_ref().unwrap().laya.status, L::Disabled);
    assert!(h.laya_pids().is_empty(), "off by default");

    // 1. Turned on: started with our settings, the model pulled and loaded
    let laya_port = free_port();
    h.cmd(Cmd::Edit(Box::new(move |s| {
        s.run_laya = true;
        s.laya_port = laya_port;
        s.laya_device = Device::Cpu;
        s.api_key = "sekrit".into();
    })));
    h.until_laya("ready", |l| *l == L::Ready);
    let env = std::fs::read_to_string(h.p.ollaya_exe.with_file_name("env.txt")).unwrap();
    for want in [
        format!("OLLAYA_HOST=127.0.0.1:{laya_port}"),
        "OLLAYA_DEVICE=cpu".into(),
        "OLLAYA_KEEP_ALIVE=-1".into(),
        "OLLAYA_API_KEY=sekrit".into(),
        format!("OLLAYA_MODELS={}", h.p.ollaya_models.display()),
    ] {
        assert!(env.lines().any(|l| l == want), "{want} in {env}");
    }
    let store = h.p.ollaya_models.clone();
    assert_eq!(
        std::fs::read_to_string(store.join("pulled.txt")).unwrap(),
        "laya:latest\n"
    );
    let loaded = std::fs::read_to_string(store.join("loaded.txt")).unwrap();
    assert!(loaded.contains(r#""keep_alive":"-1""#), "{loaded}");
    assert!(
        h.popups.contains(&"Laya ready".to_string()),
        "{:?}",
        h.popups
    );
    let snap = h.last.clone().unwrap();
    assert_eq!(snap.status, Status::Running, "the LLM is unaffected");
    assert_eq!(snap.laya.url, format!("http://127.0.0.1:{laya_port}"));
    assert_eq!(snap.laya.version.as_deref(), Some("0.5.0"));
    assert_eq!(h.laya_pids().len(), 1);

    // 2. A game: Laya pauses with the LLM, even on the CPU, and comes back when the game closes
    let game_exe = h.p.root.join("games").join("ndl-fake-game.exe");
    copy_exe(&example_exe("fake_llama_server"), &game_exe);
    let mut game = std::process::Command::new(&game_exe).spawn().unwrap();
    h.until("paused", |s| matches!(s, Status::Paused(_)));
    h.until_laya("paused", |l| *l == L::Paused);
    assert!(h.laya_pids().is_empty(), "the game gets the machine");
    game.kill().unwrap();
    let _ = game.wait();
    h.until_laya("ready after the game", |l| *l == L::Ready);
    assert_eq!(
        std::fs::read_to_string(store.join("pulled.txt")).unwrap(),
        "laya:latest\n",
        "not downloaded again"
    );

    // 3. A new model is pulled; the old one stays
    h.cmd(Cmd::Edit(Box::new(|s| s.laya_model = "laya:en".into())));
    h.until_laya("ready with laya:en", |l| *l == L::Ready);
    assert!(std::fs::read_to_string(store.join("pulled.txt"))
        .unwrap()
        .contains("laya:en\n"));

    // 4. The off switch covers Laya too
    h.cmd(Cmd::Toggle);
    h.until_laya("off", |l| *l == L::Off);
    assert!(h.laya_pids().is_empty());
    h.cmd(Cmd::Toggle);
    h.until_laya("on again", |l| *l == L::Ready);

    // 5. A model the registry doesn't have: an error that says why, retried later
    h.cmd(Cmd::Edit(Box::new(|s| s.laya_model = "missing".into())));
    h.until_laya(
        "pull error",
        |l| matches!(l, L::Error(e) if e.contains("not found in registry")),
    );
    assert!(h.popups.contains(&"Laya error".to_string()));

    // 6. Keeps crashing: an error; Restart Laya tries again
    h.cmd(Cmd::Edit(Box::new(|s| s.laya_model = "laya".into())));
    std::fs::write(h.p.ollaya_exe.with_file_name("crash"), b"").unwrap();
    h.w.stop_laya("test");
    h.until_laya(
        "crash error",
        |l| matches!(l, L::Error(e) if e.contains("keeps crashing")),
    );
    std::fs::remove_file(h.p.ollaya_exe.with_file_name("crash")).unwrap();
    h.cmd(Cmd::RestartLaya);
    h.until_laya("ready after restart", |l| *l == L::Ready);

    // 7. The LLM's port: a settings problem, nothing started
    let llm_port = h.last.as_ref().unwrap().settings.port;
    h.cmd(Cmd::Edit(Box::new(move |s| s.laya_port = llm_port)));
    h.until_laya(
        "port clash",
        |l| matches!(l, L::Error(e) if e.contains("LayaPort")),
    );
    assert!(h.laya_pids().is_empty());

    // 8. Turned off in settings: stopped
    h.cmd(Cmd::Edit(Box::new(move |s| {
        s.laya_port = laya_port;
        s.run_laya = false;
    })));
    h.until_laya("disabled", |l| *l == L::Disabled);
    assert!(h.laya_pids().is_empty());
    h.tick();
    assert_eq!(h.last.as_ref().unwrap().status, Status::Running);
}
