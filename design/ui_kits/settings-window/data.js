// Sample tray state, mirrored from ui/src/mock.ts.
window.NDL_MOCK = (() => {
  const GB = 1e9;
  const cat = [
    ["qwen3.8-27b:UD-Q6_K_XL", "Qwen 3.8 27B UD-Q6_K_XL", 24.1, "gpuAndRam", "partly in RAM - slow", false, false],
    ["qwen3.8-27b:UD-Q5_K_XL", "Qwen 3.8 27B UD-Q5_K_XL", 20.4, "gpu", "fits your GPU", true, false],
    ["qwen3.8-27b:UD-Q4_K_XL", "Qwen 3.8 27B UD-Q4_K_XL", 17.6, "gpu", "fits your GPU", false, true],
    ["qwen3.8-flash-next:UD-Q4_K_XL", "Qwen 3.8 Flash-Next UD-Q4_K_XL", 92.3, "tooBig", "too big for this PC", false, false],
  ].map(([id, family, gb, fit, note, recommended, installed]) => ({ id, label: `${family}  (${gb} GB)`, size: gb * GB, fit, note, recommended, installed }));
  return {
    version: "2.1.0",
    tone: "running",
    statusText: "Running - Qwen3.8-27B-UD-Q4_K_XL · 64K context",
    chatUrl: "http://127.0.0.1:8080",
    gpuName: "NVIDIA GeForce RTX 4090 (24 GB)",
    backend: "CUDA 13",
    nCtx: 65536,
    models: [{ name: "Qwen3.8-27B-UD-Q4_K_XL.gguf", size: 17.6 * GB }, { name: "my-own-model.Q5_K_M.gguf", size: 9.1 * GB }],
    catalog: cat,
    settings: {
      Model: "Qwen3.8-27B-UD-Q4_K_XL.gguf", Reasoning: "low", Context: "auto", ListenHost: "127.0.0.1", Port: 8080, ApiKey: "",
      PauseWhileGaming: true, DetectEmulators: true, UseWindowsGameList: true, Popups: true, PopupPosition: "TopCenter",
      ExtraGames: [], DetectionMode: "Both", GpuVramGB: 1.5, GpuLoadPct: 30, ResumeAfterSec: 60, GpuIgnore: ["obs64"],
      AutoUpdate: true, StartWithWindows: true, RunLaya: false, LayaModel: "laya", LayaPort: 11435, LayaDevice: "auto", LayaKeepAlive: "-1",
    },
  };
})();
