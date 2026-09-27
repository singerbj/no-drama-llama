import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Setup } from "./Setup";
import "../style.css";
import "./setup.css";

// Dark by default, like the settings window; follow Windows when it's set to light mode.
const light = window.matchMedia?.("(prefers-color-scheme: light)");
const applyTheme = () => {
  if (light?.matches) document.documentElement.dataset.theme = "light";
  else delete document.documentElement.dataset.theme;
};
applyTheme();
light?.addEventListener("change", applyTheme);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Setup />
  </StrictMode>,
);
