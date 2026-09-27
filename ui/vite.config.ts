import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Tauri serves ui/dist from the exe (see tauri.conf.json): index.html is the settings window,
// setup.html the setup wizard. `npm run dev` opens them in a browser with sample data
// (src/mock.ts, src/setup/mock.ts) for working on the layout.
export default defineConfig({
  plugins: [react()],
  base: "./",
  clearScreen: false,
  // The design system (tokens and fonts) lives in ../design, outside this package.
  server: { port: 5173, strictPort: true, fs: { allow: [".", "../design"] } },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
    // WebView2 is Chromium: no legacy polyfills needed.
    modulePreload: { polyfill: false },
    rollupOptions: {
      input: { settings: "index.html", setup: "setup.html" },
    },
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    restoreMocks: true,
  },
});
