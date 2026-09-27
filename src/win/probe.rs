//! Hardware probes: NVIDIA details (nvidia-smi), GPUs and free memory as llama.cpp sees them
//! (`llama-server --list-devices`), whether the build can fit itself (`--help`), system RAM.

use super::sys;
use crate::catalog::Machine;
use crate::hardware::{self, Device, NvidiaGpu};
use crate::server::ServerCaps;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn nvidia_smi() -> Option<PathBuf> {
    // From the APIs, not %SystemRoot% / %ProgramFiles%, which the user's environment can change.
    let sys32 = sys::system32().join("nvidia-smi.exe");
    if sys32.exists() {
        return Some(sys32);
    }
    let pf = sys::program_files().join(r"NVIDIA Corporation\NVSMI\nvidia-smi.exe");
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

/// GPUs as DirectX lists them, with their dedicated memory: works for any vendor, before
/// llama.cpp is installed. Software adapters (Microsoft Basic Render Driver) are skipped.
pub fn dxgi_adapters() -> Vec<Device> {
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
    };
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for i in 0.. {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(i) }) else {
            break;
        };
        let Ok(d) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        if d.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let len = d
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(d.Description.len());
        out.push(Device {
            id: format!("DXGI{i}"),
            description: String::from_utf16_lossy(&d.Description[..len]),
            total_bytes: d.DedicatedVideoMemory as u64,
            free_bytes: 0,
        });
    }
    out
}

pub fn total_ram() -> u64 {
    let mut s = sysinfo::System::new();
    s.refresh_memory();
    s.total_memory()
}

/// What this PC offers for models: primary GPU memory + RAM. Uses llama.cpp's own device list
/// when it's installed, else nvidia-smi, else DirectX.
pub fn machine(server_exe: &Path) -> (Machine, Option<String>) {
    machine_with(server_exe, &nvidia_gpus())
}

/// [`machine`] with nvidia-smi's answer already in hand.
pub fn machine_with(server_exe: &Path, nvidia: &[NvidiaGpu]) -> (Machine, Option<String>) {
    let ram = total_ram();
    let mut devs = devices(server_exe);
    if devs.is_empty() && nvidia.is_empty() {
        devs = dxgi_adapters();
    }
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
    if let Some(g) = nvidia.iter().max_by_key(|g| g.memory_mib) {
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
