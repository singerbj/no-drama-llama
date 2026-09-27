// The page's link to the installer. In Tauri: commands and events from src/win/setup_app.rs.
// In a plain browser (`npm run dev`, then open /setup.html, or /setup.html?mode=uninstall): a
// simulated PC and install.

import type { Mode, Plan, Progress, Review, Survey, UninstallPlan, UninstallSurvey } from "./types";

export interface SetupApi {
  mode(): Promise<Mode>;
  survey(): Promise<Survey>;
  uninstallSurvey(): Promise<UninstallSurvey>;
  review(plan: Plan): Promise<Review>;
  install(plan: Plan): Promise<void>;
  uninstall(plan: UninstallPlan): Promise<void>;
  cancel(): Promise<void>;
  openChat(): Promise<void>;
  openLog(): Promise<void>;
  close(): Promise<void>;
  onProgress(f: (p: Progress) => void): void;
  /** The window's close button was pressed while installing. */
  onCloseRequested(f: () => void): void;
}

function inTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

async function tauriApi(): Promise<SetupApi> {
  const { invoke } = await import("@tauri-apps/api/core");
  const { listen } = await import("@tauri-apps/api/event");
  return {
    mode: () => invoke("mode"),
    survey: () => invoke("survey"),
    uninstallSurvey: () => invoke("uninstall_survey"),
    review: (plan) => invoke("review", { plan }),
    install: (plan) => invoke("install", { plan }),
    uninstall: (plan) => invoke("uninstall", { plan }),
    cancel: () => invoke("cancel"),
    openChat: () => invoke("open_chat"),
    openLog: () => invoke("open_log"),
    close: () => invoke("close"),
    onProgress: (f) => void listen<Progress>("progress", (e) => f(e.payload)),
    onCloseRequested: (f) => void listen("close_requested", () => f()),
  };
}

export async function connect(): Promise<SetupApi> {
  if (inTauri()) return tauriApi();
  if (import.meta.env.DEV) {
    const mode = new URLSearchParams(location.search).get("mode") === "uninstall" ? "uninstall" : "install";
    return (await import("./mock")).mockSetupApi({ mode });
  }
  throw new Error("Run no-drama-llama.exe to open setup.");
}
