import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./style.css";

// The design system is dark by default; follow Windows when it's set to light mode.
const light = window.matchMedia?.("(prefers-color-scheme: light)");
const applyTheme = () => {
  if (light?.matches) document.documentElement.dataset.theme = "light";
  else delete document.documentElement.dataset.theme;
};
applyTheme();
light?.addEventListener("change", applyTheme);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
