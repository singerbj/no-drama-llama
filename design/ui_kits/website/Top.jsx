(() => {
const { PillButton, Icon, Eyebrow, StatusPopup, WindowFrame, VramBar, StateTimeline, StepCard } = window.NoDramaLlamaDesignSystem_58ce7f;
const wrap = { width: "min(1120px, 100% - 32px)", marginInline: "auto" };
const DL = "https://github.com/singerbj/no-drama-llama/releases/latest";

function Nav({ theme, setTheme }) {
  const link = { color: "var(--text-muted)", textDecoration: "none" };
  return <header style={{ position: "sticky", top: 0, zIndex: 5, background: "color-mix(in srgb, var(--bg) 85%, transparent)", backdropFilter: "blur(10px)", borderBottom: "1px solid var(--border)" }}>
    <div style={{ ...wrap, display: "flex", alignItems: "center", justifyContent: "space-between", height: 64, gap: "1rem" }}>
      <a href="#" style={{ display: "flex", alignItems: "center", gap: "0.6rem", fontWeight: 700, color: "var(--text)", textDecoration: "none", whiteSpace: "nowrap", fontFamily: "var(--font-display)", fontSize: 15 }}><img src="../../assets/logo.svg" width="32" height="32" alt="" />No Drama Llama</a>
      <nav style={{ display: "flex", alignItems: "center", gap: "1.25rem", fontSize: "0.95rem" }}>
        <a style={link} href="#">Docs</a><a style={link} href="#api">API</a><a style={link} href="https://github.com/singerbj/no-drama-llama">GitHub</a>
        <button onClick={() => setTheme(theme === "dark" ? "light" : "dark")} aria-label="Toggle theme" style={{ display: "grid", placeItems: "center", width: 36, height: 36, border: "1px solid var(--border)", borderRadius: 999, background: "transparent", color: "var(--text-muted)", cursor: "pointer" }}><Icon name={theme === "dark" ? "sun" : "moon"} size={18} /></button>
        <PillButton size="sm" href={DL}>Download</PillButton>
      </nav>
    </div>
  </header>;
}

function Hero() {
  return <section style={{ padding: "clamp(3rem, 8vw, 6rem) 0 clamp(3rem, 6vw, 5rem)", background: "var(--pattern-grid)", backgroundSize: "var(--pattern-grid-size)" }}>
    <div style={{ ...wrap, display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(min(440px, 100%), 1fr))", gap: "clamp(2rem, 5vw, 4rem)", alignItems: "center" }}>
      <div style={{ minWidth: 0 }}>
        <Eyebrow style={{ marginBottom: "1.25rem" }}>Free &amp; open source · Windows 10 and 11</Eyebrow>
        <h1 style={{ margin: 0, fontFamily: "var(--font-display)", fontWeight: 700, fontSize: "var(--text-h1)", lineHeight: 1.1, letterSpacing: "-0.035em", textWrap: "balance" }}>Your gaming PC is an AI server. <span style={{ display: "block", color: "var(--accent)" }}>It just knows when to step aside.</span></h1>
        <p style={{ fontSize: "1.2rem", color: "var(--text-muted)", marginTop: "1.25rem", maxWidth: "34em", textWrap: "pretty" }}>No Drama Llama runs a local LLM on your GPU around the clock. The moment a game starts, it stops the model and hands the game all of your VRAM. Quit, and it’s back.</p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "0.75rem", marginTop: "2rem" }}>
          <PillButton href={DL} icon={<Icon name="download" />}>Download for Windows</PillButton>
          <PillButton variant="ghost" href="#">Read the docs →</PillButton>
        </div>
        <p style={{ marginTop: "1rem", fontSize: "0.9rem", color: "var(--text-muted)" }}>One exe · installs in minutes · uninstall restores every setting</p>
      </div>
      <div style={{ position: "relative", paddingTop: "4.25rem", minWidth: 0 }}>
        <div style={{ position: "absolute", top: 0, left: "50%", translate: "-50% 0", zIndex: 1 }}><StatusPopup animated tone="paused" title="LLM paused" subtitle="Example Game (Steam) detected · GPU freed" /></div>
        <WindowFrame caption="VRAM · 24 GB">
          <VramBar rows={[{ label: "Before", segments: [{ label: "Qwen 3.8 27B", tone: "running", pct: 74 }] }, { label: "In game", segments: [{ label: "Your game", tone: "game", pct: 88 }] }]} />
          <StateTimeline steps={[{ tone: "running", title: "Running", note: "serving requests" }, { tone: "paused", title: "Paused for a game", note: "VRAM freed" }, { tone: "loading", title: "Loading", note: "game closed" }, { tone: "running", title: "Running", note: "back to work" }]} />
        </WindowFrame>
      </div>
    </div>
  </section>;
}

function HowItWorks() {
  return <section style={{ padding: "clamp(3.5rem, 8vw, 6rem) 0" }}><div style={wrap}>
    <h2 style={window.NDLSite.h2}>How it works</h2>
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))", gap: "1.5rem", marginTop: "2.5rem" }}>
      <StepCard n={1} title="Install one exe">It detects your GPU, installs the fastest llama.cpp build for it, and downloads the best model that fits. Every download is verified.</StepCard>
      <StepCard n={2} title="It serves, all day">An OpenAI-compatible API and a chat UI at 127.0.0.1:8080. It starts with Windows and runs in the tray.</StepCard>
      <StepCard n={3} title="You play, it steps aside">Launch a game and the model stops, so the game gets all of your VRAM. Quit, and it comes back a minute later.</StepCard>
    </div>
  </div></section>;
}

window.NDLSite = { ...(window.NDLSite || {}), Nav, Hero, HowItWorks, wrap, DL, h2: { margin: "0 0 0.75rem", fontFamily: "var(--font-display)", fontWeight: 700, fontSize: "var(--text-h2)", lineHeight: 1.15, letterSpacing: "-0.02em", textWrap: "balance" } };
})();
