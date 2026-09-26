//! `install` / `uninstall`: everything the PowerShell edition's install.ps1 / uninstall.ps1
//! did, plus migrating from it. Runs elevated in a console window.

use super::{laya as win_laya, models, net, probe, procs::Procs, sys};
use crate::catalog::{self, CatalogModel};
use crate::hardware::{self, Backend};
use crate::paths::{self, Paths, APP_NAME, TASK_NAME};
use crate::settings::Settings;
use crate::setup;
use crate::update::Release;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS};
use winreg::RegKey;

const OLD_TASK: &str = "Local LLM Guard";
const UNINSTALL_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NoDramaLlama";

#[derive(Debug, Default)]
pub struct InstallOptions {
    pub skip_model: bool,
    /// Catalog id to download; None = the best for this PC
    pub model: Option<String>,
    /// None = the best for this PC
    pub backend: Option<Backend>,
    pub skip_power: bool,
    pub skip_wol: bool,
    pub llama_tag: Option<String>,
    pub update_llama: bool,
    /// Turn on Laya (Ollaya) and install it now
    pub laya: bool,
}

fn step(msg: &str) {
    println!("\n== {msg}");
}

fn programs_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default())
        .join(r"Microsoft\Windows\Start Menu\Programs")
}

fn shortcut_path() -> PathBuf {
    programs_dir().join(format!("{APP_NAME}.lnk"))
}

// ------------------------------------------------------------------ stop / migrate

/// Stops the tray app (either edition), our llama-server and Ollaya.
pub fn stop_running(p: &Paths) {
    let _ = sys::run("schtasks.exe", &["/end", "/tn", TASK_NAME]);
    let _ = sys::run("schtasks.exe", &["/end", "/tn", OLD_TASK]);
    let mut procs = Procs::new();
    // The installed copy only (the tray app, any version) - never this process or the one
    // that launched it (the non-elevated copy waiting for us, or Settings > Apps).
    let me = std::process::id();
    let parent = procs.parent_of(me);
    let mut pids: Vec<u32> = procs
        .pids_of(&paths::installed_exe())
        .into_iter()
        .filter(|p| *p != me && Some(*p) != parent)
        .collect();
    pids.extend(procs.pids_with_cmdline("powershell.exe", "llm-tray.ps1"));
    pids.extend(procs.pids_with_cmdline("powershell.exe", "llm-guard.ps1"));
    procs.refresh();
    pids.extend(procs.pids_of(&p.server_exe));
    pids.extend(win_laya::pids(&procs, p));
    procs.kill(&pids);
}

fn migrate_from_powershell(p: &Paths) {
    if sys::run("schtasks.exe", &["/delete", "/tn", OLD_TASK, "/f"]).is_ok() {
        println!("Removed the PowerShell edition's logon task.");
    }
    for d in setup::migrate_legacy_layout(p) {
        println!("  {d}");
    }
    for f in setup::LEGACY_SHORTCUTS {
        let _ = std::fs::remove_file(programs_dir().join(f));
    }
}

/// C:\LLM admin-only (the elevated app runs llama-server.exe from it); models\ writable by you.
/// Folders created under C:\ let every signed-in user modify them by default.
fn set_acls(p: &Paths) -> Result<()> {
    let root = p.root.to_string_lossy().to_string();
    let me = sys::current_user_sid()?;
    sys::run(
        "icacls.exe",
        &[
            &root,
            "/grant:r",
            "*S-1-5-18:(OI)(CI)F",
            "*S-1-5-32-544:(OI)(CI)F",
            "*S-1-5-32-545:(OI)(CI)RX",
            "/Q",
        ],
    )?;
    sys::run("icacls.exe", &[&root, "/inheritance:r", "/Q"])?;
    let _ = sys::run("icacls.exe", &[&root, "/remove:g", &format!("*{me}"), "/Q"]); // left by older installs
    let _ = sys::run(
        "icacls.exe",
        &[&format!(r"{root}\*"), "/reset", "/T", "/C", "/Q"],
    );
    sys::run(
        "icacls.exe",
        &[
            &p.models_dir.to_string_lossy(),
            "/grant",
            &format!("*{me}:(OI)(CI)M"),
            "/Q",
        ],
    )?;
    Ok(())
}

// ------------------------------------------------------------------ downloads

fn print_progress(done: u64, total: u64) {
    let gb = |b: u64| b as f64 / 1e9;
    if total > 0 {
        print!(
            "\r  {:.2} / {:.2} GB ({:.0}%)   ",
            gb(done),
            gb(total),
            done as f64 * 100.0 / total as f64
        );
    } else {
        print!("\r  {:.2} GB   ", gb(done));
    }
    let _ = std::io::stdout().flush();
}

/// llama.cpp build installed in `llama\` (vulkan / cuda12 / cuda13), if known.
pub fn installed_backend(p: &Paths) -> Option<Backend> {
    std::fs::read_to_string(p.llama_dir.join("backend.txt"))
        .ok()
        .and_then(|s| Backend::parse(&s))
}

fn download_verified(asset: &crate::update::Asset, dest: &Path) -> Result<()> {
    let _ = std::fs::remove_file(dest);
    net::download(&asset.browser_download_url, dest, print_progress)?;
    println!();
    if let Some(expected) = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
    {
        let actual = net::file_sha256(dest)?;
        if !actual.eq_ignore_ascii_case(expected) {
            let _ = std::fs::remove_file(dest);
            bail!(
                "{} checksum mismatch (expected {expected}, got {actual})",
                asset.name
            );
        }
        println!("  SHA-256 verified.");
    }
    Ok(())
}

/// Installs the llama.cpp build for `backend` (falling back to Vulkan if a CUDA build is
/// missing from the release). Returns the backend actually installed.
pub fn install_llama_cpp(p: &Paths, tag: Option<&str>, backend: Backend) -> Result<Backend> {
    let api = match tag {
        Some(t) => format!("https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/{t}"),
        None => "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest".into(),
    };
    let rel = Release::parse(&net::get_text(&net::agent(), &api)?)?;
    let names: Vec<&str> = rel.assets.iter().map(|a| a.name.as_str()).collect();
    let Some((used, main, runtime)) = hardware::llama_assets_with_fallback(&names, backend) else {
        bail!(
            "llama.cpp {} has no Windows build for {} - try --llama-cpp-tag <older tag>",
            rel.tag_name,
            backend.label()
        );
    };
    if used != backend {
        println!(
            "  No {} build in this release; using {}.",
            backend.label(),
            used.label()
        );
    }
    let asset = |n: &str| rel.assets.iter().find(|a| a.name == n).unwrap();
    println!(
        "Downloading llama.cpp {} ({})...",
        rel.tag_name,
        used.label()
    );
    let zip_path = p.root.join("llama-download.zip"); // admin-only, unlike %TEMP%
    download_verified(asset(main), &zip_path)?;
    extract_llama_zip(&zip_path, &p.llama_dir)?;
    let _ = std::fs::remove_file(&zip_path);
    if let Some(rt) = runtime {
        println!("Downloading the CUDA runtime...");
        download_verified(asset(rt), &zip_path)?;
        extract_zip_into(&zip_path, &p.llama_dir)?;
        let _ = std::fs::remove_file(&zip_path);
    }
    std::fs::write(p.llama_dir.join("backend.txt"), used.as_str())?;
    Ok(used)
}

/// Extracts a zip's files flat into `dir` (the CUDA runtime DLLs go next to llama-server.exe).
pub(crate) fn extract_zip_into(zip_path: &Path, dir: &Path) -> Result<()> {
    let tmp = dir.join(".extract");
    let _ = std::fs::remove_dir_all(&tmp);
    zip::ZipArchive::new(std::fs::File::open(zip_path)?)?
        .extract(&tmp)
        .context("couldn't unpack")?;
    let mut stack = vec![tmp.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d)?.flatten() {
            if e.path().is_dir() {
                stack.push(e.path());
            } else {
                std::fs::rename(e.path(), dir.join(e.file_name()))?;
            }
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}

/// Replaces `dir` with the zip's contents, flattened so llama-server.exe sits directly in it
/// (some releases nest everything in a subfolder).
pub(crate) fn extract_llama_zip(zip_path: &Path, dir: &Path) -> Result<()> {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir)?;
    zip::ZipArchive::new(std::fs::File::open(zip_path)?)?
        .extract(dir)
        .context("couldn't unpack llama.cpp")?;
    if !dir.join("llama-server.exe").exists() {
        let nested = find_file(dir, "llama-server.exe")
            .context("llama-server.exe not found in the download")?;
        let from = nested.parent().unwrap().to_path_buf();
        for e in std::fs::read_dir(&from)?.flatten() {
            std::fs::rename(e.path(), dir.join(e.file_name()))?;
        }
    }
    Ok(())
}

pub(crate) fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let path = e.path();
        if path.is_dir() {
            if let Some(f) = find_file(&path, name) {
                return Some(f);
            }
        } else if e.file_name().eq_ignore_ascii_case(name) {
            return Some(path);
        }
    }
    None
}

/// Which model to download: an explicit choice, else nothing if the configured model is
/// already there, else the best one for this PC.
fn choose_model(p: &Paths, opts: &InstallOptions, pc: &catalog::Machine) -> Option<CatalogModel> {
    if opts.skip_model {
        return None;
    }
    if let Some(id) = &opts.model {
        return catalog::find(id);
    }
    let current = Settings::load(&p.settings).0.model;
    if p.model(&current).exists() && paths::missing_parts(&p.models_dir, &current).is_empty() {
        println!("  Keeping your model: {current}");
        return None;
    }
    let rec = catalog::recommend(pc);
    if rec.is_none() {
        println!("  No model in the catalog fits this PC. Put a smaller .gguf in {} and pick it from the tray.", p.models_dir.display());
    }
    rec
}

fn install_model(p: &Paths, m: &CatalogModel) -> Result<()> {
    println!(
        "Downloading {} ({:.1} GB). Interrupted? Run the installer again to resume.",
        m.label(),
        m.size as f64 / 1e9
    );
    models::download_model(models::HF, m, &p.models_dir, print_progress, None)?;
    println!();
    let mut s = Settings::load(&p.settings).0;
    s.model = m.primary_file();
    s.save(&p.settings)?;
    println!("  Selected {}", m.primary_file());
    Ok(())
}

/// Ollaya, when Laya is on (or `--laya` turns it on). The tray app downloads the Laya model
/// once it starts.
fn install_laya(p: &Paths, opts: &InstallOptions, nvidia: &[hardware::NvidiaGpu]) -> Result<()> {
    let mut s = Settings::load(&p.settings).0;
    if opts.laya && !s.run_laya {
        s.run_laya = true;
        s.save(&p.settings)?;
    }
    if !s.run_laya {
        return Ok(());
    }
    step("Laya (Ollaya)");
    let want_gpu = crate::laya::wants_gpu_pack(nvidia, s.laya_device);
    let record = p.ollaya_record();
    if !opts.laya && !crate::laya::needs_install(record.as_ref(), p.ollaya_exe.exists(), want_gpu) {
        println!(
            "  Keeping Ollaya {}",
            record.map(|r| r.version).unwrap_or_default()
        );
        return Ok(());
    }
    println!(
        "Downloading Ollaya{}...",
        if want_gpu {
            " and its NVIDIA GPU pack (about 1.1 GB)"
        } else {
            ""
        }
    );
    std::fs::create_dir_all(&p.ollaya_dir)?;
    let r = win_laya::install(
        p,
        &win_laya::release_api(),
        want_gpu,
        print_progress,
        &std::sync::atomic::AtomicBool::new(false),
    )?;
    println!(
        "\n  Ollaya {} installed ({}). The tray app downloads {} when it starts.",
        r.version,
        if r.gpu_pack { "NVIDIA GPU" } else { "CPU" },
        s.laya_model
    );
    Ok(())
}

// ------------------------------------------------------------------ power / Wake-on-LAN

fn ac_value(sub: &str, set: &str) -> Option<u64> {
    let out = sys::run("powercfg.exe", &["/q", "SCHEME_BALANCED", sub, set]).ok()?;
    setup::parse_ac_value(&String::from_utf8_lossy(&out.stdout))
}

const NIC_BACKUP_PS: &str = r#"
$n = Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' -and $_.MediaType -eq '802.3' }
ConvertTo-Json -Depth 5 -Compress -InputObject @($n | ForEach-Object {
  $nm = $_.Name
  @{ Name  = $nm
     Magic = [string](Get-NetAdapterPowerManagement -Name $nm -ErrorAction SilentlyContinue).WakeOnMagicPacket
     Adv   = @(Get-NetAdapterAdvancedProperty -Name $nm -ErrorAction SilentlyContinue |
               Where-Object { $_.DisplayName -match 'Wake.*Magic|Shutdown Wake' } |
               ForEach-Object { @{ DisplayName = $_.DisplayName; DisplayValue = $_.DisplayValue } }) } })
"#;

const WOL_ENABLE_PS: &str = r#"
foreach ($nic in Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' -and $_.MediaType -eq '802.3' }) {
  Set-NetAdapterPowerManagement -Name $nic.Name -WakeOnMagicPacket Enabled -ErrorAction SilentlyContinue
  Get-NetAdapterAdvancedProperty -Name $nic.Name -ErrorAction SilentlyContinue |
    Where-Object { $_.DisplayName -match 'Wake.*Magic|Shutdown Wake' } |
    ForEach-Object { Set-NetAdapterAdvancedProperty -Name $nic.Name -DisplayName $_.DisplayName -DisplayValue 'Enabled' -ErrorAction SilentlyContinue }
  "  $($nic.Name)  MAC for your WoL app: $($nic.MacAddress)"
}
"#;

fn write_backup(p: &Paths) -> Result<()> {
    let out = sys::run("powercfg.exe", &["/getactivescheme"])?;
    let active = setup::parse_active_scheme(&String::from_utf8_lossy(&out.stdout));
    let hibernate: Option<u32> = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Control\Power")
        .and_then(|k| k.get_value("HibernateEnabled"))
        .ok();
    let values: Vec<Option<u64>> = setup::POWER_SETTINGS
        .iter()
        .map(|(sub, set)| ac_value(sub, set))
        .collect();
    let nics: Value = sys::powershell(NIC_BACKUP_PS)
        .ok()
        .and_then(|t| serde_json::from_str(t.trim()).ok())
        .unwrap_or(Value::Null);
    let backup = setup::backup_json(active, hibernate, &values, nics);
    std::fs::write(&p.backup, serde_json::to_string_pretty(&backup)?)?;
    Ok(())
}

fn apply_power() -> Result<()> {
    for c in setup::power_commands() {
        let args: Vec<&str> = c.iter().map(String::as_str).collect();
        sys::run("powercfg.exe", &args)?;
    }
    Ok(())
}

fn restore_backup(p: &Paths) -> Result<()> {
    let b: Value =
        serde_json::from_str(std::fs::read_to_string(&p.backup)?.trim_start_matches('\u{feff}'))?;
    for c in setup::restore_commands(&b) {
        let args: Vec<&str> = c.iter().map(String::as_str).collect();
        let _ = sys::run("powercfg.exe", &args);
    }
    sys::powershell(&setup::nic_restore_ps(&p.backup))?;
    Ok(())
}

// ------------------------------------------------------------------ task / shortcut / registry

fn register_task(p: &Paths, exe: &Path, at_logon: bool) -> Result<()> {
    let xml = setup::task_xml(exe, &sys::current_user_sid()?, at_logon);
    let file = p.root.join("task.xml");
    std::fs::write(&file, setup::utf16le_with_bom(&xml))?;
    let r = sys::run(
        "schtasks.exe",
        &[
            "/create",
            "/tn",
            TASK_NAME,
            "/xml",
            &file.to_string_lossy(),
            "/f",
        ],
    );
    let _ = std::fs::remove_file(&file);
    r.map(|_| ())
}

pub fn task_exists() -> bool {
    sys::run("schtasks.exe", &["/query", "/tn", TASK_NAME]).is_ok()
}

/// Whether the app starts when you sign in; `None` = the logon task doesn't exist (not
/// installed). Uses PowerShell because the State enum name is the same in every display
/// language, unlike schtasks' text output.
pub fn logon_start() -> Option<bool> {
    sys::powershell(&setup::logon_start_query_ps(TASK_NAME))
        .ok()
        .and_then(|out| setup::parse_logon_start(&out))
}

/// Turns the logon task's trigger on or off (the task stays, for the Start menu entry).
pub fn set_logon_start(on: bool) -> Result<()> {
    sys::powershell(&setup::set_logon_start_ps(TASK_NAME, on)).map(|_| ())
}

/// settings.json from before `StartWithWindows` existed: the tray's old toggle disabled the
/// whole task, so carry over what it says rather than turning start-at-logon back on.
pub fn adopt_start_with_windows(p: &Paths) {
    let Ok(text) = std::fs::read_to_string(&p.settings) else {
        return;
    };
    if Settings::json_has_key(&text, "StartWithWindows") {
        return;
    }
    if let Some(on) = logon_start() {
        let (mut s, _) = Settings::from_json(&text);
        s.start_with_windows = on;
        if s.save(&p.settings).is_ok() {
            crate::log!("StartWithWindows set to {on} from the logon task");
        }
    }
}

pub fn run_task() -> Result<()> {
    sys::run("schtasks.exe", &["/run", "/tn", TASK_NAME]).map(|_| ())
}

fn create_shortcut(target: &Path, lnk: &Path) -> Result<()> {
    use windows::core::{Interface, HSTRING};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        link.SetPath(&HSTRING::from(target.as_os_str()))?;
        link.SetDescription(&HSTRING::from("Local LLM that pauses while you play"))?;
        link.SetIconLocation(&HSTRING::from(target.as_os_str()), 0)?;
        link.cast::<IPersistFile>()?
            .Save(&HSTRING::from(lnk.as_os_str()), true)?;
    }
    Ok(())
}

pub fn set_registered_version(v: &semver::Version) {
    if let Ok(k) =
        RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(UNINSTALL_KEY, KEY_ALL_ACCESS)
    {
        let _ = k.set_value("DisplayVersion", &v.to_string());
    }
}

fn register_uninstaller(exe: &Path) -> Result<()> {
    let (k, _) = RegKey::predef(HKEY_LOCAL_MACHINE).create_subkey(UNINSTALL_KEY)?;
    let exe_s = exe.to_string_lossy().to_string();
    k.set_value("DisplayName", &APP_NAME)?;
    k.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"))?;
    k.set_value("Publisher", &APP_NAME)?;
    k.set_value("DisplayIcon", &exe_s)?;
    k.set_value(
        "InstallLocation",
        &paths::install_dir().to_string_lossy().to_string(),
    )?;
    k.set_value("UninstallString", &format!("\"{exe_s}\" uninstall"))?;
    k.set_value(
        "URLInfoAbout",
        &format!("https://github.com/{}", paths::REPO),
    )?;
    k.set_value("NoModify", &1u32)?;
    k.set_value("NoRepair", &1u32)?;
    Ok(())
}

// ------------------------------------------------------------------ install / uninstall

pub fn install(opts: &InstallOptions) -> Result<()> {
    println!("{APP_NAME} {} - install", env!("CARGO_PKG_VERSION"));
    let p = Paths::system();

    step("Stopping any running copy");
    stop_running(&p);

    step("Installing the app");
    let dir = paths::install_dir();
    std::fs::create_dir_all(&dir)?;
    let exe = paths::installed_exe();
    let me = std::env::current_exe()?;
    if !me.as_os_str().eq_ignore_ascii_case(exe.as_os_str()) {
        std::fs::copy(&me, &exe).with_context(|| format!("couldn't copy to {}", exe.display()))?;
    }
    println!("  {}", exe.display());

    step("Setting up C:\\LLM");
    for d in [&p.root, &p.llama_dir, &p.models_dir, &p.data_dir] {
        std::fs::create_dir_all(d)?;
    }
    migrate_from_powershell(&p);
    set_acls(&p)?;

    step("Hardware");
    let nvidia = probe::nvidia_gpus();
    let backend = opts
        .backend
        .unwrap_or_else(|| hardware::choose_backend(&nvidia));
    for g in &nvidia {
        println!(
            "  {} - driver {}, compute {}",
            g.name, g.driver_major, g.compute_cap
        );
    }
    println!("  llama.cpp build: {}", backend.label());

    if opts.update_llama
        || !p.server_exe.exists()
        || installed_backend(&p).is_some_and(|b| b != backend)
    {
        step("llama.cpp");
        install_llama_cpp(&p, opts.llama_tag.as_deref(), backend)?;
    }
    let (pc, gpu) = probe::machine(&p.server_exe);
    println!(
        "  GPU: {} · RAM: {:.0} GB",
        gpu.as_deref()
            .unwrap_or("none found (the model will run on the CPU)"),
        hardware::gib(pc.ram)
    );
    if let Some(m) = choose_model(&p, opts, &pc) {
        step("Model");
        install_model(&p, &m)?;
    }
    install_laya(&p, opts, &nvidia)?;

    // Only on the first run, so re-running install never overwrites the original values.
    if !p.backup.exists() {
        step("Backing up your power and network settings");
        write_backup(&p)?;
    }
    if !opts.skip_power {
        step("Low-power, always-on power settings");
        apply_power()?;
    }
    if !opts.skip_wol {
        step("Wake-on-LAN on wired adapters");
        match sys::powershell(WOL_ENABLE_PS) {
            Ok(out) => print!("{out}"),
            Err(e) => println!("  skipped: {e}"),
        }
    }

    step("Start at logon, Start menu entry, Apps & features entry");
    adopt_start_with_windows(&p);
    let at_logon = Settings::load(&p.settings).0.start_with_windows;
    register_task(&p, &exe, at_logon)?;
    create_shortcut(&exe, &shortcut_path())?;
    register_uninstaller(&exe)?;
    run_task()?;

    let s = Settings::load(&p.settings).0;
    println!(
        "\nDone. Look for the dot in the system tray (click ^ if hidden, drag it onto the taskbar to pin).\n\
         Chat + OpenAI-compatible API: http://127.0.0.1:{}   Hotkey: Ctrl+Alt+L   Log: {}\n\
         Uninstall from Settings > Apps, or: \"{}\" uninstall",
        s.port,
        p.log.display(),
        exe.display()
    );
    if s.run_laya {
        println!("Laya (Ollaya API): {}", crate::laya::base_url(&s));
    }
    Ok(())
}

/// The running exe can't delete itself: have cmd remove the install folder a few seconds after
/// we exit. Call right before exiting.
pub fn delete_install_dir_after_exit() {
    let dir = paths::install_dir();
    if std::env::current_exe().is_ok_and(|e| e.starts_with(&dir)) && dir.exists() {
        let _ = std::process::Command::new("cmd.exe")
            .raw_arg(format!(
                "/c ping -n 4 127.0.0.1 >nul & rmdir /s /q \"{}\"",
                dir.display()
            ))
            .creation_flags(sys::CREATE_NO_WINDOW | sys::DETACHED_PROCESS)
            .spawn();
    }
}

fn prompt(q: &str, default_yes: bool) -> bool {
    print!("{q} {} ", if default_yes { "(Y/n)" } else { "(y/N)" });
    let _ = std::io::stdout().flush();
    let mut a = String::new();
    if std::io::stdin().read_line(&mut a).is_err() {
        return default_yes;
    }
    match a.trim().chars().next() {
        Some('y' | 'Y') => true,
        Some('n' | 'N') => false,
        _ => default_yes,
    }
}

pub fn uninstall(keep_models: bool, yes: bool) -> Result<()> {
    println!("{APP_NAME} {} - uninstall", env!("CARGO_PKG_VERSION"));
    if !yes && !prompt(
        "Remove No Drama Llama, llama.cpp, your settings and (unless --keep-models) the models?",
        false,
    ) {
        println!("Cancelled.");
        return Ok(());
    }
    let p = Paths::system();

    step("Stopping the app and the model server");
    stop_running(&p);
    let _ = sys::run("schtasks.exe", &["/delete", "/tn", TASK_NAME, "/f"]);
    let _ = std::fs::remove_file(shortcut_path());
    migrate_from_powershell(&p); // also removes the PowerShell edition's task and shortcuts

    step("Restoring your power and network settings");
    if p.backup.exists() {
        if let Err(e) = restore_backup(&p) {
            println!("  some settings couldn't be restored: {e}");
        }
    } else {
        println!("  No backup found; power settings left as they are (Settings > System > Power).");
    }

    let wl = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon";
    if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(wl, KEY_ALL_ACCESS) {
        if k.get_value::<String, _>("AutoAdminLogon")
            .is_ok_and(|v| v == "1")
            && (yes || prompt("Automatic sign-in is on. Turn it off?", true))
        {
            let _ = k.set_value("AutoAdminLogon", &"0");
            let _ = k.delete_value("DefaultPassword");
            println!("  Auto sign-in disabled. If you used Sysinternals Autologon, also click 'Disable' in it to erase the stored password.");
        }
    }

    step("Removing files");
    if keep_models {
        let dest =
            PathBuf::from(std::env::var_os("USERPROFILE").unwrap_or_default()).join("Downloads");
        for (m, _) in p.list_models() {
            if std::fs::rename(p.model(&m), dest.join(&m)).is_ok() {
                println!("  Model kept: {}", dest.join(&m).display());
            }
        }
    }
    if p.root.exists() {
        std::fs::remove_dir_all(&p.root)
            .with_context(|| format!("couldn't delete {}", p.root.display()))?;
        println!("  Deleted {}", p.root.display());
    }
    let _ = RegKey::predef(HKEY_LOCAL_MACHINE).delete_subkey_all(UNINSTALL_KEY);
    let dir = paths::install_dir();
    if !std::env::current_exe()?.starts_with(&dir) {
        let _ = std::fs::remove_dir_all(&dir);
    } // else: delete_install_dir_after_exit(), called as the very last thing
    println!(
        "\nDone. Windows-side changes are reverted. Undo these BIOS settings by hand if you changed them:\n  \
         - Restore on AC Power Loss  -> Power Off (or Last State)\n  - ErP / EuP -> Enabled\n  \
         - Wake on LAN / Power On by PCI-E -> Disabled"
    );
    Ok(())
}
