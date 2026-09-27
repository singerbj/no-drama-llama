// The setup wizard: Welcome → System check → Model → Options → Review → Install → Finish,
// or, started from Settings → Apps, the uninstall wizard: Confirm → Uninstall → Finish.

import { useCallback, useEffect, useState } from "react";
import logo from "../../../design/assets/logo.svg";
import { connect, type SetupApi } from "./api";
import {
  ChecksPage,
  DonePage,
  InstallPage,
  ModelPage,
  OptionsPage,
  ReviewPage,
  UninstallConfirmPage,
  UninstallDonePage,
  WelcomePage,
} from "./pages";
import { applyProgress, startRun, type Run } from "./progress";
import type { Mode, Plan, Review, Survey, UninstallPlan, UninstallSurvey } from "./types";

const INSTALL_PAGES = [
  ["welcome", "Welcome"],
  ["checks", "System check"],
  ["model", "Model"],
  ["options", "Options"],
  ["review", "Review"],
  ["install", "Install"],
  ["done", "Finish"],
] as const;

const UNINSTALL_PAGES = [
  ["confirm", "Confirm"],
  ["install", "Uninstall"],
  ["done", "Finish"],
] as const;

type PageId = (typeof INSTALL_PAGES)[number][0] | (typeof UNINSTALL_PAGES)[number][0];

export function Setup() {
  const [api, setApi] = useState<SetupApi | null>(null);
  const [mode, setMode] = useState<Mode | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [page, setPage] = useState<PageId>("welcome");
  const [survey, setSurvey] = useState<Survey | null>(null);
  const [surveying, setSurveying] = useState(false);
  const [plan, setPlan] = useState<Plan | null>(null);
  // The installer's review of a plan, kept with the plan it's about.
  const [reviewed, setReviewed] = useState<{ plan: Plan; review: Review } | null>(null);
  const [uninstallSurvey, setUninstallSurvey] = useState<UninstallSurvey | null>(null);
  const [uninstallPlan, setUninstallPlan] = useState<UninstallPlan>({ keepModels: true, disableAutoSignIn: true });
  const [run, setRun] = useState<Run | null>(null);
  const [confirmStop, setConfirmStop] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runSurvey = useCallback((a: SetupApi) => {
    setSurveying(true);
    a.survey()
      .then((s) => {
        setSurvey(s);
        // Keep what the user already picked; start from the defaults the first time.
        setPlan((p) => p ?? s.defaults);
      })
      .catch((e: unknown) => setError(String(e)))
      .finally(() => setSurveying(false));
  }, []);

  useEffect(() => {
    let live = true;
    connect()
      .then(async (a) => {
        if (!live) return;
        setApi(a);
        a.onProgress((p) => setRun((r) => (r ? applyProgress(r, p, Date.now()) : r)));
        a.onCloseRequested(() => setConfirmStop(true));
        const m = await a.mode();
        if (!live) return;
        setMode(m);
        if (m === "uninstall") {
          setPage("confirm");
          const u = await a.uninstallSurvey();
          if (!live) return;
          setUninstallSurvey(u);
          setUninstallPlan({ keepModels: u.models.length > 0, disableAutoSignIn: u.autoSignIn });
        } else {
          // Probe the PC while the welcome page is read.
          runSurvey(a);
        }
      })
      .catch((e: unknown) => setProblem(e instanceof Error ? e.message : String(e)));
    return () => {
      live = false;
    };
  }, [runSurvey]);

  // The review page shows what the installer says about the plan.
  useEffect(() => {
    if (page !== "review" || !plan || !api) return;
    let live = true;
    api
      .review(plan)
      .then((review) => live && setReviewed({ plan, review }))
      .catch((e: unknown) => setError(String(e)));
    return () => {
      live = false;
    };
  }, [page, plan, api]);

  if (!mode) {
    return (
      <div className="setup-loading">
        <img src={logo} alt="" width={56} height={56} />
        <p>{problem ?? "Starting setup..."}</p>
      </div>
    );
  }

  const review = reviewed && reviewed.plan === plan ? reviewed.review : null;
  // The last page follows the install page as soon as the install ends.
  const current: PageId = page === "install" && run?.finished ? "done" : page;
  const pages = mode === "install" ? INSTALL_PAGES : UNINSTALL_PAGES;
  const index = pages.findIndex(([id]) => id === current);
  const running = run !== null && run.finished === null;

  const startInstall = () => {
    if (!plan || !review || !api) return;
    setError(null);
    setRun(startRun(review.steps));
    setPage("install");
    api.install(plan).catch((e: unknown) => {
      setRun(null);
      setPage("review");
      setError(String(e));
    });
  };

  const startUninstall = () => {
    if (!api) return;
    setError(null);
    setRun(
      startRun([
        { id: "stop", label: "Stopping the app and the model server" },
        { id: "restore", label: "Restoring your power and network settings" },
        { id: "files", label: "Removing files" },
      ]),
    );
    setPage("install");
    api.uninstall(uninstallPlan).catch((e: unknown) => {
      setRun(null);
      setPage("confirm");
      setError(String(e));
    });
  };

  const stop = () => {
    void api?.cancel();
    setConfirmStop(false);
  };

  const recheck = () => api && runSurvey(api);
  const close = () => void api?.close();
  const openChat = () => void api?.openChat();
  const openLog = () => void api?.openLog();

  // ---------------------------------------------------------------- footer buttons

  const blockedChecks = survey?.checks.some((c) => c.status === "fail") ?? false;
  const selectedTooBig =
    plan?.model.kind === "download" &&
    survey?.catalog.find((c) => plan.model.kind === "download" && c.id === plan.model.id)?.fit === "tooBig";

  let next: { label: string; onClick: () => void; disabled?: boolean; danger?: boolean } | null = null;
  const go = (id: PageId) => () => setPage(id);
  switch (current) {
    case "welcome":
      next = { label: "Next", onClick: go("checks") };
      break;
    case "checks":
      next = { label: "Next", onClick: go("model"), disabled: !survey || surveying || blockedChecks };
      break;
    case "model":
      next = { label: "Next", onClick: go("options"), disabled: !plan || selectedTooBig };
      break;
    case "options":
      next = { label: "Next", onClick: go("review") };
      break;
    case "review":
      next = {
        label: survey?.existing ? "Upgrade" : "Install",
        onClick: startInstall,
        disabled: !review || review.problems.length > 0,
      };
      break;
    case "confirm":
      next = { label: "Uninstall", onClick: startUninstall, disabled: !uninstallSurvey, danger: true };
      break;
  }
  const canGoBack = index > 0 && current !== "install" && current !== "done";

  return (
    <div className="setup">
      <aside className="rail">
        <div className="brand">
          <img src={logo} alt="" width={40} height={40} />
          <div>
            <strong>No Drama Llama</strong>
            <small>
              {[mode === "install" ? "Setup" : "Uninstall", survey?.version ?? uninstallSurvey?.version]
                .filter(Boolean)
                .join(" · ")}
            </small>
          </div>
        </div>
        <ol className="rail-steps">
          {pages.map(([id, title], i) => (
            <li
              key={id}
              className={i === index ? "current" : i < index ? "past" : undefined}
              aria-current={i === index ? "step" : undefined}
            >
              <span className="rail-mark">{i < index ? "✓" : i + 1}</span>
              {title}
            </li>
          ))}
        </ol>
      </aside>

      <div className="content">
        <div className="page">
          {error && (
            <div className="banner error" role="alert">
              <span>{error}</span>
              <button onClick={() => setError(null)}>Dismiss</button>
            </div>
          )}
          {current === "welcome" && <WelcomePage survey={survey} />}
          {current === "checks" && <ChecksPage survey={survey} checking={surveying} onRecheck={recheck} />}
          {current === "model" && survey && plan && <ModelPage survey={survey} plan={plan} setPlan={setPlan} />}
          {current === "options" && survey && plan && <OptionsPage survey={survey} plan={plan} setPlan={setPlan} />}
          {current === "review" && survey && plan && <ReviewPage survey={survey} plan={plan} review={review} />}
          {current === "confirm" && (
            <UninstallConfirmPage survey={uninstallSurvey} plan={uninstallPlan} setPlan={setUninstallPlan} />
          )}
          {current === "install" && run && (
            <InstallPage run={run} title={mode === "install" ? "Installing" : "Uninstalling"} />
          )}
          {current === "done" &&
            run?.finished &&
            (mode === "install" ? (
              <DonePage finished={run.finished} log={run.log} onOpenLog={openLog} />
            ) : (
              <UninstallDonePage finished={run.finished} onOpenLog={openLog} />
            ))}
        </div>

        <footer className="wizard-footer">
          {confirmStop && running ? (
            <>
              <span className="footer-note">
                {mode === "install"
                  ? "Stop installing? Downloads pick up where they left off next time."
                  : "Stop uninstalling? It stops before removing files."}
              </span>
              <button onClick={() => setConfirmStop(false)}>Keep going</button>
              <button className="danger" onClick={stop}>
                Stop
              </button>
            </>
          ) : current === "install" ? (
            <>
              <span className="footer-note">
                {run?.finished === null && run.active === "files" ? "Removing files..." : ""}
              </span>
              <button disabled={!running || run?.active === "files"} onClick={() => setConfirmStop(true)}>
                Stop
              </button>
            </>
          ) : current === "done" ? (
            <>
              <span className="footer-note" />
              {mode === "install" && run?.finished && !run.finished.ok && (
                <button onClick={startInstall}>{run.finished.cancelled ? "Resume" : "Try again"}</button>
              )}
              {mode === "install" && run?.finished?.ok && <button onClick={openChat}>Open chat</button>}
              <button className="primary" onClick={close}>
                {run?.finished?.ok ? "Finish" : "Close"}
              </button>
            </>
          ) : (
            <>
              <button onClick={close}>Cancel</button>
              <span className="footer-note" />
              <button disabled={!canGoBack} onClick={() => setPage(pages[index - 1]![0])}>
                Back
              </button>
              {next && (
                <button
                  className={next.danger ? "danger-fill" : "primary"}
                  disabled={next.disabled}
                  onClick={next.onClick}
                >
                  {next.label}
                </button>
              )}
            </>
          )}
        </footer>
      </div>
    </div>
  );
}
