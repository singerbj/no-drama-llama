// Mirrors src/control.rs and src/settings.rs. Settings keep their settings.json names.

export type Tone = "running" | "loading" | "paused" | "off" | "error";
export type Reasoning = "none" | "low" | "medium" | "xhigh";
export type DetectionMode = "Both" | "Gpu" | "Launchers";
export type PopupPosition = "TopCenter" | "TopRight" | "BottomRight" | "BottomCenter";
export type LayaDevice = "auto" | "cpu" | "cuda";

export interface Settings {
  Model: string;
  Reasoning: Reasoning;
  /** "auto" = the largest that fits (llama.cpp --fit) */
  Context: number | "auto";
  ListenHost: "127.0.0.1" | "0.0.0.0";
  Port: number;
  ApiKey: string;
  PauseWhileGaming: boolean;
  DetectEmulators: boolean;
  UseWindowsGameList: boolean;
  Popups: boolean;
  PopupPosition: PopupPosition;
  ExtraGames: string[];
  DetectionMode: DetectionMode;
  GpuVramGB: number;
  GpuLoadPct: number;
  ResumeAfterSec: number;
  GpuIgnore: string[];
  AutoUpdate: boolean;
  StartWithWindows: boolean;
  RunLaya: boolean;
  /** An Ollaya model name: laya, laya:en, laya:multilingual, ... */
  LayaModel: string;
  LayaPort: number;
  LayaDevice: LayaDevice;
  /** Ollaya keep_alive: "-1" = always loaded, "5m", "0" = unload after each request */
  LayaKeepAlive: string;
  /** Opt-in: scrubbed crash reports to PostHog */
  SendCrashReports: boolean;
  /** Opt-in: anonymous usage events to PostHog */
  ShareUsageStats: boolean;
  /** The first-run privacy questions were answered */
  PrivacyAsked: boolean;
}

export type SettingKey = keyof Settings;
export type SettingValue = Settings[SettingKey];

export type UpdateView =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "available"; version: string }
  | { state: "installing"; version: string };

export interface InstalledModel {
  name: string;
  size: number;
}

export interface CatalogEntry {
  id: string;
  label: string;
  file: string;
  size: number;
  fit: "gpu" | "gpuAndRam" | "tooBig";
  note: string;
  recommended: boolean;
  installed: boolean;
  canDownload: boolean;
}

export interface DownloadView {
  id: string;
  label: string;
  done: number;
  total: number;
}

export interface LayaView {
  tone: Tone;
  statusText: string;
  ready: boolean;
  url: string;
  version: string | null;
  update: string | null;
  checking: boolean;
  job: DownloadView | null;
}

export interface View {
  version: string;
  tone: Tone;
  statusText: string;
  off: boolean;
  running: boolean;
  canRestart: boolean;
  gameProcess: string | null;
  settings: Settings;
  chatUrl: string;
  gpuName: string | null;
  backend: string | null;
  nCtx: number | null;
  update: UpdateView;
  models: InstalledModel[];
  catalog: CatalogEntry[];
  download: DownloadView | null;
  laya: LayaView;
}

export type Request =
  | {
      cmd:
        | "toggle"
        | "restart"
        | "open_chat"
        | "ignore_current_game"
        | "gpu_report"
        | "libraries"
        | "test_popup"
        | "check_for_updates"
        | "cancel_download"
        | "open_folder"
        | "open_models"
        | "view_log"
        | "edit_settings_file"
        | "exit"
        | "restart_laya"
        | "update_laya"
        | "view_laya_log";
    }
  | { cmd: "download"; id: string }
  | { cmd: "save"; settings: Partial<Settings> };

export interface Saved {
  warnings: string[];
}
