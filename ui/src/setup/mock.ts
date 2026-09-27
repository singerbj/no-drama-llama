// A simulated PC and install for `npm run dev` (open /setup.html) and the tests. Never part of
// the build Tauri serves.

import type { CatalogEntry } from "../types";
import type { SetupApi } from "./api";
import type { Mode, Plan, Progress, Review, StepId, StepInfo, Survey, UninstallPlan, UninstallSurvey } from "./types";

const GB = 1e9;
const GIB = 1024 ** 3;

const LABELS: Record<StepId, string> = {
  stop: "Stopping any running copy",
  app: "Installing the app",
  folders: "Setting up C:\\LLM",
  hardware: "Checking your hardware",
  llamaCpp: "Downloading llama.cpp",
  model: "Downloading the model",
  laya: "Installing Laya (Ollaya)",
  settings: "Power and network settings",
  register: "Start menu, Apps & features, start at sign-in",
  start: "Starting No Drama Llama",
  restore: "Restoring your power and network settings",
  files: "Removing files",
};

const step = (id: StepId): StepInfo => ({ id, label: LABELS[id] });

const catalog: CatalogEntry[] = (
  [
    ["qwen3.8-27b:Q8_0", "Q8_0", 29.0, "tooBig", "too big for this PC", false],
    ["qwen3.8-27b:UD-Q6_K_XL", "UD-Q6_K_XL", 25.3, "gpuAndRam", "partly in RAM - slow", false],
    ["qwen3.8-27b:UD-Q5_K_XL", "UD-Q5_K_XL", 20.9, "gpuAndRam", "partly in RAM - slow", false],
    ["qwen3.8-27b:UD-Q4_K_XL", "UD-Q4_K_XL", 17.6, "gpuAndRam", "partly in RAM - slow", false],
    ["qwen3.8-27b:UD-IQ4_XS", "UD-IQ4_XS", 14.3, "gpuAndRam", "partly in RAM - slow", false],
    ["qwen3.8-27b:UD-Q3_K_XL", "UD-Q3_K_XL", 13.1, "gpuAndRam", "partly in RAM - slow", false],
    ["qwen3.8-27b:UD-IQ3_XXS", "UD-IQ3_XXS", 10.9, "gpu", "fits your GPU", true],
    ["qwen3.8-27b:UD-Q2_K_XL", "UD-Q2_K_XL", 9.8, "gpu", "fits your GPU", false],
    ["qwen3.8-27b:UD-IQ2_XXS", "UD-IQ2_XXS", 7.3, "gpu", "fits your GPU", false],
    ["qwen3.8-flash-next:UD-Q4_K_XL", "UD-Q4_K_XL", 111.3, "tooBig", "too big for this PC", false],
    ["qwen3.8-flash-next:UD-IQ1_S", "UD-IQ1_S", 72.5, "tooBig", "too big for this PC", false],
  ] as const
).map(([id, quant, gb, fit, note, recommended]) => {
  const family = id.startsWith("qwen3.8-flash") ? "Qwen 3.8 Flash-Next (125B MoE)" : "Qwen 3.8 27B";
  return {
    id,
    label: `${family} ${quant}  (${gb} GB)`,
    file: `${id}.gguf`,
    size: gb * GB,
    fit,
    note,
    recommended,
    installed: false,
    canDownload: fit !== "tooBig",
  };
});

export function sampleSurvey(): Survey {
  return {
    version: "0.1.0",
    existing: null,
    gpu: "NVIDIA GeForce RTX 4070 (12 GB)",
    vram: 12 * GIB,
    ram: 32 * GIB,
    laptop: false,
    backends: [
      { id: "vulkan", label: "Vulkan", available: true, download: 48e6, note: "Any GPU: AMD, Intel or NVIDIA" },
      {
        id: "cuda12",
        label: "CUDA 12",
        available: true,
        download: 590e6,
        note: "NVIDIA GTX 10 series or newer, driver 528+",
      },
      {
        id: "cuda13",
        label: "CUDA 13",
        available: true,
        download: 620e6,
        note: "NVIDIA RTX 20 series or newer, driver 580+",
      },
    ],
    recommendedBackend: "cuda13",
    llamaTag: "b11222",
    llamaInstalled: null,
    layaInstalled: false,
    catalog: structuredClone(catalog),
    drive: "C:",
    diskFree: 412 * GB,
    checks: [
      { id: "windows", title: "Windows 11 (build 26100)", status: "pass", detail: "Supported." },
      {
        id: "gpu",
        title: "NVIDIA GeForce RTX 4070 (12 GB)",
        status: "pass",
        detail: "CUDA 13 build · 12 GB of video memory.",
      },
      {
        id: "ram",
        title: "32 GB of memory",
        status: "pass",
        detail: "Enough for Windows, your games and the model's overflow.",
      },
      { id: "disk", title: "412 GB free on C:", status: "pass", detail: "Room for the app and a model." },
      {
        id: "network",
        title: "Internet connection",
        status: "pass",
        detail: "GitHub (llama.cpp) and Hugging Face (models) are reachable.",
      },
      {
        id: "folder",
        title: "C:\\LLM",
        status: "pass",
        detail: "Will hold llama.cpp, your models and settings.",
      },
      {
        id: "port",
        title: "Port 8080 is in use",
        status: "warn",
        detail: "Another program uses it. After installing, pick another port in Settings → Server & API.",
      },
      { id: "power", title: "Desktop PC", status: "pass", detail: "Can stay on as a local AI server." },
    ],
    defaults: {
      model: { kind: "download", id: "qwen3.8-27b:UD-IQ3_XXS" },
      backend: "cuda13",
      power: true,
      wakeOnLan: true,
      startWithWindows: true,
      laya: false,
      crashReports: false,
      usageStats: false,
    },
    askPrivacy: true,
    installDir: "C:\\Program Files\\No Drama Llama",
    dataDir: "C:\\LLM",
  };
}

/** What the installer would decide (src/installer.rs), close enough for trying the page out. */
export function mockReview(plan: Plan, s: Survey): Review {
  const model =
    plan.model.kind === "download" ? (s.catalog.find((c) => c.id === (plan.model as { id: string }).id) ?? null) : null;
  const llama =
    s.llamaInstalled === plan.backend ? 0 : (s.backends.find((b) => b.id === plan.backend)?.download ?? 700e6);
  const laya = plan.laya && !s.layaInstalled ? 1.3e9 : 0;
  const modelBytes = model && !model.installed ? model.size : 0;
  const sizes = { download: modelBytes + llama + laya, disk: modelBytes + 3 * llama + 2 * laya };
  const problems: string[] = [];
  if (s.checks.some((c) => c.status === "fail")) problems.push("Fix the failed system checks first.");
  if (model?.fit === "tooBig") problems.push(`${model.label} is too big for this PC. Pick a smaller one.`);
  if (plan.model.kind === "keep" && !s.existing?.model) problems.push("There's no current model to keep.");
  if (!s.backends.find((b) => b.id === plan.backend)?.available)
    problems.push(`This PC can't run the ${plan.backend} build.`);
  if (s.diskFree !== null && sizes.disk + 2e9 > s.diskFree)
    problems.push(
      `Needs ${Math.ceil((sizes.disk + 2e9) / 1e9)} GB free on ${s.drive}. Free up space or pick a smaller model.`,
    );
  const steps: StepId[] = ["stop", "app", "folders", "hardware"];
  if (s.llamaInstalled !== plan.backend) steps.push("llamaCpp");
  if (plan.model.kind === "download") steps.push("model");
  if (plan.laya) steps.push("laya");
  steps.push("settings", "register", "start");
  return { problems, steps: steps.map(step), sizes };
}

export interface MockOptions {
  mode?: Mode;
  survey?: Survey;
  /** Milliseconds between simulated events */
  tick?: number;
  /** Fail at this step with this message */
  failAt?: { step: StepId; error: string };
}

export function mockSetupApi(opts: MockOptions = {}): SetupApi & { calls: string[] } {
  const tick = opts.tick ?? 120;
  const survey = opts.survey ?? sampleSurvey();
  const calls: string[] = [];
  let onProgress: ((p: Progress) => void) | null = null;
  let onClose: (() => void) | null = null;
  let cancelled = false;
  const wait = () => new Promise((r) => setTimeout(r, tick));
  const emit = (p: Progress) => onProgress?.(p);

  async function run(steps: StepInfo[], downloads: Partial<Record<StepId, number>>) {
    cancelled = false;
    const finish = (ok: boolean, error: string | null = null) =>
      emit({
        type: "finished",
        ok,
        cancelled: !ok && cancelled,
        error,
        chatUrl: ok && opts.mode !== "uninstall" ? "http://127.0.0.1:8080" : null,
        logFile: "C:\\LLM\\data\\setup.log",
      });
    for (const s of steps) {
      if (cancelled) return finish(false);
      emit({ type: "step", id: s.id });
      emit({ type: "log", text: `${s.label}...` });
      // oxlint-disable-next-line no-await-in-loop -- a simulated install is sequential
      await wait();
      if (opts.failAt?.step === s.id) return finish(false, opts.failAt.error);
      const total = downloads[s.id];
      if (total) {
        for (let i = 1; i <= 8; i++) {
          if (cancelled) return finish(false);
          emit({ type: "bytes", done: (total * i) / 8, total });
          // oxlint-disable-next-line no-await-in-loop -- sequential
          await wait();
        }
        emit({ type: "log", text: "SHA-256 verified." });
      }
    }
    finish(true);
  }

  return {
    calls,
    mode: async () => opts.mode ?? "install",
    survey: async () => {
      calls.push("survey");
      await wait();
      return structuredClone(survey);
    },
    uninstallSurvey: async () => {
      calls.push("uninstall_survey");
      return {
        version: "0.1.0",
        models: [
          { name: "Qwen3.8-27B-UD-Q4_K_XL.gguf", size: 17.6 * GB },
          { name: "my-own-model.Q5_K_M.gguf", size: 9.1 * GB },
        ],
        autoSignIn: true,
        dataDir: "C:\\LLM",
        downloadsDir: "C:\\Users\\you\\Downloads",
      } satisfies UninstallSurvey;
    },
    review: async (plan) => mockReview(plan, survey),
    install: async (plan) => {
      calls.push(`install ${JSON.stringify(plan)}`);
      const r = mockReview(plan, survey);
      const model =
        plan.model.kind === "download"
          ? (survey.catalog.find((c) => c.id === (plan.model as { id: string }).id)?.size ?? 0)
          : 0;
      void run(r.steps, {
        llamaCpp: survey.backends.find((b) => b.id === plan.backend)?.download ?? 0,
        model,
        laya: 1.3e9,
      });
    },
    uninstall: async (plan: UninstallPlan) => {
      calls.push(`uninstall ${JSON.stringify(plan)}`);
      void run([step("stop"), step("restore"), step("files")], {});
    },
    cancel: async () => {
      calls.push("cancel");
      cancelled = true;
    },
    openChat: async () => void calls.push("open_chat"),
    openLog: async () => void calls.push("open_log"),
    close: async () => void calls.push("close"),
    onProgress: (f) => (onProgress = f),
    onCloseRequested: (f) => (onClose = f),
    /** For the tests: the window's close button. */
    requestClose: () => onClose?.(),
  } as SetupApi & { calls: string[]; requestClose: () => void };
}
