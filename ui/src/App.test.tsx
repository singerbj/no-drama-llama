import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { button, cleanup, click, control, render, showTab, type, waitFor } from "./test-utils";
import type { Request } from "./types";

// The tray: the sample data from mock.ts, recording every request the page sends.
const sent = vi.hoisted(() => [] as Request[]);
vi.mock("./api", async () => {
  const { mockApi } = await import("./mock");
  return {
    connect: async () => {
      const api = mockApi();
      const send = api.send.bind(api);
      return {
        ...api,
        send: async (r: Request) => {
          sent.push(r);
          await send(r);
        },
      };
    },
  };
});

const savebar = () => document.querySelector(".savebar");
const facts = () => document.querySelector("section:not([hidden]) .facts")?.textContent;
const saves = () => sent.filter((r) => r.cmd === "save");

beforeEach(async () => {
  sent.length = 0;
  vi.spyOn(console, "info").mockImplementation(() => {});
  localStorage.clear();
  await render(<App />);
  await waitFor(() => {
    if (!document.querySelector("nav")) throw new Error("the page didn't connect");
  });
});

afterEach(cleanup);

describe("status header", () => {
  it("shows the tray's status and sends on/off", async () => {
    expect(document.querySelector(".status strong")?.textContent).toMatch(/^Running/);
    expect(document.querySelector(".dot")?.className).toBe("dot running");
    await click(button("Turn off"));
    expect(sent).toContainEqual({ cmd: "toggle" });
    await waitFor(() => expect(button("Turn on").disabled).toBe(false));
    expect(button("Open chat").disabled).toBe(true);
  });
});

describe("editing settings", () => {
  it("saves only the changed values", async () => {
    await showTab("LLM");
    await type(control("Port"), "9000");
    expect(savebar()?.textContent).toContain("1 unsaved change · saving restarts the LLM");
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { Port: 9000 } }]);
    await waitFor(() => expect(document.querySelector(".toast")?.textContent).toBe("Saved"));
    expect(savebar()).toBeNull();
    expect(control("Port").value).toBe("9000");
  });

  it("won't save an invalid value, and Revert restores the tray's", async () => {
    await showTab("LLM");
    const port = control("Port");
    await type(port, "80");
    expect(port.className).toBe("invalid");
    expect(savebar()?.textContent).toContain("Fix the highlighted value");
    expect(button("Save").disabled).toBe(true);
    await click(button("Revert"));
    expect(port.value).toBe("8080");
    expect(savebar()).toBeNull();
    expect(saves()).toEqual([]);
  });

  it("keeps an invalid value typed on another tab", async () => {
    await showTab("LLM");
    await type(control("API key"), "has spaces");
    await showTab("App");
    expect(savebar()?.textContent).toContain("Fix the highlighted value");
    await showTab("LLM");
    expect(control("API key").value).toBe("has spaces");
  });

  it("generates a valid API key", async () => {
    await showTab("LLM");
    await click(button("Generate"));
    const key = control("API key").value;
    expect(key).toMatch(/^[A-Za-z0-9_-]{32}$/);
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { ApiKey: key } }]);
  });

  it("edits lists one entry per line", async () => {
    await showTab("Game detection");
    const list = control<HTMLTextAreaElement>("Never count as a game");
    await type(list, "obs64\n");
    expect(savebar()).toBeNull(); // same list as before
    expect(list.value).toBe("obs64\n"); // and the typing isn't undone
    await type(list, "obs64\n  steamwebhelper \n\n");
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { GpuIgnore: ["obs64", "steamwebhelper"] } }]);
  });

  it("turns Start with Windows off", async () => {
    await showTab("App");
    const toggle = control("Start with Windows");
    expect(toggle.checked).toBe(true);
    await click(toggle);
    expect(savebar()?.textContent).toBe("1 unsaved changeRevertSave");
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { StartWithWindows: false } }]);
  });

  it("takes a custom context length", async () => {
    await showTab("LLM");
    await type(control<HTMLSelectElement>("Context length"), "custom");
    expect(button("Save").disabled).toBe(true); // nothing typed yet
    const tokens = document.querySelector<HTMLInputElement>('input[aria-label="Context tokens"]')!;
    await type(tokens, "100");
    expect(button("Save").disabled).toBe(true); // below 512
    await type(tokens, "50000");
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { Context: 50000 } }]);
  });

  it("switches models", async () => {
    await showTab("LLM");
    await click(control("my-own-model.Q5_K_M.gguf"));
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { Model: "my-own-model.Q5_K_M.gguf" } }]);
  });

  it("disables the GPU settings when pausing is off", async () => {
    await showTab("Game detection");
    const gpu = [...document.querySelectorAll("fieldset")].find((f) => f.textContent?.startsWith("GPU usage"))!;
    expect(gpu.disabled).toBe(false);
    await click(control("Pause while gaming"));
    expect(gpu.disabled).toBe(true);
    await type(control<HTMLSelectElement>("Detect games by"), "Launchers");
    await click(control("Pause while gaming"));
    expect(gpu.disabled).toBe(true); // launchers only
  });
});

describe("models", () => {
  it("asks before downloading, then shows progress", async () => {
    await showTab("LLM");
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
    const row = [...document.querySelectorAll(".entry")].find((e) => e.textContent?.includes("recommended"))!;
    await click(button("Download", row));
    expect(confirm).toHaveBeenCalledOnce();
    expect(sent).toEqual([]);
    confirm.mockReturnValue(true);
    await click(button("Download", row));
    expect(sent).toEqual([{ cmd: "download", id: "qwen3.8-27b:UD-Q5_K_XL" }]);
    await showTab("Overview");
    await waitFor(() => expect(document.querySelector(".download")?.textContent).toContain("(37%)"));
  });

  it("doesn't offer models that are installed or too big", async () => {
    await showTab("LLM");
    const rows = [...document.querySelectorAll(".entry")];
    const installed = rows.find((e) => e.textContent?.includes("UD-Q4_K_XL (17.6"))!;
    const tooBig = rows.find((e) => e.textContent?.includes("too big"))!;
    expect(button("Installed", installed).disabled).toBe(true);
    expect(button("Download", tooBig).disabled).toBe(true);
  });
});

describe("Laya", () => {
  it("turns Laya on, then shows its status and download", async () => {
    await showTab("Decision model");
    const model = control<HTMLSelectElement>("Model");
    const fieldset = model.closest("fieldset")!;
    expect(fieldset.disabled).toBe(true); // off by default
    await click(control("Run Laya alongside the LLM"));
    expect(fieldset.disabled).toBe(false);
    await type(model, "laya:en");
    expect(savebar()?.textContent).toContain("2 unsaved changes");
    expect(savebar()?.textContent).not.toContain("restarts"); // Laya isn't running yet
    await click(button("Save"));
    expect(saves()).toEqual([{ cmd: "save", settings: { RunLaya: true, LayaModel: "laya:en" } }]);
    await waitFor(() => expect(facts()).toContain("Downloading laya:en..."));
    expect(document.querySelector("section:not([hidden]) .download")?.textContent).toContain("(40%)");
    await click(button("Restart Laya"));
    expect(sent).toContainEqual({ cmd: "restart_laya" });
    await showTab("Overview");
    expect(facts()).toContain("Decision model Downloading laya:en... · http://127.0.0.1:11435");
  });

  it("says a Laya setting restarts Laya once it runs", async () => {
    await showTab("Decision model");
    await click(control("Run Laya alongside the LLM"));
    await click(button("Save"));
    await waitFor(() => expect(button("Restart Laya")).toBeTruthy()); // the tray confirmed it
    await type(control<HTMLSelectElement>("Keep the model loaded"), "5m");
    expect(savebar()?.textContent).toContain("1 unsaved change · saving restarts Laya");
    await showTab("LLM");
    await type(control("Port"), "9000");
    await type(control<HTMLSelectElement>("Access"), "0.0.0.0");
    expect(savebar()?.textContent).toContain("saving restarts the LLM and Laya");
  });

  it("rejects a port out of range and keeps unknown models from the settings file", async () => {
    await showTab("Decision model");
    await click(control("Run Laya alongside the LLM"));
    const port = control("Port");
    await type(port, "80");
    expect(port.className).toBe("invalid");
    expect(button("Save").disabled).toBe(true);
    await click(button("Revert"));
    const options = [...control<HTMLSelectElement>("Model").options].map((o) => o.value);
    expect(options).toEqual(["laya", "laya:en", "laya:multilingual", "laya:typed-decisions"]);
  });
});

describe("tabs", () => {
  it("opens the tab that replaced a remembered old one", async () => {
    await cleanup();
    localStorage.setItem("tab", "laya");
    await render(<App />);
    await waitFor(() => expect(document.querySelector("nav .active")?.textContent).toBe("Decision model"));
    expect(document.querySelector("section:not([hidden]) h2")?.textContent).toBe("Decision model");
  });
});

describe("keyboard", () => {
  it("saves with Ctrl+S", async () => {
    await showTab("App");
    await click(control("Update automatically"));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "s", ctrlKey: true }));
    await waitFor(() => expect(saves()).toEqual([{ cmd: "save", settings: { AutoUpdate: false } }]));
  });
});
