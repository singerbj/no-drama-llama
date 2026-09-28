//! No Drama Llama: an always-on local LLM server for Windows gaming PCs that gets out of the
//! way while you play. The platform-independent logic lives here (and is unit-tested on any
//! OS); the Windows tray app, detection probes, installer and updater live in [`win`].

pub mod catalog;
pub mod cli;
pub mod control;
pub mod detect;
pub mod gguf;
pub mod hardware;
pub mod installer;
pub mod laya;
pub mod log;
pub mod paths;
pub mod server;
pub mod settings;
pub mod setup;
pub mod state;
pub mod update;

#[cfg(windows)]
pub mod posthog;
#[cfg(windows)]
pub mod win;
