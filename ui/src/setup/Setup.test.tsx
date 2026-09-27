import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { button, cleanup, click, render, waitFor } from "../test-utils";
import type { MockOptions } from "./mock";
import { Setup } from "./Setup";
import type { Plan } from "./types";

// The installer: the simulated PC and install from mock.ts, with fast timers.
const env = vi.hoisted(() => ({
  options: {} as MockOptions,
  api: null as null | { calls: string[]; requestClose: () => void },
}));
vi.mock("./api", async () => {
  const { mockSetupApi } = await import("./mock");
  return {
    connect: async () => {
      const api = mockSetupApi({ tick: 1, ...env.options });
      env.api = api as unknown as typeof env.api;
      return api;
    },
  };
});

const calls = () => env.api!.calls;
const heading = () => document.querySelector("h1")?.textContent;
const next = () => button("Next");
const installs = () =>
  calls()
    .filter((c) => c.startsWith("install "))
    .map((c) => JSON.parse(c.slice(8)) as Plan);
const radio = (text: string) =>
  [...document.querySelectorAll<HTMLLabelElement>(".choice")]
    .find((l) => l.textContent?.includes(text))
    ?.querySelector("input") as HTMLInputElement;
const toggle = (label: string) =>
  [...document.querySelectorAll<HTMLLabelElement>("label.row")]
    .find((l) => l.textContent?.startsWith(label))
    ?.querySelector("input") as HTMLInputElement;

async function start(options: MockOptions = {}) {
  env.options = options;
  await render(<Setup />);
  await waitFor(() => expect(document.querySelector(".rail")).not.toBeNull());
}

async function toReview() {
  await click(next());
  await waitFor(() => expect(next().disabled).toBe(false));
  await click(next()); // model
  await click(next()); // options
  await click(next()); // review
  await waitFor(() => expect(button("Install").disabled).toBe(false));
}

beforeEach(() => {
  env.options = {};
  env.api = null;
});
afterEach(cleanup);

describe("installing", () => {
  it("walks from the welcome page to a running app", async () => {
    await start();
    expect(heading()).toBe("Set up No Drama Llama");
    await click(next());
    expect(heading()).toBe("System check");
    await waitFor(() => expect(document.querySelectorAll(".check")).toHaveLength(8));
    expect(document.querySelector(".banner")?.textContent).toContain("One thing is worth a look");
    await click(next());

    expect(heading()).toBe("Pick a model");
    expect(radio("UD-IQ3_XXS").checked).toBe(true); // the recommendation
    await click(radio("UD-Q2_K_XL"));
    await click(next());

    expect(heading()).toBe("Options");
    await click(toggle("Always-on power settings"));
    await click(next());

    expect(heading()).toBe("Ready to install");
    await waitFor(() => expect(document.querySelector(".facts")?.textContent).toContain("Download10.4 GB"));
    expect(document.querySelector(".facts")?.textContent).toContain("Always-on powerOff");
    await click(button("Install"));

    await waitFor(() => expect(heading()).toBe("You're all set"), 5000);
    expect(installs()).toEqual([
      {
        model: { kind: "download", id: "qwen3.8-27b:UD-Q2_K_XL" },
        backend: "cuda13",
        power: false,
        wakeOnLan: true,
        startWithWindows: true,
        laya: false,
        crashReports: false,
        usageStats: false,
      },
    ]);
    expect(document.querySelector(".done code")?.textContent).toBe("http://127.0.0.1:8080");
    await click(button("Open chat"));
    await click(button("Finish"));
    expect(calls()).toContain("open_chat");
    expect(calls()).toContain("close");
  });

  it("goes back without losing choices", async () => {
    await start();
    await click(next());
    await waitFor(() => expect(next().disabled).toBe(false));
    await click(next());
    await click(radio("UD-IQ2_XXS"));
    await click(button("Back"));
    expect(heading()).toBe("System check");
    await click(next());
    expect(radio("UD-IQ2_XXS").checked).toBe(true);
  });

  it("won't continue past a failed check, and checks again", async () => {
    const { sampleSurvey } = await import("./mock");
    const survey = sampleSurvey();
    survey.checks[5] = { id: "folder", title: "C:\\LLM", status: "fail", detail: "It's a link." };
    await start({ survey });
    await click(next());
    await waitFor(() => expect(document.querySelectorAll(".check.fail")).toHaveLength(1));
    expect(next().disabled).toBe(true);
    expect(document.querySelector(".banner")?.textContent).toContain("Fix the red item");
    await click(button("Check again"));
    await waitFor(() => expect(calls().filter((c) => c === "survey")).toHaveLength(2));
  });

  it("hides models that are too big until asked, and never selects one", async () => {
    await start();
    await click(next());
    await waitFor(() => expect(next().disabled).toBe(false));
    await click(next());
    expect(radio("Q8_0")).toBeUndefined();
    await click(button("Show 3 models too big for this PC"));
    expect(radio("Q8_0").disabled).toBe(true);
    expect(document.querySelector(".disk")?.textContent).toContain("10.9 GB of 412 GB free");
  });

  it("offers to keep the current model on an upgrade", async () => {
    const { sampleSurvey } = await import("./mock");
    const survey = sampleSurvey();
    survey.existing = { version: "0.0.1", model: "Qwen3.8-27B-UD-Q4_K_XL.gguf", models: [] };
    survey.defaults.model = { kind: "keep" };
    await start({ survey });
    expect(heading()).toBe("Upgrade No Drama Llama");
    expect(document.querySelector(".banner")?.textContent).toContain("Version 0.0.1 is installed");
    await click(next());
    await waitFor(() => expect(next().disabled).toBe(false));
    await click(next());
    expect(radio("Keep my current model").checked).toBe(true);
    await click(next());
    await click(next());
    await waitFor(() => expect(button("Upgrade").disabled).toBe(false));
    expect(document.querySelector(".facts")?.textContent).toContain("Keep Qwen3.8-27B-UD-Q4_K_XL.gguf");
  });

  it("shows why the plan can't be installed", async () => {
    const { sampleSurvey } = await import("./mock");
    const survey = sampleSurvey();
    survey.diskFree = 12e9;
    await start({ survey });
    await click(next());
    await waitFor(() => expect(next().disabled).toBe(false));
    await click(next());
    expect(document.querySelector(".disk.short")).not.toBeNull();
    await click(next());
    await click(next());
    await waitFor(() => expect(document.querySelector(".banner.error")?.textContent).toContain("Free up space"));
    expect(button("Install").disabled).toBe(true);
  });

  it("reports a failure and tries again", async () => {
    await start({ failAt: { step: "llamaCpp", error: "GET https://api.github.com: timed out" } });
    await toReview();
    await click(button("Install"));
    await waitFor(() => expect(heading()).toBe("Setup didn't finish"), 5000);
    expect(document.querySelector(".banner.error")?.textContent).toContain("timed out");
    await click(button("Try again"));
    await waitFor(() => expect(installs()).toHaveLength(2));
  });

  it("stops when asked, after confirming", async () => {
    await start({ tick: 20 });
    await toReview();
    await click(button("Install"));
    await click(button("Stop"));
    expect(document.querySelector(".wizard-footer")?.textContent).toContain("Downloads pick up where they left off");
    await click(button("Keep going"));
    expect(calls()).not.toContain("cancel");
    await click(button("Stop"));
    await click(button("Stop"));
    expect(calls()).toContain("cancel");
    await waitFor(() => expect(heading()).toBe("Setup stopped"), 5000);
    expect(button("Resume")).toBeTruthy();
  });

  it("asks before the window closes mid-install", async () => {
    await start({ tick: 20 });
    await toReview();
    await click(button("Install"));
    await waitFor(() => expect(document.querySelector(".step.active")).not.toBeNull());
    const { act } = await import("react");
    await act(async () => env.api!.requestClose());
    expect(button("Keep going")).toBeTruthy();
  });
});

describe("uninstalling", () => {
  it("keeps the models when asked and lists what to undo in the BIOS", async () => {
    await start({ mode: "uninstall" });
    await waitFor(() => expect(heading()).toBe("Uninstall No Drama Llama"));
    expect(toggle("Keep my models (26.7 GB)").checked).toBe(true);
    expect(toggle("Turn off automatic sign-in").checked).toBe(true);
    await click(toggle("Turn off automatic sign-in"));
    await click(button("Uninstall"));
    await waitFor(() => expect(heading()).toBe("No Drama Llama is removed"), 5000);
    expect(calls()).toContain('uninstall {"keepModels":true,"disableAutoSignIn":false}');
    expect(document.querySelector(".bios")?.textContent).toContain("ErP / EuP");
  });
});
