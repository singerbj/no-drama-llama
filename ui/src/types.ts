// Mirrors src/control.rs and src/settings.rs. Settings keep their settings.json names.

export type Tone = "running" | "loading" | "paused" | "off" | "error";
export type Reasoning = "none" | "low" | "medium" | "xhigh";
export type DetectionMode = "Both" | "Gpu" | "Launchers";
export type PopupPosition = "TopCenter" | "TopRight" | "BottomRight" | "BottomCenter";

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
        | "exit";
    }
  | { cmd: "download"; id: string }
  | { cmd: "save"; settings: Partial<Settings> };

export interface Saved {
  warnings: string[];
}
