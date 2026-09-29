// The window's tabs. Every settings.json key has exactly one field here (k="..."); a Rust test
// (control.rs) checks that.

import type { ReactNode } from "react";
import { Choice, ContextField, ListField, ModelPicker, NumberField, TextField, Toggle, gb, Row } from "./fields";
import { useForm } from "./form";
import type { CatalogEntry, DownloadView, Request } from "./types";

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

// ---------------------------------------------------------------- Shared by the LLM and the decision model

/** A model server's status block: its state, then the facts particular to it. */
function Status({ tone, text, children }: { tone: string; text: string; children: ReactNode }) {
  return (
    <dl className="facts">
      <dt>Status</dt>
      <dd>
        <span className={`dot inline-dot ${tone}`} /> {text}
      </dd>
      {children}
    </dl>
  );
}

function Progress({ job, children }: { job: DownloadView; children?: ReactNode }) {
  const progress = job.total ? job.done / job.total : 0;
  return (
    <div className="download">
      <div className="download-head">
        <span>
          Downloading <b>{job.label}</b>
        </span>
        {children}
      </div>
      <progress max={1} value={progress} />
      <small>
        {job.total ? `${gb(job.done)} of ${gb(job.total)} (${Math.floor(progress * 100)}%)` : "Starting..."}
      </small>
    </div>
  );
}

// ---------------------------------------------------------------- Overview

export function Overview() {
  const { view } = useForm();
  const u = view.update;
  const updateText = {
    idle: "No update found at the last check",
    checking: "Checking...",
    available: u.state === "available" ? `Version ${u.version} is available` : "",
    installing: u.state === "installing" ? `Installing ${u.version}...` : "",
  }[u.state];
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
        <dt>LLM</dt>
        <dd>
          <span className={`dot inline-dot ${view.tone}`} /> {view.statusText} · <code>{view.chatUrl}</code>
        </dd>
        <dt>Decision model</dt>
        <dd>
          <span className={`dot inline-dot ${view.laya.tone}`} /> {view.laya.statusText}
          {view.settings.RunLaya && (
            <>
              {" "}
              · <code>{view.laya.url}</code>
            </>
          )}
        </dd>
        <dt>GPU</dt>
        <dd>{view.gpuName ?? "Not detected (runs on the CPU)"}</dd>
        <dt>Version</dt>
        <dd>{view.version}</dd>
      </dl>
      <Row label="Updates" hint={updateText}>
        <Btn cmd="check_for_updates" disabled={u.state === "checking" || u.state === "installing"}>
          {u.state === "available" ? `Get ${u.version}` : "Check for updates"}
        </Btn>
      </Row>
      {view.download && (
        <Progress job={view.download}>
          <Btn cmd="cancel_download">Cancel</Btn>
        </Progress>
      )}
      {view.laya.job && <Progress job={view.laya.job} />}
      <p className="hint">Ctrl+Alt+L turns the LLM and the decision model on or off from anywhere.</p>
    </>
  );
}

// ---------------------------------------------------------------- LLM

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
          <span className="fit-dot" />
          {c.note}
          {c.recommended && " · recommended ★"}
        </small>
      </div>
      {button}
    </div>
  );
}

function generateKey(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(24));
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

export function LlmPanel() {
  const { view } = useForm();
  return (
    <>
      <p className="intro">
        The LLM writes text: chat, summaries, code. llama.cpp serves it with a chat page and an OpenAI-compatible API,
        and it pauses while you play.
      </p>
      <Status tone={view.tone} text={view.statusText}>
        <dt>API</dt>
        <dd>
          <code>{view.chatUrl}</code> (OpenAI-compatible: <code>/v1/chat/completions</code>)
        </dd>
        <dt>llama.cpp</dt>
        <dd>{view.backend ?? "-"}</dd>
        <dt>Context in use</dt>
        <dd>{view.nCtx ? `${Math.round(view.nCtx / 1024)}K tokens` : "-"}</dd>
      </Status>
      {view.download && (
        <Progress job={view.download}>
          <Btn cmd="cancel_download">Cancel</Btn>
        </Progress>
      )}
      <div className="row-buttons engine-actions">
        <Btn cmd="restart" disabled={!view.canRestart}>
          Restart LLM
        </Btn>
        <Btn cmd="view_log">View log</Btn>
      </div>
      <Fieldset legend="Model">
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
      <Fieldset legend="Server">
        <NumberField k="Port" label="Port" hint="1024 - 65535" min={1024} max={65535} integer />
        <Choice
          k="ListenHost"
          label="Access"
          hint="Who can reach the chat page and the APIs. Also applies to the decision model."
          options={[
            ["127.0.0.1", "This PC only"],
            ["0.0.0.0", "Devices on my network"],
          ]}
        />
        <TextField
          k="ApiKey"
          label="API key"
          hint={
            <>
              Optional, and also used by the decision model. Clients send it as a Bearer token. Letters, digits and{" "}
              <code>. _ - ~</code>.
            </>
          }
          pattern={/^[A-Za-z0-9._~-]*$/}
          maxLength={128}
          placeholder="(none)"
          generate={generateKey}
        />
        <p className="hint">Changes to the model and the server restart the LLM when you save.</p>
      </Fieldset>
    </>
  );
}

// ---------------------------------------------------------------- Decision model

// Mirror laya::MODELS, laya::KEEP_ALIVE_PRESETS and laya::Device::ALL.
const LAYA_MODELS = [
  ["laya", "Laya (picks English or multilingual per request)"],
  ["laya:en", "Laya English (421M, the fastest)"],
  ["laya:multilingual", "Laya multilingual (322M, 100+ languages)"],
  ["laya:typed-decisions", "Laya typed-decisions"],
] as const;

const KEEP_ALIVE = [
  ["-1", "Always (until paused or off)"],
  ["1h", "1 hour after the last request"],
  ["30m", "30 minutes after the last request"],
  ["5m", "5 minutes after the last request"],
  ["0", "Unload after each request"],
] as const;

/** The presets, plus the saved value when settings.json has something else. */
function withSaved(presets: readonly (readonly [string, string])[], saved: string) {
  return presets.some(([v]) => v === saved) ? presets : [...presets, [saved, saved] as const];
}

export function DecisionPanel() {
  const form = useForm();
  const l = form.view.laya;
  const on = form.value("RunLaya");
  const running = form.view.settings.RunLaya;
  return (
    <>
      <p className="intro">
        The decision model answers typed questions about a text (pick one of these options, score it, yes or no) in
        milliseconds, instead of writing text. Ollaya (ollaya.dev) serves Laya next to the LLM, with the same on/off
        switch and pause while gaming.
      </p>
      <Fieldset>
        <Toggle
          k="RunLaya"
          label="Run Laya alongside the LLM"
          hint="Installs Ollaya (about 25 MB, plus 1.1 GB on NVIDIA GPUs) and downloads the model (about 1 GB)."
        />
      </Fieldset>
      <Status tone={l.tone} text={l.statusText}>
        <dt>API</dt>
        <dd>
          <code>{l.url}</code> (Ollaya and TypeSafe-compatible: <code>/api/decide</code>, <code>/v1/systemone</code>)
        </dd>
        <dt>Ollaya</dt>
        <dd>{running ? (l.version ?? "not installed yet") : "-"}</dd>
      </Status>
      {l.job && <Progress job={l.job} />}
      {running && (
        <div className="row-buttons engine-actions">
          <Btn cmd="restart_laya">Restart Laya</Btn>
          <Btn cmd="update_laya" disabled={l.checking || !!l.job}>
            {l.update ? `Install Ollaya ${l.update}` : l.checking ? "Checking..." : "Check for Ollaya updates"}
          </Btn>
          <Btn cmd="view_laya_log">View Laya log</Btn>
        </div>
      )}
      <Fieldset legend="Model" disabled={!on}>
        <Choice
          k="LayaModel"
          label="Model"
          hint="Other Ollaya models (decider, nli, ...) can go in the settings file."
          options={withSaved(LAYA_MODELS, form.saved("LayaModel"))}
        />
        <Choice
          k="LayaKeepAlive"
          label="Keep the model loaded"
          options={withSaved(KEEP_ALIVE, form.saved("LayaKeepAlive"))}
        />
      </Fieldset>
      <Fieldset legend="Server" disabled={!on}>
        <NumberField k="LayaPort" label="Port" hint="1024 - 65535, not the LLM's port" min={1024} max={65535} integer />
        <Choice
          k="LayaDevice"
          label="Run on"
          hint="Ollaya runs on NVIDIA GPUs (CUDA 13). With an AMD or Intel GPU it runs on the CPU."
          options={[
            ["auto", "Auto (NVIDIA GPU if there is one, else CPU)"],
            ["cpu", "CPU only"],
            ["cuda", "NVIDIA GPU only"],
          ]}
        />
        <p className="hint">Access and the API key are shared with the LLM (LLM → Server). Changes restart Laya.</p>
      </Fieldset>
    </>
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
      <Fieldset legend="Privacy">
        <Toggle
          k="SendCrashReports"
          label="Send crash reports"
          hint="The error and where in the code it happened, with your user name, PC name and profile folder removed."
        />
        <Toggle
          k="ShareUsageStats"
          label="Share anonymous usage statistics"
          hint="Which features you use, the app version and your GPU model, with a random ID. Turning this off deletes the ID."
        />
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
