//! Small Win32 helpers: elevation, consoles, message boxes, single instance, processes.

use anyhow::{bail, Context, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_ALREADY_EXISTS, HANDLE, HLOCAL, WAIT_ABANDONED,
    WAIT_OBJECT_0,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, TokenElevation, TokenUser, TOKEN_ELEVATION, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, ReleaseMutex,
    WaitForSingleObject, INFINITE,
};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO,
    SW_SHOWNORMAL,
};

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const DETACHED_PROCESS: u32 = 0x0000_0008;

pub fn wide(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elev = TOKEN_ELEVATION::default();
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elev as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elev.TokenIsElevated != 0
    }
}

/// SID of the signed-in user (e.g. `S-1-5-21-...`). The same when elevated through UAC.
pub fn current_user_sid() -> Result<String> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;
        let mut len = 0u32;
        let _ = GetTokenInformation(token, TokenUser, None, 0, &mut len);
        let mut buf = vec![0u64; (len as usize).div_ceil(8)];
        let r = GetTokenInformation(
            token,
            TokenUser,
            Some(buf.as_mut_ptr() as *mut _),
            len,
            &mut len,
        );
        let _ = CloseHandle(token);
        r?;
        let user = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut s = PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut s)?;
        let out = s.to_string()?;
        let _ = LocalFree(Some(HLOCAL(s.0 as *mut _)));
        Ok(out)
    }
}

/// Runs this exe elevated (UAC prompt) with `args` and waits for it. Returns its exit code.
pub fn run_elevated(args: &str, wait: bool) -> Result<u32> {
    let exe = wide(std::env::current_exe()?);
    let verb = wide("runas");
    let params = wide(args);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(exe.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    unsafe {
        ShellExecuteExW(&mut info).context("elevation was cancelled")?;
        let mut code = 0u32;
        if wait && !info.hProcess.is_invalid() {
            WaitForSingleObject(info.hProcess, INFINITE);
            let _ = GetExitCodeProcess(info.hProcess, &mut code);
        }
        if !info.hProcess.is_invalid() {
            let _ = CloseHandle(info.hProcess);
        }
        Ok(code)
    }
}

/// Use the parent's console if started from a terminal; otherwise open a new one if `alloc`.
pub fn console(alloc: bool) {
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() && alloc {
            let _ = AllocConsole();
        }
    }
}

pub fn message(text: &str, error: bool) {
    let t = wide(text);
    let c = wide(crate::paths::APP_NAME);
    let style = if error {
        MB_OK | MB_ICONERROR
    } else {
        MB_OK | MB_ICONINFORMATION
    };
    unsafe {
        MessageBoxW(None, PCWSTR(t.as_ptr()), PCWSTR(c.as_ptr()), style);
    }
}

pub fn ask(text: &str) -> bool {
    let t = wide(text);
    let c = wide(crate::paths::APP_NAME);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(t.as_ptr()),
            PCWSTR(c.as_ptr()),
            MB_YESNO | MB_ICONQUESTION,
        ) == IDYES
    }
}

/// Holds the app's single-instance mutex until dropped.
pub struct SingleInstance(HANDLE);

impl SingleInstance {
    const NAME: &'static str = r"Global\NoDramaLlamaTray";

    /// `Ok(None)` if another instance holds it (after waiting up to `wait_ms`).
    pub fn acquire(wait_ms: u32) -> Result<Option<SingleInstance>> {
        Self::acquire_named(Self::NAME, wait_ms)
    }

    pub fn acquire_named(name: &str, wait_ms: u32) -> Result<Option<SingleInstance>> {
        let name = wide(name);
        unsafe {
            let h = CreateMutexW(None, true, PCWSTR(name.as_ptr()))?;
            if GetLastError() != ERROR_ALREADY_EXISTS {
                return Ok(Some(SingleInstance(h)));
            }
            let w = WaitForSingleObject(h, wait_ms);
            if w == WAIT_OBJECT_0 || w == WAIT_ABANDONED {
                return Ok(Some(SingleInstance(h)));
            }
            let _ = CloseHandle(h);
            Ok(None)
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

/// A command that runs without flashing a console window.
pub fn hidden(program: impl AsRef<OsStr>) -> Command {
    let mut c = Command::new(program);
    c.creation_flags(CREATE_NO_WINDOW).stdin(Stdio::null());
    c
}

/// Runs a system tool and returns its output; errors if it can't start or fails.
pub fn run(program: &str, args: &[&str]) -> Result<Output> {
    let out = hidden(program)
        .args(args)
        .output()
        .with_context(|| format!("couldn't run {program}"))?;
    if !out.status.success() {
        bail!(
            "{program} {} failed ({}): {}{}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out)
}

/// Runs a PowerShell snippet (used for the NetAdapter cmdlets, which have no simple API).
pub fn powershell(script: &str) -> Result<String> {
    let out = hidden("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .context("couldn't run powershell.exe")?;
    if !out.status.success() {
        bail!(
            "PowerShell failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Opens a URL, file or folder with its default app, *not* elevated (explorer hands it off).
pub fn open_unelevated(target: &Path) {
    let _ = Command::new("explorer.exe").arg(target).spawn();
}

pub fn open_in_notepad(file: &Path) {
    let _ = Command::new("notepad.exe").arg(file).spawn();
}
