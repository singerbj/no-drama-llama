// The wizard's pages. Each one only shows the survey and edits the plan; Setup.tsx moves
// between them and talks to the installer.

import { useEffect, useState, type Dispatch, type ReactNode, type SetStateAction } from "react";
import logo from "../../../design/assets/logo.svg";
import type { CatalogEntry } from "../types";
import { gb, memory, speed, timeLeft } from "./format";
import type { Run, RowState } from "./progress";
import type { Backend, Check, Finished, Plan, Review, Survey, UninstallPlan, UninstallSurvey } from "./types";

type SetPlan = Dispatch<SetStateAction<Plan | null>>;

const TONE: Record<Check["status"], string> = { pass: "running", warn: "loading", fail: "error" };

function Heading({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <>
      <h1>{title}</h1>
      {children && <p className="lede">{children}</p>}
    </>
  );
}

function Switch({
  label,
  hint,
  checked,
  onChange,
  disabled,
}: {
  label: string;
  hint: ReactNode;
  checked: boolean;
  onChange: (on: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className="row">
      <span className="label">
        {label}
        <small>{hint}</small>
      </span>
      <input
        type="checkbox"
        className="switch"
        role="switch"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
    </label>
  );
}

// ---------------------------------------------------------------- welcome

export function WelcomePage({ survey }: { survey: Survey | null }) {
  const upgrading = survey?.existing;
  return (
    <div className="welcome">
      <img className="hero-logo" src={logo} alt="No Drama Llama" width={96} height={96} />
      <Heading title={upgrading ? "Upgrade No Drama Llama" : "Set up No Drama Llama"}>
        Your gaming PC becomes a local AI server. It steps aside when you play.
      </Heading>
      {upgrading && (
        <div className="banner">
          <span>
            {upgrading.version ? `Version ${upgrading.version} is installed. ` : "An earlier install was found. "}
            Your models and settings are kept.
          </span>
        </div>
      )}
      <ul className="what">
        <li>
          <strong>Checks your PC</strong>
          <span>Windows, GPU, memory, disk space and the connection, before anything changes.</span>
        </li>
        <li>
          <strong>Picks what fits</strong>
          <span>The llama.cpp build and the model that suit your graphics card. You can change both.</span>
        </li>
        <li>
          <strong>Installs properly</strong>
          <span>
            The app in Program Files, llama.cpp and models in <code>C:\LLM</code>, a Start menu entry and an Apps &
            features entry.
          </span>
        </li>
        <li>
          <strong>Undoes itself</strong>
          <span>Uninstall from Settings → Apps puts every Windows setting back the way it was.</span>
        </li>
      </ul>
      <p className="hint">
        Windows asked for administrator rights because setup changes power settings and Program Files.
      </p>
    </div>
  );
}

// ---------------------------------------------------------------- system check

export function ChecksPage({
  survey,
  checking,
  onRecheck,
}: {
  survey: Survey | null;
  checking: boolean;
  onRecheck: () => void;
}) {
  const checks = survey?.checks ?? [];
  const failed = checks.filter((c) => c.status === "fail").length;
  const warned = checks.filter((c) => c.status === "warn").length;
  return (
    <>
      <Heading title="System check">Setup looks at this PC before it changes anything.</Heading>
      {!survey ? (
        <div className="checking">
          <span className="dot loading" />
          Checking your PC, GitHub and Hugging Face...
        </div>
      ) : (
        <>
          <div className={`banner ${failed ? "error" : warned ? "warn" : "ok"}`} role="status">
            <span>
              {failed
                ? `Fix ${failed === 1 ? "the red item" : `the ${failed} red items`}, then check again.`
                : warned
                  ? `Ready. ${warned === 1 ? "One thing is" : `${warned} things are`} worth a look, but you can continue.`
                  : "Everything looks good."}
            </span>
            <button disabled={checking} onClick={onRecheck}>
              {checking ? "Checking..." : "Check again"}
            </button>
          </div>
          <ul className="checks">
            {checks.map((c) => (
              <li key={c.id} className={`check ${c.status}`}>
                <span className={`dot ${TONE[c.status]}`} aria-label={c.status} />
                <div>
                  <strong>{c.title}</strong>
                  <small>{c.detail}</small>
                </div>
              </li>
            ))}
          </ul>
        </>
      )}
    </>
  );
}

// ---------------------------------------------------------------- model

function ModelRow({ c, selected, onSelect }: { c: CatalogEntry; selected: boolean; onSelect: () => void }) {
  const disabled = c.fit === "tooBig";
  const [name] = c.label.split("  (");
  return (
    <label className={`choice fit-${c.fit}${selected ? " selected" : ""}${disabled ? " disabled" : ""}`}>
      <input type="radio" name="model" checked={selected} disabled={disabled} onChange={onSelect} />
      <span className="choice-main">
        <span>
          {name}
          {c.recommended && <em className="badge">★ Recommended</em>}
          {c.installed && <em className="badge muted">Downloaded</em>}
        </span>
        <small>
          <span className="fit-dot" />
          {c.note}
        </small>
      </span>
      <span className="choice-size">{gb(c.size)}</span>
    </label>
  );
}

export function ModelPage({ survey, plan, setPlan }: { survey: Survey; plan: Plan; setPlan: SetPlan }) {
  const [showAll, setShowAll] = useState(false);
  const fits = survey.catalog.filter((c) => c.fit !== "tooBig");
  const shown = showAll ? survey.catalog : fits;
  const hidden = survey.catalog.length - fits.length;
  const keep = survey.existing?.model;
  const pick = (model: Plan["model"]) => setPlan((p) => p && { ...p, model });
  const selected =
    plan.model.kind === "download" ? survey.catalog.find((c) => c.id === (plan.model as { id: string }).id) : undefined;
  const need = selected && !selected.installed ? selected.size : 0;
  // The recommendation can be far down the list: start with the selected model in view.
  useEffect(() => {
    document.querySelector(".choice.selected")?.scrollIntoView?.({ block: "nearest" });
  }, []);
  return (
    <>
      <Heading title="Pick a model">
        {survey.gpu
          ? `Sized for your ${survey.gpu} and ${memory(survey.ram)} of memory.`
          : `No graphics card was found, so models run on the CPU (slowly) with ${memory(survey.ram)} of memory.`}{" "}
        Green fits entirely on the GPU, which is fastest.
      </Heading>
      <div className="choices" role="radiogroup" aria-label="Model">
        {keep && (
          <label className={`choice${plan.model.kind === "keep" ? " selected" : ""}`}>
            <input
              type="radio"
              name="model"
              checked={plan.model.kind === "keep"}
              onChange={() => pick({ kind: "keep" })}
            />
            <span className="choice-main">
              <span>
                Keep my current model <em className="badge muted">Installed</em>
              </span>
              <small>{keep}</small>
            </span>
            <span className="choice-size" />
          </label>
        )}
        {shown.map((c) => (
          <ModelRow
            key={c.id}
            c={c}
            selected={plan.model.kind === "download" && plan.model.id === c.id}
            onSelect={() => pick({ kind: "download", id: c.id })}
          />
        ))}
        <label className={`choice${plan.model.kind === "none" ? " selected" : ""}`}>
          <input
            type="radio"
            name="model"
            checked={plan.model.kind === "none"}
            onChange={() => pick({ kind: "none" })}
          />
          <span className="choice-main">
            <span>No model for now</span>
            <small>
              Download one later from the tray, or drop a <code>.gguf</code> into <code>{survey.dataDir}\models</code>.
            </small>
          </span>
          <span className="choice-size" />
        </label>
      </div>
      {hidden > 0 && (
        <button className="link" onClick={() => setShowAll((v) => !v)}>
          {showAll ? "Hide models that don't fit" : `Show ${hidden} model${hidden > 1 ? "s" : ""} too big for this PC`}
        </button>
      )}
      <DiskMeter survey={survey} need={need} />
    </>
  );
}

function DiskMeter({ survey, need }: { survey: Survey; need: number }) {
  if (survey.diskFree === null) return null;
  const free = survey.diskFree;
  const pct = Math.min(100, (need / Math.max(free, 1)) * 100);
  const short = need + 2e9 > free;
  return (
    <div className={`disk${short ? " short" : ""}`}>
      <div className="disk-head">
        <span>Space on {survey.drive}</span>
        <span>{need > 0 ? `${gb(need)} of ${gb(free)} free` : `${gb(free)} free`}</span>
      </div>
      <div className="meter" role="meter" aria-valuenow={Math.round(pct)} aria-valuemin={0} aria-valuemax={100}>
        <span style={{ width: `${pct}%` }} />
      </div>
      {short && <small>Not enough space. Free some up, or pick a smaller model.</small>}
    </div>
  );
}

// ---------------------------------------------------------------- options

export function OptionsPage({ survey, plan, setPlan }: { survey: Survey; plan: Plan; setPlan: SetPlan }) {
  const set = <K extends keyof Plan>(k: K, v: Plan[K]) => setPlan((p) => p && { ...p, [k]: v });
  return (
    <>
      <Heading title="Options">
        The defaults suit an always-on gaming PC. Everything here can be changed later in settings.
      </Heading>
      <fieldset>
        <legend>llama.cpp</legend>
        <label className="row">
          <span className="label">
            Build
            <small>{survey.backends.find((b) => b.id === plan.backend)?.note}</small>
          </span>
          <select value={plan.backend} onChange={(e) => set("backend", e.target.value as Backend)}>
            {survey.backends.map((b) => (
              <option key={b.id} value={b.id} disabled={!b.available}>
                {b.label}
                {b.id === survey.recommendedBackend ? " (recommended)" : ""}
                {!b.available ? " - not for this PC" : ""}
              </option>
            ))}
          </select>
        </label>
      </fieldset>
      <fieldset>
        <legend>This PC</legend>
        {survey.laptop && (
          <div className="banner warn">
            <span>This looks like a laptop. Always-on power settings would keep it awake and drain the battery.</span>
          </div>
        )}
        <Switch
          label="Always-on power settings"
          hint="Never sleep or hibernate on AC power, and use little power while idle. Uninstalling restores yours."
          checked={plan.power}
          onChange={(v) => set("power", v)}
        />
        <Switch
          label="Wake-on-LAN"
          hint="Lets a phone app wake this PC over a wired network. Uninstalling restores your adapters."
          checked={plan.wakeOnLan}
          onChange={(v) => set("wakeOnLan", v)}
        />
        <Switch
          label="Start with Windows"
          hint="Starts in the system tray when you sign in, without a UAC prompt."
          checked={plan.startWithWindows}
          onChange={(v) => set("startWithWindows", v)}
        />
      </fieldset>
      <fieldset>
        <legend>Extras</legend>
        <Switch
          label="Laya decision model"
          hint={
            <>
              Also run Laya, a small model for yes/no and pick-one decisions, through Ollaya (about{" "}
              {survey.layaInstalled ? "no extra download" : "1.3 GB"}).
            </>
          }
          checked={plan.laya}
          onChange={(v) => set("laya", v)}
        />
      </fieldset>
      {survey.askPrivacy && (
        <fieldset>
          <legend>Privacy</legend>
          <Switch
            label="Send crash reports"
            hint="Scrubbed of paths and names. Off unless you turn it on."
            checked={plan.crashReports}
            onChange={(v) => set("crashReports", v)}
          />
          <Switch
            label="Share anonymous usage stats"
            hint="Which features are used, never what you type. Off unless you turn it on."
            checked={plan.usageStats}
            onChange={(v) => set("usageStats", v)}
          />
        </fieldset>
      )}
    </>
  );
}

// ---------------------------------------------------------------- review

function onOff(v: boolean) {
  return v ? "On" : "Off";
}

export function ReviewPage({ survey, plan, review }: { survey: Survey; plan: Plan; review: Review | null }) {
  const backend = survey.backends.find((b) => b.id === plan.backend);
  const model =
    plan.model.kind === "keep"
      ? `Keep ${survey.existing?.model ?? "the current model"}`
      : plan.model.kind === "none"
        ? "None for now"
        : (survey.catalog
            .find((c) => plan.model.kind === "download" && c.id === plan.model.id)
            ?.label.replace("  (", " (") ?? plan.model.id);
  return (
    <>
      <Heading title="Ready to install">
        Check the summary. Setup does the rest, and the tray icon appears when it's done.
      </Heading>
      {review?.problems.map((p) => (
        <div key={p} className="banner error" role="alert">
          <span>{p}</span>
        </div>
      ))}
      <dl className="facts">
        <dt>App</dt>
        <dd>{survey.installDir}</dd>
        <dt>llama.cpp and models</dt>
        <dd>{survey.dataDir}</dd>
        <dt>llama.cpp build</dt>
        <dd>
          {backend?.label}
          {survey.llamaInstalled === plan.backend
            ? " (already installed)"
            : survey.llamaTag
              ? ` · ${survey.llamaTag}`
              : ""}
        </dd>
        <dt>Model</dt>
        <dd>{model}</dd>
        <dt>Always-on power</dt>
        <dd>{onOff(plan.power)}</dd>
        <dt>Wake-on-LAN</dt>
        <dd>{onOff(plan.wakeOnLan)}</dd>
        <dt>Start with Windows</dt>
        <dd>{onOff(plan.startWithWindows)}</dd>
        <dt>Laya</dt>
        <dd>{onOff(plan.laya)}</dd>
        <dt>Download</dt>
        <dd>{review ? (review.sizes.download > 0 ? gb(review.sizes.download) : "Nothing new") : "..."}</dd>
        <dt>Disk space</dt>
        <dd>
          {review
            ? `${gb(review.sizes.disk)}${survey.diskFree !== null ? ` of ${gb(survey.diskFree)} free on ${survey.drive}` : ""}`
            : "..."}
        </dd>
      </dl>
    </>
  );
}

// ---------------------------------------------------------------- progress

const ROW_TONE: Record<RowState, string> = {
  pending: "off",
  active: "loading",
  done: "running",
  skipped: "off",
  failed: "error",
};

export function InstallPage({ run, title }: { run: Run; title: string }) {
  const [details, setDetails] = useState(false);
  const active = run.steps.find((s) => s.id === run.active);
  const last = run.log.at(-1);
  const b = run.bytes;
  const left = b && run.speed ? (b.total - b.done) / run.speed : NaN;
  return (
    <>
      <Heading title={title}>
        {active ? active.label : "Starting..."}
        {last && active && !last.startsWith(active.label) && <span className="lede-detail"> · {last}</span>}
      </Heading>
      {b && b.total > 0 && (
        <div className="download" aria-label="Download progress">
          <progress value={b.done} max={b.total} />
          <small>
            {gb(b.done)} of {gb(b.total)}
            {run.speed ? ` · ${speed(run.speed)} · ${timeLeft(left)}` : ""}
          </small>
        </div>
      )}
      <ol className="steps">
        {run.steps.map((s) => {
          const state = run.states[s.id] ?? "pending";
          return (
            <li key={s.id} className={`step ${state}`}>
              <span className={`dot ${ROW_TONE[state]}`} />
              <span>{s.label}</span>
              {state === "skipped" && <small>not needed</small>}
            </li>
          );
        })}
      </ol>
      <button className="link" onClick={() => setDetails((d) => !d)}>
        {details ? "Hide details" : "Show details"}
      </button>
      {details && (
        <pre className="log" ref={(el) => el?.scrollTo(0, el.scrollHeight)}>
          {run.log.join("\n")}
        </pre>
      )}
    </>
  );
}

// ---------------------------------------------------------------- finish

export function DonePage({ finished, log, onOpenLog }: { finished: Finished; log: string[]; onOpenLog: () => void }) {
  if (finished.ok) {
    return (
      <div className="done">
        <img className="hero-logo" src={logo} alt="" width={80} height={80} />
        <Heading title="You're all set">No Drama Llama is running. The model loads in the background.</Heading>
        <ul className="what">
          <li>
            <strong>System tray</strong>
            <span>Look for the llama by the clock (click ^ if it's hidden). Its dot shows the status.</span>
          </li>
          <li>
            <strong>Chat and API</strong>
            <span>
              <code>{finished.chatUrl}</code> is the chat page and an OpenAI-compatible API.
            </span>
          </li>
          <li>
            <strong>Games</strong>
            <span>Start one and the model stops so the game gets all of your VRAM. It comes back after you quit.</span>
          </li>
          <li>
            <strong>Shortcut</strong>
            <span>
              <code>Ctrl+Alt+L</code> turns it on or off.
            </span>
          </li>
        </ul>
      </div>
    );
  }
  if (finished.cancelled) {
    return (
      <>
        <Heading title="Setup stopped">Nothing is lost. Downloads pick up where they left off when you resume.</Heading>
      </>
    );
  }
  return (
    <>
      <Heading title="Setup didn't finish">Fix the problem below and try again. Finished downloads are kept.</Heading>
      <div className="banner error" role="alert">
        <span>{finished.error}</span>
      </div>
      <FailureTools log={log} logFile={finished.logFile} onOpenLog={onOpenLog} />
    </>
  );
}

function FailureTools({ log, logFile, onOpenLog }: { log: string[]; logFile: string | null; onOpenLog: () => void }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard
      ?.writeText(log.join("\n"))
      .then(() => setCopied(true))
      .catch(() => setCopied(false));
  };
  return (
    <div className="row-buttons">
      {logFile && <button onClick={onOpenLog}>Open setup log</button>}
      <button onClick={copy}>{copied ? "Copied" : "Copy details"}</button>
    </div>
  );
}

// ---------------------------------------------------------------- uninstall

export function UninstallConfirmPage({
  survey,
  plan,
  setPlan,
}: {
  survey: UninstallSurvey | null;
  plan: UninstallPlan;
  setPlan: Dispatch<SetStateAction<UninstallPlan>>;
}) {
  if (!survey) {
    return (
      <div className="checking">
        <span className="dot loading" />
        Looking at what's installed...
      </div>
    );
  }
  const modelBytes = survey.models.reduce((n, m) => n + m.size, 0);
  return (
    <>
      <Heading title="Uninstall No Drama Llama">Removes the app and puts your Windows settings back.</Heading>
      <ul className="what">
        <li>
          <strong>Removed</strong>
          <span>
            The app, llama.cpp, your settings and logs (<code>{survey.dataDir}</code>), the Start menu entry and the
            sign-in task.
          </span>
        </li>
        <li>
          <strong>Restored</strong>
          <span>Your original power plan settings and network adapter wake settings.</span>
        </li>
      </ul>
      <fieldset>
        {survey.models.length > 0 && (
          <Switch
            label={`Keep my models (${gb(modelBytes)})`}
            hint={
              <>
                Moves {survey.models.length === 1 ? "it" : `all ${survey.models.length}`} to{" "}
                <code>{survey.downloadsDir}</code> instead of deleting {survey.models.length === 1 ? "it" : "them"}.
              </>
            }
            checked={plan.keepModels}
            onChange={(v) => setPlan((p) => ({ ...p, keepModels: v }))}
          />
        )}
        {survey.autoSignIn && (
          <Switch
            label="Turn off automatic sign-in"
            hint="Windows signs in by itself at startup. Turn it off unless you set it up for something else."
            checked={plan.disableAutoSignIn}
            onChange={(v) => setPlan((p) => ({ ...p, disableAutoSignIn: v }))}
          />
        )}
        {survey.models.length === 0 && !survey.autoSignIn && (
          <p className="hint">Nothing to choose: no models are installed.</p>
        )}
      </fieldset>
    </>
  );
}

export function UninstallDonePage({ finished, onOpenLog }: { finished: Finished; onOpenLog: () => void }) {
  if (!finished.ok) {
    return (
      <>
        <Heading title={finished.cancelled ? "Uninstall stopped" : "Uninstall didn't finish"}>
          {finished.cancelled
            ? "Nothing was removed. Run it again from Settings → Apps."
            : "Some parts may be left. Fix the problem below and run it again from Settings → Apps."}
        </Heading>
        {finished.error && (
          <div className="banner error" role="alert">
            <span>{finished.error}</span>
          </div>
        )}
        {finished.logFile && (
          <div className="row-buttons">
            <button onClick={onOpenLog}>Open log</button>
          </div>
        )}
      </>
    );
  }
  return (
    <>
      <Heading title="No Drama Llama is removed">Windows settings are back the way they were.</Heading>
      <p>If you changed these BIOS settings for it, set them back by hand:</p>
      <ul className="bios">
        <li>Restore on AC Power Loss → Power Off (or Last State)</li>
        <li>ErP / EuP → Enabled</li>
        <li>Wake on LAN / Power On by PCI-E → Disabled</li>
      </ul>
    </>
  );
}
