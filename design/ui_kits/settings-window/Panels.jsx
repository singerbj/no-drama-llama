(() => {
const { Button, Fieldset, SettingRow, Switch, Select, NumberInput, TextInput, TextArea, FactList, Banner, DownloadProgress, ModelPicker, CatalogRow, StatusDot, StatusPopup } = window.NoDramaLlamaDesignSystem_58ce7f;
const hint = { margin: "8px 0 0", color: "var(--text-muted)", fontSize: "var(--text-hint)" };
const rowButtons = { display: "flex", flexWrap: "wrap", gap: 8, paddingTop: 10 };

const Tog = ({ f, k, label, hint: h, last }) => <SettingRow label={label} hint={h} last={last}><Switch checked={f.value(k)} onChange={(v) => f.edit(k, v)} /></SettingRow>;
const Num = ({ f, k, label, hint: h, min, max, step = 1, last }) => {
  const v = f.value(k); const bad = v === "" || Number(v) < min || Number(v) > max;
  return <SettingRow label={label} hint={h} last={last}><NumberInput value={v} min={min} max={max} step={step} invalid={bad} onChange={(e) => { const t = e.target.value; const n = t === "" ? "" : Number(t); f.edit(k, n, t === "" || n < min || n > max); }} /></SettingRow>;
};
const Pick = ({ f, k, label, hint: h, options, last }) => <SettingRow label={label} hint={h} last={last}><Select value={String(f.value(k))} options={options} onChange={(e) => f.edit(k, e.target.value)} /></SettingRow>;
const List = ({ f, k, label, hint: h }) => <SettingRow label={label} hint={h} stacked last><TextArea value={f.value(k).join("\n")} onChange={(e) => f.edit(k, e.target.value.split(/\r?\n/))} /></SettingRow>;

function Overview({ f, view, send }) {
  const u = view.update;
  const updateText = { idle: "No update found at the last check", checking: "Checking...", available: `Version ${u.version} is available` }[u.state];
  return <>
    {view.gameProcess && <Banner tone="paused" action={<Button onClick={() => send("ignore_current_game")}>Ignore this app</Button>}>Paused for <b>{view.gameProcess}</b>. Not a game?</Banner>}
    <FactList items={[["GPU", view.gpuName], ["llama.cpp build", view.backend], ["Context in use", view.running ? `${Math.round(view.nCtx / 1024)}K tokens` : "-"], ["Chat & OpenAI API", <code>{view.chatUrl}</code>], ["Version", view.version]]} />
    <SettingRow label="Updates" hint={updateText} last><Button disabled={u.state === "checking"} onClick={() => send("check_for_updates")}>{u.state === "available" ? `Get ${u.version}` : "Check for updates"}</Button></SettingRow>
    {view.download && <DownloadProgress label={view.download.label} done={view.download.done} total={view.download.total} onCancel={() => send("cancel_download")} note="The app switches to it when it's done." />}
    <p style={hint}>Ctrl+Alt+L turns the LLM on or off from anywhere.</p>
  </>;
}

function ModelPanel({ f, view, send }) {
  return <>
    <Fieldset legend="Installed models">
      <ModelPicker models={view.models} value={f.value("Model")} onChange={(m) => f.edit("Model", m)} />
      <div style={rowButtons}><Button>Open models folder</Button></div>
      <p style={hint}>Drop your own <code>.gguf</code> files into the models folder and they show up here.</p>
    </Fieldset>
    <Fieldset legend="Download a model">
      {view.catalog.map((c, i) => <CatalogRow key={c.id} label={c.label} note={c.note} fit={c.fit} recommended={c.recommended} last={i === view.catalog.length - 1}
        state={c.installed ? "installed" : view.download?.id === c.id ? "downloading" : view.download ? "unavailable" : "download"}
        onAction={() => send(view.download?.id === c.id ? "cancel_download" : "download", c.id)} />)}
    </Fieldset>
    <Fieldset legend="Generation">
      <Pick f={f} k="Reasoning" label="Reasoning" hint="How much the model thinks before it answers." options={[["none", "None"], ["low", "Low"], ["medium", "Medium"], ["xhigh", "Extra high"]]} />
      <Pick f={f} k="Context" label="Context length" hint="Auto lets llama.cpp use the largest that fits your GPU." last options={[["auto", "Auto (largest that fits)"], ...[8192, 16384, 32768, 65536, 131072, 262144].map((n) => [String(n), `${n / 1024}K tokens`])]} />
    </Fieldset>
  </>;
}

function ServerPanel({ f }) {
  const key = f.value("ApiKey"); const bad = !/^[A-Za-z0-9._~-]*$/.test(key);
  const gen = () => btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(24)))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  return <Fieldset>
    <Pick f={f} k="ListenHost" label="Access" hint="Who can reach the chat page and the API." options={[["127.0.0.1", "This PC only"], ["0.0.0.0", "Devices on my network"]]} />
    <Num f={f} k="Port" label="Port" hint="1024 - 65535" min={1024} max={65535} />
    <SettingRow label="API key" hint={<>Optional. Clients must send it as a Bearer token. Letters, digits and <code>. _ - ~</code>.</>} stacked last>
      <span style={{ display: "flex", gap: 8 }}><TextInput value={key} placeholder="(none)" invalid={bad} onChange={(e) => f.edit("ApiKey", e.target.value, !/^[A-Za-z0-9._~-]*$/.test(e.target.value))} /><Button onClick={() => f.edit("ApiKey", gen())}>Generate</Button></span>
    </SettingRow>
    <p style={hint}>Changes on this page and to the model restart the server when you save.</p>
  </Fieldset>;
}

function LayaPanel({ f, view }) {
  const on = f.value("RunLaya");
  return <>
    <p style={{ ...hint, margin: "0 0 14px", fontSize: "var(--text-app)" }}>Laya is a <i>decision model</i>: it answers typed questions about a text (pick one of these options, score it, yes or no) in milliseconds, instead of writing text. Ollaya (ollaya.dev) serves it next to the LLM, with the same on/off switch and pause while gaming.</p>
    <Fieldset><Tog f={f} k="RunLaya" last label="Run Laya alongside the LLM" hint="Installs Ollaya (about 25 MB, plus 1.1 GB on NVIDIA GPUs) and downloads the model (about 1 GB)." /></Fieldset>
    {view.settings.RunLaya && <FactList items={[["Status", <span style={{ display: "inline-flex", alignItems: "center", gap: 8 }}><StatusDot tone="loading" /> Downloading laya...</span>], ["API", <code>http://127.0.0.1:11435</code>], ["Ollaya", "0.5.0"]]} />}
    <Fieldset legend="Model" disabled={!on}>
      <Pick f={f} k="LayaModel" label="Model" hint="Other Ollaya models (decider, nli, ...) can go in the settings file." options={[["laya", "Laya (picks English or multilingual per request)"], ["laya:en", "Laya English (421M, the fastest)"], ["laya:multilingual", "Laya multilingual (322M, 100+ languages)"], ["laya:typed-decisions", "Laya typed-decisions"]]} />
      <Pick f={f} k="LayaKeepAlive" label="Keep the model loaded" last options={[["-1", "Always (until paused or off)"], ["1h", "1 hour after the last request"], ["30m", "30 minutes after the last request"], ["5m", "5 minutes after the last request"], ["0", "Unload after each request"]]} />
    </Fieldset>
    <Fieldset legend="Server" disabled={!on}>
      <Num f={f} k="LayaPort" label="Port" hint="1024 - 65535, not the LLM's port" min={1024} max={65535} />
      <Pick f={f} k="LayaDevice" label="Run on" last hint="On the CPU, Laya keeps running while you play; on the GPU it pauses with the LLM." options={[["auto", "Auto (NVIDIA GPU if there is one)"], ["cpu", "CPU only"], ["cuda", "NVIDIA GPU only"]]} />
    </Fieldset>
  </>;
}

function GamesPanel({ f }) {
  const pause = f.value("PauseWhileGaming"), mode = f.value("DetectionMode");
  return <>
    <Fieldset>
      <Tog f={f} k="PauseWhileGaming" label="Pause while gaming" hint="Stop the model so the game gets all of your VRAM." />
      <Pick f={f} k="DetectionMode" label="Detect games by" options={[["Both", "GPU usage + launchers (recommended)"], ["Gpu", "GPU usage only"], ["Launchers", "Launchers only"]]} />
      <Num f={f} k="ResumeAfterSec" label="Resume after the game closes" hint="Seconds, 0 - 3600" min={0} max={3600} last />
    </Fieldset>
    <Fieldset legend="GPU usage" disabled={!pause || mode === "Launchers"}>
      <Num f={f} k="GpuVramGB" label="VRAM threshold" hint="Another app using this many GB or more counts as a game." min={0.1} max={256} step={0.1} />
      <Num f={f} k="GpuLoadPct" label="3D load threshold" hint="Another app using this much of the 3D engine (%) counts as a game." min={1} max={100} />
      <List f={f} k="GpuIgnore" label="Never count as a game" hint={<>Process names, one per line (e.g. <code>obs64</code>).</>} />
      <div style={rowButtons}><Button>Show GPU usage now</Button></div>
    </Fieldset>
    <Fieldset legend="Launchers" disabled={!pause || mode === "Gpu"}>
      <Tog f={f} k="DetectEmulators" label="Count emulators as games" />
      <Tog f={f} k="UseWindowsGameList" label="Use Windows' game list" hint="Games the Xbox Game Bar knows about." />
      <List f={f} k="ExtraGames" label="Extra games" hint="Process names or folders, one per line." />
      <div style={rowButtons}><Button>Rescan and show detected libraries</Button></div>
    </Fieldset>
  </>;
}

function AppPanel({ f, send }) {
  const [popup, setPopup] = React.useState(false);
  const test = () => { setPopup(true); setTimeout(() => setPopup(false), 2600); };
  return <>
    <Fieldset>
      <Tog f={f} k="StartWithWindows" label="Start with Windows" hint="Start No Drama Llama when you sign in." />
      <Tog f={f} k="AutoUpdate" label="Update automatically" hint="Installs signed releases from GitHub." last />
    </Fieldset>
    <Fieldset legend="On-screen popups">
      <Tog f={f} k="Popups" label="Show popups" hint="When the model pauses or resumes. They never take focus." />
      <Pick f={f} k="PopupPosition" label="Position" last options={[["TopCenter", "Top center"], ["TopRight", "Top right"], ["BottomRight", "Bottom right"], ["BottomCenter", "Bottom center"]]} />
      <div style={rowButtons}><Button onClick={test}>Test popup</Button></div>
    </Fieldset>
    <Fieldset legend="Files"><div style={rowButtons}><Button>Edit settings file</Button><Button>Open folder</Button><Button>View log</Button></div></Fieldset>
    <Fieldset legend="Quit"><SettingRow label="Exit No Drama Llama" hint="Stops the model and removes the tray icon." last><Button variant="danger" onClick={() => confirm("Exit No Drama Llama? This stops the model until you start the app again.")}>Exit</Button></SettingRow></Fieldset>
    {popup && <div style={{ position: "fixed", top: 40, left: "50%", transform: "translateX(-50%)", zIndex: 5 }}><StatusPopup tone="paused" title="LLM paused" subtitle="Test popup · this is where it shows" /></div>}
  </>;
}

window.NDLPanels = { Overview, ModelPanel, ServerPanel, LayaPanel, GamesPanel, AppPanel };
})();
