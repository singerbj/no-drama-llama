// Follows the installer's progress events: which step is running, which finished or were
// skipped, the current download's speed, and the log. Pure, so it's easy to test.

import type { Finished, Progress, StepId, StepInfo } from "./types";

export type RowState = "pending" | "active" | "done" | "skipped" | "failed";

export interface Run {
  steps: StepInfo[];
  states: Partial<Record<StepId, RowState>>;
  active: StepId | null;
  bytes: { done: number; total: number } | null;
  /** Bytes per second, smoothed */
  speed: number | null;
  /** The last bytes sample, for the speed */
  sample: { at: number; done: number } | null;
  log: string[];
  finished: Finished | null;
}

const MAX_LOG = 400;

export function startRun(steps: StepInfo[]): Run {
  return {
    steps,
    states: Object.fromEntries(steps.map((s) => [s.id, "pending"])),
    active: null,
    bytes: null,
    speed: null,
    sample: null,
    log: [],
    finished: null,
  };
}

/** Settles every step before `upTo` (all steps if -1): the running one is done, the rest were skipped. */
function settle(run: Run, upTo: number): Run["states"] {
  const states = { ...run.states };
  run.steps.forEach((s, i) => {
    if (upTo >= 0 && i >= upTo) return;
    if (states[s.id] === "active") states[s.id] = "done";
    else if (states[s.id] === "pending") states[s.id] = "skipped";
  });
  return states;
}

export function applyProgress(run: Run, p: Progress, now: number): Run {
  switch (p.type) {
    case "step": {
      let steps = run.steps;
      let i = steps.findIndex((s) => s.id === p.id);
      if (i < 0) {
        // A step the plan didn't list: show it after the one running now.
        const after = steps.findIndex((s) => s.id === run.active) + 1;
        steps = [...steps.slice(0, after), { id: p.id, label: p.id }, ...steps.slice(after)];
        i = after;
      }
      const states = settle({ ...run, steps }, i);
      states[p.id] = "active";
      return { ...run, steps, states, active: p.id, bytes: null, speed: null, sample: null };
    }
    case "bytes": {
      const fresh = !run.bytes || run.bytes.total !== p.total || p.done < run.bytes.done;
      let speed = fresh ? null : run.speed;
      let sample = fresh || !run.sample ? { at: now, done: p.done } : run.sample;
      if (!fresh && run.sample && now - run.sample.at >= 500) {
        const current = ((p.done - run.sample.done) * 1000) / (now - run.sample.at);
        speed = speed === null ? current : speed * 0.7 + current * 0.3;
        sample = { at: now, done: p.done };
      }
      return { ...run, bytes: { done: p.done, total: p.total }, speed, sample };
    }
    case "log": {
      const log = run.log.length >= MAX_LOG ? run.log.slice(-MAX_LOG + 1) : run.log.slice();
      log.push(p.text);
      return { ...run, log };
    }
    case "finished": {
      const { type: _, ...finished } = p;
      let states: Run["states"];
      if (finished.ok) {
        states = settle(run, -1);
      } else {
        states = { ...run.states };
        if (run.active) states[run.active] = finished.cancelled ? "skipped" : "failed";
      }
      return { ...run, states, finished, bytes: finished.ok ? null : run.bytes };
    }
  }
}
