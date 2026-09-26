import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Tauri serves ui/dist from the exe (see tauri.conf.json). `npm run dev` opens the page in a
// browser with sample data (src/mock.ts) for working on the layout.
export default defineConfig({
  plugins: [react()],
  base: "./",
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
    // WebView2 is Chromium: no legacy polyfills needed.
    modulePreload: { polyfill: false },
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    restoreMocks: true,
  },
});
