// Sample tray state for `npm run dev` in a browser. Never part of the build Tauri serves.

import type { Api } from "./api";
import type { Request, Saved, Settings, View } from "./types";

const GB = 1e9;

const settings: Settings = {
  Model: "Qwen3.8-27B-UD-Q4_K_XL.gguf",
  Reasoning: "low",
  Context: "auto",
  ListenHost: "127.0.0.1",
  Port: 8080,
  ApiKey: "",
  PauseWhileGaming: true,
  DetectEmulators: true,
  UseWindowsGameList: true,
  Popups: true,
  PopupPosition: "TopCenter",
  ExtraGames: [],
  DetectionMode: "Both",
  GpuVramGB: 1.5,
  GpuLoadPct: 30,
  ResumeAfterSec: 60,
  GpuIgnore: ["obs64"],
  AutoUpdate: true,
  StartWithWindows: true,
  RunLaya: false,
  LayaModel: "laya",
  LayaPort: 11435,
  LayaDevice: "auto",
  LayaKeepAlive: "-1",
  SendCrashReports: false,
  ShareUsageStats: false,
  PrivacyAsked: true,
};

export function mockApi(): Api {
  let onState: ((v: View) => void) | null = null;
  let onSaved: ((s: Saved) => void) | null = null;
  const view: View = {
    version: "2.1.0",
    tone: "running",
    statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context",
    off: false,
    running: true,
    canRestart: true,
    gameProcess: null,
    settings: structuredClone(settings),
    chatUrl: "http://127.0.0.1:8080",
    gpuName: "NVIDIA GeForce RTX 4090 (24 GB)",
    backend: "CUDA 13",
    nCtx: 65536,
    update: { state: "idle" },
    models: [
      { name: "Qwen3.8-27B-UD-Q4_K_XL.gguf", size: 17.6 * GB },
      { name: "my-own-model.Q5_K_M.gguf", size: 9.1 * GB },
    ],
    catalog: [
      ["qwen3.8-27b:UD-Q6_K_XL", "Qwen 3.8 27B UD-Q6_K_XL", 24.1, "gpuAndRam", "partly in RAM - slow", false, false],
      ["qwen3.8-27b:UD-Q5_K_XL", "Qwen 3.8 27B UD-Q5_K_XL", 20.4, "gpu", "fits your GPU", true, false],
      ["qwen3.8-27b:UD-Q4_K_XL", "Qwen 3.8 27B UD-Q4_K_XL", 17.6, "gpu", "fits your GPU", false, true],
      [
        "qwen3.8-flash-next:UD-Q4_K_XL",
        "Qwen 3.8 Flash-Next UD-Q4_K_XL",
        92.3,
        "tooBig",
        "too big for this PC",
        false,
        false,
      ],
    ].map(([id, family, gb, fit, note, recommended, installed]) => ({
      id: id as string,
      label: `${family}  (${gb} GB)`,
      file: `${id}.gguf`,
      size: (gb as number) * GB,
      fit: fit as "gpu",
      note: note as string,
      recommended: recommended as boolean,
      installed: installed as boolean,
      canDownload: !installed && fit !== "tooBig",
    })),
    download: null,
    laya: {
      tone: "off",
      statusText: "Not running (turn it on in settings)",
      ready: false,
      url: "http://127.0.0.1:11435",
      version: null,
      update: null,
      checking: false,
      job: null,
    },
  };
  const push = () => setTimeout(() => onState?.(structuredClone(view)), 50);
  return {
    async send(r: Request) {
      console.info("request", JSON.stringify(r));
      switch (r.cmd) {
        case "toggle":
          view.off = !view.off;
          view.running = !view.off;
          view.tone = view.off ? "off" : "running";
          view.statusText = view.off ? "Off" : "Running - Qwen3.8-27B-UD-Q4_K_XL";
          break;
        case "save": {
          const warnings =
            r.settings.Port !== undefined && r.settings.Port < 1024 ? ["invalid Port - not changed"] : [];
          Object.assign(view.settings, r.settings);
          if (r.settings.RunLaya) {
            view.laya = {
              ...view.laya,
              tone: "loading",
              statusText: `Downloading ${view.settings.LayaModel}...`,
              version: "0.5.0",
              job: { id: "laya-download", label: view.settings.LayaModel, done: 0.4e9, total: 1e9 },
            };
          }
          setTimeout(() => onSaved?.({ warnings }), 30);
          break;
        }
        case "check_for_updates":
          view.update = { state: "available", version: "2.2.0" };
          break;
        case "update_laya":
          view.laya = { ...view.laya, update: "0.6.0" };
          break;
        case "download": {
          const m = view.catalog.find((c) => c.id === r.id)!;
          view.download = { id: m.id, label: m.label, done: 0.37 * m.size, total: m.size };
          view.catalog.forEach((c) => (c.canDownload = false));
          break;
        }
        case "cancel_download":
          view.download = null;
          break;
      }
      push();
    },
    state: async () => structuredClone(view),
    onState: (f) => (onState = f),
    onSaved: (f) => (onSaved = f),
  };
}
