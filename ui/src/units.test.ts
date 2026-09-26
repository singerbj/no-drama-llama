import { afterEach, describe, expect, it, vi } from "vitest";
import { parseNumber } from "./fields";

describe("parseNumber", () => {
  it("accepts numbers in range", () => {
    expect(parseNumber("8080", 1024, 65535, true)).toBe(8080);
    expect(parseNumber(" 1.5 ", 0.1, 256)).toBe(1.5);
    expect(parseNumber("0", 0, 3600, true)).toBe(0);
  });

  it.each(["", "  ", "80", "70000", "12.5", "abc", "1e400", "NaN", "Infinity"])("rejects %j", (t) => {
    expect(parseNumber(t, 1024, 65535, true)).toBeUndefined();
  });
});

describe("the Tauri connection", () => {
  afterEach(() => {
    delete (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    vi.resetModules();
  });

  it("sends requests and reads state through the window's commands", async () => {
    const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>(async (cmd) =>
      cmd === "state" ? { statusText: "Off" } : undefined,
    );
    const listen = vi.fn<(event: string) => Promise<() => void>>(async () => () => {});
    vi.doMock("@tauri-apps/api/core", () => ({ invoke }));
    vi.doMock("@tauri-apps/api/event", () => ({ listen }));
    (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    const { connect } = await import("./api");
    const api = await connect();
    await api.send({ cmd: "toggle" });
    expect(invoke).toHaveBeenCalledWith("send", { request: { cmd: "toggle" } });
    expect(await api.state()).toEqual({ statusText: "Off" });
    api.onState(() => {});
    api.onSaved(() => {});
    expect(listen.mock.calls.map((c) => c[0])).toEqual(["state", "saved"]);
  });
});
