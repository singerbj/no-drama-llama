//! Running processes (names + exe paths) via sysinfo, which reads paths with
//! `QueryFullProcessImageNameW` and caches them per process.

use crate::detect::ProcInfo;
use std::collections::HashMap;
use std::path::Path;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

pub struct Procs {
    sys: System,
}

impl Default for Procs {
    fn default() -> Self {
        Self::new()
    }
}

impl Procs {
    pub fn new() -> Procs {
        let mut p = Procs { sys: System::new() };
        p.refresh();
        p
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
    }

    pub fn list(&self) -> Vec<ProcInfo> {
        self.sys
            .processes()
            .values()
            .map(|p| {
                let name = p.name().to_string_lossy();
                let name = name
                    .strip_suffix(".exe")
                    .or_else(|| name.strip_suffix(".EXE"))
                    .unwrap_or(&name)
                    .to_string();
                ProcInfo {
                    pid: p.pid().as_u32(),
                    name,
                    path: p.exe().map(|e| e.to_string_lossy().into_owned()),
                    title: None,
                }
            })
            .collect()
    }

    pub fn by_pid(&self) -> HashMap<u32, ProcInfo> {
        self.list().into_iter().map(|p| (p.pid, p)).collect()
    }

    /// PIDs of processes running this exact executable.
    pub fn pids_of(&self, exe: &Path) -> Vec<u32> {
        self.sys
            .processes()
            .values()
            .filter(|p| {
                p.exe()
                    .is_some_and(|e| e.as_os_str().eq_ignore_ascii_case(exe.as_os_str()))
            })
            .map(|p| p.pid().as_u32())
            .collect()
    }

    /// Kills the processes and waits (up to 5 s) for them to exit.
    pub fn kill(&mut self, pids: &[u32]) {
        for pid in pids {
            if let Some(p) = self.sys.process(Pid::from_u32(*pid)) {
                p.kill();
            }
        }
        for _ in 0..50 {
            self.refresh();
            if pids
                .iter()
                .all(|p| self.sys.process(Pid::from_u32(*p)).is_none())
            {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Other processes whose command line contains `needle` (case-insensitive), e.g. the
    /// PowerShell edition's `llm-tray.ps1`.
    pub fn pids_with_cmdline(&mut self, exe_name: &str, needle: &str) -> Vec<u32> {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always),
        );
        let needle = needle.to_lowercase();
        self.sys
            .processes()
            .values()
            .filter(|p| p.name().eq_ignore_ascii_case(exe_name))
            .filter(|p| {
                p.cmd()
                    .iter()
                    .any(|a| a.to_string_lossy().to_lowercase().contains(&needle))
            })
            .map(|p| p.pid().as_u32())
            .collect()
    }
}
