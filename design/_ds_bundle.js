/* @ds-bundle: {"format":4,"namespace":"NoDramaLlamaDesignSystem_58ce7f","components":[{"name":"Button","sourcePath":"components/core/Button.jsx"},{"name":"Icon","sourcePath":"components/core/Icon.jsx"},{"name":"PillButton","sourcePath":"components/core/PillButton.jsx"},{"name":"StatusDot","sourcePath":"components/core/StatusDot.jsx"},{"name":"Banner","sourcePath":"components/feedback/Banner.jsx"},{"name":"DownloadProgress","sourcePath":"components/feedback/DownloadProgress.jsx"},{"name":"SaveBar","sourcePath":"components/feedback/SaveBar.jsx"},{"name":"StatusPopup","sourcePath":"components/feedback/StatusPopup.jsx"},{"name":"Toast","sourcePath":"components/feedback/Toast.jsx"},{"name":"NumberInput","sourcePath":"components/forms/NumberInput.jsx"},{"name":"Select","sourcePath":"components/forms/Select.jsx"},{"name":"SettingRow","sourcePath":"components/forms/SettingRow.jsx"},{"name":"Switch","sourcePath":"components/forms/Switch.jsx"},{"name":"TextArea","sourcePath":"components/forms/TextArea.jsx"},{"name":"TextInput","sourcePath":"components/forms/TextInput.jsx"},{"name":"FactList","sourcePath":"components/layout/FactList.jsx"},{"name":"Fieldset","sourcePath":"components/layout/Fieldset.jsx"},{"name":"SideNav","sourcePath":"components/layout/SideNav.jsx"},{"name":"StatusHeader","sourcePath":"components/layout/StatusHeader.jsx"},{"name":"Chip","sourcePath":"components/marketing/Chip.jsx"},{"name":"CodeWindow","sourcePath":"components/marketing/CodeWindow.jsx"},{"name":"DataTable","sourcePath":"components/marketing/DataTable.jsx"},{"name":"Eyebrow","sourcePath":"components/marketing/Eyebrow.jsx"},{"name":"FaqItem","sourcePath":"components/marketing/FaqItem.jsx"},{"name":"FeatureCard","sourcePath":"components/marketing/FeatureCard.jsx"},{"name":"StateTimeline","sourcePath":"components/marketing/StateTimeline.jsx"},{"name":"StepCard","sourcePath":"components/marketing/StepCard.jsx"},{"name":"VramBar","sourcePath":"components/marketing/VramBar.jsx"},{"name":"WindowFrame","sourcePath":"components/marketing/WindowFrame.jsx"},{"name":"CatalogRow","sourcePath":"components/models/CatalogRow.jsx"},{"name":"ModelPicker","sourcePath":"components/models/ModelPicker.jsx"}],"sourceHashes":{"components/core/Button.jsx":"930d384deba9","components/core/Icon.jsx":"1d48ce95a16c","components/core/PillButton.jsx":"e2029d1f60a9","components/core/StatusDot.jsx":"e08d94a8b6a5","components/feedback/Banner.jsx":"f7e3f8e45aa0","components/feedback/DownloadProgress.jsx":"f3e527ad95da","components/feedback/SaveBar.jsx":"6496a4fedca9","components/feedback/StatusPopup.jsx":"fe08199d5e53","components/feedback/Toast.jsx":"65cdedae577f","components/forms/NumberInput.jsx":"37fa3f30346c","components/forms/Select.jsx":"e43f9229fa7f","components/forms/SettingRow.jsx":"39e0662cfaf5","components/forms/Switch.jsx":"e13b6c970415","components/forms/TextArea.jsx":"85322862ef90","components/forms/TextInput.jsx":"8b6bef3b088b","components/layout/FactList.jsx":"8c083e428cd3","components/layout/Fieldset.jsx":"63fb0bb01b06","components/layout/SideNav.jsx":"17321879b40c","components/layout/StatusHeader.jsx":"e6a123f9b982","components/marketing/Chip.jsx":"6d8b49940fbc","components/marketing/CodeWindow.jsx":"01f5c957451f","components/marketing/DataTable.jsx":"3874ee5b9e65","components/marketing/Eyebrow.jsx":"f146018029b6","components/marketing/FaqItem.jsx":"335f94ebb417","components/marketing/FeatureCard.jsx":"bceec9d7b754","components/marketing/StateTimeline.jsx":"6ee005ed6e40","components/marketing/StepCard.jsx":"b240326edea2","components/marketing/VramBar.jsx":"39f24ea6f8b0","components/marketing/WindowFrame.jsx":"8e77f0e3524a","components/models/CatalogRow.jsx":"7ad52a70c5f1","components/models/ModelPicker.jsx":"04fbc981d5a2","ui_kits/settings-window/App.jsx":"40f10bda2b07","ui_kits/settings-window/Panels.jsx":"b2b6b5221938","ui_kits/settings-window/data.js":"1b4fea5678f8","ui_kits/website/Sections.jsx":"f245e43edde5","ui_kits/website/Top.jsx":"cb69732b68ed"},"inlinedExternals":[],"unexposedExports":[]} */

(() => {

const __ds_ns = (window.NoDramaLlamaDesignSystem_58ce7f = window.NoDramaLlamaDesignSystem_58ce7f || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/core/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const {
  useState
} = React;
function useHover() {
  const [h, setH] = useState(false);
  return [h, {
    onMouseEnter: () => setH(true),
    onMouseLeave: () => setH(false)
  }];
}
/** Settings-window button: default, primary, danger, or a side-nav tab. Values from ui/src/style.css. */
function Button({
  variant = 'default',
  active = false,
  disabled = false,
  type = 'button',
  onClick,
  children,
  style,
  ...rest
}) {
  const [h, hp] = useHover();
  const base = {
    font: 'inherit',
    fontFamily: 'var(--font-sans)',
    fontSize: 'var(--text-app)',
    lineHeight: 'var(--leading-app)',
    color: 'var(--text)',
    background: 'var(--surface)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-sm)',
    padding: '5px 14px',
    cursor: disabled ? 'default' : 'pointer',
    whiteSpace: 'nowrap',
    opacity: disabled ? 0.5 : 1,
    transition: 'background var(--dur-fast), filter var(--dur-fast)'
  };
  const live = h && !disabled;
  const v = {
    default: {
      background: live ? 'var(--hover)' : 'var(--surface)'
    },
    primary: {
      background: 'var(--accent)',
      borderColor: 'var(--accent)',
      color: 'var(--on-accent)',
      fontWeight: 600,
      filter: live ? 'brightness(1.08)' : 'none'
    },
    danger: {
      color: 'var(--danger)',
      borderColor: 'var(--danger)',
      background: live ? 'color-mix(in srgb, var(--danger) 10%, var(--surface))' : 'var(--surface)'
    },
    tab: {
      textAlign: 'left',
      border: '1px solid ' + (active ? 'color-mix(in srgb, var(--accent) 40%, transparent)' : 'transparent'),
      background: active ? 'var(--accent-soft)' : live ? 'var(--hover)' : 'none',
      padding: '7px 11px',
      fontWeight: active ? 600 : 400,
      color: active ? 'var(--accent-strong)' : 'var(--text-muted)'
    }
  }[variant];
  return /*#__PURE__*/React.createElement("button", _extends({
    type: type,
    disabled: disabled,
    onClick: onClick,
    style: {
      ...base,
      ...v,
      ...style
    }
  }, hp, rest), children);
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Button.jsx", error: String((e && e.message) || e) }); }

// components/core/Icon.jsx
try { (() => {
/** Lucide glyph (CDN, lucide-static) painted with currentColor via CSS mask, so it takes any token colour. */
function Icon({
  name,
  size = 18,
  color = 'currentColor',
  style
}) {
  const url = 'url(https://unpkg.com/lucide-static@0.460.0/icons/' + name + '.svg)';
  return /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true",
    style: {
      display: 'inline-block',
      flex: 'none',
      width: size,
      height: size,
      background: color,
      WebkitMask: url + ' center / contain no-repeat',
      mask: url + ' center / contain no-repeat',
      ...style
    }
  });
}
Object.assign(__ds_scope, { Icon });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Icon.jsx", error: String((e && e.message) || e) }); }

// components/core/PillButton.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
const {
  useState
} = React;
function useHover() {
  const [h, setH] = useState(false);
  return [h, {
    onMouseEnter: () => setH(true),
    onMouseLeave: () => setH(false)
  }];
}
/** Website CTA: pill-shaped, mint fill or ghost outline. From site .btn / .btn-ghost / .btn-sm. */
function PillButton({
  variant = 'primary',
  size = 'md',
  href,
  icon,
  onClick,
  children,
  style
}) {
  const [h, hp] = useHover();
  const s = {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.5rem',
    padding: size === 'sm' ? '0.45rem 0.95rem' : '0.8rem 1.3rem',
    fontSize: size === 'sm' ? '0.9rem' : 'inherit',
    fontFamily: 'var(--font-sans)',
    borderRadius: 'var(--radius-pill)',
    fontWeight: 600,
    textDecoration: 'none',
    cursor: 'pointer',
    transition: 'background var(--dur-fast), transform var(--dur-fast)',
    transform: h ? 'translateY(-1px)' : 'none',
    whiteSpace: 'nowrap'
  };
  const v = variant === 'ghost' ? {
    background: h ? 'var(--bg-alt)' : 'transparent',
    color: 'var(--text)',
    border: '1px solid var(--border)'
  } : {
    background: h ? 'var(--accent-strong)' : 'var(--accent)',
    color: 'var(--on-accent)',
    border: '1px solid var(--accent)'
  };
  const Tag = href ? 'a' : 'button';
  return /*#__PURE__*/React.createElement(Tag, _extends({
    href: href,
    onClick: onClick,
    style: {
      ...s,
      ...v,
      ...style
    }
  }, hp), icon, children);
}
Object.assign(__ds_scope, { PillButton });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/PillButton.jsx", error: String((e && e.message) || e) }); }

// components/core/StatusDot.jsx
try { (() => {
/** The tray's status tone as a dot. loading pulses (1.2s). */
function StatusDot({
  tone = 'off',
  size = 'md',
  style
}) {
  const px = size === 'lg' ? 14 : size === 'sm' ? 8 : 10;
  const c = 'var(--tone-' + tone + ')';
  return /*#__PURE__*/React.createElement("span", {
    role: "img",
    "aria-label": tone,
    style: {
      display: 'inline-block',
      flex: 'none',
      width: px,
      height: px,
      borderRadius: '50%',
      background: c,
      animation: tone === 'loading' ? 'ndl-pulse 1.2s ease-in-out infinite' : 'none',
      verticalAlign: size === 'lg' ? 'middle' : '-1px',
      ...style
    }
  });
}
Object.assign(__ds_scope, { StatusDot });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/StatusDot.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Banner.jsx
try { (() => {
/** Tone-tinted notice with an optional action (default tone: paused). */
function Banner({
  tone = 'paused',
  action,
  children,
  style
}) {
  const c = 'var(--tone-' + tone + ')';
  return /*#__PURE__*/React.createElement("div", {
    role: "status",
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      gap: 12,
      padding: '10px 14px',
      marginBottom: 14,
      borderRadius: 'var(--radius-md)',
      border: '1px solid ' + c,
      background: 'color-mix(in srgb, ' + c + ' 12%, var(--surface))'
    }
  }, /*#__PURE__*/React.createElement("span", null, children), action);
}
Object.assign(__ds_scope, { Banner });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Banner.jsx", error: String((e && e.message) || e) }); }

// components/feedback/DownloadProgress.jsx
try { (() => {
const gb = b => (b / 1e9).toFixed(1) + ' GB';
/** Background download panel with an 8px progress bar. done/total in bytes. */
function DownloadProgress({
  label,
  done = 0,
  total = 0,
  onCancel,
  note,
  style
}) {
  const p = total ? done / total : 0;
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '12px 16px',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-md)',
      background: 'var(--surface)',
      marginTop: 8,
      ...style
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      gap: 12,
      marginBottom: 8
    }
  }, /*#__PURE__*/React.createElement("span", null, "Downloading ", /*#__PURE__*/React.createElement("b", null, label)), onCancel && /*#__PURE__*/React.createElement(__ds_scope.Button, {
    onClick: onCancel
  }, "Cancel")), /*#__PURE__*/React.createElement("div", {
    style: {
      height: 8,
      borderRadius: 'var(--radius-pill)',
      background: 'var(--bg-alt)',
      overflow: 'hidden',
      border: '1px solid var(--border)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      width: p * 100 + '%',
      height: '100%',
      background: 'var(--tone-running)',
      transition: 'width 0.3s var(--ease-out)'
    }
  })), /*#__PURE__*/React.createElement("small", {
    style: {
      display: 'block',
      marginTop: 6,
      color: 'var(--text-muted)',
      fontSize: 'var(--text-hint)'
    }
  }, total ? gb(done) + ' of ' + gb(total) + ' (' + Math.floor(p * 100) + '%).' : 'Starting...', note ? ' ' + note : ''));
}
Object.assign(__ds_scope, { DownloadProgress });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/DownloadProgress.jsx", error: String((e && e.message) || e) }); }

// components/feedback/SaveBar.jsx
try { (() => {
/** Unsaved-changes bar with Revert + Save. floating = fixed to the window bottom. */
function SaveBar({
  message,
  onRevert,
  onSave,
  saveDisabled = false,
  floating = false,
  style
}) {
  return /*#__PURE__*/React.createElement("footer", {
    style: {
      ...(floating ? {
        position: 'fixed',
        left: 0,
        right: 0,
        bottom: 0
      } : {}),
      display: 'flex',
      alignItems: 'center',
      gap: 10,
      padding: '12px 20px',
      background: 'var(--surface)',
      borderTop: '1px solid var(--border)',
      boxShadow: 'var(--shadow-savebar)',
      ...style
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      flex: 1
    }
  }, message), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    onClick: onRevert
  }, "Revert"), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "primary",
    disabled: saveDisabled,
    onClick: onSave
  }, "Save"));
}
Object.assign(__ds_scope, { SaveBar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/SaveBar.jsx", error: String((e && e.message) || e) }); }

// components/feedback/StatusPopup.jsx
try { (() => {
/** The on-screen popup (osd.rs): near-black card, tone dot, title + subtitle. Never takes focus. */
function StatusPopup({
  tone = 'paused',
  title,
  subtitle,
  animated = false,
  style
}) {
  const c = 'var(--tone-' + tone + ')';
  return /*#__PURE__*/React.createElement("div", {
    role: "status",
    style: {
      position: 'relative',
      display: 'inline-flex',
      alignItems: 'flex-start',
      gap: 12,
      minWidth: 300,
      padding: 18,
      borderRadius: 'var(--radius-xl)',
      background: 'var(--osd-bg)',
      border: '1px solid var(--osd-border)',
      boxShadow: 'var(--shadow-card)',
      overflow: 'hidden',
      fontFamily: 'var(--font-sans)',
      pointerEvents: 'none',
      animation: animated ? 'ndl-popup 6s ease-in-out infinite' : 'none',
      ...style
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.StatusDot, {
    tone: tone,
    size: "md",
    style: {
      width: 12,
      height: 12,
      marginTop: 6
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 2
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: '#fff',
      fontSize: 18,
      fontWeight: 600,
      lineHeight: 1.3
    }
  }, title), subtitle && /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--osd-sub)',
      fontSize: 14,
      lineHeight: 1.3
    }
  }, subtitle)));
}
Object.assign(__ds_scope, { StatusPopup });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/StatusPopup.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Toast.jsx
try { (() => {
/** Inverted pill notice. floating = fixed 72px above the bottom, centred. */
function Toast({
  error = false,
  floating = false,
  children,
  style
}) {
  const pos = floating ? {
    position: 'fixed',
    bottom: 72,
    left: '50%',
    transform: 'translateX(-50%)',
    animation: 'ndl-toast 0.15s var(--ease-out)'
  } : {
    display: 'inline-block'
  };
  return /*#__PURE__*/React.createElement("div", {
    role: "status",
    style: {
      ...pos,
      maxWidth: 'min(640px, 90vw)',
      padding: '10px 16px',
      borderRadius: 'var(--radius-md)',
      background: error ? 'var(--danger)' : 'var(--text)',
      color: error ? 'var(--on-danger)' : 'var(--bg)',
      boxShadow: 'var(--shadow-toast)',
      ...style
    }
  }, children);
}
Object.assign(__ds_scope, { Toast });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Toast.jsx", error: String((e && e.message) || e) }); }

// components/forms/NumberInput.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** Numeric field (110px). Use for ports, thresholds, seconds. */
function NumberInput({
  value,
  onChange,
  min,
  max,
  step = 1,
  invalid = false,
  width = 110,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("input", _extends({
    type: "number",
    value: value,
    onChange: onChange,
    min: min,
    max: max,
    step: step,
    "aria-invalid": invalid,
    style: {
      ...{
        fontFamily: 'var(--font-sans)',
        fontSize: 'var(--text-app)',
        color: 'var(--text)',
        background: 'var(--field)',
        border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'),
        borderRadius: 'var(--radius-sm)',
        padding: '5px 8px',
        minWidth: 0,
        outlineColor: invalid ? 'var(--danger)' : 'var(--accent)'
      },
      width,
      ...style
    }
  }, rest));
}
Object.assign(__ds_scope, { NumberInput });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/NumberInput.jsx", error: String((e && e.message) || e) }); }

// components/forms/Select.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** Native select (220px min). options = [[value, label], ...]. */
function Select({
  value,
  onChange,
  options = [],
  invalid = false,
  minWidth = 220,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", {
    style: {
      position: 'relative',
      display: 'inline-flex',
      alignItems: 'center',
      minWidth: 0
    }
  }, /*#__PURE__*/React.createElement("select", _extends({
    value: value,
    onChange: onChange,
    style: {
      ...{
        fontFamily: 'var(--font-sans)',
        fontSize: 'var(--text-app)',
        color: 'var(--text)',
        background: 'var(--field)',
        border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'),
        borderRadius: 'var(--radius-sm)',
        padding: '5px 8px',
        minWidth: 0,
        outlineColor: invalid ? 'var(--danger)' : 'var(--accent)'
      },
      minWidth,
      appearance: 'none',
      WebkitAppearance: 'none',
      paddingRight: 30,
      cursor: 'pointer',
      ...style
    }
  }, rest), options.map(([v, l]) => /*#__PURE__*/React.createElement("option", {
    key: String(v),
    value: String(v)
  }, l))), /*#__PURE__*/React.createElement(__ds_scope.Icon, {
    name: "chevron-down",
    size: 16,
    color: "var(--text-muted)",
    style: {
      position: 'absolute',
      right: 9,
      pointerEvents: 'none'
    }
  }));
}
Object.assign(__ds_scope, { Select });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Select.jsx", error: String((e && e.message) || e) }); }

// components/forms/SettingRow.jsx
try { (() => {
/** One setting: label + muted hint on the left, control on the right (or below when stacked). */
function SettingRow({
  label,
  hint,
  stacked = false,
  htmlFor,
  last = false,
  children
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: stacked ? 'column' : 'row',
      alignItems: stacked ? 'stretch' : 'center',
      justifyContent: 'space-between',
      gap: stacked ? 6 : 16,
      padding: '10px 0',
      borderBottom: last ? 'none' : '1px solid var(--border)'
    }
  }, /*#__PURE__*/React.createElement("label", {
    htmlFor: htmlFor,
    style: {
      display: 'flex',
      flexDirection: 'column',
      minWidth: 0
    }
  }, label, hint && /*#__PURE__*/React.createElement("small", {
    style: {
      color: 'var(--text-muted)',
      fontSize: 'var(--text-hint)'
    }
  }, hint)), children);
}
Object.assign(__ds_scope, { SettingRow });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/SettingRow.jsx", error: String((e && e.message) || e) }); }

// components/forms/Switch.jsx
try { (() => {
/** 40×22 toggle switch; mint when on. */
function Switch({
  checked = false,
  onChange,
  disabled = false,
  id,
  style
}) {
  return /*#__PURE__*/React.createElement("button", {
    type: "button",
    role: "switch",
    id: id,
    "aria-checked": checked,
    disabled: disabled,
    onClick: () => onChange && onChange(!checked),
    style: {
      flex: 'none',
      width: 40,
      height: 22,
      borderRadius: 11,
      border: 'none',
      padding: 0,
      margin: 0,
      position: 'relative',
      cursor: disabled ? 'default' : 'pointer',
      opacity: disabled ? 0.55 : 1,
      background: checked ? 'var(--accent)' : 'var(--border-strong)',
      transition: 'background var(--dur-fast)',
      ...style
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      position: 'absolute',
      top: 3,
      left: 3,
      width: 16,
      height: 16,
      borderRadius: '50%',
      background: '#fff',
      boxShadow: 'var(--shadow-knob)',
      transform: checked ? 'translateX(18px)' : 'none',
      transition: 'transform var(--dur-fast)'
    }
  }));
}
Object.assign(__ds_scope, { Switch });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Switch.jsx", error: String((e && e.message) || e) }); }

// components/forms/TextArea.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** Multi-line list field, one entry per line, code font 13px. */
function TextArea({
  value,
  onChange,
  rows = 3,
  invalid = false,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("textarea", _extends({
    value: value,
    onChange: onChange,
    rows: rows,
    spellCheck: false,
    style: {
      ...{
        fontFamily: 'var(--font-sans)',
        fontSize: 'var(--text-app)',
        color: 'var(--text)',
        background: 'var(--field)',
        border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'),
        borderRadius: 'var(--radius-sm)',
        padding: '5px 8px',
        minWidth: 0,
        outlineColor: invalid ? 'var(--danger)' : 'var(--accent)'
      },
      width: '100%',
      resize: 'vertical',
      fontFamily: 'var(--font-mono)',
      fontSize: 'var(--text-code)',
      ...style
    }
  }, rest));
}
Object.assign(__ds_scope, { TextArea });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/TextArea.jsx", error: String((e && e.message) || e) }); }

// components/forms/TextInput.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/** Single-line text field (260px default). Invalid = tomato border. */
function TextInput({
  value,
  onChange,
  placeholder,
  invalid = false,
  width = 260,
  style,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("input", _extends({
    type: "text",
    value: value,
    onChange: onChange,
    placeholder: placeholder,
    "aria-invalid": invalid,
    spellCheck: false,
    autoComplete: "off",
    style: {
      ...{
        fontFamily: 'var(--font-sans)',
        fontSize: 'var(--text-app)',
        color: 'var(--text)',
        background: 'var(--field)',
        border: '1px solid ' + (invalid ? 'var(--danger)' : 'var(--border)'),
        borderRadius: 'var(--radius-sm)',
        padding: '5px 8px',
        minWidth: 0,
        outlineColor: invalid ? 'var(--danger)' : 'var(--accent)'
      },
      width,
      ...style
    }
  }, rest));
}
Object.assign(__ds_scope, { TextInput });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/TextInput.jsx", error: String((e && e.message) || e) }); }

// components/layout/FactList.jsx
try { (() => {
/** Two-column definition list panel (170px term column). items = [[term, value], ...]. */
function FactList({
  items = [],
  style
}) {
  return /*#__PURE__*/React.createElement("dl", {
    style: {
      display: 'grid',
      gridTemplateColumns: '170px 1fr',
      gap: '8px 16px',
      margin: '0 0 8px',
      padding: '14px 16px',
      background: 'var(--surface)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-md)',
      ...style
    }
  }, items.map(([k, v], i) => /*#__PURE__*/React.createElement(React.Fragment, {
    key: i
  }, /*#__PURE__*/React.createElement("dt", {
    style: {
      color: 'var(--text-muted)'
    }
  }, k), /*#__PURE__*/React.createElement("dd", {
    style: {
      margin: 0,
      overflowWrap: 'anywhere'
    }
  }, v))));
}
Object.assign(__ds_scope, { FactList });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/FactList.jsx", error: String((e && e.message) || e) }); }

// components/layout/Fieldset.jsx
try { (() => {
/** Settings group: 8px panel with an inline legend. Disabled dims to 55%. */
function Fieldset({
  legend,
  disabled = false,
  children,
  style
}) {
  return /*#__PURE__*/React.createElement("fieldset", {
    disabled: disabled,
    style: {
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-md)',
      background: 'var(--surface)',
      margin: '0 0 16px',
      padding: '4px 16px 12px',
      opacity: disabled ? 0.55 : 1,
      minWidth: 0,
      ...style
    }
  }, legend && /*#__PURE__*/React.createElement("legend", {
    style: {
      fontWeight: 600,
      padding: '0 6px',
      marginLeft: -6
    }
  }, legend), children);
}
Object.assign(__ds_scope, { Fieldset });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/Fieldset.jsx", error: String((e && e.message) || e) }); }

// components/layout/SideNav.jsx
try { (() => {
/** 180px left nav of tab buttons. items = [{ id, title }]. */
function SideNav({
  items = [],
  active,
  onSelect,
  style
}) {
  return /*#__PURE__*/React.createElement("nav", {
    style: {
      width: 'var(--nav-width)',
      flex: 'none',
      padding: '12px 8px',
      display: 'flex',
      flexDirection: 'column',
      gap: 2,
      borderRight: '1px solid var(--border)',
      ...style
    }
  }, items.map(it => /*#__PURE__*/React.createElement(__ds_scope.Button, {
    key: it.id,
    variant: "tab",
    active: it.id === active,
    onClick: () => onSelect && onSelect(it.id)
  }, it.title)));
}
Object.assign(__ds_scope, { SideNav });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/SideNav.jsx", error: String((e && e.message) || e) }); }

// components/layout/StatusHeader.jsx
try { (() => {
/** Top of the settings window: big tone dot, status line, chat URL, actions. */
function StatusHeader({
  tone = 'running',
  title,
  subtitle,
  actions,
  style
}) {
  return /*#__PURE__*/React.createElement("header", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12,
      padding: '14px 20px',
      background: 'var(--surface)',
      borderBottom: '1px solid var(--border)',
      ...style
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.StatusDot, {
    tone: tone,
    size: "lg"
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1,
      minWidth: 0,
      display: 'flex',
      flexDirection: 'column'
    }
  }, /*#__PURE__*/React.createElement("strong", {
    style: {
      fontSize: 'var(--text-status)',
      fontWeight: 600,
      overflow: 'hidden',
      textOverflow: 'ellipsis',
      whiteSpace: 'nowrap'
    }
  }, title), subtitle && /*#__PURE__*/React.createElement("small", {
    style: {
      color: 'var(--text-muted)',
      fontSize: 'var(--text-hint)'
    }
  }, subtitle)), actions && /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 8
    }
  }, actions));
}
Object.assign(__ds_scope, { StatusHeader });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/layout/StatusHeader.jsx", error: String((e && e.message) || e) }); }

// components/marketing/Chip.jsx
try { (() => {
/** Outline pill for lists of names (launchers, GPUs). Optional leading tone dot. */
function Chip({
  tone,
  children,
  style
}) {
  return /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: 6,
      padding: '0.35rem 0.85rem',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-pill)',
      background: 'var(--surface)',
      fontSize: '0.9rem',
      lineHeight: 1.3,
      ...style
    }
  }, tone && /*#__PURE__*/React.createElement("span", {
    style: {
      width: 7,
      height: 7,
      borderRadius: '50%',
      background: 'var(--tone-' + tone + ')'
    }
  }), children);
}
Object.assign(__ds_scope, { Chip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/Chip.jsx", error: String((e && e.message) || e) }); }

// components/marketing/CodeWindow.jsx
try { (() => {
/** Dark code panel with a filename bar. Always dark, in both themes. */
function CodeWindow({
  filename,
  children,
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: 'var(--code-bg)',
      color: 'var(--code-text)',
      borderRadius: 'var(--radius-card)',
      boxShadow: 'var(--shadow-card)',
      overflow: 'hidden',
      minWidth: 0,
      ...style
    }
  }, filename && /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '0.6rem 1rem',
      font: '0.8rem var(--font-mono)',
      color: 'var(--code-muted)',
      borderBottom: '1px solid var(--wool-700)'
    }
  }, filename), /*#__PURE__*/React.createElement("pre", {
    style: {
      margin: 0,
      padding: '1.25rem',
      overflowX: 'auto',
      font: '0.86rem/1.65 var(--font-mono)'
    }
  }, /*#__PURE__*/React.createElement("code", null, children)));
}
Object.assign(__ds_scope, { CodeWindow });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/CodeWindow.jsx", error: String((e && e.message) || e) }); }

// components/marketing/DataTable.jsx
try { (() => {
/** Bordered table with caption and caps headers. rows = arrays of cells. */
function DataTable({
  caption,
  columns = [],
  rows = [],
  style
}) {
  const cell = {
    textAlign: 'left',
    padding: '0.75rem 1.25rem',
    borderBottom: '1px solid var(--border)',
    verticalAlign: 'top'
  };
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: 'var(--surface)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-card)',
      overflowX: 'auto',
      ...style
    }
  }, /*#__PURE__*/React.createElement("table", {
    style: {
      width: '100%',
      borderCollapse: 'collapse',
      fontSize: '0.95rem'
    }
  }, caption && /*#__PURE__*/React.createElement("caption", {
    style: {
      textAlign: 'left',
      padding: '1rem 1.25rem 0.25rem',
      fontWeight: 600,
      color: 'var(--text-muted)',
      fontSize: '0.85rem'
    }
  }, caption), /*#__PURE__*/React.createElement("thead", null, /*#__PURE__*/React.createElement("tr", null, columns.map((c, i) => /*#__PURE__*/React.createElement("th", {
    key: i,
    style: {
      ...cell,
      fontSize: '0.8rem',
      textTransform: 'uppercase',
      letterSpacing: '0.06em',
      color: 'var(--text-muted)'
    }
  }, c)))), /*#__PURE__*/React.createElement("tbody", null, rows.map((r, i) => /*#__PURE__*/React.createElement("tr", {
    key: i
  }, r.map((c, j) => /*#__PURE__*/React.createElement("td", {
    key: j,
    style: {
      ...cell,
      borderBottom: i === rows.length - 1 ? 0 : cell.borderBottom
    }
  }, c)))))));
}
Object.assign(__ds_scope, { DataTable });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/DataTable.jsx", error: String((e && e.message) || e) }); }

// components/marketing/Eyebrow.jsx
try { (() => {
/** Mint pill label above a hero headline. */
function Eyebrow({
  children,
  style
}) {
  return /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-block',
      fontSize: '0.85rem',
      fontWeight: 600,
      color: 'var(--accent)',
      background: 'var(--accent-soft)',
      padding: '0.3rem 0.8rem',
      borderRadius: 'var(--radius-pill)',
      ...style
    }
  }, children);
}
Object.assign(__ds_scope, { Eyebrow });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/Eyebrow.jsx", error: String((e && e.message) || e) }); }

// components/marketing/FaqItem.jsx
try { (() => {
const {
  useState
} = React;
/** Disclosure row with a mint +/− marker. */
function FaqItem({
  question,
  children,
  defaultOpen = false,
  style
}) {
  const [open, setOpen] = useState(defaultOpen);
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: 'var(--surface)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-xl)',
      padding: '0 1.25rem',
      ...style
    }
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    onClick: () => setOpen(!open),
    "aria-expanded": open,
    style: {
      all: 'unset',
      boxSizing: 'border-box',
      width: '100%',
      cursor: 'pointer',
      padding: '1rem 0',
      fontWeight: 600,
      display: 'flex',
      justifyContent: 'space-between',
      gap: '1rem'
    }
  }, question, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--accent)',
      fontSize: '1.3rem',
      lineHeight: 1
    }
  }, open ? '−' : '+')), open && /*#__PURE__*/React.createElement("p", {
    style: {
      margin: 0,
      paddingBottom: '1.1rem',
      color: 'var(--text-muted)'
    }
  }, children));
}
Object.assign(__ds_scope, { FaqItem });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/FaqItem.jsx", error: String((e && e.message) || e) }); }

// components/marketing/FeatureCard.jsx
try { (() => {
/** Feature tile: accent line icon, display-font title, muted copy. */
function FeatureCard({
  icon,
  title,
  children,
  tone = 'running',
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '1.4rem',
      background: 'var(--surface)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-card)',
      ...style
    }
  }, icon && /*#__PURE__*/React.createElement(__ds_scope.Icon, {
    name: icon,
    size: 28,
    color: 'var(--tone-' + tone + ')'
  }), /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: '0.8rem 0 0.35rem',
      fontFamily: 'var(--font-display)',
      fontWeight: 600,
      fontSize: 'var(--text-h3)',
      lineHeight: 1.15,
      letterSpacing: '-0.02em'
    }
  }, title), /*#__PURE__*/React.createElement("p", {
    style: {
      margin: 0,
      color: 'var(--text-muted)',
      fontSize: '0.96rem',
      lineHeight: 1.6
    }
  }, children));
}
Object.assign(__ds_scope, { FeatureCard });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/FeatureCard.jsx", error: String((e && e.message) || e) }); }

// components/marketing/StateTimeline.jsx
try { (() => {
/** Horizontal tone timeline: Running → Paused → Loading → Running. */
function StateTimeline({
  steps = [],
  style
}) {
  return /*#__PURE__*/React.createElement("ol", {
    style: {
      listStyle: 'none',
      margin: 0,
      padding: '1rem 1.25rem 1.25rem',
      display: 'grid',
      gridTemplateColumns: 'repeat(' + steps.length + ', minmax(0,1fr))',
      gap: '0.5rem',
      fontSize: '0.8rem',
      ...style
    }
  }, steps.map((s, i) => {
    const c = 'var(--tone-' + s.tone + ')';
    return /*#__PURE__*/React.createElement("li", {
      key: i,
      style: {
        display: 'flex',
        flexDirection: 'column',
        gap: '0.2rem',
        padding: '0.65rem',
        borderRadius: 'var(--radius-lg)',
        background: 'var(--bg-alt)'
      }
    }, /*#__PURE__*/React.createElement("span", {
      style: {
        width: 10,
        height: 10,
        borderRadius: '50%',
        background: c,
        animation: s.tone === 'loading' ? 'ndl-pulse 1.2s ease-in-out infinite' : 'none'
      }
    }), /*#__PURE__*/React.createElement("strong", {
      style: {
        fontSize: '0.82rem',
        lineHeight: 1.25,
        marginTop: 4
      }
    }, s.title), /*#__PURE__*/React.createElement("em", {
      style: {
        fontStyle: 'normal',
        color: 'var(--text-muted)'
      }
    }, s.note));
  }));
}
Object.assign(__ds_scope, { StateTimeline });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/StateTimeline.jsx", error: String((e && e.message) || e) }); }

// components/marketing/StepCard.jsx
try { (() => {
/** Numbered "How it works" step. */
function StepCard({
  n,
  title,
  children,
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'relative',
      padding: '1.5rem',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-card)',
      background: 'var(--surface)',
      ...style
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'grid',
      placeItems: 'center',
      width: 36,
      height: 36,
      borderRadius: '50%',
      background: 'var(--accent-soft)',
      color: 'var(--accent)',
      fontWeight: 700,
      fontFamily: 'var(--font-display)'
    }
  }, n), /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: '0.9rem 0 0.4rem',
      fontFamily: 'var(--font-display)',
      fontWeight: 600,
      fontSize: 'var(--text-h3)',
      lineHeight: 1.15
    }
  }, title), /*#__PURE__*/React.createElement("p", {
    style: {
      margin: 0,
      color: 'var(--text-muted)',
      fontSize: '0.96rem',
      lineHeight: 1.6
    }
  }, children));
}
Object.assign(__ds_scope, { StepCard });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/StepCard.jsx", error: String((e && e.message) || e) }); }

// components/marketing/VramBar.jsx
try { (() => {
/** Stacked VRAM bars that grow in. rows = [{ label, segments: [{ label, tone, pct }] }]. */
function VramBar({
  rows = [],
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '1.25rem 1.25rem 0.5rem',
      display: 'grid',
      gap: '0.8rem',
      ...style
    }
  }, rows.map((r, i) => /*#__PURE__*/React.createElement("div", {
    key: i,
    style: {
      display: 'grid',
      gridTemplateColumns: '4.5rem 1fr',
      alignItems: 'center',
      gap: '0.75rem',
      fontSize: '0.85rem'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--text-muted)'
    }
  }, r.label), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      height: 30,
      background: 'var(--bg-alt)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-md)',
      overflow: 'hidden'
    }
  }, r.segments.map((s, j) => /*#__PURE__*/React.createElement("div", {
    key: j,
    style: {
      width: s.pct + '%',
      display: 'flex',
      alignItems: 'center',
      height: '100%',
      paddingInline: '0.7rem',
      fontSize: '0.78rem',
      fontWeight: 600,
      color: 'var(--on-tone)',
      whiteSpace: 'nowrap',
      background: 'var(--tone-' + s.tone + ')',
      transformOrigin: 'left',
      animation: 'ndl-grow 1.2s var(--ease-out) both',
      animationDelay: i * 0.3 + 's'
    }
  }, s.label))))));
}
Object.assign(__ds_scope, { VramBar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/VramBar.jsx", error: String((e && e.message) || e) }); }

// components/marketing/WindowFrame.jsx
try { (() => {
/** Faux app window for marketing demos: three grey dots + mono caption. */
function WindowFrame({
  caption,
  children,
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    style: {
      background: 'var(--surface)',
      border: '1px solid var(--border)',
      borderRadius: 'var(--radius-card)',
      boxShadow: 'var(--shadow-card)',
      overflow: 'hidden',
      ...style
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 6,
      padding: '0.7rem 1rem',
      borderBottom: '1px solid var(--border)',
      background: 'var(--bg-alt)'
    }
  }, [0, 1, 2].map(i => /*#__PURE__*/React.createElement("span", {
    key: i,
    style: {
      width: 10,
      height: 10,
      borderRadius: '50%',
      background: 'var(--border-strong)'
    }
  })), caption && /*#__PURE__*/React.createElement("em", {
    style: {
      marginLeft: 'auto',
      fontStyle: 'normal',
      fontSize: '0.8rem',
      color: 'var(--text-muted)',
      fontFamily: 'var(--font-mono)'
    }
  }, caption)), children);
}
Object.assign(__ds_scope, { WindowFrame });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/marketing/WindowFrame.jsx", error: String((e && e.message) || e) }); }

// components/models/CatalogRow.jsx
try { (() => {
const FIT = {
  gpu: 'running',
  gpuAndRam: 'loading',
  tooBig: 'off'
};
/** Catalog entry: label, fit note (tone-coloured), and Download / Installed / Cancel. */
function CatalogRow({
  label,
  note,
  fit = 'gpu',
  recommended = false,
  state = 'download',
  onAction,
  last = false
}) {
  const tone = 'var(--tone-' + FIT[fit] + ')';
  const btn = state === 'installed' ? /*#__PURE__*/React.createElement(__ds_scope.Button, {
    disabled: true
  }, "Installed") : state === 'downloading' ? /*#__PURE__*/React.createElement(__ds_scope.Button, {
    onClick: onAction
  }, "Cancel") : /*#__PURE__*/React.createElement(__ds_scope.Button, {
    disabled: fit === 'tooBig' || state === 'unavailable',
    onClick: onAction
  }, "Download");
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      gap: 12,
      padding: '8px 0',
      borderBottom: last ? 'none' : '1px solid var(--border)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      minWidth: 0
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: fit === 'tooBig' ? 'var(--text-muted)' : 'var(--text)'
    }
  }, label), /*#__PURE__*/React.createElement("small", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 6,
      fontSize: 'var(--text-hint)',
      color: recommended ? 'var(--accent)' : 'var(--text-muted)',
      fontWeight: recommended ? 600 : 400
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      width: 6,
      height: 6,
      borderRadius: '50%',
      background: tone
    }
  }), note, recommended && ' · recommended ★')), btn);
}
Object.assign(__ds_scope, { CatalogRow });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/models/CatalogRow.jsx", error: String((e && e.message) || e) }); }

// components/models/ModelPicker.jsx
try { (() => {
const {
  useState
} = React;
const gb = b => (b / 1e9).toFixed(1) + ' GB';
function Opt({
  m,
  checked,
  onPick
}) {
  const [h, setH] = useState(false);
  return /*#__PURE__*/React.createElement("label", {
    onMouseEnter: () => setH(true),
    onMouseLeave: () => setH(false),
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10,
      padding: '7px 8px',
      borderRadius: 'var(--radius-sm)',
      cursor: 'pointer',
      background: h ? 'var(--hover)' : 'transparent'
    }
  }, /*#__PURE__*/React.createElement("input", {
    type: "radio",
    checked: checked,
    onChange: onPick,
    style: {
      accentColor: 'var(--accent)',
      margin: 0
    }
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      flex: 1,
      overflowWrap: 'anywhere'
    }
  }, m.name), /*#__PURE__*/React.createElement("small", {
    style: {
      color: m.size < 0 ? 'var(--tone-error)' : 'var(--text-muted)',
      fontSize: 'var(--text-hint)'
    }
  }, m.size < 0 ? 'missing' : gb(m.size)));
}
/** Radio list of installed .gguf models. size in bytes; size < 0 = missing. */
function ModelPicker({
  models = [],
  value,
  onChange,
  style
}) {
  return /*#__PURE__*/React.createElement("div", {
    role: "radiogroup",
    "aria-label": "Installed models",
    style: {
      display: 'flex',
      flexDirection: 'column',
      paddingTop: 6,
      ...style
    }
  }, models.map(m => /*#__PURE__*/React.createElement(Opt, {
    key: m.name,
    m: m,
    checked: m.name === value,
    onPick: () => onChange && onChange(m.name)
  })));
}
Object.assign(__ds_scope, { ModelPicker });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/models/ModelPicker.jsx", error: String((e && e.message) || e) }); }

// ui_kits/settings-window/App.jsx
try { (() => {
(() => {
  const {
    Button,
    StatusHeader,
    SideNav,
    SaveBar,
    Toast
  } = window.NoDramaLlamaDesignSystem_58ce7f;
  const {
    Overview,
    ModelPanel,
    ServerPanel,
    LayaPanel,
    GamesPanel,
    AppPanel
  } = window.NDLPanels;
  const TABS = [["overview", "Overview", Overview], ["model", "Model", ModelPanel], ["server", "Server & API", ServerPanel], ["laya", "Laya", LayaPanel], ["games", "Game detection", GamesPanel], ["app", "App", AppPanel]];
  const RESTARTS = ["Model", "Reasoning", "Context", "ListenHost", "Port", "ApiKey"];
  function App() {
    const [view, setView] = React.useState(() => ({
      ...window.NDL_MOCK,
      off: false,
      running: true,
      gameProcess: null,
      update: {
        state: "idle"
      },
      download: null
    }));
    const [draft, setDraft] = React.useState({});
    const [invalid, setInvalid] = React.useState({});
    const [tab, setTab] = React.useState(() => localStorage.getItem("ndl-tab") || "overview");
    const [toast, setToast] = React.useState(null);
    React.useEffect(() => {
      if (!toast) return;
      const t = setTimeout(() => setToast(null), toast.error ? 8000 : 2500);
      return () => clearTimeout(t);
    }, [toast]);
    const f = {
      value: k => k in draft ? draft[k] : view.settings[k],
      edit: (k, v, bad = false) => {
        setInvalid(s => {
          const n = {
            ...s
          };
          if (bad) n[k] = true;else delete n[k];
          return n;
        });
        setDraft(d => {
          const n = {
            ...d
          };
          if (JSON.stringify(v) === JSON.stringify(view.settings[k])) delete n[k];else n[k] = v;
          return n;
        });
      }
    };
    const changed = Object.keys(draft),
      bad = Object.keys(invalid).length;
    const save = () => {
      if (!changed.length || bad) return;
      setView(v => ({
        ...v,
        settings: {
          ...v.settings,
          ...draft
        }
      }));
      setDraft({});
      setToast({
        text: "Saved"
      });
    };
    const revert = () => {
      setDraft({});
      setInvalid({});
    };
    React.useEffect(() => {
      const k = e => {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
          e.preventDefault();
          save();
        } else if (e.key === "g" && !/INPUT|TEXTAREA|SELECT/.test(e.target.tagName)) send("game");
      };
      document.addEventListener("keydown", k);
      return () => document.removeEventListener("keydown", k);
    });
    const send = (cmd, id) => setView(v => {
      switch (cmd) {
        case "toggle":
          return v.off ? {
            ...v,
            off: false,
            running: true,
            tone: "running",
            statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL"
          } : {
            ...v,
            off: true,
            running: false,
            tone: "off",
            statusText: "Off"
          };
        case "restart":
          setTimeout(() => setView(w => ({
            ...w,
            tone: "running",
            running: true,
            statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context"
          })), 1800);
          return {
            ...v,
            tone: "loading",
            running: false,
            statusText: "Loading Qwen3.8-27B-UD-Q4_K_XL..."
          };
        case "game":
          return {
            ...v,
            tone: "paused",
            running: false,
            gameProcess: "eldenring.exe",
            statusText: "Paused for a game - eldenring.exe"
          };
        case "ignore_current_game":
          return {
            ...v,
            tone: "running",
            running: true,
            gameProcess: null,
            statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context"
          };
        case "check_for_updates":
          return {
            ...v,
            update: {
              state: "available",
              version: "2.2.0"
            }
          };
        case "download":
          {
            const m = v.catalog.find(c => c.id === id);
            return {
              ...v,
              download: {
                id,
                label: m.label,
                done: 0.37 * m.size,
                total: m.size
              }
            };
          }
        case "cancel_download":
          return {
            ...v,
            download: null
          };
        default:
          return v;
      }
    });
    const pick = id => {
      setTab(id);
      localStorage.setItem("ndl-tab", id);
    };
    const restarting = changed.some(k => RESTARTS.includes(k));
    const dirty = changed.length > 0 || bad > 0;
    return /*#__PURE__*/React.createElement("div", {
      style: {
        height: "100vh",
        display: "flex",
        flexDirection: "column"
      }
    }, /*#__PURE__*/React.createElement(StatusHeader, {
      tone: view.tone,
      title: view.statusText,
      subtitle: view.running ? `Chat and API at ${view.chatUrl}` : view.tone === "paused" ? "Comes back 60 s after the game closes" : "",
      actions: /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Button, {
        onClick: () => send("toggle")
      }, view.off ? "Turn on" : "Turn off"), /*#__PURE__*/React.createElement(Button, {
        disabled: view.off,
        onClick: () => send("restart")
      }, "Restart"), /*#__PURE__*/React.createElement(Button, {
        variant: "primary",
        disabled: !view.running
      }, "Open chat"))
    }), /*#__PURE__*/React.createElement("div", {
      style: {
        flex: 1,
        display: "flex",
        minHeight: 0
      }
    }, /*#__PURE__*/React.createElement(SideNav, {
      items: TABS.map(([id, title]) => ({
        id,
        title
      })),
      active: tab,
      onSelect: pick
    }), /*#__PURE__*/React.createElement("main", {
      style: {
        flex: 1,
        overflowY: "auto",
        padding: dirty ? "8px 24px 80px" : "8px 24px 24px"
      }
    }, TABS.map(([id, title, Panel]) => /*#__PURE__*/React.createElement("section", {
      key: id,
      hidden: id !== tab
    }, /*#__PURE__*/React.createElement("h2", {
      style: {
        fontFamily: "var(--font-display)",
        fontSize: "var(--text-app-h2)",
        fontWeight: 600,
        letterSpacing: "-0.02em",
        margin: "12px 0 14px"
      }
    }, title), /*#__PURE__*/React.createElement(Panel, {
      f: f,
      view: view,
      send: send
    }))))), dirty && /*#__PURE__*/React.createElement(SaveBar, {
      floating: true,
      saveDisabled: bad > 0 || !changed.length,
      onRevert: revert,
      onSave: save,
      message: bad ? `Fix the highlighted value${bad > 1 ? "s" : ""} to save` : `${changed.length} unsaved change${changed.length > 1 ? "s" : ""}${restarting ? " · saving restarts the model" : ""}`
    }), toast && /*#__PURE__*/React.createElement(Toast, {
      floating: true,
      error: toast.error
    }, toast.text));
  }
  ReactDOM.createRoot(document.getElementById("root")).render(/*#__PURE__*/React.createElement(App, null));
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/settings-window/App.jsx", error: String((e && e.message) || e) }); }

// ui_kits/settings-window/Panels.jsx
try { (() => {
(() => {
  const {
    Button,
    Fieldset,
    SettingRow,
    Switch,
    Select,
    NumberInput,
    TextInput,
    TextArea,
    FactList,
    Banner,
    DownloadProgress,
    ModelPicker,
    CatalogRow,
    StatusDot,
    StatusPopup
  } = window.NoDramaLlamaDesignSystem_58ce7f;
  const hint = {
    margin: "8px 0 0",
    color: "var(--text-muted)",
    fontSize: "var(--text-hint)"
  };
  const rowButtons = {
    display: "flex",
    flexWrap: "wrap",
    gap: 8,
    paddingTop: 10
  };
  const Tog = ({
    f,
    k,
    label,
    hint: h,
    last
  }) => /*#__PURE__*/React.createElement(SettingRow, {
    label: label,
    hint: h,
    last: last
  }, /*#__PURE__*/React.createElement(Switch, {
    checked: f.value(k),
    onChange: v => f.edit(k, v)
  }));
  const Num = ({
    f,
    k,
    label,
    hint: h,
    min,
    max,
    step = 1,
    last
  }) => {
    const v = f.value(k);
    const bad = v === "" || Number(v) < min || Number(v) > max;
    return /*#__PURE__*/React.createElement(SettingRow, {
      label: label,
      hint: h,
      last: last
    }, /*#__PURE__*/React.createElement(NumberInput, {
      value: v,
      min: min,
      max: max,
      step: step,
      invalid: bad,
      onChange: e => {
        const t = e.target.value;
        const n = t === "" ? "" : Number(t);
        f.edit(k, n, t === "" || n < min || n > max);
      }
    }));
  };
  const Pick = ({
    f,
    k,
    label,
    hint: h,
    options,
    last
  }) => /*#__PURE__*/React.createElement(SettingRow, {
    label: label,
    hint: h,
    last: last
  }, /*#__PURE__*/React.createElement(Select, {
    value: String(f.value(k)),
    options: options,
    onChange: e => f.edit(k, e.target.value)
  }));
  const List = ({
    f,
    k,
    label,
    hint: h
  }) => /*#__PURE__*/React.createElement(SettingRow, {
    label: label,
    hint: h,
    stacked: true,
    last: true
  }, /*#__PURE__*/React.createElement(TextArea, {
    value: f.value(k).join("\n"),
    onChange: e => f.edit(k, e.target.value.split(/\r?\n/))
  }));
  function Overview({
    f,
    view,
    send
  }) {
    const u = view.update;
    const updateText = {
      idle: "No update found at the last check",
      checking: "Checking...",
      available: `Version ${u.version} is available`
    }[u.state];
    return /*#__PURE__*/React.createElement(React.Fragment, null, view.gameProcess && /*#__PURE__*/React.createElement(Banner, {
      tone: "paused",
      action: /*#__PURE__*/React.createElement(Button, {
        onClick: () => send("ignore_current_game")
      }, "Ignore this app")
    }, "Paused for ", /*#__PURE__*/React.createElement("b", null, view.gameProcess), ". Not a game?"), /*#__PURE__*/React.createElement(FactList, {
      items: [["GPU", view.gpuName], ["llama.cpp build", view.backend], ["Context in use", view.running ? `${Math.round(view.nCtx / 1024)}K tokens` : "-"], ["Chat & OpenAI API", /*#__PURE__*/React.createElement("code", null, view.chatUrl)], ["Version", view.version]]
    }), /*#__PURE__*/React.createElement(SettingRow, {
      label: "Updates",
      hint: updateText,
      last: true
    }, /*#__PURE__*/React.createElement(Button, {
      disabled: u.state === "checking",
      onClick: () => send("check_for_updates")
    }, u.state === "available" ? `Get ${u.version}` : "Check for updates")), view.download && /*#__PURE__*/React.createElement(DownloadProgress, {
      label: view.download.label,
      done: view.download.done,
      total: view.download.total,
      onCancel: () => send("cancel_download"),
      note: "The app switches to it when it's done."
    }), /*#__PURE__*/React.createElement("p", {
      style: hint
    }, "Ctrl+Alt+L turns the LLM on or off from anywhere."));
  }
  function ModelPanel({
    f,
    view,
    send
  }) {
    return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Installed models"
    }, /*#__PURE__*/React.createElement(ModelPicker, {
      models: view.models,
      value: f.value("Model"),
      onChange: m => f.edit("Model", m)
    }), /*#__PURE__*/React.createElement("div", {
      style: rowButtons
    }, /*#__PURE__*/React.createElement(Button, null, "Open models folder")), /*#__PURE__*/React.createElement("p", {
      style: hint
    }, "Drop your own ", /*#__PURE__*/React.createElement("code", null, ".gguf"), " files into the models folder and they show up here.")), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Download a model"
    }, view.catalog.map((c, i) => /*#__PURE__*/React.createElement(CatalogRow, {
      key: c.id,
      label: c.label,
      note: c.note,
      fit: c.fit,
      recommended: c.recommended,
      last: i === view.catalog.length - 1,
      state: c.installed ? "installed" : view.download?.id === c.id ? "downloading" : view.download ? "unavailable" : "download",
      onAction: () => send(view.download?.id === c.id ? "cancel_download" : "download", c.id)
    }))), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Generation"
    }, /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "Reasoning",
      label: "Reasoning",
      hint: "How much the model thinks before it answers.",
      options: [["none", "None"], ["low", "Low"], ["medium", "Medium"], ["xhigh", "Extra high"]]
    }), /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "Context",
      label: "Context length",
      hint: "Auto lets llama.cpp use the largest that fits your GPU.",
      last: true,
      options: [["auto", "Auto (largest that fits)"], ...[8192, 16384, 32768, 65536, 131072, 262144].map(n => [String(n), `${n / 1024}K tokens`])]
    })));
  }
  function ServerPanel({
    f
  }) {
    const key = f.value("ApiKey");
    const bad = !/^[A-Za-z0-9._~-]*$/.test(key);
    const gen = () => btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(24)))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
    return /*#__PURE__*/React.createElement(Fieldset, null, /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "ListenHost",
      label: "Access",
      hint: "Who can reach the chat page and the API.",
      options: [["127.0.0.1", "This PC only"], ["0.0.0.0", "Devices on my network"]]
    }), /*#__PURE__*/React.createElement(Num, {
      f: f,
      k: "Port",
      label: "Port",
      hint: "1024 - 65535",
      min: 1024,
      max: 65535
    }), /*#__PURE__*/React.createElement(SettingRow, {
      label: "API key",
      hint: /*#__PURE__*/React.createElement(React.Fragment, null, "Optional. Clients must send it as a Bearer token. Letters, digits and ", /*#__PURE__*/React.createElement("code", null, ". _ - ~"), "."),
      stacked: true,
      last: true
    }, /*#__PURE__*/React.createElement("span", {
      style: {
        display: "flex",
        gap: 8
      }
    }, /*#__PURE__*/React.createElement(TextInput, {
      value: key,
      placeholder: "(none)",
      invalid: bad,
      onChange: e => f.edit("ApiKey", e.target.value, !/^[A-Za-z0-9._~-]*$/.test(e.target.value))
    }), /*#__PURE__*/React.createElement(Button, {
      onClick: () => f.edit("ApiKey", gen())
    }, "Generate"))), /*#__PURE__*/React.createElement("p", {
      style: hint
    }, "Changes on this page and to the model restart the server when you save."));
  }
  function LayaPanel({
    f,
    view
  }) {
    const on = f.value("RunLaya");
    return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("p", {
      style: {
        ...hint,
        margin: "0 0 14px",
        fontSize: "var(--text-app)"
      }
    }, "Laya is a ", /*#__PURE__*/React.createElement("i", null, "decision model"), ": it answers typed questions about a text (pick one of these options, score it, yes or no) in milliseconds, instead of writing text. Ollaya (ollaya.dev) serves it next to the LLM, with the same on/off switch and pause while gaming."), /*#__PURE__*/React.createElement(Fieldset, null, /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "RunLaya",
      last: true,
      label: "Run Laya alongside the LLM",
      hint: "Installs Ollaya (about 25 MB, plus 1.1 GB on NVIDIA GPUs) and downloads the model (about 1 GB)."
    })), view.settings.RunLaya && /*#__PURE__*/React.createElement(FactList, {
      items: [["Status", /*#__PURE__*/React.createElement("span", {
        style: {
          display: "inline-flex",
          alignItems: "center",
          gap: 8
        }
      }, /*#__PURE__*/React.createElement(StatusDot, {
        tone: "loading"
      }), " Downloading laya...")], ["API", /*#__PURE__*/React.createElement("code", null, "http://127.0.0.1:11435")], ["Ollaya", "0.5.0"]]
    }), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Model",
      disabled: !on
    }, /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "LayaModel",
      label: "Model",
      hint: "Other Ollaya models (decider, nli, ...) can go in the settings file.",
      options: [["laya", "Laya (picks English or multilingual per request)"], ["laya:en", "Laya English (421M, the fastest)"], ["laya:multilingual", "Laya multilingual (322M, 100+ languages)"], ["laya:typed-decisions", "Laya typed-decisions"]]
    }), /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "LayaKeepAlive",
      label: "Keep the model loaded",
      last: true,
      options: [["-1", "Always (until paused or off)"], ["1h", "1 hour after the last request"], ["30m", "30 minutes after the last request"], ["5m", "5 minutes after the last request"], ["0", "Unload after each request"]]
    })), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Server",
      disabled: !on
    }, /*#__PURE__*/React.createElement(Num, {
      f: f,
      k: "LayaPort",
      label: "Port",
      hint: "1024 - 65535, not the LLM's port",
      min: 1024,
      max: 65535
    }), /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "LayaDevice",
      label: "Run on",
      last: true,
      hint: "On the CPU, Laya keeps running while you play; on the GPU it pauses with the LLM.",
      options: [["auto", "Auto (NVIDIA GPU if there is one)"], ["cpu", "CPU only"], ["cuda", "NVIDIA GPU only"]]
    })));
  }
  function GamesPanel({
    f
  }) {
    const pause = f.value("PauseWhileGaming"),
      mode = f.value("DetectionMode");
    return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Fieldset, null, /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "PauseWhileGaming",
      label: "Pause while gaming",
      hint: "Stop the model so the game gets all of your VRAM."
    }), /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "DetectionMode",
      label: "Detect games by",
      options: [["Both", "GPU usage + launchers (recommended)"], ["Gpu", "GPU usage only"], ["Launchers", "Launchers only"]]
    }), /*#__PURE__*/React.createElement(Num, {
      f: f,
      k: "ResumeAfterSec",
      label: "Resume after the game closes",
      hint: "Seconds, 0 - 3600",
      min: 0,
      max: 3600,
      last: true
    })), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "GPU usage",
      disabled: !pause || mode === "Launchers"
    }, /*#__PURE__*/React.createElement(Num, {
      f: f,
      k: "GpuVramGB",
      label: "VRAM threshold",
      hint: "Another app using this many GB or more counts as a game.",
      min: 0.1,
      max: 256,
      step: 0.1
    }), /*#__PURE__*/React.createElement(Num, {
      f: f,
      k: "GpuLoadPct",
      label: "3D load threshold",
      hint: "Another app using this much of the 3D engine (%) counts as a game.",
      min: 1,
      max: 100
    }), /*#__PURE__*/React.createElement(List, {
      f: f,
      k: "GpuIgnore",
      label: "Never count as a game",
      hint: /*#__PURE__*/React.createElement(React.Fragment, null, "Process names, one per line (e.g. ", /*#__PURE__*/React.createElement("code", null, "obs64"), ").")
    }), /*#__PURE__*/React.createElement("div", {
      style: rowButtons
    }, /*#__PURE__*/React.createElement(Button, null, "Show GPU usage now"))), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Launchers",
      disabled: !pause || mode === "Gpu"
    }, /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "DetectEmulators",
      label: "Count emulators as games"
    }), /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "UseWindowsGameList",
      label: "Use Windows' game list",
      hint: "Games the Xbox Game Bar knows about."
    }), /*#__PURE__*/React.createElement(List, {
      f: f,
      k: "ExtraGames",
      label: "Extra games",
      hint: "Process names or folders, one per line."
    }), /*#__PURE__*/React.createElement("div", {
      style: rowButtons
    }, /*#__PURE__*/React.createElement(Button, null, "Rescan and show detected libraries"))));
  }
  function AppPanel({
    f,
    send
  }) {
    const [popup, setPopup] = React.useState(false);
    const test = () => {
      setPopup(true);
      setTimeout(() => setPopup(false), 2600);
    };
    return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(Fieldset, null, /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "StartWithWindows",
      label: "Start with Windows",
      hint: "Start No Drama Llama when you sign in."
    }), /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "AutoUpdate",
      label: "Update automatically",
      hint: "Installs signed releases from GitHub.",
      last: true
    })), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "On-screen popups"
    }, /*#__PURE__*/React.createElement(Tog, {
      f: f,
      k: "Popups",
      label: "Show popups",
      hint: "When the model pauses or resumes. They never take focus."
    }), /*#__PURE__*/React.createElement(Pick, {
      f: f,
      k: "PopupPosition",
      label: "Position",
      last: true,
      options: [["TopCenter", "Top center"], ["TopRight", "Top right"], ["BottomRight", "Bottom right"], ["BottomCenter", "Bottom center"]]
    }), /*#__PURE__*/React.createElement("div", {
      style: rowButtons
    }, /*#__PURE__*/React.createElement(Button, {
      onClick: test
    }, "Test popup"))), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Files"
    }, /*#__PURE__*/React.createElement("div", {
      style: rowButtons
    }, /*#__PURE__*/React.createElement(Button, null, "Edit settings file"), /*#__PURE__*/React.createElement(Button, null, "Open folder"), /*#__PURE__*/React.createElement(Button, null, "View log"))), /*#__PURE__*/React.createElement(Fieldset, {
      legend: "Quit"
    }, /*#__PURE__*/React.createElement(SettingRow, {
      label: "Exit No Drama Llama",
      hint: "Stops the model and removes the tray icon.",
      last: true
    }, /*#__PURE__*/React.createElement(Button, {
      variant: "danger",
      onClick: () => confirm("Exit No Drama Llama? This stops the model until you start the app again.")
    }, "Exit"))), popup && /*#__PURE__*/React.createElement("div", {
      style: {
        position: "fixed",
        top: 40,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 5
      }
    }, /*#__PURE__*/React.createElement(StatusPopup, {
      tone: "paused",
      title: "LLM paused",
      subtitle: "Test popup \xB7 this is where it shows"
    })));
  }
  window.NDLPanels = {
    Overview,
    ModelPanel,
    ServerPanel,
    LayaPanel,
    GamesPanel,
    AppPanel
  };
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/settings-window/Panels.jsx", error: String((e && e.message) || e) }); }

// ui_kits/settings-window/data.js
try { (() => {
// Sample tray state, mirrored from ui/src/mock.ts.
window.NDL_MOCK = (() => {
  const GB = 1e9;
  const cat = [["qwen3.8-27b:UD-Q6_K_XL", "Qwen 3.8 27B UD-Q6_K_XL", 24.1, "gpuAndRam", "partly in RAM - slow", false, false], ["qwen3.8-27b:UD-Q5_K_XL", "Qwen 3.8 27B UD-Q5_K_XL", 20.4, "gpu", "fits your GPU", true, false], ["qwen3.8-27b:UD-Q4_K_XL", "Qwen 3.8 27B UD-Q4_K_XL", 17.6, "gpu", "fits your GPU", false, true], ["qwen3.8-flash-next:UD-Q4_K_XL", "Qwen 3.8 Flash-Next UD-Q4_K_XL", 92.3, "tooBig", "too big for this PC", false, false]].map(([id, family, gb, fit, note, recommended, installed]) => ({
    id,
    label: `${family}  (${gb} GB)`,
    size: gb * GB,
    fit,
    note,
    recommended,
    installed
  }));
  return {
    version: "2.1.0",
    tone: "running",
    statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context",
    chatUrl: "http://127.0.0.1:8080",
    gpuName: "NVIDIA GeForce RTX 4090 (24 GB)",
    backend: "CUDA 13",
    nCtx: 65536,
    models: [{
      name: "Qwen3.8-27B-UD-Q4_K_XL.gguf",
      size: 17.6 * GB
    }, {
      name: "my-own-model.Q5_K_M.gguf",
      size: 9.1 * GB
    }],
    catalog: cat,
    settings: {
      Model: "Qwen3.8-27B-UD-Q4_K_XL.gguf",
      Reasoning: "low",
      Context: "auto",
      ListenHost: "127.0.0.1",
      Port: 8080,
      ApiKey: "",
      PauseWhileGaming: true,
      DetectEmulators: true,
      UseWindowsGameList: true,
      Popups: true,
      PopupPosition: "TopCenter",
      ExtraGames: [],
      DetectionMode: "Both",
      GpuVramGB: 1.5,
      GpuLoadPct: 30,
      ResumeAfterSec: 60,
      GpuIgnore: ["obs64"],
      AutoUpdate: true,
      StartWithWindows: true,
      RunLaya: false,
      LayaModel: "laya",
      LayaPort: 11435,
      LayaDevice: "auto",
      LayaKeepAlive: "-1"
    }
  };
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/settings-window/data.js", error: String((e && e.message) || e) }); }

// ui_kits/website/Sections.jsx
try { (() => {
(() => {
  const {
    PillButton,
    Icon,
    FeatureCard,
    Chip,
    CodeWindow,
    DataTable,
    FaqItem
  } = window.NoDramaLlamaDesignSystem_58ce7f;
  const S = window.NDLSite;
  const sec = alt => ({
    padding: "clamp(3.5rem, 8vw, 6rem) 0",
    ...(alt ? {
      background: "var(--bg-alt)",
      borderBlock: "1px solid var(--border)"
    } : {})
  });
  const lede = {
    color: "var(--text-muted)",
    fontSize: "1.1rem",
    maxWidth: "40em",
    margin: 0,
    textWrap: "pretty"
  };
  const link = {
    display: "inline-block",
    marginTop: "1.25rem",
    color: "var(--accent)",
    fontWeight: 600,
    textDecoration: "none"
  };
  const split = {
    ...S.wrap,
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(min(420px, 100%), 1fr))",
    gap: "clamp(2rem, 5vw, 4rem)",
    alignItems: "center"
  };
  const FEATURES = [["cpu", "running", "Fits any GPU", "NVIDIA, AMD or Intel. CUDA or Vulkan is chosen for you, and llama.cpp sizes the context and GPU layers to your free VRAM."], ["gamepad-2", "game", "Game detection that works", "Reads the records of a dozen-plus launchers and watches per-process GPU use, so it catches the games launchers miss."], ["code-xml", "paused", "OpenAI-compatible API", "Point any OpenAI client, editor plugin or agent at http://127.0.0.1:8080/v1. Add an API key to share it on your network."], ["app-window", "loading", "Lives in the tray", "A status icon, every setting in the right-click menu, and a click-through popup that never steals focus from your game."], ["keyboard", "game", "Ctrl+Alt+L", "Turn the model on or off from anywhere, even full-screen."], ["box", "running", "Pick your model", "A catalog of Qwen 3.8 quants rated for your PC, downloaded in the background. Or drop in any .gguf of your own."], ["battery-low", "loading", "Always on, low power", "No sleep, screen off after 10 minutes, PCIe/CPU/USB power saving and Wake-on-LAN. Uninstalling puts it all back."], ["shield-check", "paused", "Signed updates", "Updates itself from GitHub releases, but only ones signed with the project key for that exact version."]];
  const LAUNCHERS = ["Steam", "Epic", "GOG", "EA", "Ubisoft", "Battle.net", "Riot", "Rockstar", "Xbox / Game Pass", "Heroic", "Humble", "HoYoPlay", "Meta / Oculus", "Emulators"];
  function Features() {
    return /*#__PURE__*/React.createElement("section", {
      style: sec(true)
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...S.wrap,
        textAlign: "center"
      }
    }, /*#__PURE__*/React.createElement("h2", {
      style: S.h2
    }, "Everything a shared GPU needs"), /*#__PURE__*/React.createElement("p", {
      style: {
        ...lede,
        marginInline: "auto"
      }
    }, "Built for the PC that\u2019s a game machine at night and a model server the rest of the time."), /*#__PURE__*/React.createElement("div", {
      style: {
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))",
        gap: "1.25rem",
        marginTop: "2.5rem",
        textAlign: "left"
      }
    }, FEATURES.map(([i, t, h, p]) => /*#__PURE__*/React.createElement(FeatureCard, {
      key: h,
      icon: i,
      tone: t,
      title: h
    }, p))), /*#__PURE__*/React.createElement("div", {
      style: {
        marginTop: "2.5rem"
      }
    }, /*#__PURE__*/React.createElement("p", {
      style: {
        fontSize: "0.85rem",
        fontWeight: 600,
        textTransform: "uppercase",
        letterSpacing: "0.08em",
        color: "var(--text-muted)",
        margin: 0
      }
    }, "Knows where your games live"), /*#__PURE__*/React.createElement("div", {
      style: {
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "center",
        gap: "0.5rem",
        marginTop: "1rem"
      }
    }, LAUNCHERS.map(l => /*#__PURE__*/React.createElement(Chip, {
      key: l
    }, l))))));
  }
  const k = t => /*#__PURE__*/React.createElement("span", {
    style: {
      color: "var(--grape-300)"
    }
  }, t);
  const s = t => /*#__PURE__*/React.createElement("span", {
    style: {
      color: "var(--mint-300)"
    }
  }, t);
  function Api() {
    return /*#__PURE__*/React.createElement("section", {
      id: "api",
      style: sec(false)
    }, /*#__PURE__*/React.createElement("div", {
      style: split
    }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("h2", {
      style: S.h2
    }, "Talks OpenAI"), /*#__PURE__*/React.createElement("p", {
      style: lede
    }, "llama.cpp\u2019s server speaks the OpenAI API, so the tools you already use work unchanged: SDKs, editor assistants, agents and chat front-ends. There\u2019s also a built-in chat UI: just double-click the tray icon."), /*#__PURE__*/React.createElement("a", {
      href: "#",
      style: link
    }, "Connect your tools \u2192")), /*#__PURE__*/React.createElement(CodeWindow, {
      filename: "chat.ts"
    }, k("import"), " OpenAI ", k("from"), " ", s("'openai'"), ";", "\n\n", k("const"), " llama = ", k("new"), " OpenAI(", "{", "\n", "  baseURL: ", s("'http://127.0.0.1:8080/v1'"), ",", "\n", "  apiKey: ", s("'not-needed-on-localhost'"), ",", "\n", "}", ");", "\n\n", k("const"), " reply = ", k("await"), " llama.chat.completions.create(", "{", "\n", "  model: ", s("'local'"), ",", "\n", "  messages: [", "{", " role: ", s("'user'"), ", content: ", s("'Hello!'"), " ", "}", "],", "\n", "}", ");", "\n", "console.log(reply.choices[0].message.content);")));
  }
  function Models() {
    const c = t => /*#__PURE__*/React.createElement("code", {
      style: {
        color: "var(--accent)"
      }
    }, t);
    const g = (a, b) => /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("b", null, a), b && /*#__PURE__*/React.createElement("span", {
      style: {
        display: "block",
        fontSize: "0.82rem",
        color: "var(--text-muted)"
      }
    }, b));
    return /*#__PURE__*/React.createElement("section", {
      style: sec(true)
    }, /*#__PURE__*/React.createElement("div", {
      style: split
    }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("h2", {
      style: S.h2
    }, "The best model your PC can run"), /*#__PURE__*/React.createElement("p", {
      style: lede
    }, "The installer measures your GPU and RAM and picks a model for you. Every model in the catalog is rated ", /*#__PURE__*/React.createElement("i", null, "fits your GPU"), ", ", /*#__PURE__*/React.createElement("i", null, "GPU + RAM"), ", ", /*#__PURE__*/React.createElement("i", null, "partly in RAM"), " or ", /*#__PURE__*/React.createElement("i", null, "too big"), ", and you can switch any time from the tray."), /*#__PURE__*/React.createElement("a", {
      href: "#",
      style: link
    }, "How models are chosen \u2192")), /*#__PURE__*/React.createElement(DataTable, {
      caption: "Recommended model by GPU memory",
      columns: ["GPU memory", "Recommended"],
      rows: [[g("48 GB"), c("27B Q8_0")], [g("32 GB", "RTX 5090"), c("27B UD-Q6_K_XL")], [g("24 GB", "RX 7900 XTX, RTX 4090"), c("27B UD-Q4_K_XL")], [g("20 GB", "RX 7900 XT"), c("27B UD-IQ4_XS")], [g("16 GB", "RTX 4080, RX 7800 XT"), c("27B UD-IQ3_XXS")], [g("8 – 12 GB"), c("27B UD-IQ2_XXS, or Flash-Next with 96 GB+ RAM")]]
    })));
  }
  function Trust() {
    const items = [["Verified downloads.", "llama.cpp and every model are checked against their published SHA-256 before use."], ["Signed updates.", "The updater only installs releases signed for that exact version with the project’s minisign key."], ["Locked-down install.", "Everything that runs elevated lives in admin-only folders, and every setting is validated before it reaches a command line."], ["Leaves no trace.", "Uninstalling restores your original power and Wake-on-LAN settings and removes the app."]];
    return /*#__PURE__*/React.createElement("section", {
      style: sec(false)
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...S.wrap,
        width: "min(760px, 100% - 32px)"
      }
    }, /*#__PURE__*/React.createElement("h2", {
      style: S.h2
    }, "Careful with your machine"), /*#__PURE__*/React.createElement("ul", {
      style: {
        listStyle: "none",
        padding: 0,
        margin: "2rem 0 0",
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
        gap: "1rem 2.5rem"
      }
    }, items.map(([a, b]) => /*#__PURE__*/React.createElement("li", {
      key: a,
      style: {
        paddingLeft: "1.75rem",
        position: "relative",
        color: "var(--text-muted)"
      }
    }, /*#__PURE__*/React.createElement("span", {
      style: {
        position: "absolute",
        left: 0,
        top: "0.45em",
        width: 12,
        height: 12,
        borderRadius: "50%",
        background: "var(--accent)"
      }
    }), /*#__PURE__*/React.createElement("strong", {
      style: {
        color: "var(--text)"
      }
    }, a), " ", b)))));
  }
  function Faq() {
    const qs = [["Is it free?", "Yes. No Drama Llama is open source, and everything runs on your own PC. Nothing is sent to a cloud service."], ["Will it slow down my games?", "No. When a game starts, the model is stopped completely, so the game gets all of your VRAM. It only comes back after the game has been closed for a while (60 seconds by default)."], ["What if it pauses for something that isn’t a game?", "Open the tray menu while it’s paused and choose “Not a game – ignore app”. You can also tune the GPU thresholds or switch detection to launchers only."], ["Can I use it from my laptop or phone?", "Yes. Set Access to “Devices on my network” and set an API key. Then any device on your network can use the chat UI or the API."], ["Does it work without a GPU?", "It runs on the CPU, slowly. Any GPU with a Vulkan or CUDA driver works, from 8 GB cards up."], ["Why does Windows say “Windows protected your PC”?", "The exe isn’t code-signed yet. Choose More info → Run anyway, or build it yourself from source. Updates are still verified with the project’s signing key."]];
    return /*#__PURE__*/React.createElement("section", {
      style: sec(true)
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...S.wrap,
        width: "min(760px, 100% - 32px)"
      }
    }, /*#__PURE__*/React.createElement("h2", {
      style: S.h2
    }, "Questions"), /*#__PURE__*/React.createElement("div", {
      style: {
        marginTop: "2rem",
        display: "grid",
        gap: "0.75rem"
      }
    }, qs.map(([q, a]) => /*#__PURE__*/React.createElement(FaqItem, {
      key: q,
      question: q
    }, a)))));
  }
  function FinalCta() {
    return /*#__PURE__*/React.createElement("section", {
      style: sec(false)
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...S.wrap,
        width: "min(760px, 100% - 32px)",
        textAlign: "center"
      }
    }, /*#__PURE__*/React.createElement("img", {
      src: "../../assets/logo.svg",
      width: "72",
      height: "72",
      alt: "",
      style: {
        marginBottom: "1.25rem"
      }
    }), /*#__PURE__*/React.createElement("h2", {
      style: {
        ...S.h2,
        fontSize: "var(--text-cta)"
      }
    }, "Game on. ", /*#__PURE__*/React.createElement("span", {
      style: {
        color: "var(--accent)"
      }
    }, "The llama will wait.")), /*#__PURE__*/React.createElement("p", {
      style: {
        ...lede,
        marginInline: "auto"
      }
    }, "Download the exe, choose ", /*#__PURE__*/React.createElement("i", null, "Yes"), " to install, and it takes care of the rest."), /*#__PURE__*/React.createElement("div", {
      style: {
        display: "flex",
        flexWrap: "wrap",
        gap: "0.75rem",
        marginTop: "2rem",
        justifyContent: "center"
      }
    }, /*#__PURE__*/React.createElement(PillButton, {
      href: S.DL,
      icon: /*#__PURE__*/React.createElement(Icon, {
        name: "download"
      })
    }, "Download for Windows"), /*#__PURE__*/React.createElement(PillButton, {
      variant: "ghost",
      href: "#"
    }, "Installation guide"))));
  }
  function Footer() {
    const a = {
      color: "var(--text-muted)",
      textDecoration: "none"
    };
    return /*#__PURE__*/React.createElement("footer", {
      style: {
        borderTop: "1px solid var(--border)",
        padding: "2rem 0",
        fontSize: "0.9rem",
        color: "var(--text-muted)"
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...S.wrap,
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "space-between",
        gap: "1rem"
      }
    }, /*#__PURE__*/React.createElement("span", null, "No Drama Llama \xB7 runs ", /*#__PURE__*/React.createElement("a", {
      style: {
        ...a,
        color: "var(--text)"
      },
      href: "https://github.com/ggml-org/llama.cpp"
    }, "llama.cpp")), /*#__PURE__*/React.createElement("nav", {
      style: {
        display: "flex",
        gap: "1.25rem"
      }
    }, /*#__PURE__*/React.createElement("a", {
      style: a,
      href: "#"
    }, "Docs"), /*#__PURE__*/React.createElement("a", {
      style: a,
      href: "#"
    }, "Changelog"), /*#__PURE__*/React.createElement("a", {
      style: a,
      href: "#"
    }, "Releases"), /*#__PURE__*/React.createElement("a", {
      style: a,
      href: "https://github.com/singerbj/no-drama-llama"
    }, "GitHub"))));
  }
  function Site() {
    const [theme, setTheme] = React.useState(() => localStorage.getItem("ndl-site-theme") || "dark");
    React.useEffect(() => {
      document.documentElement.dataset.theme = theme;
      localStorage.setItem("ndl-site-theme", theme);
    }, [theme]);
    return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement(S.Nav, {
      theme: theme,
      setTheme: setTheme
    }), /*#__PURE__*/React.createElement("main", null, /*#__PURE__*/React.createElement(S.Hero, null), /*#__PURE__*/React.createElement(S.HowItWorks, null), /*#__PURE__*/React.createElement(Features, null), /*#__PURE__*/React.createElement(Api, null), /*#__PURE__*/React.createElement(Models, null), /*#__PURE__*/React.createElement(Trust, null), /*#__PURE__*/React.createElement(Faq, null), /*#__PURE__*/React.createElement(FinalCta, null)), /*#__PURE__*/React.createElement(Footer, null));
  }
  ReactDOM.createRoot(document.getElementById("root")).render(/*#__PURE__*/React.createElement(Site, null));
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/website/Sections.jsx", error: String((e && e.message) || e) }); }

// ui_kits/website/Top.jsx
try { (() => {
(() => {
  const {
    PillButton,
    Icon,
    Eyebrow,
    StatusPopup,
    WindowFrame,
    VramBar,
    StateTimeline,
    StepCard
  } = window.NoDramaLlamaDesignSystem_58ce7f;
  const wrap = {
    width: "min(1120px, 100% - 32px)",
    marginInline: "auto"
  };
  const DL = "https://github.com/singerbj/no-drama-llama/releases/latest";
  function Nav({
    theme,
    setTheme
  }) {
    const link = {
      color: "var(--text-muted)",
      textDecoration: "none"
    };
    return /*#__PURE__*/React.createElement("header", {
      style: {
        position: "sticky",
        top: 0,
        zIndex: 5,
        background: "color-mix(in srgb, var(--bg) 85%, transparent)",
        backdropFilter: "blur(10px)",
        borderBottom: "1px solid var(--border)"
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...wrap,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        height: 64,
        gap: "1rem"
      }
    }, /*#__PURE__*/React.createElement("a", {
      href: "#",
      style: {
        display: "flex",
        alignItems: "center",
        gap: "0.6rem",
        fontWeight: 700,
        color: "var(--text)",
        textDecoration: "none",
        whiteSpace: "nowrap",
        fontFamily: "var(--font-display)",
        fontSize: 15
      }
    }, /*#__PURE__*/React.createElement("img", {
      src: "../../assets/logo.svg",
      width: "32",
      height: "32",
      alt: ""
    }), "No Drama Llama"), /*#__PURE__*/React.createElement("nav", {
      style: {
        display: "flex",
        alignItems: "center",
        gap: "1.25rem",
        fontSize: "0.95rem"
      }
    }, /*#__PURE__*/React.createElement("a", {
      style: link,
      href: "#"
    }, "Docs"), /*#__PURE__*/React.createElement("a", {
      style: link,
      href: "#api"
    }, "API"), /*#__PURE__*/React.createElement("a", {
      style: link,
      href: "https://github.com/singerbj/no-drama-llama"
    }, "GitHub"), /*#__PURE__*/React.createElement("button", {
      onClick: () => setTheme(theme === "dark" ? "light" : "dark"),
      "aria-label": "Toggle theme",
      style: {
        display: "grid",
        placeItems: "center",
        width: 36,
        height: 36,
        border: "1px solid var(--border)",
        borderRadius: 999,
        background: "transparent",
        color: "var(--text-muted)",
        cursor: "pointer"
      }
    }, /*#__PURE__*/React.createElement(Icon, {
      name: theme === "dark" ? "sun" : "moon",
      size: 18
    })), /*#__PURE__*/React.createElement(PillButton, {
      size: "sm",
      href: DL
    }, "Download"))));
  }
  function Hero() {
    return /*#__PURE__*/React.createElement("section", {
      style: {
        padding: "clamp(3rem, 8vw, 6rem) 0 clamp(3rem, 6vw, 5rem)",
        background: "var(--pattern-grid)",
        backgroundSize: "var(--pattern-grid-size)"
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        ...wrap,
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(min(440px, 100%), 1fr))",
        gap: "clamp(2rem, 5vw, 4rem)",
        alignItems: "center"
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        minWidth: 0
      }
    }, /*#__PURE__*/React.createElement(Eyebrow, {
      style: {
        marginBottom: "1.25rem"
      }
    }, "Free & open source \xB7 Windows 10 and 11"), /*#__PURE__*/React.createElement("h1", {
      style: {
        margin: 0,
        fontFamily: "var(--font-display)",
        fontWeight: 700,
        fontSize: "var(--text-h1)",
        lineHeight: 1.1,
        letterSpacing: "-0.035em",
        textWrap: "balance"
      }
    }, "Your gaming PC is an AI server. ", /*#__PURE__*/React.createElement("span", {
      style: {
        display: "block",
        color: "var(--accent)"
      }
    }, "It just knows when to step aside.")), /*#__PURE__*/React.createElement("p", {
      style: {
        fontSize: "1.2rem",
        color: "var(--text-muted)",
        marginTop: "1.25rem",
        maxWidth: "34em",
        textWrap: "pretty"
      }
    }, "No Drama Llama runs a local LLM on your GPU around the clock. The moment a game starts, it stops the model and hands the game all of your VRAM. Quit, and it\u2019s back."), /*#__PURE__*/React.createElement("div", {
      style: {
        display: "flex",
        flexWrap: "wrap",
        gap: "0.75rem",
        marginTop: "2rem"
      }
    }, /*#__PURE__*/React.createElement(PillButton, {
      href: DL,
      icon: /*#__PURE__*/React.createElement(Icon, {
        name: "download"
      })
    }, "Download for Windows"), /*#__PURE__*/React.createElement(PillButton, {
      variant: "ghost",
      href: "#"
    }, "Read the docs \u2192")), /*#__PURE__*/React.createElement("p", {
      style: {
        marginTop: "1rem",
        fontSize: "0.9rem",
        color: "var(--text-muted)"
      }
    }, "One exe \xB7 installs in minutes \xB7 uninstall restores every setting")), /*#__PURE__*/React.createElement("div", {
      style: {
        position: "relative",
        paddingTop: "4.25rem",
        minWidth: 0
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        position: "absolute",
        top: 0,
        left: "50%",
        translate: "-50% 0",
        zIndex: 1
      }
    }, /*#__PURE__*/React.createElement(StatusPopup, {
      animated: true,
      tone: "paused",
      title: "LLM paused",
      subtitle: "Example Game (Steam) detected \xB7 GPU freed"
    })), /*#__PURE__*/React.createElement(WindowFrame, {
      caption: "VRAM \xB7 24 GB"
    }, /*#__PURE__*/React.createElement(VramBar, {
      rows: [{
        label: "Before",
        segments: [{
          label: "Qwen 3.8 27B",
          tone: "running",
          pct: 74
        }]
      }, {
        label: "In game",
        segments: [{
          label: "Your game",
          tone: "game",
          pct: 88
        }]
      }]
    }), /*#__PURE__*/React.createElement(StateTimeline, {
      steps: [{
        tone: "running",
        title: "Running",
        note: "serving requests"
      }, {
        tone: "paused",
        title: "Paused for a game",
        note: "VRAM freed"
      }, {
        tone: "loading",
        title: "Loading",
        note: "game closed"
      }, {
        tone: "running",
        title: "Running",
        note: "back to work"
      }]
    })))));
  }
  function HowItWorks() {
    return /*#__PURE__*/React.createElement("section", {
      style: {
        padding: "clamp(3.5rem, 8vw, 6rem) 0"
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: wrap
    }, /*#__PURE__*/React.createElement("h2", {
      style: window.NDLSite.h2
    }, "How it works"), /*#__PURE__*/React.createElement("div", {
      style: {
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))",
        gap: "1.5rem",
        marginTop: "2.5rem"
      }
    }, /*#__PURE__*/React.createElement(StepCard, {
      n: 1,
      title: "Install one exe"
    }, "It detects your GPU, installs the fastest llama.cpp build for it, and downloads the best model that fits. Every download is verified."), /*#__PURE__*/React.createElement(StepCard, {
      n: 2,
      title: "It serves, all day"
    }, "An OpenAI-compatible API and a chat UI at 127.0.0.1:8080. It starts with Windows and runs in the tray."), /*#__PURE__*/React.createElement(StepCard, {
      n: 3,
      title: "You play, it steps aside"
    }, "Launch a game and the model stops, so the game gets all of your VRAM. Quit, and it comes back a minute later."))));
  }
  window.NDLSite = {
    ...(window.NDLSite || {}),
    Nav,
    Hero,
    HowItWorks,
    wrap,
    DL,
    h2: {
      margin: "0 0 0.75rem",
      fontFamily: "var(--font-display)",
      fontWeight: 700,
      fontSize: "var(--text-h2)",
      lineHeight: 1.15,
      letterSpacing: "-0.02em",
      textWrap: "balance"
    }
  };
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/website/Top.jsx", error: String((e && e.message) || e) }); }

__ds_ns.Button = __ds_scope.Button;

__ds_ns.Icon = __ds_scope.Icon;

__ds_ns.PillButton = __ds_scope.PillButton;

__ds_ns.StatusDot = __ds_scope.StatusDot;

__ds_ns.Banner = __ds_scope.Banner;

__ds_ns.DownloadProgress = __ds_scope.DownloadProgress;

__ds_ns.SaveBar = __ds_scope.SaveBar;

__ds_ns.StatusPopup = __ds_scope.StatusPopup;

__ds_ns.Toast = __ds_scope.Toast;

__ds_ns.NumberInput = __ds_scope.NumberInput;

__ds_ns.Select = __ds_scope.Select;

__ds_ns.SettingRow = __ds_scope.SettingRow;

__ds_ns.Switch = __ds_scope.Switch;

__ds_ns.TextArea = __ds_scope.TextArea;

__ds_ns.TextInput = __ds_scope.TextInput;

__ds_ns.FactList = __ds_scope.FactList;

__ds_ns.Fieldset = __ds_scope.Fieldset;

__ds_ns.SideNav = __ds_scope.SideNav;

__ds_ns.StatusHeader = __ds_scope.StatusHeader;

__ds_ns.Chip = __ds_scope.Chip;

__ds_ns.CodeWindow = __ds_scope.CodeWindow;

__ds_ns.DataTable = __ds_scope.DataTable;

__ds_ns.Eyebrow = __ds_scope.Eyebrow;

__ds_ns.FaqItem = __ds_scope.FaqItem;

__ds_ns.FeatureCard = __ds_scope.FeatureCard;

__ds_ns.StateTimeline = __ds_scope.StateTimeline;

__ds_ns.StepCard = __ds_scope.StepCard;

__ds_ns.VramBar = __ds_scope.VramBar;

__ds_ns.WindowFrame = __ds_scope.WindowFrame;

__ds_ns.CatalogRow = __ds_scope.CatalogRow;

__ds_ns.ModelPicker = __ds_scope.ModelPicker;

})();
