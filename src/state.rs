//! The LLM state machine: Off > Paused (game) > Loading/Running, with crash counting and a
//! resume delay after the game closes. Pure: the caller gathers inputs and executes actions.

use crate::detect::GameHit;
use std::mem::discriminant;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Starting,
    Loading,
    Running,
    Paused(GameHit),
    Off,
    Error(String),
}

/// Colour/meaning of a status, shared by the tray icon and the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    Running,
    Loading,
    Paused,
    Off,
    Error,
}

impl Status {
    pub fn tone(&self) -> Tone {
        match self {
            Status::Running => Tone::Running,
            Status::Loading | Status::Starting => Tone::Loading,
            Status::Paused(_) => Tone::Paused,
            Status::Off => Tone::Off,
            Status::Error(_) => Tone::Error,
        }
    }

    pub fn text(&self, model_short: &str) -> String {
        match self {
            Status::Starting => "Starting...".into(),
            Status::Loading => format!("Loading - {model_short}"),
            Status::Running => format!("Running - {model_short}"),
            Status::Paused(g) => format!("Paused - {}", g.name),
            Status::Off => "Off".into(),
            Status::Error(e) => format!("Error - {e}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popup {
    pub title: String,
    pub subtitle: String,
    pub tone: Tone,
}

fn popup(title: &str, subtitle: impl Into<String>, tone: Tone) -> Option<Popup> {
    Some(Popup {
        title: title.into(),
        subtitle: subtitle.into(),
        tone,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    StartServer,
    StopServer(String),
}

#[derive(Debug, Clone)]
pub struct Inputs {
    pub now: Instant,
    pub off: bool,
    /// Game detected this tick (already `None` if pausing is disabled).
    pub game: Option<GameHit>,
    pub resume_after: Duration,
    pub server_running: bool,
    pub server_ready: bool,
    pub model_exists: bool,
    pub server_exe_exists: bool,
    pub model_name: String,
}

#[derive(Debug, Default)]
pub struct Output {
    pub actions: Vec<Action>,
    pub popup: Option<Popup>,
    pub logs: Vec<String>,
}

pub fn model_short(model: &str) -> String {
    let m = model.strip_suffix(".gguf").unwrap_or(model);
    m.replace("-UD-", " ").replace('-', " ")
}

#[derive(Debug)]
pub struct Machine {
    status: Status,
    game: Option<GameHit>,
    last_game: Option<(GameHit, Instant)>,
    failed: u32,
    pending: bool,
    error_msg: String,
    announce_ready: bool,
    load_start: Option<Instant>,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            status: Status::Starting,
            game: None,
            last_game: None,
            failed: 0,
            pending: false,
            error_msg: String::new(),
            announce_ready: false,
            load_start: None,
        }
    }
}

impl Machine {
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// The game currently holding the pause, if any.
    pub fn game(&self) -> Option<&GameHit> {
        self.game.as_ref()
    }

    /// Clear crash counting (turn on, restart, settings change).
    pub fn reset_failures(&mut self) {
        self.failed = 0;
        self.error_msg.clear();
    }

    /// Forget the held game ("Not a game - ignore this app").
    pub fn forget_game(&mut self) {
        self.last_game = None;
    }

    /// llama-server could not be launched at all.
    pub fn start_failed(&mut self, msg: String) {
        self.pending = false;
        self.failed = 3;
        self.error_msg = msg;
    }

    fn effective_game(&mut self, i: &Inputs) -> Option<GameHit> {
        if let Some(g) = &i.game {
            self.last_game = Some((g.clone(), i.now));
            return Some(g.clone());
        }
        // Hold the pause a while after the game disappears (loading screens, launcher hand-offs)
        match &self.last_game {
            Some((g, at)) if i.now.duration_since(*at) < i.resume_after => Some(g.clone()),
            _ => {
                self.last_game = None;
                None
            }
        }
    }

    pub fn step(&mut self, i: &Inputs) -> Output {
        let mut out = Output::default();
        let game = self.effective_game(i);

        if i.off {
            if i.server_running {
                out.actions.push(Action::StopServer("turned off".into()));
                self.pending = false;
            }
            self.game = None;
            self.set_status(Status::Off, i, &mut out);
            return out;
        }
        if let Some(g) = game {
            if i.server_running {
                out.actions.push(Action::StopServer(format!(
                    "gaming: {} via {}",
                    g.name, g.launcher
                )));
                self.pending = false;
            }
            self.game = Some(g.clone());
            self.set_status(Status::Paused(g), i, &mut out);
            return out;
        }
        if self.game.take().is_some() {
            out.logs.push("game closed".into());
        }

        if !i.server_running {
            if self.pending {
                self.failed += 1;
                self.pending = false;
                out.logs
                    .push(format!("server exited before ready (fail {})", self.failed));
            }
            if self.failed >= 3 {
                if self.error_msg.is_empty() {
                    self.error_msg = "server keeps crashing (see log)".into();
                }
                let e = Status::Error(self.error_msg.clone());
                self.set_status(e, i, &mut out);
                return out;
            }
            // Checked every tick, so adding the model (or llama.cpp) recovers on its own.
            let missing = if !i.model_exists {
                Some(format!("Model missing: {}", i.model_name))
            } else if !i.server_exe_exists {
                Some("llama-server.exe missing".to_string())
            } else {
                None
            };
            if let Some(m) = missing {
                self.set_status(Status::Error(m), i, &mut out);
                return out;
            }
            out.actions.push(Action::StartServer);
            self.pending = true;
            self.set_status(Status::Loading, i, &mut out);
            return out;
        }
        if i.server_ready {
            if self.pending {
                out.logs.push("server ready".into());
            }
            self.pending = false;
            self.reset_failures();
            self.set_status(Status::Running, i, &mut out);
        } else {
            self.set_status(Status::Loading, i, &mut out);
        }
        out
    }

    fn set_status(&mut self, status: Status, i: &Inputs, out: &mut Output) {
        let prev = std::mem::replace(&mut self.status, status);
        if prev != Status::Starting && discriminant(&prev) != discriminant(&self.status) {
            out.popup = self.announce(&prev, i);
        }
    }

    /// Popup for a state change (none on the app's own startup).
    fn announce(&mut self, prev: &Status, i: &Inputs) -> Option<Popup> {
        let m = model_short(&i.model_name);
        match &self.status {
            Status::Paused(g) => {
                let who = format!("{} ({})", g.name, g.launcher);
                if *prev == Status::Off {
                    popup(
                        "LLM on - waiting",
                        format!("{who} is running. Starts when you quit."),
                        Tone::Paused,
                    )
                } else {
                    popup(
                        "LLM paused",
                        format!("{who} detected  -  GPU freed"),
                        Tone::Paused,
                    )
                }
            }
            Status::Off => popup("LLM off", "Turned off  -  GPU freed", Tone::Off),
            Status::Loading => {
                self.announce_ready = true;
                self.load_start = Some(i.now);
                match prev {
                    Status::Paused(_) => {
                        popup("Game closed", format!("Reloading {m}..."), Tone::Loading)
                    }
                    Status::Running => {
                        popup("LLM restarting", "Applying new settings...", Tone::Loading)
                    }
                    _ => popup("LLM on", format!("Loading {m}..."), Tone::Loading),
                }
            }
            Status::Running if self.announce_ready => {
                self.announce_ready = false;
                let secs = self
                    .load_start
                    .map(|t| i.now.duration_since(t).as_secs())
                    .unwrap_or(0);
                popup(
                    "LLM ready",
                    format!("{m}  -  loaded in {secs}s"),
                    Tone::Running,
                )
            }
            Status::Error(e) => popup("LLM error", e.clone(), Tone::Error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Sim {
        m: Machine,
        i: Inputs,
    }

    impl Sim {
        fn new() -> Sim {
            Sim {
                m: Machine::default(),
                i: Inputs {
                    now: Instant::now(),
                    off: false,
                    game: None,
                    resume_after: Duration::from_secs(60),
                    server_running: false,
                    server_ready: false,
                    model_exists: true,
                    server_exe_exists: true,
                    model_name: "Qwen3.8-27B-UD-Q4_K_XL.gguf".into(),
                },
            }
        }
        /// Advance time, step, and apply start/stop actions to the simulated server.
        fn tick(&mut self, secs: u64) -> Output {
            self.i.now += Duration::from_secs(secs);
            let out = self.m.step(&self.i);
            for a in &out.actions {
                match a {
                    Action::StartServer => self.i.server_running = true,
                    Action::StopServer(_) => {
                        self.i.server_running = false;
                        self.i.server_ready = false;
                    }
                }
            }
            out
        }
    }

    fn game() -> GameHit {
        GameHit {
            name: "ELDEN RING".into(),
            launcher: "Steam".into(),
            process: Some("eldenring".into()),
        }
    }

    #[test]
    fn starts_loads_and_runs() {
        let mut s = Sim::new();
        let out = s.tick(0);
        assert_eq!(out.actions, vec![Action::StartServer]);
        assert_eq!(out.popup, None, "no popup on startup");
        assert_eq!(*s.m.status(), Status::Loading);
        s.i.server_ready = true;
        let out = s.tick(5);
        assert_eq!(*s.m.status(), Status::Running);
        assert!(out.actions.is_empty());
    }

    #[test]
    fn pauses_for_game_and_resumes_after_delay() {
        let mut s = Sim::new();
        s.tick(0);
        s.i.server_ready = true;
        s.tick(5);

        s.i.game = Some(game());
        let out = s.tick(5);
        assert!(matches!(out.actions[..], [Action::StopServer(_)]));
        assert_eq!(out.popup.unwrap().title, "LLM paused");
        assert!(matches!(s.m.status(), Status::Paused(_)));

        s.i.game = None;
        s.tick(30);
        assert!(
            matches!(s.m.status(), Status::Paused(_)),
            "held during resume delay"
        );
        let out = s.tick(31);
        assert_eq!(out.actions, vec![Action::StartServer]);
        assert_eq!(out.popup.unwrap().title, "Game closed");

        s.i.server_ready = true;
        let out = s.tick(20);
        let p = out.popup.unwrap();
        assert_eq!(p.title, "LLM ready");
        assert!(p.subtitle.contains("loaded in 20s"), "{}", p.subtitle);
    }

    #[test]
    fn off_wins_over_everything() {
        let mut s = Sim::new();
        s.tick(0);
        s.i.off = true;
        s.i.game = Some(game());
        let out = s.tick(1);
        assert!(matches!(out.actions[..], [Action::StopServer(_)]));
        assert_eq!(*s.m.status(), Status::Off);
        // turning on while a game runs: waiting popup
        s.i.off = false;
        let out = s.tick(1);
        assert_eq!(out.popup.unwrap().title, "LLM on - waiting");
    }

    #[test]
    fn three_crashes_is_an_error() {
        let mut s = Sim::new();
        for _ in 0..3 {
            s.tick(5); // start
            s.i.server_running = false; // crashed before ready
        }
        let out = s.tick(5);
        assert!(out.actions.is_empty());
        assert_eq!(
            *s.m.status(),
            Status::Error("server keeps crashing (see log)".into())
        );
        assert_eq!(out.popup.unwrap().tone, Tone::Error);
        // stays in error until reset
        assert!(s.tick(5).actions.is_empty());
        s.m.reset_failures();
        assert_eq!(s.tick(5).actions, vec![Action::StartServer]);
    }

    #[test]
    fn missing_model_is_an_error_without_starting() {
        let mut s = Sim::new();
        s.i.model_exists = false;
        let out = s.tick(0);
        assert!(out.actions.is_empty());
        assert_eq!(
            *s.m.status(),
            Status::Error("Model missing: Qwen3.8-27B-UD-Q4_K_XL.gguf".into())
        );
        s.i.model_exists = true;
        assert_eq!(
            s.tick(5).actions,
            vec![Action::StartServer],
            "recovers once the model is there"
        );
    }

    #[test]
    fn forget_game_resumes_immediately() {
        let mut s = Sim::new();
        s.i.game = Some(game());
        s.tick(0);
        s.i.game = None;
        s.m.forget_game();
        assert_eq!(s.tick(1).actions, vec![Action::StartServer]);
    }

    #[test]
    fn short_model_name() {
        assert_eq!(
            model_short("Qwen3.8-27B-UD-Q4_K_XL.gguf"),
            "Qwen3.8 27B Q4_K_XL"
        );
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use proptest::prelude::*;

    /// Simulated world: the machine plus a fake llama-server that loads in `load_ticks`.
    struct World {
        m: Machine,
        i: Inputs,
        load_left: Option<u32>,
        load_ticks: u32,
        popups: Vec<Popup>,
        starts: u32,
        stops: u32,
    }

    impl World {
        fn new() -> World {
            World {
                m: Machine::default(),
                i: Inputs {
                    now: Instant::now(),
                    off: false,
                    game: None,
                    resume_after: Duration::from_secs(60),
                    server_running: false,
                    server_ready: false,
                    model_exists: true,
                    server_exe_exists: true,
                    model_name: "Qwen3.8-27B-UD-Q4_K_XL.gguf".into(),
                },
                load_left: None,
                load_ticks: 2,
                popups: Vec::new(),
                starts: 0,
                stops: 0,
            }
        }

        fn tick(&mut self, secs: u64) -> Output {
            self.i.now += Duration::from_secs(secs);
            if let Some(n) = self.load_left.as_mut() {
                if *n == 0 {
                    self.i.server_ready = true;
                } else {
                    *n -= 1;
                }
            }
            let out = self.m.step(&self.i);
            for a in &out.actions {
                match a {
                    Action::StartServer => {
                        assert!(!self.i.server_running, "started while already running");
                        self.i.server_running = true;
                        self.i.server_ready = false;
                        self.load_left = Some(self.load_ticks);
                        self.starts += 1;
                    }
                    Action::StopServer(_) => {
                        assert!(self.i.server_running, "stopped while not running");
                        self.i.server_running = false;
                        self.i.server_ready = false;
                        self.load_left = None;
                        self.stops += 1;
                    }
                }
            }
            if let Some(p) = &out.popup {
                self.popups.push(p.clone());
            }
            out
        }

        fn crash(&mut self) {
            self.i.server_running = false;
            self.i.server_ready = false;
            self.load_left = None;
        }

        fn run_until_running(&mut self) {
            for _ in 0..10 {
                self.tick(5);
                if *self.m.status() == Status::Running {
                    return;
                }
            }
            panic!("never became ready: {:?}", self.m.status());
        }

        fn titles(&self) -> Vec<&str> {
            self.popups.iter().map(|p| p.title.as_str()).collect()
        }
    }

    fn game(name: &str) -> GameHit {
        GameHit {
            name: name.into(),
            launcher: "Steam".into(),
            process: Some(name.to_lowercase()),
        }
    }

    #[test]
    fn status_text_and_tone_for_every_status() {
        let cases = [
            (Status::Starting, "Starting...", Tone::Loading),
            (Status::Loading, "Loading - M", Tone::Loading),
            (Status::Running, "Running - M", Tone::Running),
            (
                Status::Paused(game("Hades")),
                "Paused - Hades",
                Tone::Paused,
            ),
            (Status::Off, "Off", Tone::Off),
            (Status::Error("boom".into()), "Error - boom", Tone::Error),
        ];
        for (s, text, tone) in cases {
            assert_eq!(s.text("M"), text);
            assert_eq!(s.tone(), tone);
        }
    }

    #[test]
    fn a_whole_evening() {
        let mut w = World::new();
        w.run_until_running();
        assert!(
            w.titles().is_empty(),
            "the app's own startup is silent (no popup over a game at boot)"
        );

        // game starts: stop immediately
        w.i.game = Some(game("Hades"));
        w.tick(5);
        assert!(!w.i.server_running);
        // game keeps running for an hour: nothing else happens
        for _ in 0..720 {
            let out = w.tick(5);
            assert!(out.actions.is_empty() && out.popup.is_none());
        }
        // game closes: held for 60 s, then reload
        w.i.game = None;
        for _ in 0..11 {
            w.tick(5);
            assert!(matches!(w.m.status(), Status::Paused(_)));
        }
        w.tick(5);
        assert_eq!(*w.m.status(), Status::Loading);
        w.run_until_running();

        // hotkey off, then on
        w.i.off = true;
        w.tick(1);
        assert_eq!(*w.m.status(), Status::Off);
        w.i.off = false;
        w.tick(1);
        w.run_until_running();
        assert_eq!(
            w.titles(),
            vec![
                "LLM paused",
                "Game closed",
                "LLM ready",
                "LLM off",
                "LLM on",
                "LLM ready"
            ]
        );
        assert_eq!((w.starts, w.stops), (3, 2));
    }

    #[test]
    fn game_during_loading_stops_the_load() {
        let mut w = World::new();
        w.load_ticks = 100;
        w.tick(0);
        w.tick(5);
        assert_eq!(*w.m.status(), Status::Loading);
        w.i.game = Some(game("Hades"));
        w.tick(5);
        assert!(!w.i.server_running);
        assert!(matches!(w.m.status(), Status::Paused(_)));
        // the interrupted load is not counted as a crash
        w.i.game = None;
        w.i.resume_after = Duration::ZERO;
        w.m.forget_game();
        w.tick(5);
        assert_eq!(*w.m.status(), Status::Loading);
        assert_eq!(w.m.failed, 0);
    }

    #[test]
    fn switching_games_does_not_spam_popups() {
        let mut w = World::new();
        w.run_until_running();
        w.i.game = Some(game("Hades"));
        w.tick(5);
        w.i.game = Some(game("Celeste"));
        w.tick(5);
        assert_eq!(*w.m.status(), Status::Paused(game("Celeste")));
        assert_eq!(w.titles().iter().filter(|t| **t == "LLM paused").count(), 1);
    }

    #[test]
    fn resume_delay_restarts_when_game_comes_back() {
        let mut w = World::new();
        w.i.game = Some(game("Hades"));
        w.tick(0);
        w.i.game = None;
        w.tick(50); // 50 s after
        w.i.game = Some(game("Hades")); // launcher hand-off: game back
        w.tick(5);
        w.i.game = None;
        w.tick(50); // 50 s after the *last* sighting: still held
        assert!(matches!(w.m.status(), Status::Paused(_)));
        w.tick(11);
        assert_eq!(*w.m.status(), Status::Loading);
    }

    #[test]
    fn zero_resume_delay_resumes_on_the_next_tick() {
        let mut w = World::new();
        w.i.resume_after = Duration::ZERO;
        w.i.game = Some(game("Hades"));
        w.tick(0);
        w.i.game = None;
        assert_eq!(w.tick(1).actions, vec![Action::StartServer]);
    }

    #[test]
    fn crash_count_resets_after_a_successful_load() {
        let mut w = World::new();
        for _ in 0..2 {
            w.tick(5);
            w.crash();
        }
        w.run_until_running();
        assert_eq!(w.m.failed, 0);
        // two more crashes are fine again
        for _ in 0..2 {
            w.crash();
            w.tick(5);
        }
        assert_eq!(*w.m.status(), Status::Loading);
    }

    #[test]
    fn crash_while_running_restarts_without_counting_twice() {
        let mut w = World::new();
        w.run_until_running();
        w.crash();
        let out = w.tick(5);
        assert_eq!(out.actions, vec![Action::StartServer]);
        assert_eq!(w.m.failed, 0, "a crash after 'ready' isn't a failed start");
    }

    #[test]
    fn error_popup_is_shown_once() {
        let mut w = World::new();
        for _ in 0..3 {
            w.tick(5);
            w.crash();
        }
        for _ in 0..20 {
            w.tick(5);
        }
        assert_eq!(w.titles().iter().filter(|t| **t == "LLM error").count(), 1);
        assert_eq!(w.starts, 3);
    }

    #[test]
    fn off_clears_error_via_reset_and_turn_on() {
        let mut w = World::new();
        for _ in 0..3 {
            w.tick(5);
            w.crash();
        }
        w.tick(5);
        assert!(matches!(w.m.status(), Status::Error(_)));
        w.i.off = true;
        w.tick(1);
        assert_eq!(*w.m.status(), Status::Off);
        w.i.off = false;
        w.m.reset_failures(); // what "Turn on" does
        w.run_until_running();
    }

    #[test]
    fn start_failure_is_an_immediate_error_until_reset() {
        let mut w = World::new();
        w.tick(0);
        w.crash();
        w.m.start_failed("can't start llama-server: access denied".into());
        let out = w.tick(5);
        assert!(out.actions.is_empty());
        assert_eq!(
            *w.m.status(),
            Status::Error("can't start llama-server: access denied".into())
        );
        w.m.reset_failures();
        assert_eq!(w.tick(5).actions, vec![Action::StartServer]);
    }

    #[test]
    fn missing_llama_cpp_then_model() {
        let mut w = World::new();
        w.i.server_exe_exists = false;
        w.tick(0);
        assert_eq!(
            *w.m.status(),
            Status::Error("llama-server.exe missing".into())
        );
        w.i.server_exe_exists = true;
        w.i.model_exists = false;
        w.tick(5);
        assert!(matches!(w.m.status(), Status::Error(e) if e.starts_with("Model missing")));
        assert_eq!(w.starts, 0);
    }

    #[test]
    fn missing_model_while_paused_or_off_is_not_an_error() {
        let mut w = World::new();
        w.i.model_exists = false;
        w.i.off = true;
        w.tick(0);
        assert_eq!(*w.m.status(), Status::Off);
        w.i.off = false;
        w.i.game = Some(game("Hades"));
        w.tick(1);
        assert!(matches!(w.m.status(), Status::Paused(_)));
    }

    #[test]
    fn restart_via_stop_shows_restarting_popup() {
        let mut w = World::new();
        w.run_until_running();
        w.crash(); // what Cmd::Restart does: stop + reset
        w.m.reset_failures();
        w.tick(1);
        assert_eq!(w.popups.last().unwrap().title, "LLM restarting");
    }

    #[test]
    fn ready_popup_reports_load_time() {
        let mut w = World::new();
        w.run_until_running();
        w.i.off = true;
        w.tick(1);
        w.i.off = false;
        w.load_ticks = 3;
        w.run_until_running();
        let p = w.popups.last().unwrap();
        assert_eq!(p.title, "LLM ready");
        assert_eq!(p.tone, Tone::Running);
        assert!(
            p.subtitle.starts_with("Qwen3.8 27B Q4_K_XL"),
            "{}",
            p.subtitle
        );
    }

    #[test]
    fn paused_popup_names_game_and_launcher() {
        let mut w = World::new();
        w.run_until_running();
        w.i.game = Some(GameHit {
            name: "ELDEN RING".into(),
            launcher: "Steam".into(),
            process: None,
        });
        w.tick(1);
        assert_eq!(
            w.popups.last().unwrap().subtitle,
            "ELDEN RING (Steam) detected  -  GPU freed"
        );
    }

    #[test]
    fn logs_explain_what_happened() {
        let mut w = World::new();
        w.tick(0);
        w.crash();
        let out = w.tick(5);
        assert!(out
            .logs
            .iter()
            .any(|l| l.contains("server exited before ready (fail 1)")));
        w.run_until_running();
        w.i.game = Some(game("Hades"));
        w.tick(1);
        w.i.game = None;
        w.i.resume_after = Duration::ZERO;
        let out = w.tick(1);
        assert!(out.logs.contains(&"game closed".to_string()));
    }

    #[test]
    fn model_short_edge_cases() {
        assert_eq!(model_short("plain"), "plain");
        assert_eq!(model_short("a-b.gguf"), "a b");
        assert_eq!(model_short(".gguf"), "");
    }

    #[derive(Debug, Clone)]
    enum Ev {
        Tick(u64),
        Off(bool),
        Game(Option<u8>),
        Crash,
        Ready,
        ModelExists(bool),
        Reset,
        Forget,
    }

    fn ev() -> impl Strategy<Value = Ev> {
        prop_oneof![
            4 => (0u64..90).prop_map(Ev::Tick),
            1 => any::<bool>().prop_map(Ev::Off),
            2 => prop::option::of(0u8..3).prop_map(Ev::Game),
            1 => Just(Ev::Crash),
            2 => Just(Ev::Ready),
            1 => any::<bool>().prop_map(Ev::ModelExists),
            1 => Just(Ev::Reset),
            1 => Just(Ev::Forget),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3000))]

        /// Safety rules that must hold after any sequence of events.
        #[test]
        fn invariants_hold(events in prop::collection::vec(ev(), 1..120)) {
            let mut w = World::new();
            w.load_ticks = 1;
            let mut last_seen: Option<Instant> = None;
            for e in events {
                match e {
                    Ev::Tick(s) => {
                        let before_running = w.i.server_running;
                        let out = w.tick(s);
                        let starts = out.actions.iter().filter(|a| **a == Action::StartServer).count();
                        let stops = out.actions.iter().filter(|a| matches!(a, Action::StopServer(_))).count();
                        prop_assert!(starts + stops <= 1, "at most one action per step: {:?}", out.actions);
                        if w.i.game.is_some() { last_seen = Some(w.i.now); }
                        let held = last_seen.is_some_and(|t| w.i.now.duration_since(t) < w.i.resume_after);
                        // 1. Off always wins, and nothing runs while off
                        if w.i.off {
                            prop_assert_eq!(w.m.status(), &Status::Off);
                            prop_assert!(!w.i.server_running);
                        }
                        // 2. Never run the model while a game is (or was just) running
                        if w.i.game.is_some() {
                            prop_assert!(!w.i.server_running);
                            prop_assert!(matches!(w.m.status(), Status::Paused(_) | Status::Off));
                        }
                        if starts == 1 {
                            prop_assert!(!before_running);
                            prop_assert!(!w.i.off && !held);
                            prop_assert!(w.i.model_exists);
                        }
                        // 3. Running means the server really is ready
                        if *w.m.status() == Status::Running {
                            prop_assert!(w.i.server_running && w.i.server_ready);
                        }
                        // 4. The text/tone never panic and are consistent
                        let _ = w.m.status().text("m");
                    }
                    Ev::Off(b) => w.i.off = b,
                    Ev::Game(g) => w.i.game = g.map(|n| game(&format!("G{n}"))),
                    Ev::Crash => w.crash(),
                    Ev::Ready => if w.i.server_running { w.i.server_ready = true },
                    Ev::ModelExists(b) => w.i.model_exists = b,
                    Ev::Reset => w.m.reset_failures(),
                    Ev::Forget => { w.m.forget_game(); last_seen = None; }
                }
            }
            // at most one popup per tick is guaranteed by Output; crash retries are bounded:
            prop_assert!(w.m.failed <= 3);
        }
    }
}
