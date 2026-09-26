// The window's tabs. Every settings.json key has exactly one field here (k="..."); a Rust test
// (control.rs) checks that.

import type { ReactNode } from "react";
import { Choice, ContextField, ListField, ModelPicker, NumberField, TextField, Toggle, gb, Row } from "./fields";
import { useForm } from "./form";
import type { CatalogEntry, Request } from "./types";

type Cmd = Extract<Request, { cmd: string }>["cmd"];

function Btn({
  cmd,
  children,
  disabled,
}: {
  cmd: Exclude<Cmd, "download" | "save">;
  children: ReactNode;
  disabled?: boolean;
}) {
  const { send } = useForm();
  return (
    <button disabled={disabled} onClick={() => send({ cmd })}>
      {children}
    </button>
  );
}

function Fieldset({ legend, disabled, children }: { legend?: string; disabled?: boolean; children: ReactNode }) {
  return (
    <fieldset disabled={disabled}>
      {legend && <legend>{legend}</legend>}
      {children}
    </fieldset>
  );
}

// ---------------------------------------------------------------- Overview

export function Overview() {
  const { view } = useForm();
  const u = view.update;
  const d = view.download;
  const updateText = {
    idle: "No update found at the last check",
    checking: "Checking...",
    available: u.state === "available" ? `Version ${u.version} is available` : "",
    installing: u.state === "installing" ? `Installing ${u.version}...` : "",
  }[u.state];
  const progress = d && d.total ? d.done / d.total : 0;
  return (
    <>
      {view.gameProcess && (
        <div className="banner">
          <span>
            Paused for <b>{view.gameProcess}</b>. Not a game?
          </span>
          <Btn cmd="ignore_current_game">Ignore this app</Btn>
        </div>
      )}
      <dl className="facts">
        <dt>GPU</dt>
        <dd>{view.gpuName ?? "Not detected (runs on the CPU)"}</dd>
        <dt>llama.cpp build</dt>
        <dd>{view.backend ?? "-"}</dd>
        <dt>Context in use</dt>
        <dd>{view.nCtx ? `${Math.round(view.nCtx / 1024)}K tokens` : "-"}</dd>
        <dt>Chat &amp; OpenAI API</dt>
        <dd>
          <code>{view.chatUrl}</code>
        </dd>
        <dt>Version</dt>
        <dd>{view.version}</dd>
      </dl>
      <Row label="Updates" hint={updateText}>
        <Btn cmd="check_for_updates" disabled={u.state === "checking" || u.state === "installing"}>
          {u.state === "available" ? `Get ${u.version}` : "Check for updates"}
        </Btn>
      </Row>
      {d && (
        <div className="download">
          <div className="download-head">
            <span>
              Downloading <b>{d.label}</b>
            </span>
            <Btn cmd="cancel_download">Cancel</Btn>
          </div>
          <progress max={1} value={progress} />
          <small>
            {gb(d.done)} of {gb(d.total)} ({Math.floor(progress * 100)}%). The app switches to it when it's done.
          </small>
        </div>
      )}
      <p className="hint">Ctrl+Alt+L turns the LLM on or off from anywhere.</p>
    </>
  );
}

// ---------------------------------------------------------------- Model

function CatalogRow({ c }: { c: CatalogEntry }) {
  const { view, send } = useForm();
  let button: ReactNode;
  if (view.download?.id === c.id) {
    button = <button onClick={() => send({ cmd: "cancel_download" })}>Cancel</button>;
  } else if (c.installed) {
    button = <button disabled>Installed</button>;
  } else {
    const download = () => {
      const q = `Download ${c.label}?\n\n${gb(c.size)} from Hugging Face (${c.note}). It keeps running in the background and switches over when it's done.`;
      if (confirm(q)) send({ cmd: "download", id: c.id });
    };
    button = (
      <button disabled={!c.canDownload} onClick={download}>
        Download
      </button>
    );
  }
  return (
    <div className={`entry fit-${c.fit}${c.recommended ? " recommended" : ""}`}>
      <div>
        <span>{c.label}</span>
        <small>
          {c.note}
          {c.recommended && " · recommended"}
        </small>
      </div>
      {button}
    </div>
  );
}

export function ModelPanel() {
  const { view } = useForm();
  return (
    <>
      <Fieldset legend="Installed models">
        <ModelPicker k="Model" />
        <div className="row-buttons">
          <Btn cmd="open_models">Open models folder</Btn>
        </div>
        <p className="hint">
          Drop your own <code>.gguf</code> files into the models folder and they show up here.
        </p>
      </Fieldset>
      <Fieldset legend="Download a model">
        <div className="catalog">
          {view.catalog.map((c) => (
            <CatalogRow key={c.id} c={c} />
          ))}
        </div>
      </Fieldset>
      <Fieldset legend="Generation">
        <Choice
          k="Reasoning"
          label="Reasoning"
          hint="How much the model thinks before it answers."
          options={[
            ["none", "None"],
            ["low", "Low"],
            ["medium", "Medium"],
            ["xhigh", "Extra high"],
          ]}
        />
        <ContextField
          k="Context"
          label="Context length"
          hint="Auto lets llama.cpp use the largest that fits your GPU."
        />
      </Fieldset>
    </>
  );
}

// ---------------------------------------------------------------- Server & API

function generateKey(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(24));
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

export function ServerPanel() {
  return (
    <Fieldset>
      <Choice
        k="ListenHost"
        label="Access"
        hint="Who can reach the chat page and the API."
        options={[
          ["127.0.0.1", "This PC only"],
          ["0.0.0.0", "Devices on my network"],
        ]}
      />
      <NumberField k="Port" label="Port" hint="1024 - 65535" min={1024} max={65535} integer />
      <TextField
        k="ApiKey"
        label="API key"
        hint={
          <>
            Optional. Clients must send it as a Bearer token. Letters, digits and <code>. _ - ~</code>.
          </>
        }
        pattern={/^[A-Za-z0-9._~-]*$/}
        maxLength={128}
        placeholder="(none)"
        generate={generateKey}
      />
      <p className="hint">Changes on this page and to the model restart the server when you save.</p>
    </Fieldset>
  );
}

// ---------------------------------------------------------------- Game detection

export function GamesPanel() {
  const form = useForm();
  const pause = form.value("PauseWhileGaming");
  const mode = form.value("DetectionMode");
  return (
    <>
      <Fieldset>
        <Toggle
          k="PauseWhileGaming"
          label="Pause while gaming"
          hint="Stop the model so the game gets all of your VRAM."
        />
        <Choice
          k="DetectionMode"
          label="Detect games by"
          options={[
            ["Both", "GPU usage + launchers (recommended)"],
            ["Gpu", "GPU usage only"],
            ["Launchers", "Launchers only"],
          ]}
        />
        <NumberField
          k="ResumeAfterSec"
          label="Resume after the game closes"
          hint="Seconds, 0 - 3600"
          min={0}
          max={3600}
          integer
        />
      </Fieldset>
      <Fieldset legend="GPU usage" disabled={!pause || mode === "Launchers"}>
        <NumberField
          k="GpuVramGB"
          label="VRAM threshold"
          hint="Another app using this many GB or more counts as a game."
          min={0.1}
          max={256}
          step={0.1}
        />
        <NumberField
          k="GpuLoadPct"
          label="3D load threshold"
          hint="Another app using this much of the 3D engine (%) counts as a game."
          min={1}
          max={100}
        />
        <ListField
          k="GpuIgnore"
          label="Never count as a game"
          hint={
            <>
              Process names, one per line (e.g. <code>obs64</code>).
            </>
          }
        />
        <div className="row-buttons">
          <Btn cmd="gpu_report">Show GPU usage now</Btn>
        </div>
      </Fieldset>
      <Fieldset legend="Launchers" disabled={!pause || mode === "Gpu"}>
        <Toggle k="DetectEmulators" label="Count emulators as games" />
        <Toggle k="UseWindowsGameList" label="Use Windows' game list" hint="Games the Xbox Game Bar knows about." />
        <ListField k="ExtraGames" label="Extra games" hint="Process names or folders, one per line." />
        <div className="row-buttons">
          <Btn cmd="libraries">Rescan and show detected libraries</Btn>
        </div>
      </Fieldset>
    </>
  );
}

// ---------------------------------------------------------------- App

export function AppPanel() {
  const { send } = useForm();
  const exit = () => {
    if (confirm("Exit No Drama Llama? This stops the model until you start the app again.")) send({ cmd: "exit" });
  };
  return (
    <>
      <Fieldset>
        <Toggle k="StartWithWindows" label="Start with Windows" hint="Start No Drama Llama when you sign in." />
        <Toggle k="AutoUpdate" label="Update automatically" hint="Installs signed releases from GitHub." />
      </Fieldset>
      <Fieldset legend="On-screen popups">
        <Toggle k="Popups" label="Show popups" hint="When the model pauses or resumes. They never take focus." />
        <Choice
          k="PopupPosition"
          label="Position"
          options={[
            ["TopCenter", "Top center"],
            ["TopRight", "Top right"],
            ["BottomRight", "Bottom right"],
            ["BottomCenter", "Bottom center"],
          ]}
        />
        <div className="row-buttons">
          <Btn cmd="test_popup">Test popup</Btn>
        </div>
      </Fieldset>
      <Fieldset legend="Files">
        <div className="row-buttons">
          <Btn cmd="edit_settings_file">Edit settings file</Btn>
          <Btn cmd="open_folder">Open folder</Btn>
          <Btn cmd="view_log">View log</Btn>
        </div>
      </Fieldset>
      <Fieldset legend="Quit">
        <Row label="Exit No Drama Llama" hint="Stops the model and removes the tray icon.">
          <button className="danger" onClick={exit}>
            Exit
          </button>
        </Row>
      </Fieldset>
    </>
  );
}
