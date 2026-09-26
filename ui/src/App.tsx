import { useCallback, useEffect, useRef, useState, type ComponentType } from "react";
import { connect, type Api } from "./api";
import { FormProvider, RESTARTS_SERVER, useSettingsForm } from "./form";
import { AppPanel, GamesPanel, ModelPanel, Overview, ServerPanel } from "./panels";
import type { Request, View } from "./types";

const TABS: [id: string, title: string, Panel: ComponentType][] = [
  ["overview", "Overview", Overview],
  ["model", "Model", ModelPanel],
  ["server", "Server & API", ServerPanel],
  ["games", "Game detection", GamesPanel],
  ["app", "App", AppPanel],
];

function storedTab(): string {
  try {
    return localStorage.getItem("tab") ?? "overview";
  } catch {
    return "overview"; // storage unavailable: the tab just isn't remembered
  }
}

interface Toast {
  text: string;
  error: boolean;
}

export function App() {
  const api = useRef<Api | null>(null);
  const [view, setView] = useState<View | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [toast, setToast] = useState<Toast | null>(null);
  const [tab, setTab] = useState(() => (TABS.some(([id]) => id === storedTab()) ? storedTab() : "overview"));

  const showToast = useCallback((text: string, error = false) => {
    setToast({ text, error });
  }, []);
  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), toast.error ? 8000 : 2500);
    return () => clearTimeout(t);
  }, [toast]);

  const send = useCallback(
    (r: Request) => {
      api.current?.send(r).catch((e: unknown) => showToast(String(e), true));
    },
    [showToast],
  );

  useEffect(() => {
    let live = true;
    connect()
      .then(async (a) => {
        if (!live) return;
        api.current = a;
        a.onState(setView);
        a.onSaved(({ warnings }) =>
          showToast(warnings.length ? `Not saved: ${warnings.join("; ")}` : "Saved", warnings.length > 0),
        );
        const first = await a.state();
        if (live && first) setView((v) => v ?? first);
      })
      .catch((e: unknown) => setProblem(e instanceof Error ? e.message : String(e)));
    return () => {
      live = false;
    };
  }, [showToast]);

  const { form, changed, invalid, save, revert } = useSettingsForm(view, send);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        save();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [save]);

  useEffect(() => {
    document.title = view ? `No Drama Llama - ${view.statusText}` : "No Drama Llama";
  }, [view]);

  const pickTab = (id: string) => {
    setTab(id);
    try {
      localStorage.setItem("tab", id);
    } catch {
      // ignore
    }
  };

  if (!form || !view) {
    return (
      <header className="status">
        <span className="dot loading" />
        <div className="status-text">
          <strong>{problem ?? "Connecting to the tray app..."}</strong>
        </div>
      </header>
    );
  }

  const dirty = changed.length > 0 || invalid.size > 0;
  const restarts = changed.some((k) => RESTARTS_SERVER.includes(k));

  return (
    <FormProvider value={form}>
      <header className="status">
        <span className={`dot ${view.tone}`} />
        <div className="status-text">
          <strong>{view.statusText}</strong>
          <small>{view.running ? `Chat and API at ${view.chatUrl}` : ""}</small>
        </div>
        <div className="actions">
          <button onClick={() => send({ cmd: "toggle" })}>{view.off ? "Turn on" : "Turn off"}</button>
          <button disabled={!view.canRestart} onClick={() => send({ cmd: "restart" })}>
            Restart
          </button>
          <button className="primary" disabled={!view.running} onClick={() => send({ cmd: "open_chat" })}>
            Open chat
          </button>
        </div>
      </header>

      <div className="layout">
        <nav>
          {TABS.map(([id, title]) => (
            <button key={id} className={id === tab ? "tab active" : "tab"} onClick={() => pickTab(id)}>
              {title}
            </button>
          ))}
        </nav>
        <main className={dirty ? "dirty" : undefined}>
          {/* All tabs stay mounted, so text typed on a hidden tab isn't lost. */}
          {TABS.map(([id, title, Panel]) => (
            <section key={id} hidden={id !== tab}>
              <h2>{title}</h2>
              <Panel />
            </section>
          ))}
        </main>
      </div>

      {dirty && (
        <footer className="savebar">
          <span>
            {invalid.size > 0
              ? `Fix the highlighted value${invalid.size > 1 ? "s" : ""} to save`
              : `${changed.length} unsaved change${changed.length > 1 ? "s" : ""}${restarts ? " · saving restarts the model" : ""}`}
          </span>
          <button onClick={revert}>Revert</button>
          <button className="primary" disabled={invalid.size > 0 || changed.length === 0} onClick={save}>
            Save
          </button>
        </footer>
      )}
      {toast && (
        <div className={toast.error ? "toast error" : "toast"} role="status">
          {toast.text}
        </div>
      )}
    </FormProvider>
  );
}
