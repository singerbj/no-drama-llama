//! Hardware probes: NVIDIA details (nvidia-smi), GPUs and free memory as llama.cpp sees them
//! (`llama-server --list-devices`), whether the build can fit itself (`--help`), system RAM.

use super::sys;
use crate::catalog::Machine;
use crate::hardware::{self, Device, NvidiaGpu};
use crate::server::ServerCaps;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn nvidia_smi() -> Option<PathBuf> {
    let sys32 =
        PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()))
            .join(r"System32\nvidia-smi.exe");
    if sys32.exists() {
        return Some(sys32);
    }
    let pf = PathBuf::from(
        std::env::var_os("ProgramFiles").unwrap_or_else(|| r"C:\Program Files".into()),
    )
    .join(r"NVIDIA Corporation\NVSMI\nvidia-smi.exe");
    pf.exists().then_some(pf)
}

/// NVIDIA GPUs (empty on AMD/Intel-only PCs or without a working NVIDIA driver).
pub fn nvidia_gpus() -> Vec<NvidiaGpu> {
    let Some(smi) = nvidia_smi() else {
        return Vec::new();
    };
    sys::run_with_timeout(
        &smi,
        &[
            "--query-gpu=name,driver_version,compute_cap,memory.total",
            "--format=csv,noheader,nounits",
        ],
        Duration::from_secs(15),
    )
    .map(|o| hardware::parse_nvidia_smi(&o))
    .unwrap_or_default()
}

/// Devices llama.cpp can use, with free memory right now.
pub fn devices(server_exe: &Path) -> Vec<Device> {
    if !server_exe.exists() {
        return Vec::new();
    }
    sys::run_with_timeout(server_exe, &["--list-devices"], Duration::from_secs(30))
        .map(|o| hardware::parse_list_devices(&o))
        .unwrap_or_default()
}

pub fn server_caps(server_exe: &Path) -> ServerCaps {
    match sys::run_with_timeout(server_exe, &["--help"], Duration::from_secs(30)) {
        Ok(help) => ServerCaps {
            fit: hardware::supports_fit(&help),
        },
        Err(_) => ServerCaps::default(),
    }
}

pub fn total_ram() -> u64 {
    let mut s = sysinfo::System::new();
    s.refresh_memory();
    s.total_memory()
}

/// What this PC offers for models: primary GPU memory + RAM. Uses llama.cpp's own device list
/// when it's installed, else nvidia-smi.
pub fn machine(server_exe: &Path) -> (Machine, Option<String>) {
    let ram = total_ram();
    let devs = devices(server_exe);
    if let Some(d) = hardware::primary_device(&devs) {
        let vram = if hardware::is_integrated(&d.description) {
            0
        } else {
            d.total_bytes
        };
        return (
            Machine { vram, ram },
            Some(format!(
                "{} ({:.0} GB)",
                d.description,
                hardware::gib(d.total_bytes)
            )),
        );
    }
    if let Some(g) = nvidia_gpus().into_iter().max_by_key(|g| g.memory_mib) {
        return (
            Machine {
                vram: g.memory_mib * 1024 * 1024,
                ram,
            },
            Some(format!(
                "{} ({:.0} GB)",
                g.name,
                g.memory_mib as f64 / 1024.0
            )),
        );
    }
    (Machine { vram: 0, ram }, None)
}
