//! Small Win32 helpers: elevation, consoles, message boxes, single instance, processes.

use anyhow::{bail, Context, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_ALREADY_EXISTS, HANDLE, HLOCAL, WAIT_ABANDONED,
    WAIT_OBJECT_0,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
    GetNamedSecurityInfoW, SetNamedSecurityInfoW, SDDL_REVISION_1, SE_FILE_OBJECT,
};
use windows::Win32::Security::{
    GetSecurityDescriptorDacl, GetTokenInformation, TokenElevation, TokenUser,
    DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES, TOKEN_ELEVATION, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, ReleaseMutex,
    WaitForSingleObject, INFINITE,
};
use windows::Win32::UI::Shell::{
    FOLDERID_ProgramFiles, SHGetKnownFolderPath, ShellExecuteExW, KF_FLAG_DEFAULT,
    SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK,
    MB_YESNO, SW_SHOWNORMAL,
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

/// SYSTEM, Administrators and TrustedInstaller: the owners a folder the elevated app runs
/// programs from may have. Any other owner can change the folder's permissions at will.
pub const TRUSTED_OWNER_SIDS: &[&str] = &[
    "S-1-5-18",
    "S-1-5-32-544",
    "S-1-5-80-956008885-3418522649-1831038044-1850803918-3017456440",
];

/// Whether `path` itself is a symlink or junction (without following it).
pub fn is_reparse_point(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    std::fs::symlink_metadata(path)
        .is_ok_and(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
}

/// Whether `path` or any folder above it is a symlink or junction. For files the elevated app
/// creates or deletes in the user's own folders, where a link could send it elsewhere.
pub fn path_has_link(path: &Path) -> bool {
    path.ancestors().any(is_reparse_point)
}

/// SID of `path`'s owner (e.g. `S-1-5-32-544`).
pub fn owner_sid(path: &Path) -> Result<String> {
    let name = wide(path);
    unsafe {
        let mut owner = PSID::default();
        let mut sd = PSECURITY_DESCRIPTOR::default();
        GetNamedSecurityInfoW(
            PCWSTR(name.as_ptr()),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            Some(&mut owner),
            None,
            None,
            None,
            &mut sd,
        )
        .ok()
        .with_context(|| format!("couldn't read the owner of {}", path.display()))?;
        let mut s = PWSTR::null();
        let r = ConvertSidToStringSidW(owner, &mut s);
        let out = r.and_then(|_| s.to_string().map_err(Into::into));
        if !s.is_null() {
            let _ = LocalFree(Some(HLOCAL(s.0 as *mut _)));
        }
        let _ = LocalFree(Some(HLOCAL(sd.0)));
        Ok(out?)
    }
}

/// A security descriptor parsed from SDDL, freed on drop.
struct Sddl(PSECURITY_DESCRIPTOR);

impl Sddl {
    fn parse(sddl: &str) -> Result<Sddl> {
        let w = wide(sddl);
        let mut sd = PSECURITY_DESCRIPTOR::default();
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(w.as_ptr()),
                SDDL_REVISION_1,
                &mut sd,
                None,
            )?;
        }
        Ok(Sddl(sd))
    }
}

impl Drop for Sddl {
    fn drop(&mut self) {
        unsafe {
            let _ = LocalFree(Some(HLOCAL(self.0 .0)));
        }
    }
}

/// Creates the folder `path` with the permissions in `sddl` from the start, so there is no
/// moment when it has the parent's (for `C:\` folders: every user may modify it).
pub fn create_dir_with_sddl(path: &Path, sddl: &str) -> Result<()> {
    let sd = Sddl::parse(sddl)?;
    let sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0 .0,
        bInheritHandle: false.into(),
    };
    let name = wide(path);
    unsafe { CreateDirectoryW(PCWSTR(name.as_ptr()), Some(&sa)) }
        .with_context(|| format!("couldn't create {}", path.display()))
}

/// Replaces `path`'s whole DACL with the one in `sddl`, in one step, and stops it inheriting
/// from its parent. ACEs anyone else added are gone afterwards.
pub fn set_dacl(path: &Path, sddl: &str) -> Result<()> {
    let sd = Sddl::parse(sddl)?;
    let name = wide(path);
    unsafe {
        let mut present = windows::core::BOOL::default();
        let mut defaulted = windows::core::BOOL::default();
        let mut dacl = std::ptr::null_mut();
        GetSecurityDescriptorDacl(sd.0, &mut present, &mut dacl, &mut defaulted)?;
        SetNamedSecurityInfoW(
            PCWSTR(name.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(dacl),
            None,
        )
        .ok()
        .with_context(|| format!("couldn't set the permissions of {}", path.display()))
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
/// Returns true if a new console window was opened (so the caller should pause before exit).
pub fn console(alloc: bool) -> bool {
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS).is_err() && alloc && AllocConsole().is_ok() }
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

/// A yes/no question asking for consent: No is the default button.
pub fn consent(text: &str) -> bool {
    let t = wide(text);
    let c = wide(crate::paths::APP_NAME);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(t.as_ptr()),
            PCWSTR(c.as_ptr()),
            MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
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

/// `C:\Windows\System32`, from the API: %SystemRoot% comes from the environment, which the
/// signed-in user can change for the elevated app.
pub fn system32() -> PathBuf {
    let mut buf = [0u16; 260];
    let n = unsafe { GetSystemDirectoryW(Some(&mut buf)) } as usize;
    if n == 0 || n > buf.len() {
        return PathBuf::from(r"C:\Windows\System32");
    }
    PathBuf::from(String::from_utf16_lossy(&buf[..n]))
}

/// `C:\Windows`
pub fn windows_dir() -> PathBuf {
    let s32 = system32();
    s32.parent().map(Path::to_path_buf).unwrap_or(s32)
}

/// `C:\Program Files` (64-bit), from the known-folder API rather than %ProgramFiles%.
pub fn program_files() -> PathBuf {
    unsafe {
        match SHGetKnownFolderPath(&FOLDERID_ProgramFiles, KF_FLAG_DEFAULT, None) {
            Ok(p) => {
                let out = p.to_string().ok().map(PathBuf::from);
                CoTaskMemFree(Some(p.0 as *const _));
                out.unwrap_or_else(|| r"C:\Program Files".into())
            }
            Err(_) => r"C:\Program Files".into(),
        }
    }
}

/// Where a Windows tool given by bare name (`schtasks.exe`) lives. Always an absolute path:
/// a bare name is looked up next to this exe first, and the installer usually runs from
/// Downloads, where any program can drop a fake `schtasks.exe` to be run elevated.
pub fn system_tool(name: &str) -> PathBuf {
    match name.to_ascii_lowercase().as_str() {
        "powershell.exe" => system32().join(r"WindowsPowerShell\v1.0\powershell.exe"),
        "explorer.exe" => windows_dir().join("explorer.exe"),
        _ => system32().join(name),
    }
}

/// Environment variables that make programs load code or change what they run. The elevated
/// app inherits the signed-in user's environment (HKCU\Environment), which unelevated
/// programs can write, so these would otherwise reach elevated children: .NET profilers in
/// PowerShell (COR_PROFILER), WebView2's browser folder and arguments, llama.cpp and Ollaya
/// options. Matched as case-insensitive prefixes.
const UNSAFE_ENV_PREFIXES: &[&str] = &[
    "COR_",
    "CORECLR_",
    "COMPLUS_",
    "DOTNET_",
    "WEBVIEW2_",
    "LLAMA_",
    "GGML_",
    "OLLAYA_",
    "PSEXECUTIONPOLICYPREFERENCE",
    "PSMODULEPATH",
];

/// Drops [`UNSAFE_ENV_PREFIXES`] from this process's environment and rebuilds PATH and
/// PSModulePath from admin-only locations, so every child starts from a clean environment.
/// Call first thing in `main`, before any thread starts.
pub fn scrub_environment() {
    for (k, _) in std::env::vars_os() {
        let upper = k.to_string_lossy().to_ascii_uppercase();
        if UNSAFE_ENV_PREFIXES.iter().any(|p| upper.starts_with(p)) {
            std::env::remove_var(&k);
        }
    }
    let win = windows_dir();
    let s32 = system32();
    std::env::set_var("SystemRoot", &win);
    std::env::set_var("windir", &win);
    std::env::set_var(
        "PSModulePath",
        format!(
            r"{};{}",
            program_files().join(r"WindowsPowerShell\Modules").display(),
            s32.join(r"WindowsPowerShell\v1.0\Modules").display()
        ),
    );
    std::env::set_var("PATH", machine_path(&win, &s32));
}

/// The machine-wide PATH (admin-only in the registry), without the user's own entries.
fn machine_path(win: &Path, s32: &Path) -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let base = [
        s32.display().to_string(),
        win.display().to_string(),
        s32.join("Wbem").display().to_string(),
        s32.join(r"WindowsPowerShell\v1.0").display().to_string(),
    ];
    let raw: String = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
        .and_then(|k| k.get_value("Path"))
        .unwrap_or_default();
    let mut out: Vec<String> = base.to_vec();
    for e in raw.split(';').map(str::trim).filter(|e| !e.is_empty()) {
        let e = replace_ci(
            &replace_ci(e, "%SystemRoot%", &base[1]),
            "%windir%",
            &base[1],
        );
        if !e.contains('%') && !out.iter().any(|o| o.eq_ignore_ascii_case(&e)) {
            out.push(e);
        }
    }
    out.join(";")
}

fn replace_ci(s: &str, from: &str, to: &str) -> String {
    match s.to_ascii_lowercase().find(&from.to_ascii_lowercase()) {
        Some(i) => format!("{}{to}{}", &s[..i], &s[i + from.len()..]),
        None => s.to_string(),
    }
}

/// A command that runs without flashing a console window. A bare program name is resolved
/// to the Windows tool of that name ([`system_tool`]).
pub fn hidden(program: impl AsRef<OsStr>) -> Command {
    let program = program.as_ref();
    let mut c = if Path::new(program).components().count() == 1 {
        Command::new(system_tool(&program.to_string_lossy()))
    } else {
        Command::new(program)
    };
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

/// Runs a program, killing it if it takes longer than `timeout`. Returns stdout even when the
/// exit code is non-zero (some tools print what we need and then fail).
pub fn run_with_timeout(
    program: &Path,
    args: &[&str],
    timeout: std::time::Duration,
) -> Result<String> {
    use std::io::Read;
    let mut child = hidden(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("couldn't run {}", program.display()))?;
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        let mut e = String::new();
        let _ = err.read_to_string(&mut e);
        s + &e
    });
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("{} timed out", program.display());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(reader.join().unwrap_or_default())
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

/// Bytes this user can still write on the drive holding `path` (or its nearest existing parent).
pub fn disk_free(path: &Path) -> Option<u64> {
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let dir = path.ancestors().find(|d| d.exists())?;
    let mut free = 0u64;
    let name = wide(dir);
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(name.as_ptr()),
            Some(&mut free as *mut u64),
            None,
            None,
        )
    }
    .ok()?;
    Some(free)
}

/// Windows' build number (19045 = Windows 10 22H2, 22000+ = Windows 11).
pub fn windows_build() -> Option<u32> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .and_then(|k| k.get_value::<String, _>("CurrentBuildNumber"))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// True if the PC has a battery (a laptop, or a desktop on a UPS that reports as one).
pub fn has_battery() -> bool {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut s = SYSTEM_POWER_STATUS::default();
    // 128: no system battery; 255: unknown
    unsafe { GetSystemPowerStatus(&mut s) }.is_ok() && s.BatteryFlag != 128 && s.BatteryFlag != 255
}

/// Opens a URL, file or folder with its default app, *not* elevated (explorer hands it off).
pub fn open_unelevated(target: &Path) {
    let _ = Command::new(system_tool("explorer.exe"))
        .arg(target)
        .spawn();
}

pub fn open_in_notepad(file: &Path) {
    let _ = Command::new(system_tool("notepad.exe")).arg(file).spawn();
}
