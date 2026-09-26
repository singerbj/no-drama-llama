// The page's link to the tray app. In Tauri: commands and events from src/win/settings_app.rs.
// In a plain browser (`npm run dev`): sample data, for working on the layout.

import type { Request, Saved, View } from "./types";

export interface Api {
  send(r: Request): Promise<void>;
  /** The latest state, if the tray has sent one yet. */
  state(): Promise<View | null>;
  onState(f: (v: View) => void): void;
  onSaved(f: (s: Saved) => void): void;
}

function inTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

async function tauriApi(): Promise<Api> {
  const { invoke } = await import("@tauri-apps/api/core");
  const { listen } = await import("@tauri-apps/api/event");
  return {
    send: (request) => invoke("send", { request }),
    state: () => invoke<View | null>("state"),
    onState: (f) => void listen<View>("state", (e) => f(e.payload)),
    onSaved: (f) => void listen<Saved>("saved", (e) => f(e.payload)),
  };
}

export async function connect(): Promise<Api> {
  if (inTauri()) return tauriApi();
  if (import.meta.env.DEV) return (await import("./mock")).mockApi();
  throw new Error("Open this from the No Drama Llama tray icon.");
}
