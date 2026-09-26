// Settings form state: the tray's settings, the user's unsaved edits and what's been saved but
// not yet confirmed by a state update from the tray.

import { createContext, useCallback, useContext, useMemo, useState } from "react";
import type { Request, SettingKey, Settings, View } from "./types";

/** Settings that change llama-server's command line (see settings::server_changed). */
export const RESTARTS_SERVER: SettingKey[] = ["Model", "Reasoning", "Context", "ListenHost", "Port", "ApiKey"];

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

export interface Form {
  view: View;
  send(r: Request): void;
  /** What the field shows: the edit, else what was just saved, else the tray's value. */
  value<K extends SettingKey>(k: K): Settings[K];
  /** What the tray has (or is about to have, right after Save). */
  saved<K extends SettingKey>(k: K): Settings[K];
  /** `undefined` = what's typed isn't valid. */
  edit<K extends SettingKey>(k: K, v: Settings[K] | undefined): void;
  isInvalid(k: SettingKey): boolean;
  isEdited(k: SettingKey): boolean;
  /** Changes on Revert, so fields holding typed text reset. */
  revision: number;
}

const FormContext = createContext<Form | null>(null);
export const FormProvider = FormContext.Provider;

export function useForm(): Form {
  const f = useContext(FormContext);
  if (!f) throw new Error("useForm outside FormProvider");
  return f;
}

export function useSettingsForm(view: View | null, send: (r: Request) => void) {
  const [draft, setDraft] = useState<Partial<Settings>>({});
  const [pending, setPending] = useState<Partial<Settings>>({});
  const [invalid, setInvalid] = useState<ReadonlySet<SettingKey>>(new Set());
  const [revision, setRevision] = useState(0);

  // Drop what the tray now confirms (or has had long enough to).
  const [lastView, setLastView] = useState(view);
  if (view !== lastView) {
    setLastView(view);
    if (view) {
      const left = Object.fromEntries(
        Object.entries(pending).filter(([k, v]) => !same(view.settings[k as SettingKey], v)),
      ) as Partial<Settings>;
      if (Object.keys(left).length !== Object.keys(pending).length) setPending(left);
    }
  }

  const edit = useCallback(
    <K extends SettingKey>(k: K, v: Settings[K] | undefined) => {
      setInvalid((s) => {
        const has = s.has(k);
        if ((v === undefined) === has) return s;
        const n = new Set(s);
        if (v === undefined) n.add(k);
        else n.delete(k);
        return n;
      });
      if (v === undefined || !view) return;
      const base = pending[k] !== undefined ? pending[k] : view.settings[k];
      setDraft((d) => {
        const n = { ...d };
        if (same(v, base)) delete n[k];
        else n[k] = v;
        return n;
      });
    },
    [view, pending],
  );

  const save = useCallback(() => {
    if (Object.keys(draft).length === 0 || invalid.size > 0) return;
    send({ cmd: "save", settings: draft });
    setPending((p) => ({ ...p, ...draft }));
    setDraft({});
    // If the tray rejected a value, show its settings again after a moment.
    setTimeout(() => setPending({}), 4000);
  }, [draft, invalid, send]);

  const revert = useCallback(() => {
    setDraft({});
    setInvalid(new Set());
    setRevision((r) => r + 1);
  }, []);

  const form = useMemo<Form | null>(() => {
    if (!view) return null;
    const saved = <K extends SettingKey>(k: K): Settings[K] =>
      (pending[k] !== undefined ? pending[k] : view.settings[k]) as Settings[K];
    return {
      view,
      send,
      saved,
      value: (k) => (draft[k] !== undefined ? draft[k] : saved(k)) as Settings[typeof k],
      edit,
      isInvalid: (k) => invalid.has(k),
      isEdited: (k) => draft[k] !== undefined || invalid.has(k),
      revision,
    };
  }, [view, send, draft, pending, invalid, edit, revision]);

  const changed = Object.keys(draft) as SettingKey[];
  return { form, changed, invalid, save, revert };
}
