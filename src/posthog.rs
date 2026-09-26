//! Opt-in PostHog analytics for the Windows tray application.
//!
//! Nothing is sent unless the user said yes (`ShareUsageStats`, `SendCrashReports`), which
//! the tray asks once on first run. Product events, crash reports and a few purpose-written
//! log records go straight to PostHog's HTTP endpoints through the app's own `ureq` agent
//! (Windows' SChannel TLS). The application's file log is never exported, and every event
//! asks PostHog not to store a person profile or look up a location from the IP address.
//! Builds without a baked-in configuration send nothing.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How long exit waits for queued records before giving up.
const FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

struct Client {
    token: &'static str,
    host: &'static str,
    /// Where the random install id lives while usage stats are on (deleted when turned off).
    id_file: PathBuf,
    queue: Mutex<Option<Sender<Post>>>,
    done: Mutex<Option<mpsc::Receiver<()>>>,
}

struct Post {
    path: &'static str,
    body: Value,
}

/// What the user agreed to, mirrored from the settings on every tray refresh.
struct Consent {
    crash: AtomicBool,
    usage: AtomicBool,
    /// The first refresh after startup logs "tray application ready".
    started: AtomicBool,
    install_id: Mutex<Option<String>>,
    gpu: Mutex<Option<String>>,
}

static CLIENT: OnceLock<Option<Client>> = OnceLock::new();
static CONSENT: Consent = Consent {
    crash: AtomicBool::new(false),
    usage: AtomicBool::new(false),
    started: AtomicBool::new(false),
    install_id: Mutex::new(None),
    gpu: Mutex::new(None),
};

/// PostHog project token and host, baked in from the build environment like
/// `NDL_UPDATE_PUBKEY`: an installed app is started by Windows, not from a shell, so
/// it never sees these as runtime environment variables. CI passes an empty string
/// when the repository variable isn't set: that means "no analytics".
fn config() -> Option<(&'static str, &'static str)> {
    let set = |v: Option<&'static str>| v.map(str::trim).filter(|v| !v.is_empty());
    Some((
        set(option_env!("POSTHOG_PROJECT_TOKEN"))?,
        set(option_env!("POSTHOG_HOST"))?.trim_end_matches('/'),
    ))
}

/// Whether this build can send anything at all (so the tray only asks when it matters).
pub fn available() -> bool {
    config().is_some()
}

/// Starts the sender thread and the panic reporter. Sends nothing until [`set_consent`].
pub fn init(data_dir: &Path) {
    CLIENT.get_or_init(|| {
        let (token, host) = config()?;
        let (tx, rx) = mpsc::channel::<Post>();
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let agent = crate::win::net::telemetry_agent();
            for post in rx {
                send(&agent, host, token, &post);
            }
            let _ = done_tx.send(());
        });
        Some(Client {
            token,
            host,
            id_file: data_dir.join("analytics-id"),
            queue: Mutex::new(Some(tx)),
            done: Mutex::new(Some(done_rx)),
        })
    });
    if client().is_some() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            report_panic(info);
            previous(info);
        }));
    }
}

/// Applies the user's current choices. Turning usage stats off deletes the install id, so
/// turning them back on starts over as a new, unconnected install.
pub fn set_consent(crash: bool, usage: bool, gpu: Option<&str>) {
    let Some(c) = client() else { return };
    CONSENT.crash.store(crash, Ordering::Relaxed);
    if let Ok(mut g) = CONSENT.gpu.lock() {
        *g = gpu.map(str::to_owned);
    }
    let was_on = CONSENT.usage.swap(usage, Ordering::Relaxed);
    if let Ok(mut id) = CONSENT.install_id.lock() {
        if usage && id.is_none() {
            *id = Some(load_or_create_id(&c.id_file));
        } else if !usage && (was_on || c.id_file.exists()) {
            *id = None;
            let _ = std::fs::remove_file(&c.id_file);
        }
    }
    if !CONSENT.started.swap(true, Ordering::Relaxed) {
        log("tray application ready");
    }
}

fn client() -> Option<&'static Client> {
    CLIENT.get().and_then(Option::as_ref)
}

fn load_or_create_id(file: &Path) -> String {
    let valid = |s: &str| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit());
    if let Ok(id) = std::fs::read_to_string(file) {
        if valid(id.trim()) {
            return id.trim().to_owned();
        }
    }
    let id = random_id();
    let _ = std::fs::write(file, &id);
    id
}

/// 128 random bits from the OS-seeded hasher keys: not tied to the user, the PC or its hardware.
fn random_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let half = || {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(now_nanos());
        h.finish()
    };
    format!("{:016x}{:016x}", half(), half())
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn send(agent: &ureq::Agent, host: &str, token: &str, post: &Post) {
    // Best effort: analytics never surfaces errors to the user or the app log.
    let _ = agent
        .post(format!("{host}{}", post.path))
        .header("Authorization", format!("Bearer {token}"))
        .content_type("application/json")
        .send(post.body.to_string());
}

fn enqueue(path: &'static str, body: Value) {
    let Some(c) = client() else { return };
    if let Some(tx) = c.queue.lock().ok().and_then(|q| q.clone()) {
        let _ = tx.send(Post { path, body });
    }
}

fn event_body(c: &Client, event: &str, distinct_id: String, mut properties: Value) -> Value {
    properties["$process_person_profile"] = json!(false);
    properties["$geoip_disable"] = json!(true);
    properties["$lib"] = json!("no-drama-llama");
    properties["app_version"] = json!(env!("CARGO_PKG_VERSION"));
    if let Some(gpu) = CONSENT.gpu.lock().ok().and_then(|g| g.clone()) {
        properties["gpu"] = json!(gpu);
    }
    json!({
        "api_key": c.token,
        "event": event,
        "distinct_id": distinct_id,
        "properties": properties,
    })
}

/// Records a product action, if the user shares usage statistics.
pub fn capture(event: &str) {
    let Some(c) = client() else { return };
    let Some(id) = CONSENT.install_id.lock().ok().and_then(|id| id.clone()) else {
        return;
    };
    enqueue("/i/v0/e/", event_body(c, event, id, json!({})));
}

/// Sends the panic as a PostHog exception synchronously: in release builds (`panic = "abort"`)
/// the process dies as soon as the hook returns, so the sender thread wouldn't get to it.
fn report_panic(info: &std::panic::PanicHookInfo) {
    let Some(c) = client() else { return };
    if !CONSENT.crash.load(Ordering::Relaxed) {
        return;
    }
    let message = info
        .payload()
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
        .unwrap_or("panic");
    let value = match info.location() {
        Some(l) => format!("{message} at {}:{}", l.file(), l.line()),
        None => message.to_string(),
    };
    let props = json!({
        "$exception_list": [{
            "type": "panic",
            "value": scrub(&value, &identifying_values()),
            "mechanism": { "handled": false, "synthetic": false },
        }],
    });
    // Crash reports never carry the usage install id: a fresh id each time keeps them unlinked.
    let post = Post {
        path: "/i/v0/e/",
        body: event_body(c, "$exception", random_id(), props),
    };
    let agent = crate::win::net::telemetry_agent();
    send(&agent, c.host, c.token, &post);
}

/// Things a panic message could contain that identify the user or PC, with their placeholders.
fn identifying_values() -> Vec<(String, &'static str)> {
    ["USERPROFILE", "USERNAME", "COMPUTERNAME", "USERDOMAIN"]
        .iter()
        .filter_map(|var| {
            let v = std::env::var(var).ok()?;
            (v.trim().len() >= 2).then_some((v, *var))
        })
        .collect()
}

/// Replaces each identifying value (case-insensitively) with `%VAR%`, longest first so the
/// profile path goes before the user name inside it.
fn scrub(text: &str, values: &[(String, &str)]) -> String {
    let mut values: Vec<_> = values.iter().collect();
    values.sort_by_key(|(v, _)| std::cmp::Reverse(v.len()));
    let mut out = text.to_owned();
    for (value, var) in values {
        if let Ok(re) = regex::Regex::new(&format!("(?i){}", regex::escape(value))) {
            out = re.replace_all(&out, format!("%{var}%")).into_owned();
        }
    }
    out
}

/// Emits one INFO record to PostHog Logs as OTLP/HTTP JSON, if the user shares usage statistics.
fn log(message: &'static str) {
    if !CONSENT.usage.load(Ordering::Relaxed) {
        return;
    }
    let attr = |k: &str, v: &str| json!({ "key": k, "value": { "stringValue": v } });
    enqueue(
        "/i/v1/logs",
        json!({
            "resourceLogs": [{
                "resource": { "attributes": [
                    attr("service.name", "no-drama-llama"),
                    attr("service.version", env!("CARGO_PKG_VERSION")),
                ]},
                "scopeLogs": [{
                    "scope": { "name": "no-drama-llama.posthog" },
                    "logRecords": [{
                        "timeUnixNano": now_nanos().to_string(),
                        "severityNumber": 9,
                        "severityText": "INFO",
                        "body": { "stringValue": message },
                    }],
                }],
            }],
        }),
    );
}

/// Records a successfully submitted settings change without its values.
pub fn settings_saved() {
    log("settings change submitted");
}

/// Records the tray application's graceful exit and waits briefly for queued records.
pub fn shutdown() {
    let Some(c) = client() else { return };
    log("tray application exiting");
    if let Ok(mut q) = c.queue.lock() {
        q.take(); // closing the queue ends the sender thread once it's drained
    }
    if let Some(done) = c.done.lock().ok().and_then(|mut d| d.take()) {
        let _ = done.recv_timeout(FLUSH_TIMEOUT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_removes_profile_user_and_pc_names() {
        let values = vec![
            (r"C:\Users\Jane.Doe".to_string(), "USERPROFILE"),
            ("Jane.Doe".to_string(), "USERNAME"),
            ("JANES-PC".to_string(), "COMPUTERNAME"),
        ];
        let text = r"can't open c:\users\jane.doe\models\x.gguf (owner jane.doe on janes-pc)";
        assert_eq!(
            scrub(text, &values),
            r"can't open %USERPROFILE%\models\x.gguf (owner %USERNAME% on %COMPUTERNAME%)"
        );
    }

    #[test]
    fn install_id_is_random_hex_and_persists() {
        let a = random_id();
        assert_eq!(a.len(), 32);
        assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(a, random_id());

        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("analytics-id");
        let id = load_or_create_id(&file);
        assert_eq!(load_or_create_id(&file), id);
        std::fs::write(&file, "not an id").unwrap();
        assert_ne!(load_or_create_id(&file), id);
    }
}
