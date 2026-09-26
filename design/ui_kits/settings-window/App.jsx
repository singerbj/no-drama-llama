(() => {
const { Button, StatusHeader, SideNav, SaveBar, Toast } = window.NoDramaLlamaDesignSystem_58ce7f;
const { Overview, ModelPanel, ServerPanel, LayaPanel, GamesPanel, AppPanel } = window.NDLPanels;
const TABS = [["overview", "Overview", Overview], ["model", "Model", ModelPanel], ["server", "Server & API", ServerPanel], ["laya", "Laya", LayaPanel], ["games", "Game detection", GamesPanel], ["app", "App", AppPanel]];
const RESTARTS = ["Model", "Reasoning", "Context", "ListenHost", "Port", "ApiKey"];

function App() {
  const [view, setView] = React.useState(() => ({ ...window.NDL_MOCK, off: false, running: true, gameProcess: null, update: { state: "idle" }, download: null }));
  const [draft, setDraft] = React.useState({});
  const [invalid, setInvalid] = React.useState({});
  const [tab, setTab] = React.useState(() => localStorage.getItem("ndl-tab") || "overview");
  const [toast, setToast] = React.useState(null);
  React.useEffect(() => { if (!toast) return; const t = setTimeout(() => setToast(null), toast.error ? 8000 : 2500); return () => clearTimeout(t); }, [toast]);

  const f = {
    value: (k) => (k in draft ? draft[k] : view.settings[k]),
    edit: (k, v, bad = false) => {
      setInvalid((s) => { const n = { ...s }; if (bad) n[k] = true; else delete n[k]; return n; });
      setDraft((d) => { const n = { ...d }; if (JSON.stringify(v) === JSON.stringify(view.settings[k])) delete n[k]; else n[k] = v; return n; });
    },
  };
  const changed = Object.keys(draft), bad = Object.keys(invalid).length;
  const save = () => { if (!changed.length || bad) return; setView((v) => ({ ...v, settings: { ...v.settings, ...draft } })); setDraft({}); setToast({ text: "Saved" }); };
  const revert = () => { setDraft({}); setInvalid({}); };
  React.useEffect(() => { const k = (e) => { if ((e.ctrlKey || e.metaKey) && e.key === "s") { e.preventDefault(); save(); } else if (e.key === "g" && !/INPUT|TEXTAREA|SELECT/.test(e.target.tagName)) send("game"); }; document.addEventListener("keydown", k); return () => document.removeEventListener("keydown", k); });

  const send = (cmd, id) => setView((v) => {
    switch (cmd) {
      case "toggle": return v.off ? { ...v, off: false, running: true, tone: "running", statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL" } : { ...v, off: true, running: false, tone: "off", statusText: "Off" };
      case "restart": setTimeout(() => setView((w) => ({ ...w, tone: "running", running: true, statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context" })), 1800); return { ...v, tone: "loading", running: false, statusText: "Loading Qwen3.8-27B-UD-Q4_K_XL..." };
      case "game": return { ...v, tone: "paused", running: false, gameProcess: "eldenring.exe", statusText: "Paused for a game - eldenring.exe" };
      case "ignore_current_game": return { ...v, tone: "running", running: true, gameProcess: null, statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context" };
      case "check_for_updates": return { ...v, update: { state: "available", version: "2.2.0" } };
      case "download": { const m = v.catalog.find((c) => c.id === id); return { ...v, download: { id, label: m.label, done: 0.37 * m.size, total: m.size } }; }
      case "cancel_download": return { ...v, download: null };
      default: return v;
    }
  });

  const pick = (id) => { setTab(id); localStorage.setItem("ndl-tab", id); };
  const restarting = changed.some((k) => RESTARTS.includes(k));
  const dirty = changed.length > 0 || bad > 0;
  return <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
    <StatusHeader tone={view.tone} title={view.statusText} subtitle={view.running ? `Chat and API at ${view.chatUrl}` : view.tone === "paused" ? "Comes back 60 s after the game closes" : ""}
      actions={<><Button onClick={() => send("toggle")}>{view.off ? "Turn on" : "Turn off"}</Button><Button disabled={view.off} onClick={() => send("restart")}>Restart</Button><Button variant="primary" disabled={!view.running}>Open chat</Button></>} />
    <div style={{ flex: 1, display: "flex", minHeight: 0 }}>
      <SideNav items={TABS.map(([id, title]) => ({ id, title }))} active={tab} onSelect={pick} />
      <main style={{ flex: 1, overflowY: "auto", padding: dirty ? "8px 24px 80px" : "8px 24px 24px" }}>
        {TABS.map(([id, title, Panel]) => <section key={id} hidden={id !== tab}>
          <h2 style={{ fontFamily: "var(--font-display)", fontSize: "var(--text-app-h2)", fontWeight: 600, letterSpacing: "-0.02em", margin: "12px 0 14px" }}>{title}</h2>
          <Panel f={f} view={view} send={send} />
        </section>)}
      </main>
    </div>
    {dirty && <SaveBar floating saveDisabled={bad > 0 || !changed.length} onRevert={revert} onSave={save}
      message={bad ? `Fix the highlighted value${bad > 1 ? "s" : ""} to save` : `${changed.length} unsaved change${changed.length > 1 ? "s" : ""}${restarting ? " · saving restarts the model" : ""}`} />}
    {toast && <Toast floating error={toast.error}>{toast.text}</Toast>}
  </div>;
}
ReactDOM.createRoot(document.getElementById("root")).render(<App />);
})();
