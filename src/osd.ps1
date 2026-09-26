# On-screen popup (OSD) for LLM state changes.
# Borderless, always-on-top, click-through, and it never takes focus from your game.
# Visible over borderless/windowed games; exclusive-fullscreen games draw over everything.
Add-Type -ReferencedAssemblies System.Windows.Forms, System.Drawing -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
using System.Runtime.InteropServices;
using System.Windows.Forms;

public class LlmOsd : Form
{
    [DllImport("user32.dll")] static extern bool SetProcessDPIAware();
    public static void EnableDpiAwareness() { try { SetProcessDPIAware(); } catch { } }

    static LlmOsd current;

    readonly string title, subtitle;
    readonly Color accent;
    readonly int holdMs;
    readonly float scale;
    readonly Font titleFont, subFont;
    readonly System.Windows.Forms.Timer timer;
    int phase, elapsed;

    public static void Popup(string title, string subtitle, Color accent, int holdMs, string position)
    {
        if (current != null && !current.IsDisposed) { current.timer.Stop(); current.Close(); current.Dispose(); }
        current = new LlmOsd(title, subtitle, accent, holdMs, position);
        current.Show();
    }

    LlmOsd(string title, string subtitle, Color accent, int holdMs, string position)
    {
        this.title = title; this.subtitle = subtitle; this.accent = accent; this.holdMs = holdMs;
        using (Graphics g = Graphics.FromHwnd(IntPtr.Zero)) scale = g.DpiX / 96f;
        titleFont = new Font("Segoe UI Semibold", 13f);
        subFont   = new Font("Segoe UI", 10f);

        FormBorderStyle = FormBorderStyle.None;
        ShowInTaskbar = false;
        StartPosition = FormStartPosition.Manual;
        BackColor = Color.FromArgb(28, 28, 32);
        DoubleBuffered = true;
        Opacity = 0;

        int pad = S(18), dot = S(12), gap = S(12);
        Size t = TextRenderer.MeasureText(title, titleFont);
        Size s = TextRenderer.MeasureText(subtitle ?? "", subFont);
        int w = Math.Max(S(300), pad + dot + gap + Math.Max(t.Width, s.Width) + pad);
        int h = pad + t.Height + S(2) + s.Height + pad;
        Size = new Size(w, h);

        using (GraphicsPath p = Rounded(new Rectangle(0, 0, w, h), S(12))) Region = new Region(p);

        Rectangle wa = Screen.PrimaryScreen.WorkingArea;
        int m = S(24);
        switch ((position ?? "").ToLowerInvariant())
        {
            case "topright":     Location = new Point(wa.Right - w - m, wa.Top + m); break;
            case "bottomright":  Location = new Point(wa.Right - w - m, wa.Bottom - h - m); break;
            case "bottomcenter": Location = new Point(wa.Left + (wa.Width - w) / 2, wa.Bottom - h - S(96)); break;
            default:             Location = new Point(wa.Left + (wa.Width - w) / 2, wa.Top + S(40)); break; // topcenter
        }

        timer = new System.Windows.Forms.Timer();
        timer.Interval = 15;
        timer.Tick += Step;
        timer.Start();
    }

    int S(int px) { return (int)Math.Round(px * scale); }

    static GraphicsPath Rounded(Rectangle r, int radius)
    {
        int d = radius * 2;
        GraphicsPath p = new GraphicsPath();
        p.AddArc(r.X, r.Y, d, d, 180, 90);
        p.AddArc(r.Right - d, r.Y, d, d, 270, 90);
        p.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
        p.AddArc(r.X, r.Bottom - d, d, d, 90, 90);
        p.CloseFigure();
        return p;
    }

    void Step(object sender, EventArgs e)
    {
        if (phase == 0) { Opacity = Math.Min(0.96, Opacity + 0.12); if (Opacity >= 0.96) phase = 1; }
        else if (phase == 1) { elapsed += timer.Interval; if (elapsed >= holdMs) phase = 2; }
        else { Opacity -= 0.06; if (Opacity <= 0.02) { timer.Stop(); Close(); } }
    }

    // Never steal focus; stay on top; let clicks pass through to the game.
    protected override bool ShowWithoutActivation { get { return true; } }
    protected override CreateParams CreateParams
    {
        get
        {
            CreateParams cp = base.CreateParams;
            cp.ExStyle |= 0x08000000 /*NOACTIVATE*/ | 0x00000080 /*TOOLWINDOW*/ | 0x00000008 /*TOPMOST*/
                        | 0x00000020 /*TRANSPARENT*/ | 0x00080000 /*LAYERED*/;
            return cp;
        }
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        Graphics g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;
        int pad = S(18), dot = S(12), gap = S(12);
        int titleH = TextRenderer.MeasureText(title, titleFont).Height;

        using (SolidBrush a = new SolidBrush(accent))
        {
            g.FillRectangle(a, 0, 0, S(4), Height);                               // accent edge
            g.FillEllipse(a, pad, pad + (titleH - dot) / 2, dot, dot);             // status dot
        }
        int x = pad + dot + gap;
        TextRenderer.DrawText(g, title, titleFont, new Point(x, pad), Color.White);
        TextRenderer.DrawText(g, subtitle ?? "", subFont, new Point(x, pad + titleH + S(2)), Color.FromArgb(190, 190, 198));

        using (GraphicsPath p = Rounded(new Rectangle(0, 0, Width - 1, Height - 1), S(12)))
        using (Pen border = new Pen(Color.FromArgb(70, 255, 255, 255)))
            g.DrawPath(border, p);
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing) { titleFont.Dispose(); subFont.Dispose(); timer.Dispose(); }
        base.Dispose(disposing);
    }
}
'@
