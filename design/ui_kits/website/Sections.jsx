(() => {
const { PillButton, Icon, FeatureCard, Chip, CodeWindow, DataTable, FaqItem } = window.NoDramaLlamaDesignSystem_58ce7f;
const S = window.NDLSite;
const sec = (alt) => ({ padding: "clamp(3.5rem, 8vw, 6rem) 0", ...(alt ? { background: "var(--bg-alt)", borderBlock: "1px solid var(--border)" } : {}) });
const lede = { color: "var(--text-muted)", fontSize: "1.1rem", maxWidth: "40em", margin: 0, textWrap: "pretty" };
const link = { display: "inline-block", marginTop: "1.25rem", color: "var(--accent)", fontWeight: 600, textDecoration: "none" };
const split = { ...S.wrap, display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(min(420px, 100%), 1fr))", gap: "clamp(2rem, 5vw, 4rem)", alignItems: "center" };

const FEATURES = [
  ["cpu", "running", "Fits any GPU", "NVIDIA, AMD or Intel. CUDA or Vulkan is chosen for you, and llama.cpp sizes the context and GPU layers to your free VRAM."],
  ["gamepad-2", "game", "Game detection that works", "Reads the records of a dozen-plus launchers and watches per-process GPU use, so it catches the games launchers miss."],
  ["code-xml", "paused", "OpenAI-compatible API", "Point any OpenAI client, editor plugin or agent at http://127.0.0.1:8080/v1. Add an API key to share it on your network."],
  ["app-window", "loading", "Lives in the tray", "A status icon, every setting in the right-click menu, and a click-through popup that never steals focus from your game."],
  ["keyboard", "game", "Ctrl+Alt+L", "Turn the model on or off from anywhere, even full-screen."],
  ["box", "running", "Pick your model", "A catalog of Qwen 3.8 quants rated for your PC, downloaded in the background. Or drop in any .gguf of your own."],
  ["battery-low", "loading", "Always on, low power", "No sleep, screen off after 10 minutes, PCIe/CPU/USB power saving and Wake-on-LAN. Uninstalling puts it all back."],
  ["shield-check", "paused", "Signed updates", "Updates itself from GitHub releases, but only ones signed with the project key for that exact version."],
];
const LAUNCHERS = ["Steam", "Epic", "GOG", "EA", "Ubisoft", "Battle.net", "Riot", "Rockstar", "Xbox / Game Pass", "Heroic", "Humble", "HoYoPlay", "Meta / Oculus", "Emulators"];

function Features() {
  return <section style={sec(true)}><div style={{ ...S.wrap, textAlign: "center" }}>
    <h2 style={S.h2}>Everything a shared GPU needs</h2>
    <p style={{ ...lede, marginInline: "auto" }}>Built for the PC that’s a game machine at night and a model server the rest of the time.</p>
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: "1.25rem", marginTop: "2.5rem", textAlign: "left" }}>
      {FEATURES.map(([i, t, h, p]) => <FeatureCard key={h} icon={i} tone={t} title={h}>{p}</FeatureCard>)}
    </div>
    <div style={{ marginTop: "2.5rem" }}>
      <p style={{ fontSize: "0.85rem", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.08em", color: "var(--text-muted)", margin: 0 }}>Knows where your games live</p>
      <div style={{ display: "flex", flexWrap: "wrap", justifyContent: "center", gap: "0.5rem", marginTop: "1rem" }}>{LAUNCHERS.map((l) => <Chip key={l}>{l}</Chip>)}</div>
    </div>
  </div></section>;
}

const k = (t) => <span style={{ color: "var(--grape-300)" }}>{t}</span>;
const s = (t) => <span style={{ color: "var(--mint-300)" }}>{t}</span>;
function Api() {
  return <section id="api" style={sec(false)}><div style={split}>
    <div><h2 style={S.h2}>Talks OpenAI</h2>
      <p style={lede}>llama.cpp’s server speaks the OpenAI API, so the tools you already use work unchanged: SDKs, editor assistants, agents and chat front-ends. There’s also a built-in chat UI: just double-click the tray icon.</p>
      <a href="#" style={link}>Connect your tools →</a></div>
    <CodeWindow filename="chat.ts">{k("import")} OpenAI {k("from")} {s("'openai'")};{"\n\n"}{k("const")} llama = {k("new")} OpenAI({"{"}{"\n"}  baseURL: {s("'http://127.0.0.1:8080/v1'")},{"\n"}  apiKey: {s("'not-needed-on-localhost'")},{"\n"}{"}"});{"\n\n"}{k("const")} reply = {k("await")} llama.chat.completions.create({"{"}{"\n"}  model: {s("'local'")},{"\n"}  messages: [{"{"} role: {s("'user'")}, content: {s("'Hello!'")} {"}"}],{"\n"}{"}"});{"\n"}console.log(reply.choices[0].message.content);</CodeWindow>
  </div></section>;
}

function Models() {
  const c = (t) => <code style={{ color: "var(--accent)" }}>{t}</code>;
  const g = (a, b) => <><b>{a}</b>{b && <span style={{ display: "block", fontSize: "0.82rem", color: "var(--text-muted)" }}>{b}</span>}</>;
  return <section style={sec(true)}><div style={split}>
    <div><h2 style={S.h2}>The best model your PC can run</h2>
      <p style={lede}>The installer measures your GPU and RAM and picks a model for you. Every model in the catalog is rated <i>fits your GPU</i>, <i>GPU + RAM</i>, <i>partly in RAM</i> or <i>too big</i>, and you can switch any time from the tray.</p>
      <a href="#" style={link}>How models are chosen →</a></div>
    <DataTable caption="Recommended model by GPU memory" columns={["GPU memory", "Recommended"]} rows={[[g("48 GB"), c("27B Q8_0")], [g("32 GB", "RTX 5090"), c("27B UD-Q6_K_XL")], [g("24 GB", "RX 7900 XTX, RTX 4090"), c("27B UD-Q4_K_XL")], [g("20 GB", "RX 7900 XT"), c("27B UD-IQ4_XS")], [g("16 GB", "RTX 4080, RX 7800 XT"), c("27B UD-IQ3_XXS")], [g("8 – 12 GB"), c("27B UD-IQ2_XXS, or Flash-Next with 96 GB+ RAM")]]} />
  </div></section>;
}

function Trust() {
  const items = [["Verified downloads.", "llama.cpp and every model are checked against their published SHA-256 before use."], ["Signed updates.", "The updater only installs releases signed for that exact version with the project’s minisign key."], ["Locked-down install.", "Everything that runs elevated lives in admin-only folders, and every setting is validated before it reaches a command line."], ["Leaves no trace.", "Uninstalling restores your original power and Wake-on-LAN settings and removes the app."]];
  return <section style={sec(false)}><div style={{ ...S.wrap, width: "min(760px, 100% - 32px)" }}>
    <h2 style={S.h2}>Careful with your machine</h2>
    <ul style={{ listStyle: "none", padding: 0, margin: "2rem 0 0", display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: "1rem 2.5rem" }}>
      {items.map(([a, b]) => <li key={a} style={{ paddingLeft: "1.75rem", position: "relative", color: "var(--text-muted)" }}><span style={{ position: "absolute", left: 0, top: "0.45em", width: 12, height: 12, borderRadius: "50%", background: "var(--accent)" }} /><strong style={{ color: "var(--text)" }}>{a}</strong> {b}</li>)}
    </ul>
  </div></section>;
}

function Faq() {
  const qs = [["Is it free?", "Yes. No Drama Llama is open source, and everything runs on your own PC. Nothing is sent to a cloud service."], ["Will it slow down my games?", "No. When a game starts, the model is stopped completely, so the game gets all of your VRAM. It only comes back after the game has been closed for a while (60 seconds by default)."], ["What if it pauses for something that isn’t a game?", "Open the tray menu while it’s paused and choose “Not a game – ignore app”. You can also tune the GPU thresholds or switch detection to launchers only."], ["Can I use it from my laptop or phone?", "Yes. Set Access to “Devices on my network” and set an API key. Then any device on your network can use the chat UI or the API."], ["Does it work without a GPU?", "It runs on the CPU, slowly. Any GPU with a Vulkan or CUDA driver works, from 8 GB cards up."], ["Why does Windows say “Windows protected your PC”?", "The exe isn’t code-signed yet. Choose More info → Run anyway, or build it yourself from source. Updates are still verified with the project’s signing key."]];
  return <section style={sec(true)}><div style={{ ...S.wrap, width: "min(760px, 100% - 32px)" }}>
    <h2 style={S.h2}>Questions</h2>
    <div style={{ marginTop: "2rem", display: "grid", gap: "0.75rem" }}>{qs.map(([q, a]) => <FaqItem key={q} question={q}>{a}</FaqItem>)}</div>
  </div></section>;
}

function FinalCta() {
  return <section style={sec(false)}><div style={{ ...S.wrap, width: "min(760px, 100% - 32px)", textAlign: "center" }}>
    <img src="../../assets/logo.svg" width="72" height="72" alt="" style={{ marginBottom: "1.25rem" }} />
    <h2 style={{ ...S.h2, fontSize: "var(--text-cta)" }}>Game on. <span style={{ color: "var(--accent)" }}>The llama will wait.</span></h2>
    <p style={{ ...lede, marginInline: "auto" }}>Download the exe, choose <i>Yes</i> to install, and it takes care of the rest.</p>
    <div style={{ display: "flex", flexWrap: "wrap", gap: "0.75rem", marginTop: "2rem", justifyContent: "center" }}><PillButton href={S.DL} icon={<Icon name="download" />}>Download for Windows</PillButton><PillButton variant="ghost" href="#">Installation guide</PillButton></div>
  </div></section>;
}

function Footer() {
  const a = { color: "var(--text-muted)", textDecoration: "none" };
  return <footer style={{ borderTop: "1px solid var(--border)", padding: "2rem 0", fontSize: "0.9rem", color: "var(--text-muted)" }}>
    <div style={{ ...S.wrap, display: "flex", flexWrap: "wrap", justifyContent: "space-between", gap: "1rem" }}>
      <span>No Drama Llama · runs <a style={{ ...a, color: "var(--text)" }} href="https://github.com/ggml-org/llama.cpp">llama.cpp</a></span>
      <nav style={{ display: "flex", gap: "1.25rem" }}><a style={a} href="#">Docs</a><a style={a} href="#">Changelog</a><a style={a} href="#">Releases</a><a style={a} href="https://github.com/singerbj/no-drama-llama">GitHub</a></nav>
    </div>
  </footer>;
}

function Site() {
  const [theme, setTheme] = React.useState(() => localStorage.getItem("ndl-site-theme") || "dark");
  React.useEffect(() => { document.documentElement.dataset.theme = theme; localStorage.setItem("ndl-site-theme", theme); }, [theme]);
  return <><S.Nav theme={theme} setTheme={setTheme} /><main><S.Hero /><S.HowItWorks /><Features /><Api /><Models /><Trust /><Faq /><FinalCta /></main><Footer /></>;
}
ReactDOM.createRoot(document.getElementById("root")).render(<Site />);
})();
