import { describe, expect, it, vi } from "vitest";
import { gb, speed, timeLeft } from "./format";
import { applyProgress, startRun } from "./progress";
import type { Progress, StepInfo } from "./types";

const steps: StepInfo[] = [
  { id: "stop", label: "Stop" },
  { id: "llamaCpp", label: "llama.cpp" },
  { id: "model", label: "Model" },
  { id: "start", label: "Start" },
];

function play(events: Progress[], times?: number[]) {
  return events.reduce((r, p, i) => applyProgress(r, p, times?.[i] ?? 0), startRun(steps));
}

describe("install progress", () => {
  it("marks steps done, running and skipped", () => {
    const r = play([
      { type: "step", id: "stop" },
      { type: "step", id: "model" },
    ]);
    expect(r.states).toEqual({ stop: "done", llamaCpp: "skipped", model: "active", start: "pending" });
    const done = applyProgress(
      r,
      { type: "finished", ok: true, cancelled: false, error: null, chatUrl: "x", logFile: null },
      0,
    );
    expect(done.states).toEqual({ stop: "done", llamaCpp: "skipped", model: "done", start: "skipped" });
  });

  it("marks the running step failed or stopped", () => {
    const r = play([{ type: "step", id: "llamaCpp" }]);
    const failed = applyProgress(
      r,
      { type: "finished", ok: false, cancelled: false, error: "boom", chatUrl: null, logFile: null },
      0,
    );
    expect(failed.states.llamaCpp).toBe("failed");
    const stopped = applyProgress(
      r,
      { type: "finished", ok: false, cancelled: true, error: null, chatUrl: null, logFile: null },
      0,
    );
    expect(stopped.states.llamaCpp).toBe("skipped");
  });

  it("shows a step the plan didn't list after the running one", () => {
    const r = play([
      { type: "step", id: "stop" },
      { type: "step", id: "laya" },
    ]);
    expect(r.steps.map((s) => s.id)).toEqual(["stop", "laya", "llamaCpp", "model", "start"]);
    expect(r.states.laya).toBe("active");
  });

  it("measures download speed and resets it for the next file", () => {
    const r = play(
      [
        { type: "step", id: "model" },
        { type: "bytes", done: 0, total: 100e6 },
        { type: "bytes", done: 10e6, total: 100e6 },
        { type: "bytes", done: 30e6, total: 100e6 },
      ],
      [0, 0, 1000, 2000],
    );
    expect(r.speed).toBeCloseTo(13e6);
    const next = applyProgress(r, { type: "bytes", done: 1e6, total: 5e6 }, 2100);
    expect(next.speed).toBeNull();
  });

  it("keeps the log to a few hundred lines", () => {
    const r = play(Array.from({ length: 500 }, (_, i) => ({ type: "log", text: `line ${i}` }) as Progress));
    expect(r.log).toHaveLength(400);
    expect(r.log.at(-1)).toBe("line 499");
  });
});

describe("formatting", () => {
  it("writes sizes, speeds and time left", () => {
    expect(gb(17.6e9)).toBe("17.6 GB");
    expect(gb(412e9)).toBe("412 GB");
    expect(gb(9.8e9)).toBe("9.8 GB");
    expect(gb(48e6)).toBe("48 MB");
    expect(speed(45.3e6)).toBe("45.3 MB/s");
    expect(timeLeft(30)).toBe("less than a minute left");
    expect(timeLeft(300)).toBe("5 min left");
    expect(timeLeft(4000)).toBe("1 h 7 min left");
    expect(timeLeft(NaN)).toBe("");
  });
});

describe("the Tauri connection", () => {
  it("calls the wizard's commands and listens for progress", async () => {
    const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>(async (cmd) =>
      cmd === "mode" ? "uninstall" : undefined,
    );
    const listen = vi.fn<(event: string) => Promise<() => void>>(async () => () => {});
    vi.doMock("@tauri-apps/api/core", () => ({ invoke }));
    vi.doMock("@tauri-apps/api/event", () => ({ listen }));
    (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    try {
      const { connect } = await import("./api");
      const api = await connect();
      expect(await api.mode()).toBe("uninstall");
      await api.uninstall({ keepModels: true, disableAutoSignIn: false });
      expect(invoke).toHaveBeenCalledWith("uninstall", { plan: { keepModels: true, disableAutoSignIn: false } });
      await api.uninstallSurvey();
      expect(invoke).toHaveBeenCalledWith("uninstall_survey");
      api.onProgress(() => {});
      api.onCloseRequested(() => {});
      expect(listen.mock.calls.map((c) => c[0])).toEqual(["progress", "close_requested"]);
    } finally {
      delete (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
      vi.resetModules();
    }
  });
});
