import { connect, type Api } from "./api";
import type { CatalogEntry, Request, SettingKey, Settings, SettingValue, View } from "./types";

/** Settings that change llama-server's command line (see settings::server_changed). */
const RESTARTS_SERVER: SettingKey[] = ["Model", "Reasoning", "Context", "ListenHost", "Port", "ApiKey"];

let api: Api;
let view: View | null = null;
/** Edited, valid values that differ from the tray's settings. */
const draft = new Map<SettingKey, SettingValue>();
/** Saved but not yet reflected in a state update (keeps fields from flickering back). */
const saving = new Map<SettingKey, SettingValue>();
const invalid = new Set<SettingKey>();

function el<T extends HTMLElement = HTMLElement>(id: string): T {
  const e = document.getElementById(id);
  if (!e) throw new Error(`#${id} missing`);
  return e as T;
}

const fields = [...document.querySelectorAll<HTMLElement>("[data-key]")];
const keyOf = (f: HTMLElement) => f.dataset.key as SettingKey;
const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);
const gb = (bytes: number) => `${(bytes / 1e9).toFixed(1)} GB`;

function effective<K extends SettingKey>(key: K): Settings[K] | undefined {
  return (draft.get(key) ?? view?.settings[key]) as Settings[K] | undefined;
}

// ---------------------------------------------------------------- reading and writing fields

/** The field's value, or undefined if what's typed isn't valid. */
function read(f: HTMLElement): SettingValue | undefined {
  switch (f.dataset.type) {
    case "bool":
      return (f as HTMLInputElement).checked;
    case "int":
    case "float": {
      const i = f as HTMLInputElement;
      const n = i.valueAsNumber;
      if (i.value.trim() === "" || !i.checkValidity() || Number.isNaN(n)) return undefined;
      if (f.dataset.type === "int" && !Number.isInteger(n)) return undefined;
      return n;
    }
    case "text": {
      const i = f as HTMLInputElement;
      return i.checkValidity() ? i.value.trim() : undefined;
    }
    case "choice":
      return (f as HTMLSelectElement).value as SettingValue;
    case "list":
      return (f as HTMLTextAreaElement).value
        .split(/\r?\n/)
        .map((s) => s.trim())
        .filter(Boolean);
    case "context": {
      const choice = el<HTMLSelectElement>("context-choice").value;
      if (choice === "auto") return "auto";
      if (choice !== "custom") return Number(choice);
      const i = el<HTMLInputElement>("context-custom");
      const n = i.valueAsNumber;
      return i.value.trim() !== "" && i.checkValidity() && Number.isInteger(n) ? n : undefined;
    }
    case "model":
      return f.querySelector<HTMLInputElement>("input:checked")?.value;
  }
  return undefined;
}

function write(f: HTMLElement, v: SettingValue) {
  switch (f.dataset.type) {
    case "bool":
      (f as HTMLInputElement).checked = v as boolean;
      break;
    case "int":
    case "float":
    case "text":
    case "choice":
      (f as HTMLInputElement).value = String(v);
      break;
    case "list":
      (f as HTMLTextAreaElement).value = (v as string[]).join("\n");
      break;
    case "context": {
      const choice = el<HTMLSelectElement>("context-choice");
      const custom = el<HTMLInputElement>("context-custom");
      const preset = [...choice.options].some((o) => o.value === String(v) && o.value !== "custom");
      choice.value = preset ? String(v) : "custom";
      custom.hidden = preset;
      if (!preset) custom.value = String(v);
      break;
    }
    case "model":
      for (const r of f.querySelectorAll<HTMLInputElement>("input[type=radio]")) r.checked = r.value === v;
      break;
  }
}

function onEdit(f: HTMLElement) {
  if (!view) return;
  const key = keyOf(f);
  const v = read(f);
  f.classList.toggle("invalid", v === undefined);
  if (v === undefined) {
    invalid.add(key);
  } else {
    invalid.delete(key);
    if (same(v, view.settings[key])) draft.delete(key);
    else draft.set(key, v);
  }
  if (key === "Context") el("context-custom").hidden = el<HTMLSelectElement>("context-choice").value !== "custom";
  renderSavebar();
  renderDependent();
}

/** Shows the tray's values in every field the user isn't editing. */
function syncFields(s: Settings) {
  for (const f of fields) {
    const key = keyOf(f);
    if (draft.has(key) || invalid.has(key) || f.contains(document.activeElement)) continue;
    if (saving.has(key)) {
      if (!same(saving.get(key), s[key])) continue;
      saving.delete(key);
    }
    write(f, s[key]);
  }
}

// ---------------------------------------------------------------- rendering

function renderStatus(v: View) {
  const dot = el("dot");
  dot.className = `dot ${v.tone}`;
  el("status-text").textContent = v.statusText;
  el("status-sub").textContent = v.running ? `Chat and API at ${v.chatUrl}` : "";
  el("toggle").textContent = v.off ? "Turn on" : "Turn off";
  el<HTMLButtonElement>("restart").disabled = !v.canRestart;
  el<HTMLButtonElement>("open-chat").disabled = !v.running;
  document.title = `No Drama Llama - ${v.statusText}`;

  el("paused-banner").hidden = !v.gameProcess;
  el("paused-process").textContent = v.gameProcess ?? "";
  el("fact-gpu").textContent = v.gpuName ?? "Not detected (runs on the CPU)";
  el("fact-backend").textContent = v.backend ?? "-";
  el("fact-ctx").textContent = v.nCtx ? `${Math.round(v.nCtx / 1024)}K tokens` : "-";
  el("fact-url").textContent = v.chatUrl;
  el("fact-version").textContent = v.version;

  const btn = el<HTMLButtonElement>("update-btn");
  const text = el("update-text");
  const u = v.update;
  btn.disabled = u.state === "checking" || u.state === "installing";
  btn.textContent = u.state === "available" ? `Get ${u.version}` : "Check for updates";
  text.textContent = {
    idle: "No update found at the last check",
    checking: "Checking...",
    available: u.state === "available" ? `Version ${u.version} is available` : "",
    installing: u.state === "installing" ? `Installing ${u.version}...` : "",
  }[u.state];

  const d = v.download;
  el("download-box").hidden = !d;
  if (d) {
    el("download-label").textContent = d.label;
    const p = el<HTMLProgressElement>("download-progress");
    p.value = d.total ? d.done / d.total : 0;
    el("download-text").textContent = `${gb(d.done)} of ${gb(d.total)} (${Math.floor(p.value * 100)}%). The app switches to it when it's done.`;
  }
}

let modelsKey = "";
function renderModels(v: View) {
  const key = JSON.stringify(v.models);
  const box = el("models");
  if (key !== modelsKey) {
    modelsKey = key;
    box.replaceChildren();
    const names = v.models.map((m) => m.name);
    // A configured model that isn't on disk still shows, so it's clear what's selected.
    const rows = names.includes(v.settings.Model) ? v.models : [{ name: v.settings.Model, size: -1 }, ...v.models];
    for (const m of rows) {
      const label = document.createElement("label");
      label.className = "model";
      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "model";
      radio.value = m.name;
      radio.addEventListener("change", () => onEdit(box));
      const name = document.createElement("span");
      name.textContent = m.name;
      const size = document.createElement("small");
      size.textContent = m.size < 0 ? "missing" : gb(m.size);
      label.append(radio, name, size);
      box.append(label);
    }
    write(box, effective("Model") ?? v.settings.Model);
  }
}

function catalogRow(c: CatalogEntry, v: View): HTMLElement {
  const row = document.createElement("div");
  row.className = `entry fit-${c.fit}`;
  const text = document.createElement("div");
  const title = document.createElement("span");
  title.textContent = c.label;
  const note = document.createElement("small");
  note.textContent = c.note + (c.recommended ? " · recommended" : "");
  text.append(title, note);
  const btn = document.createElement("button");
  if (v.download?.id === c.id) {
    btn.textContent = "Cancel";
    btn.onclick = () => send({ cmd: "cancel_download" });
  } else if (c.installed) {
    btn.textContent = "Installed";
    btn.disabled = true;
  } else {
    btn.textContent = "Download";
    btn.disabled = !c.canDownload;
    btn.onclick = () => {
      const q = `Download ${c.label}?\n\n${gb(c.size)} from Hugging Face (${c.note}). It keeps running in the background and switches over when it's done.`;
      if (confirm(q)) send({ cmd: "download", id: c.id });
    };
  }
  if (c.recommended) row.classList.add("recommended");
  row.append(text, btn);
  return row;
}

function renderCatalog(v: View) {
  el("catalog").replaceChildren(...v.catalog.map((c) => catalogRow(c, v)));
}

/** Parts of the form that depend on other settings. */
function renderDependent() {
  const mode = effective("DetectionMode");
  const pause = effective("PauseWhileGaming") !== false;
  for (const fs of document.querySelectorAll<HTMLFieldSetElement>("fieldset.gpu-only")) fs.disabled = !pause || mode === "Launchers";
  for (const fs of document.querySelectorAll<HTMLFieldSetElement>("fieldset.launchers-only")) fs.disabled = !pause || mode === "Gpu";
}

function renderSavebar() {
  const dirty = draft.size > 0 || invalid.size > 0;
  el("savebar").hidden = !dirty;
  document.body.classList.toggle("dirty", dirty);
  el<HTMLButtonElement>("save").disabled = invalid.size > 0 || draft.size === 0;
  const restarts = [...draft.keys()].some((k) => RESTARTS_SERVER.includes(k));
  el("savebar-text").textContent =
    invalid.size > 0
      ? `Fix the highlighted value${invalid.size > 1 ? "s" : ""} to save`
      : `${draft.size} unsaved change${draft.size > 1 ? "s" : ""}${restarts ? " · saving restarts the model" : ""}`;
}

function render(v: View) {
  view = v;
  renderStatus(v);
  renderModels(v);
  renderCatalog(v);
  syncFields(v.settings);
  renderDependent();
  renderSavebar();
}

// ---------------------------------------------------------------- actions

let toastTimer = 0;
function toast(text: string, error = false) {
  const t = el("toast");
  t.textContent = text;
  t.classList.toggle("error", error);
  t.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (t.hidden = true), error ? 8000 : 2500);
}

async function send(r: Request) {
  try {
    await api.send(r);
  } catch (e) {
    toast(String(e), true);
  }
}

function save() {
  if (!view || draft.size === 0 || invalid.size > 0) return;
  const settings = Object.fromEntries(draft) as Partial<Settings>;
  for (const [k, v] of draft) saving.set(k, v);
  draft.clear();
  setTimeout(() => saving.clear(), 4000);
  void send({ cmd: "save", settings });
  renderSavebar();
}

function revert() {
  draft.clear();
  invalid.clear();
  for (const f of fields) f.classList.remove("invalid");
  if (view) {
    for (const f of fields) write(f, view.settings[keyOf(f)]);
    render(view);
  }
}

function generateKey(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(24));
  return btoa(String.fromCharCode(...bytes)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function showTab(name: string) {
  for (const t of document.querySelectorAll<HTMLElement>(".tab")) t.classList.toggle("active", t.dataset.tab === name);
  for (const p of document.querySelectorAll<HTMLElement>(".panel")) p.hidden = p.id !== `tab-${name}`;
  try {
    localStorage.setItem("tab", name);
  } catch {
    // storage unavailable: the tab just isn't remembered
  }
}

function wire() {
  for (const f of fields) {
    if (f.dataset.type === "model") continue; // radios wire themselves
    f.addEventListener("input", () => onEdit(f));
    f.addEventListener("change", () => onEdit(f));
    f.addEventListener("focusout", () => view && !draft.has(keyOf(f)) && !invalid.has(keyOf(f)) && write(f, view.settings[keyOf(f)]));
  }
  for (const b of document.querySelectorAll<HTMLButtonElement>("button[data-cmd]")) {
    b.addEventListener("click", () => send({ cmd: b.dataset.cmd } as Request));
  }
  for (const t of document.querySelectorAll<HTMLElement>(".tab")) t.addEventListener("click", () => showTab(t.dataset.tab!));
  el("save").addEventListener("click", save);
  el("revert").addEventListener("click", revert);
  el("exit").addEventListener("click", () => {
    if (confirm("Exit No Drama Llama? This stops the model until you start the app again.")) void send({ cmd: "exit" });
  });
  el("gen-key").addEventListener("click", () => {
    const i = document.querySelector<HTMLInputElement>('[data-key="ApiKey"]')!;
    i.value = generateKey();
    onEdit(i);
  });
  document.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      save();
    }
  });
  let tab = "overview";
  try {
    tab = localStorage.getItem("tab") ?? tab;
  } catch {
    // ignore
  }
  showTab(document.getElementById(`tab-${tab}`) ? tab : "overview");
}

async function main() {
  wire();
  try {
    api = await connect();
  } catch (e) {
    el("status-text").textContent = String(e instanceof Error ? e.message : e);
    return;
  }
  api.onState(render);
  api.onSaved(({ warnings }) => {
    if (warnings.length) toast(`Not saved: ${warnings.join("; ")}`, true);
    else toast("Saved");
  });
  const first = await api.state();
  if (first) render(first);
}

void main();
