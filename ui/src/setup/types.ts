// Mirrors src/installer.rs (and the parts of src/control.rs it reuses).

import type { CatalogEntry, InstalledModel } from "../types";

export type Mode = "install" | "uninstall";
export type Backend = "vulkan" | "cuda12" | "cuda13";
export type CheckStatus = "pass" | "warn" | "fail";

export interface Check {
  id: string;
  title: string;
  status: CheckStatus;
  detail: string;
}

export interface BackendOption {
  id: Backend;
  label: string;
  available: boolean;
  /** Bytes, when the llama.cpp release could be read */
  download: number | null;
  note: string;
}

export interface Existing {
  version: string | null;
  /** The configured model, if it's on disk */
  model: string | null;
  models: InstalledModel[];
}

export type ModelChoice = { kind: "keep" } | { kind: "none" } | { kind: "download"; id: string };

export interface Plan {
  model: ModelChoice;
  backend: Backend;
  power: boolean;
  wakeOnLan: boolean;
  startWithWindows: boolean;
  laya: boolean;
  crashReports: boolean;
  usageStats: boolean;
}

export interface Survey {
  version: string;
  existing: Existing | null;
  gpu: string | null;
  vram: number;
  ram: number;
  laptop: boolean;
  backends: BackendOption[];
  recommendedBackend: Backend;
  llamaTag: string | null;
  llamaInstalled: Backend | null;
  layaInstalled: boolean;
  catalog: CatalogEntry[];
  drive: string;
  diskFree: number | null;
  checks: Check[];
  defaults: Plan;
  askPrivacy: boolean;
  installDir: string;
  dataDir: string;
}

export type StepId =
  | "stop"
  | "app"
  | "folders"
  | "hardware"
  | "llamaCpp"
  | "model"
  | "laya"
  | "settings"
  | "register"
  | "start"
  | "restore"
  | "files";

export interface StepInfo {
  id: StepId;
  label: string;
}

export interface Sizes {
  download: number;
  disk: number;
}

export interface Review {
  problems: string[];
  steps: StepInfo[];
  sizes: Sizes;
}

export interface Finished {
  ok: boolean;
  cancelled: boolean;
  error: string | null;
  chatUrl: string | null;
  logFile: string | null;
}

export type Progress =
  | { type: "step"; id: StepId }
  | { type: "log"; text: string }
  | { type: "bytes"; done: number; total: number }
  | ({ type: "finished" } & Finished);

export interface UninstallSurvey {
  version: string | null;
  models: InstalledModel[];
  autoSignIn: boolean;
  dataDir: string;
  downloadsDir: string;
}

export interface UninstallPlan {
  keepModels: boolean;
  disableAutoSignIn: boolean;
}
