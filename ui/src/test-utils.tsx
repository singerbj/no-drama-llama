// Small helpers for rendering components in Vitest (jsdom) and driving them like a user.

import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

let mounted: { root: Root; container: HTMLElement }[] = [];

export async function render(ui: ReactNode): Promise<HTMLElement> {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  await act(async () => root.render(ui));
  mounted.push({ root, container });
  return container;
}

export async function cleanup() {
  for (const { root, container } of mounted) {
    // oxlint-disable-next-line no-await-in-loop -- unmount one at a time
    await act(async () => root.unmount());
    container.remove();
  }
  mounted = [];
}

/** Waits (up to `ms`, 2 s by default) for `check` to stop throwing, letting React and timers run. */
export async function waitFor<T>(check: () => T, ms = 2000): Promise<T> {
  const end = Date.now() + ms;
  for (;;) {
    try {
      return check();
    } catch (e) {
      if (Date.now() > end) throw e;
      // oxlint-disable-next-line no-await-in-loop -- polling: each wait depends on the last check
      await act(() => new Promise((r) => setTimeout(r, 10)));
    }
  }
}

/** The form control a visible <label> (matched by the start of its text) points at. */
export function control<T extends HTMLElement = HTMLInputElement>(labelText: string): T {
  const label = [...document.querySelectorAll<HTMLLabelElement>("main section:not([hidden]) label")].find((l) =>
    l.textContent?.startsWith(labelText),
  );
  if (!label) throw new Error(`no label "${labelText}"`);
  const el = label.htmlFor ? document.getElementById(label.htmlFor) : label.querySelector("input");
  if (!el) throw new Error(`label "${labelText}" has no control`);
  return el as T;
}

export function button(text: string, scope: ParentNode = document): HTMLButtonElement {
  const b = [...scope.querySelectorAll("button")].find((x) => x.textContent?.trim() === text);
  if (!b) throw new Error(`no button "${text}"`);
  return b;
}

export async function click(el: HTMLElement) {
  await act(async () => el.click());
}

/** Sets a value the way typing does, so React's onChange fires. */
export async function type(el: HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), "value")!.set!;
  await act(async () => {
    setter.call(el, value);
    el.dispatchEvent(new Event(el instanceof HTMLSelectElement ? "change" : "input", { bubbles: true }));
  });
}

export async function showTab(title: string) {
  await click(button(title, document.querySelector("nav")!));
}
