//! Per-process GPU use from the counters Task Manager's GPU columns use:
//! `GPU Engine\Utilization Percentage` (3D engine load) and
//! `GPU Process Memory\Dedicated Usage` (VRAM), plus "a full-screen D3D app is running".

use crate::detect::GpuProc;
use std::collections::HashMap;
use windows::core::{w, PCWSTR};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT,
    PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_FMT_LARGE, PDH_HCOUNTER, PDH_HQUERY,
    PDH_MORE_DATA,
};
use windows::Win32::UI::Shell::{SHQueryUserNotificationState, QUNS_RUNNING_D3D_FULL_SCREEN};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub struct GpuSampler {
    query: PDH_HQUERY,
    engine: PDH_HCOUNTER,
    memory: PDH_HCOUNTER,
}

/// `pid_1234_luid_0x..._engtype_3D` -> 1234
fn pid_of(instance: &str) -> Option<u32> {
    instance
        .strip_prefix("pid_")?
        .split('_')
        .next()?
        .parse()
        .ok()
}

impl GpuSampler {
    /// `None` if the GPU counters aren't available (old driver / Windows).
    pub fn new() -> Option<GpuSampler> {
        unsafe {
            let mut query = PDH_HQUERY::default();
            if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != 0 {
                return None;
            }
            let mut engine = PDH_HCOUNTER::default();
            let mut memory = PDH_HCOUNTER::default();
            let a = PdhAddEnglishCounterW(
                query,
                w!(r"\GPU Engine(*engtype_3D)\Utilization Percentage"),
                0,
                &mut engine,
            );
            let b = PdhAddEnglishCounterW(
                query,
                w!(r"\GPU Process Memory(*)\Dedicated Usage"),
                0,
                &mut memory,
            );
            if a != 0 || b != 0 {
                PdhCloseQuery(query);
                return None;
            }
            PdhCollectQueryData(query); // utilization is a rate: needs a first sample
            Some(GpuSampler {
                query,
                engine,
                memory,
            })
        }
    }

    fn read(
        counter: PDH_HCOUNTER,
        fmt: PDH_FMT,
        mut f: impl FnMut(&str, &PDH_FMT_COUNTERVALUE_ITEM_W),
    ) {
        unsafe {
            let (mut size, mut count) = (0u32, 0u32);
            if PdhGetFormattedCounterArrayW(counter, fmt, &mut size, &mut count, None)
                != PDH_MORE_DATA
            {
                return;
            }
            // u64 buffer: the items hold pointers and f64s, so they need 8-byte alignment
            let mut buf = vec![0u64; (size as usize).div_ceil(8)];
            let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
            if PdhGetFormattedCounterArrayW(counter, fmt, &mut size, &mut count, Some(items)) != 0 {
                return;
            }
            for item in std::slice::from_raw_parts(items, count as usize) {
                let status = item.FmtValue.CStatus;
                if status != PDH_CSTATUS_VALID_DATA && status != PDH_CSTATUS_NEW_DATA {
                    continue;
                }
                if let Ok(name) = item.szName.to_string() {
                    f(&name, item);
                }
            }
        }
    }

    pub fn sample(&mut self) -> Vec<GpuProc> {
        let mut map: HashMap<u32, GpuProc> = HashMap::new();
        unsafe {
            if PdhCollectQueryData(self.query) != 0 {
                return Vec::new();
            }
        }
        Self::read(self.engine, PDH_FMT_DOUBLE, |name, item| {
            if let Some(pid) = pid_of(name) {
                let g = map.entry(pid).or_insert(GpuProc {
                    pid,
                    ..Default::default()
                });
                g.engine_3d =
                    (g.engine_3d + unsafe { item.FmtValue.Anonymous.doubleValue }).min(100.0);
            }
        });
        Self::read(self.memory, PDH_FMT_LARGE, |name, item| {
            if let Some(pid) = pid_of(name) {
                let g = map.entry(pid).or_insert(GpuProc {
                    pid,
                    ..Default::default()
                });
                g.dedicated_bytes += unsafe { item.FmtValue.Anonymous.largeValue }.max(0) as u64;
            }
        });
        map.into_values().collect()
    }
}

impl Drop for GpuSampler {
    fn drop(&mut self) {
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

/// PID of the foreground app if it is an exclusive full-screen Direct3D app.
pub fn fullscreen_foreground_pid() -> Option<u32> {
    unsafe {
        if SHQueryUserNotificationState().ok()? != QUNS_RUNNING_D3D_FULL_SCREEN {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid));
        (pid != 0).then_some(pid)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_instance_pid() {
        assert_eq!(
            super::pid_of("pid_1234_luid_0x00000000_0x0000D1A5_phys_0_eng_0_engtype_3D"),
            Some(1234)
        );
        assert_eq!(super::pid_of("luid_0x0"), None);
    }
}
