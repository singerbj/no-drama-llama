//! End-to-end simulation of the app's decision loop (detection -> state machine -> actions)
//! with scripted processes and GPU samples, exactly as the worker wires them together.

use no_drama_llama::detect::{find_running_game, GpuDetector, GpuProc, ProcInfo, Scan};
use no_drama_llama::settings::{DetectionMode, Settings};
use no_drama_llama::state::{Action, Inputs, Machine, Status};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const GB: u64 = 1 << 30;
const LLM_PID: u32 = 900;

struct Sim {
    s: Settings,
    scan: Scan,
    m: Machine,
    gpu: GpuDetector,
    now: Instant,
    procs: Vec<ProcInfo>,
    samples: Vec<GpuProc>,
    fullscreen: Option<u32>,
    steam: Option<(u32, String)>,
    off: bool,
    server: bool,
    ready_in: Option<u32>,
    log: Vec<String>,
}

impl Sim {
    fn new(s: Settings) -> Sim {
        let mut scan = Scan::default();
        scan.add_lib(r"E:\Epic Games\Fortnite", "Epic", Some("Fortnite"));
        Sim {
            s,
            scan,
            m: Machine::default(),
            gpu: GpuDetector::default(),
            now: Instant::now(),
            procs: vec![
                p(1, "explorer", r"C:\Windows\explorer.exe"),
                p(2, "chrome", r"C:\Program Files\Google\Chrome\chrome.exe"),
            ],
            samples: Vec::new(),
            fullscreen: None,
            steam: None,
            off: false,
            server: false,
            ready_in: None,
            log: Vec::new(),
        }
    }

    /// One worker tick, `secs` after the previous one.
    fn tick(&mut self, secs: u64) -> &Status {
        self.now += Duration::from_secs(secs);
        let mut game = None;
        if self.s.pause_while_gaming {
            if self.s.detection_mode.uses_launchers() {
                game = find_running_game(&self.procs, &self.scan, &self.s, self.steam.clone());
            }
            if self.s.detection_mode.uses_gpu() {
                let by_pid: HashMap<u32, ProcInfo> =
                    self.procs.iter().map(|p| (p.pid, p.clone())).collect();
                let exclude = if self.server { vec![LLM_PID] } else { vec![] };
                let g = self.gpu.check(
                    &self.samples,
                    &by_pid,
                    &self.s,
                    &exclude,
                    self.fullscreen,
                    &self.scan,
                );
                game = game.or(g);
            }
        }
        let ready = match self.ready_in.as_mut() {
            Some(0) => true,
            Some(n) => {
                *n -= 1;
                false
            }
            None => false,
        };
        let out = self.m.step(&Inputs {
            now: self.now,
            off: self.off,
            game,
            resume_after: Duration::from_secs(self.s.resume_after_sec.into()),
            server_running: self.server,
            server_ready: self.server && ready,
            model_exists: true,
            server_exe_exists: true,
            model_name: self.s.model.clone(),
        });
        for a in out.actions {
            match a {
                Action::StartServer => {
                    self.server = true;
                    self.ready_in = Some(2);
                    self.samples.retain(|g| g.pid != LLM_PID);
                    self.samples.push(GpuProc {
                        pid: LLM_PID,
                        engine_3d: 5.0,
                        dedicated_bytes: 18 * GB,
                    });
                    self.log.push("start".into());
                }
                Action::StopServer(why) => {
                    self.server = false;
                    self.ready_in = None;
                    self.samples.retain(|g| g.pid != LLM_PID);
                    self.log.push(format!("stop: {why}"));
                }
            }
        }
        if let Some(p) = out.popup {
            self.log.push(format!("popup: {}", p.title));
        }
        self.m.status()
    }

    fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.tick(5);
        }
    }

    fn launch(&mut self, proc: ProcInfo) {
        self.procs.push(proc);
    }

    fn quit(&mut self, pid: u32) {
        self.procs.retain(|p| p.pid != pid);
        self.samples.retain(|g| g.pid != pid);
        if self.fullscreen == Some(pid) {
            self.fullscreen = None;
        }
    }

    fn running(&self) -> bool {
        *self.m.status() == Status::Running
    }
}

fn p(pid: u32, name: &str, path: &str) -> ProcInfo {
    ProcInfo {
        pid,
        name: name.into(),
        path: Some(path.into()),
        title: None,
    }
}

fn fast() -> Settings {
    Settings {
        resume_after_sec: 10,
        ..Default::default()
    }
}

#[test]
fn boots_and_serves() {
    let mut s = Sim::new(Settings::default());
    s.ticks(4);
    assert!(s.running());
    assert_eq!(s.log, vec!["start"]);
}

#[test]
fn steam_game_by_library_folder() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.launch(p(
        50,
        "eldenring",
        r"D:\SteamLibrary\steamapps\common\ELDEN RING\Game\eldenring.exe",
    ));
    assert!(matches!(s.tick(5), Status::Paused(g) if g.launcher == "Steam"));
    assert!(!s.server);
    s.quit(50);
    s.ticks(1);
    assert!(matches!(s.m.status(), Status::Paused(_)), "resume delay");
    s.ticks(6);
    assert!(s.running());
}

#[test]
fn steam_running_app_id_without_a_known_folder() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.steam = Some((1245620, "ELDEN RING".into()));
    assert!(matches!(s.tick(5), Status::Paused(g) if g.name == "ELDEN RING"));
    s.steam = None;
    s.ticks(8);
    assert!(s.running());
}

#[test]
fn epic_game_via_discovered_library() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.launch(p(
        60,
        "FortniteClient-Win64-Shipping",
        r"E:\Epic Games\Fortnite\FortniteGame\Binaries\Win64\FortniteClient-Win64-Shipping.exe",
    ));
    assert!(matches!(s.tick(5), Status::Paused(g) if g.name == "Fortnite" && g.launcher == "Epic"));
}

#[test]
fn unknown_game_caught_by_gpu_on_second_check() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.launch(p(70, "IndieGame", r"C:\Stuff\IndieGame\IndieGame.exe"));
    s.samples.push(GpuProc {
        pid: 70,
        engine_3d: 85.0,
        dedicated_bytes: 3 * GB,
    });
    assert!(s.tick(5) == &Status::Running, "one sample is not enough");
    assert!(matches!(s.tick(5), Status::Paused(g) if g.launcher.starts_with("GPU")));
    s.quit(70);
    s.ticks(8);
    assert!(s.running());
}

#[test]
fn the_llm_itself_never_triggers_a_pause() {
    let mut s = Sim::new(Settings {
        detection_mode: DetectionMode::Gpu,
        ..fast()
    });
    s.procs
        .push(p(LLM_PID, "llama-server", r"C:\LLM\llama\llama-server.exe"));
    s.ticks(20);
    assert!(s.running());
    assert_eq!(s.log, vec!["start"]);
}

#[test]
fn browser_video_does_not_pause() {
    let mut s = Sim::new(fast());
    s.samples.push(GpuProc {
        pid: 2,
        engine_3d: 60.0,
        dedicated_bytes: 2 * GB,
    }); // chrome
    s.ticks(20);
    assert!(s.running());
}

#[test]
fn fullscreen_app_pauses_immediately_and_ignore_list_fixes_false_positive() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.launch(p(80, "Blender", r"C:\Program Files\Blender\blender.exe"));
    s.fullscreen = Some(80);
    assert!(matches!(s.tick(5), Status::Paused(g) if g.launcher.ends_with("full-screen")));
    // user: "Not a game - ignore blender"
    s.s.gpu_ignore.push("Blender".into());
    s.m.forget_game();
    s.ticks(4);
    assert!(s.running());
}

#[test]
fn launcher_only_mode_ignores_gpu_and_gpu_only_mode_ignores_folders() {
    let mut s = Sim::new(Settings {
        detection_mode: DetectionMode::Launchers,
        ..fast()
    });
    s.ticks(4);
    s.launch(p(70, "IndieGame", r"C:\Stuff\IndieGame.exe"));
    s.samples.push(GpuProc {
        pid: 70,
        engine_3d: 90.0,
        dedicated_bytes: 6 * GB,
    });
    s.ticks(5);
    assert!(s.running());

    let mut s = Sim::new(Settings {
        detection_mode: DetectionMode::Gpu,
        ..fast()
    });
    s.ticks(4);
    s.launch(p(
        50,
        "eldenring",
        r"D:\SteamLibrary\steamapps\common\ELDEN RING\eldenring.exe",
    ));
    s.ticks(5);
    assert!(
        s.running(),
        "an idle game process with no GPU use is ignored in GPU-only mode"
    );
}

#[test]
fn pausing_disabled_keeps_running_through_games() {
    let mut s = Sim::new(Settings {
        pause_while_gaming: false,
        ..fast()
    });
    s.ticks(4);
    s.launch(p(50, "GTA5", r"C:\x\GTA5.exe"));
    s.samples.push(GpuProc {
        pid: 50,
        engine_3d: 99.0,
        dedicated_bytes: 8 * GB,
    });
    s.fullscreen = Some(50);
    s.ticks(10);
    assert!(s.running());
}

#[test]
fn emulator_toggle_and_extra_games() {
    let mut s = Sim::new(Settings {
        detect_emulators: false,
        detection_mode: DetectionMode::Launchers,
        ..fast()
    });
    s.ticks(4);
    s.launch(p(90, "retroarch", r"C:\RetroArch\retroarch.exe"));
    s.ticks(3);
    assert!(s.running());
    s.s.extra_games.push("retroarch".into());
    assert!(matches!(s.tick(5), Status::Paused(g) if g.launcher == "your list"));
}

#[test]
fn off_during_game_then_on_after_it_closes() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.launch(p(50, "GTA5", r"C:\x\GTA5.exe"));
    s.ticks(1);
    s.off = true;
    assert_eq!(s.tick(5), &Status::Off);
    s.quit(50);
    s.ticks(10);
    assert_eq!(s.m.status(), &Status::Off, "stays off after the game");
    s.off = false;
    s.ticks(4);
    assert!(s.running());
    assert_eq!(s.log.iter().filter(|l| l.as_str() == "start").count(), 2);
}

#[test]
fn launcher_hand_off_does_not_bounce_the_model() {
    // Launcher -> updater -> game, with gaps shorter than the resume delay
    let mut s = Sim::new(Settings {
        resume_after_sec: 60,
        ..Default::default()
    });
    s.ticks(4);
    let starts_before = s.log.iter().filter(|l| *l == "start").count();
    s.launch(p(50, "GTA5", r"C:\x\GTA5.exe"));
    s.ticks(2);
    s.quit(50);
    s.ticks(6); // 30 s gap
    s.launch(p(51, "GTA5", r"C:\x\GTA5.exe"));
    s.ticks(20);
    s.quit(51);
    s.ticks(6);
    let starts_after = s.log.iter().filter(|l| *l == "start").count();
    assert_eq!(
        starts_after, starts_before,
        "no reload between hand-offs: {:?}",
        s.log
    );
}

#[test]
fn full_session_log() {
    let mut s = Sim::new(fast());
    s.ticks(4);
    s.launch(p(
        50,
        "eldenring",
        r"D:\SteamLibrary\steamapps\common\ELDEN RING\Game\eldenring.exe",
    ));
    s.ticks(10);
    s.quit(50);
    s.ticks(8);
    s.off = true;
    s.ticks(1);
    assert_eq!(
        s.log,
        vec![
            "start",
            "stop: gaming: eldenring via Steam",
            "popup: LLM paused",
            "start",
            "popup: Game closed",
            "popup: LLM ready",
            "stop: turned off",
            "popup: LLM off",
        ]
    );
}
