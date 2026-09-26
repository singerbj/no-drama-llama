// One component per kind of setting. Each takes `k`, the settings.json key it edits.

import { useEffect, useId, useState, type ReactNode } from "react";
import { useForm } from "./form";
import type { SettingKey, Settings } from "./types";

type KeysOf<T> = { [K in SettingKey]: Settings[K] extends T ? K : never }[SettingKey];
type BoolKey = KeysOf<boolean>;
type NumberKey = KeysOf<number>;
type StringKey = KeysOf<string>;
type ListKey = KeysOf<string[]>;

interface RowProps {
  label: string;
  hint?: ReactNode;
  stacked?: boolean;
  htmlFor?: string;
  children: ReactNode;
}

export function Row({ label, hint, stacked, htmlFor, children }: RowProps) {
  return (
    <div className={stacked ? "row stacked" : "row"}>
      <label className="label" htmlFor={htmlFor}>
        {label}
        {hint && <small>{hint}</small>}
      </label>
      {children}
    </div>
  );
}

/** Text the user is typing, reset to the setting whenever the setting changes elsewhere. */
function useText<K extends SettingKey>(k: K, format: (v: Settings[K]) => string) {
  const form = useForm();
  const saved = form.saved(k);
  const [text, setText] = useState(() => format(form.value(k)));
  const savedKey = JSON.stringify(saved);
  const edited = form.isEdited(k);
  useEffect(() => {
    if (!edited) setText(format(saved));
    // Only when the setting itself changes (or on Revert), never while typing.
  }, [savedKey, form.revision]);
  return [text, setText] as const;
}

export function Toggle({ k, label, hint }: { k: BoolKey; label: string; hint?: ReactNode }) {
  const form = useForm();
  const id = useId();
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <input
        id={id}
        type="checkbox"
        className="switch"
        checked={form.value(k)}
        onChange={(e) => form.edit(k, e.target.checked)}
      />
    </Row>
  );
}

interface NumberProps {
  k: NumberKey;
  label: string;
  hint?: ReactNode;
  min: number;
  max: number;
  step?: number;
  integer?: boolean;
}

export function parseNumber(text: string, min: number, max: number, integer = false): number | undefined {
  const t = text.trim();
  if (t === "") return undefined;
  const n = Number(t);
  if (!Number.isFinite(n) || n < min || n > max || (integer && !Number.isInteger(n))) return undefined;
  return n;
}

export function NumberField({ k, label, hint, min, max, step = 1, integer }: NumberProps) {
  const form = useForm();
  const id = useId();
  const [text, setText] = useText(k, String);
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <input
        id={id}
        type="number"
        min={min}
        max={max}
        step={step}
        value={text}
        aria-invalid={form.isInvalid(k)}
        className={form.isInvalid(k) ? "invalid" : undefined}
        onChange={(e) => {
          setText(e.target.value);
          form.edit(k, parseNumber(e.target.value, min, max, integer));
        }}
      />
    </Row>
  );
}

interface TextProps {
  k: StringKey;
  label: string;
  hint?: ReactNode;
  pattern: RegExp;
  maxLength: number;
  placeholder?: string;
  /** Replaces the text, e.g. a Generate button. */
  extra?: (set: (text: string) => void) => ReactNode;
}

export function TextField({ k, label, hint, pattern, maxLength, placeholder, extra }: TextProps) {
  const form = useForm();
  const id = useId();
  const [text, setText] = useText(k, String);
  const set = (t: string) => {
    setText(t);
    const v = t.trim();
    form.edit(k, v.length <= maxLength && pattern.test(v) ? v : undefined);
  };
  return (
    <Row label={label} hint={hint} htmlFor={id} stacked>
      <span className="inline">
        <input
          id={id}
          type="text"
          value={text}
          maxLength={maxLength}
          spellCheck={false}
          autoComplete="off"
          placeholder={placeholder}
          className={form.isInvalid(k) ? "invalid" : undefined}
          aria-invalid={form.isInvalid(k)}
          onChange={(e) => set(e.target.value)}
        />
        {extra?.(set)}
      </span>
    </Row>
  );
}

interface ChoiceProps<K extends SettingKey> {
  k: K;
  label: string;
  hint?: ReactNode;
  options: readonly (readonly [Settings[K], string])[];
}

export function Choice<K extends SettingKey>({ k, label, hint, options }: ChoiceProps<K>) {
  const form = useForm();
  const id = useId();
  const current = form.value(k);
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <select
        id={id}
        value={String(current)}
        onChange={(e) => {
          const hit = options.find(([v]) => String(v) === e.target.value);
          if (hit) form.edit(k, hit[0]);
        }}
      >
        {options.map(([v, text]) => (
          <option key={String(v)} value={String(v)}>
            {text}
          </option>
        ))}
      </select>
    </Row>
  );
}

export function ListField({ k, label, hint }: { k: ListKey; label: string; hint?: ReactNode }) {
  const form = useForm();
  const id = useId();
  const [text, setText] = useText(k, (v) => v.join("\n"));
  return (
    <Row label={label} hint={hint} htmlFor={id} stacked>
      <textarea
        id={id}
        rows={3}
        spellCheck={false}
        value={text}
        onChange={(e) => {
          setText(e.target.value);
          form.edit(
            k,
            e.target.value
              .split(/\r?\n/)
              .map((s) => s.trim())
              .filter(Boolean),
          );
        }}
      />
    </Row>
  );
}

const CONTEXT_PRESETS = [8192, 16384, 32768, 65536, 131072, 262144];

/** Auto, a preset, or any number of tokens (512 - 1048576). */
export function ContextField({ k, label, hint }: { k: "Context"; label: string; hint?: ReactNode }) {
  const form = useForm();
  const id = useId();
  const value = form.value(k);
  const isPreset = value === "auto" || CONTEXT_PRESETS.includes(value);
  const [custom, setCustom] = useState(!isPreset);
  const [text, setText] = useText(k, (v) => (v === "auto" ? "" : String(v)));
  const showCustom = custom || !isPreset;
  return (
    <Row label={label} hint={hint} htmlFor={id}>
      <span className="inline">
        <select
          id={id}
          value={showCustom ? "custom" : String(value)}
          onChange={(e) => {
            const v = e.target.value;
            setCustom(v === "custom");
            if (v === "auto") form.edit(k, "auto");
            else if (v === "custom") form.edit(k, parseNumber(text, 512, 1048576, true));
            else form.edit(k, Number(v));
          }}
        >
          <option value="auto">Auto (largest that fits)</option>
          {CONTEXT_PRESETS.map((n) => (
            <option key={n} value={n}>
              {n / 1024}K tokens
            </option>
          ))}
          <option value="custom">Custom...</option>
        </select>
        {showCustom && (
          <input
            type="number"
            min={512}
            max={1048576}
            step={1}
            aria-label="Context tokens"
            placeholder="tokens"
            value={text}
            className={form.isInvalid(k) ? "invalid" : undefined}
            onChange={(e) => {
              setText(e.target.value);
              form.edit(k, parseNumber(e.target.value, 512, 1048576, true));
            }}
          />
        )}
      </span>
    </Row>
  );
}

const gb = (bytes: number) => `${(bytes / 1e9).toFixed(1)} GB`;

/** The installed models; a configured model that isn't on disk still shows. */
export function ModelPicker({ k }: { k: "Model" }) {
  const form = useForm();
  const { models } = form.view;
  const configured = form.saved(k);
  const rows = models.some((m) => m.name === configured) ? models : [{ name: configured, size: -1 }, ...models];
  const selected = form.value(k);
  return (
    <div className="models" role="radiogroup" aria-label="Installed models">
      {rows.map((m) => (
        <label className="model" key={m.name}>
          <input type="radio" name="model" value={m.name} checked={m.name === selected} onChange={() => form.edit(k, m.name)} />
          <span>{m.name}</span>
          <small>{m.size < 0 ? "missing" : gb(m.size)}</small>
        </label>
      ))}
    </div>
  );
}

export { gb };
