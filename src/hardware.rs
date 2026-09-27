//! What the PC can run: GPUs as llama.cpp sees them, NVIDIA driver details, and which
//! llama.cpp build to install. Pure parsing/decisions; the Windows side runs the tools.

use regex::Regex;
use std::sync::LazyLock;

const MIB: u64 = 1024 * 1024;

/// A device from `llama-server --list-devices`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// llama.cpp's name, e.g. `Vulkan0`, `CUDA0`
    pub id: String,
    pub description: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

/// Parses `llama-server --list-devices`:
/// `  Vulkan0: AMD Radeon RX 7900 XTX (24560 MiB, 23800 MiB free)`
pub fn parse_list_devices(output: &str) -> Vec<Device> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s*([A-Za-z]+\d+):\s*(.*?)\s*\((\d+) MiB, (\d+) MiB free\)\s*$").unwrap()
    });
    output
        .lines()
        .filter_map(|l| {
            let c = RE.captures(l)?;
            Some(Device {
                id: c[1].to_string(),
                description: c[2].to_string(),
                total_bytes: c[3].parse::<u64>().ok()? * MIB,
                free_bytes: c[4].parse::<u64>().ok()? * MIB,
            })
        })
        .collect()
}

/// The GPU that matters for sizing: the one with the most memory. Integrated GPUs report
/// shared memory; they're only used if nothing else is there.
pub fn primary_device(devices: &[Device]) -> Option<&Device> {
    devices
        .iter()
        .max_by_key(|d| (!is_integrated(&d.description), d.total_bytes))
}

pub fn is_integrated(description: &str) -> bool {
    let d = description.to_lowercase();
    (d.contains("intel") && !d.contains("arc"))
        || d.contains("radeon(tm) graphics")
        || d.contains("radeon graphics")
        || d.contains("microsoft basic")
}

/// One line of `nvidia-smi --query-gpu=name,driver_version,compute_cap,memory.total --format=csv,noheader,nounits`
#[derive(Debug, Clone, PartialEq)]
pub struct NvidiaGpu {
    pub name: String,
    pub driver_major: u32,
    pub compute_cap: f32,
    pub memory_mib: u64,
}

pub fn parse_nvidia_smi(output: &str) -> Vec<NvidiaGpu> {
    output
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            if f.len() != 4 {
                return None;
            }
            Some(NvidiaGpu {
                name: f[0].to_string(),
                driver_major: f[1].split('.').next()?.parse().ok()?,
                compute_cap: f[2].parse().ok()?,
                memory_mib: f[3].parse().ok()?,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// AMD, Intel, anything with a Vulkan driver (and NVIDIA as a fallback)
    Vulkan,
    /// CUDA 12.x build: NVIDIA Pascal and newer, driver 528+
    Cuda12,
    /// CUDA 13.x build: NVIDIA Turing and newer, driver 580+
    Cuda13,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Vulkan => "vulkan",
            Backend::Cuda12 => "cuda12",
            Backend::Cuda13 => "cuda13",
        }
    }

    pub fn parse(s: &str) -> Option<Backend> {
        match s.trim().to_ascii_lowercase().as_str() {
            "vulkan" => Some(Backend::Vulkan),
            "cuda12" | "cuda-12" => Some(Backend::Cuda12),
            "cuda13" | "cuda-13" | "cuda" => Some(Backend::Cuda13),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Backend::Vulkan => "Vulkan",
            Backend::Cuda12 => "CUDA 12",
            Backend::Cuda13 => "CUDA 13",
        }
    }
}

/// Best llama.cpp build for this PC. CUDA is faster than Vulkan on NVIDIA, if the driver and
/// card are new enough; AMD stays on Vulkan (avoids ROCm's idle-power problem on RDNA3).
pub fn choose_backend(nvidia: &[NvidiaGpu]) -> Backend {
    [Backend::Cuda13, Backend::Cuda12]
        .into_iter()
        .find(|b| backend_supported(*b, nvidia))
        .unwrap_or(Backend::Vulkan)
}

/// Whether this PC's biggest NVIDIA card and its driver can run `backend`'s build.
pub fn backend_supported(backend: Backend, nvidia: &[NvidiaGpu]) -> bool {
    let Some(best) = nvidia.iter().max_by_key(|g| g.memory_mib) else {
        return backend == Backend::Vulkan;
    };
    match backend {
        Backend::Vulkan => true,
        Backend::Cuda12 => best.driver_major >= 528 && best.compute_cap >= 6.0,
        Backend::Cuda13 => best.driver_major >= 580 && best.compute_cap >= 7.5,
    }
}

/// Release assets to download for a backend: the llama.cpp zip and (CUDA) the runtime DLLs.
pub fn llama_assets<'a>(names: &[&'a str], backend: Backend) -> Option<(&'a str, Option<&'a str>)> {
    match backend {
        Backend::Vulkan => names
            .iter()
            .find(|n| n.starts_with("llama-") && n.ends_with("-bin-win-vulkan-x64.zip"))
            .map(|n| (*n, None)),
        Backend::Cuda12 | Backend::Cuda13 => {
            let major = if backend == Backend::Cuda12 {
                "12"
            } else {
                "13"
            };
            static RE: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r"^llama-.*-bin-win-cuda-(\d+)\.(\d+)-x64\.zip$").unwrap()
            });
            let main = names
                .iter()
                .filter(|n| RE.captures(n).is_some_and(|c| &c[1] == major))
                .max_by_key(|n| {
                    let c = RE.captures(n).unwrap();
                    c[2].parse::<u32>().unwrap_or(0)
                })?;
            let version = RE.captures(main).map(|c| format!("{}.{}", &c[1], &c[2]))?;
            let cudart = names.iter().find(|n| {
                n.starts_with("cudart-") && n.ends_with(&format!("-cuda-{version}-x64.zip"))
            })?;
            Some((*main, Some(*cudart)))
        }
    }
}

/// Picks the backend's assets, falling back to Vulkan if a CUDA build is missing.
pub fn llama_assets_with_fallback<'a>(
    names: &[&'a str],
    backend: Backend,
) -> Option<(Backend, &'a str, Option<&'a str>)> {
    let order: &[Backend] = match backend {
        Backend::Cuda13 => &[Backend::Cuda13, Backend::Cuda12, Backend::Vulkan],
        Backend::Cuda12 => &[Backend::Cuda12, Backend::Vulkan],
        Backend::Vulkan => &[Backend::Vulkan],
    };
    order
        .iter()
        .find_map(|b| llama_assets(names, *b).map(|(m, c)| (*b, m, c)))
}

/// Which of `releases` (asset names, newest first) to install from: the newest with the
/// backend's build, else the newest with a fallback build. llama.cpp ships its builds as
/// pre-releases and keeps unrelated releases (no Windows zips) marked "latest", so the
/// whole list has to be searched.
pub fn pick_llama_release(releases: &[Vec<&str>], backend: Backend) -> Option<usize> {
    let order: &[Backend] = match backend {
        Backend::Cuda13 => &[Backend::Cuda13, Backend::Cuda12, Backend::Vulkan],
        Backend::Cuda12 => &[Backend::Cuda12, Backend::Vulkan],
        Backend::Vulkan => &[Backend::Vulkan],
    };
    order.iter().find_map(|b| {
        releases
            .iter()
            .position(|names| llama_assets(names, *b).is_some())
    })
}

/// Context size llama-server settled on, from its log (`n_ctx = 131072` / `n_ctx_seq ...`).
pub fn parse_n_ctx(server_log: &str) -> Option<u32> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bn_ctx\s*=\s*(\d+)").unwrap());
    RE.captures_iter(server_log)
        .last()
        .and_then(|c| c[1].parse().ok())
}

/// Whether this llama.cpp build fits parameters to device memory by itself (`--fit`).
pub fn supports_fit(help_output: &str) -> bool {
    help_output.contains("--fit ")
        || help_output.contains("--fit,")
        || help_output
            .lines()
            .any(|l| l.trim_start().starts_with("-fit,"))
}

pub fn gib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "ggml_vulkan: Found 2 Vulkan devices:
ggml_vulkan: 0 = AMD Radeon RX 7900 XTX (AMD proprietary driver) | uma: 0 | fp16: 1
Available devices:
  Vulkan0: AMD Radeon RX 7900 XTX (24560 MiB, 23812 MiB free)
  Vulkan1: AMD Radeon(TM) Graphics (32768 MiB, 30000 MiB free)
";

    #[test]
    fn parses_devices_including_parentheses_in_names() {
        let d = parse_list_devices(LIST);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].id, "Vulkan0");
        assert_eq!(d[0].description, "AMD Radeon RX 7900 XTX");
        assert_eq!(d[0].total_bytes, 24560 * MIB);
        assert_eq!(d[0].free_bytes, 23812 * MIB);
        let n = parse_list_devices(
            "  CUDA0: NVIDIA GeForce RTX 4090 (Founders (FE)) (24563 MiB, 22000 MiB free)",
        );
        assert_eq!(n[0].description, "NVIDIA GeForce RTX 4090 (Founders (FE))");
        assert!(parse_list_devices("Available devices:\n").is_empty());
        assert!(parse_list_devices("garbage (1 MiB)").is_empty());
    }

    #[test]
    fn primary_device_prefers_discrete() {
        let d = parse_list_devices(LIST);
        assert_eq!(
            primary_device(&d).unwrap().id,
            "Vulkan0",
            "the iGPU reports more (shared) memory but isn't used"
        );
        let only_igpu =
            parse_list_devices("  Vulkan0: Intel(R) UHD Graphics 770 (16000 MiB, 15000 MiB free)");
        assert_eq!(primary_device(&only_igpu).unwrap().id, "Vulkan0");
        assert!(primary_device(&[]).is_none());
        assert!(!is_integrated("Intel(R) Arc(TM) A770 Graphics"));
        assert!(is_integrated("Intel(R) Iris(R) Xe Graphics"));
    }

    #[test]
    fn parses_nvidia_smi() {
        let g = parse_nvidia_smi("NVIDIA GeForce RTX 4070, 581.29, 8.9, 12282\nNVIDIA GeForce GTX 1080, 546.01, 6.1, 8192\nbad line\n");
        assert_eq!(g.len(), 2);
        assert_eq!(
            g[0],
            NvidiaGpu {
                name: "NVIDIA GeForce RTX 4070".into(),
                driver_major: 581,
                compute_cap: 8.9,
                memory_mib: 12282
            }
        );
        assert!(parse_nvidia_smi("").is_empty());
        assert!(parse_nvidia_smi(
            "NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver."
        )
        .is_empty());
    }

    #[test]
    fn backend_choice() {
        let gpu = |drv: u32, cc: f32, mem: u64| NvidiaGpu {
            name: "x".into(),
            driver_major: drv,
            compute_cap: cc,
            memory_mib: mem,
        };
        assert_eq!(choose_backend(&[]), Backend::Vulkan, "AMD / Intel");
        assert_eq!(choose_backend(&[gpu(581, 8.9, 12000)]), Backend::Cuda13);
        assert_eq!(
            choose_backend(&[gpu(581, 12.0, 32000)]),
            Backend::Cuda13,
            "Blackwell"
        );
        assert_eq!(
            choose_backend(&[gpu(560, 8.9, 12000)]),
            Backend::Cuda12,
            "older driver"
        );
        assert_eq!(
            choose_backend(&[gpu(581, 6.1, 8000)]),
            Backend::Cuda12,
            "Pascal isn't supported by CUDA 13"
        );
        assert_eq!(
            choose_backend(&[gpu(470, 7.5, 8000)]),
            Backend::Vulkan,
            "driver too old for CUDA 12"
        );
        assert_eq!(
            choose_backend(&[gpu(581, 5.2, 4000)]),
            Backend::Vulkan,
            "Maxwell"
        );
        // the biggest card decides
        assert_eq!(
            choose_backend(&[gpu(581, 6.1, 4000), gpu(581, 8.9, 16000)]),
            Backend::Cuda13
        );
    }

    const ASSETS: [&str; 12] = [
        "llama-b7000-bin-win-cpu-x64.zip",
        "llama-b7000-bin-win-cuda-12.4-x64.zip",
        "cudart-llama-bin-win-cuda-12.4-x64.zip",
        "llama-b7000-bin-win-cuda-13.4-x64.zip",
        "cudart-llama-bin-win-cuda-13.4-x64.zip",
        "llama-b7000-bin-win-cuda-13.4-arm64.zip",
        "cudart-llama-bin-win-cuda-13.4-arm64.zip",
        "llama-b7000-bin-win-vulkan-x64.zip",
        "llama-b7000-bin-win-sycl-x64.zip",
        "llama-b7000-bin-win-rocm-10.0-x64.zip",
        "llama-b7000-bin-ubuntu-vulkan-x64.tar.gz",
        "llama-b7000-xcframework.zip",
    ];

    #[test]
    fn picks_release_assets() {
        assert_eq!(
            llama_assets(&ASSETS, Backend::Vulkan),
            Some(("llama-b7000-bin-win-vulkan-x64.zip", None))
        );
        assert_eq!(
            llama_assets(&ASSETS, Backend::Cuda13),
            Some((
                "llama-b7000-bin-win-cuda-13.4-x64.zip",
                Some("cudart-llama-bin-win-cuda-13.4-x64.zip")
            ))
        );
        assert_eq!(
            llama_assets(&ASSETS, Backend::Cuda12),
            Some((
                "llama-b7000-bin-win-cuda-12.4-x64.zip",
                Some("cudart-llama-bin-win-cuda-12.4-x64.zip")
            ))
        );
        // highest minor version wins, and the runtime must match it
        let two = [
            "llama-b1-bin-win-cuda-12.4-x64.zip",
            "llama-b1-bin-win-cuda-12.8-x64.zip",
            "cudart-llama-bin-win-cuda-12.8-x64.zip",
        ];
        assert_eq!(
            llama_assets(&two, Backend::Cuda12),
            Some((
                "llama-b1-bin-win-cuda-12.8-x64.zip",
                Some("cudart-llama-bin-win-cuda-12.8-x64.zip")
            ))
        );
        // CUDA zip without its runtime is unusable
        assert_eq!(
            llama_assets(&["llama-b1-bin-win-cuda-13.4-x64.zip"], Backend::Cuda13),
            None
        );
    }

    #[test]
    fn falls_back_when_a_build_is_missing() {
        let no13: Vec<&str> = ASSETS
            .iter()
            .copied()
            .filter(|n| !n.contains("13.4"))
            .collect();
        assert_eq!(
            llama_assets_with_fallback(&no13, Backend::Cuda13)
                .unwrap()
                .0,
            Backend::Cuda12
        );
        let vulkan_only = ["llama-b1-bin-win-vulkan-x64.zip"];
        assert_eq!(
            llama_assets_with_fallback(&vulkan_only, Backend::Cuda13)
                .unwrap()
                .0,
            Backend::Vulkan
        );
        assert_eq!(llama_assets_with_fallback(&[], Backend::Vulkan), None);
    }

    #[test]
    fn picks_the_newest_release_with_a_windows_build() {
        let nightly = vec!["nightly-tag.txt"];
        let vulkan_only = vec!["llama-b2-bin-win-vulkan-x64.zip"];
        let all = ASSETS.to_vec();
        let releases = vec![nightly.clone(), vulkan_only, all];
        assert_eq!(pick_llama_release(&releases, Backend::Vulkan), Some(1));
        // an older release with CUDA beats a newer one without it
        assert_eq!(pick_llama_release(&releases, Backend::Cuda13), Some(2));
        assert_eq!(pick_llama_release(&releases[..2], Backend::Cuda13), Some(1));
        assert_eq!(pick_llama_release(&[nightly], Backend::Vulkan), None);
        assert_eq!(pick_llama_release(&[], Backend::Vulkan), None);
    }

    #[test]
    fn backend_names_round_trip() {
        for b in [Backend::Vulkan, Backend::Cuda12, Backend::Cuda13] {
            assert_eq!(Backend::parse(b.as_str()), Some(b));
        }
        assert_eq!(Backend::parse("CUDA"), Some(Backend::Cuda13));
        assert_eq!(Backend::parse("rocm"), None);
    }

    #[test]
    fn reads_context_from_server_log() {
        let log = "llama_context: constructing\nllama_context: n_ctx         = 4096\n...fit...\nllama_context: n_ctx         = 131072\nllama_context: n_ctx_seq     = 131072\n";
        assert_eq!(parse_n_ctx(log), Some(131072));
        assert_eq!(parse_n_ctx("nothing"), None);
    }

    #[test]
    fn detects_fit_support() {
        assert!(supports_fit(
            "-fit,   --fit [on|off]                   whether to adjust unset arguments"
        ));
        assert!(!supports_fit("-ngl, --gpu-layers N\n-c, --ctx-size N"));
    }
}
