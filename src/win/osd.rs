//! On-screen popup for LLM state changes: borderless, always on top, click-through, fades in
//! and out, and never takes focus from your game. Visible over borderless/windowed games;
//! exclusive full-screen games draw over everything. Must be used from the UI thread.

use crate::settings::PopupPosition;
use crate::state::Tone;
use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DeleteObject,
    Ellipse, EndPaint, FillRect, GetDC, GetStockObject, GetTextExtentPoint32W, ReleaseDC,
    RoundRect, SelectObject, SetBkMode, SetTextColor, SetWindowRgn, TextOutW, CLEARTYPE_QUALITY,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, HFONT, HGDIOBJ, NULL_BRUSH, NULL_PEN, OUT_DEFAULT_PRECIS,
    PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, KillTimer, RegisterClassW,
    SetLayeredWindowAttributes, SetTimer, SetWindowPos, ShowWindow, SystemParametersInfoW,
    HTTRANSPARENT, HWND_TOPMOST, LWA_ALPHA, MA_NOACTIVATE, SPI_GETWORKAREA, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WM_DESTROY,
    WM_MOUSEACTIVATE, WM_NCHITTEST, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

const CLASS: PCWSTR = w!("NoDramaLlamaOsd");
const HOLD_MS: u32 = 2600;
const TICK_MS: u32 = 15;

pub fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

/// The design system's status tones (design/tokens/colors.css, the dark-ground set): mint,
/// marigold, sky, pebble and tomato. Running is also the logo's disc.
pub fn tone_rgb(t: Tone) -> (u8, u8, u8) {
    match t {
        Tone::Running => (0x3D, 0xDC, 0x84),
        Tone::Loading => (0xFF, 0xB6, 0x27),
        Tone::Paused => (0x5A, 0xB0, 0xFF),
        Tone::Off => (0xA8, 0x9C, 0x90),
        Tone::Error => (0xFF, 0x5A, 0x4E),
    }
}

struct Osd {
    hwnd: HWND,
    title: Vec<u16>,
    sub: Vec<u16>,
    accent: COLORREF,
    title_font: HFONT,
    sub_font: HFONT,
    scale: f32,
    title_h: i32,
    phase: u8,
    alpha: f32,
    elapsed: u32,
}

thread_local! {
    static CURRENT: RefCell<Option<Osd>> = const { RefCell::new(None) };
    static REGISTERED: RefCell<bool> = const { RefCell::new(false) };
}

fn font(px: i32, weight: i32) -> HFONT {
    unsafe {
        CreateFontW(
            -px,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Segoe UI"),
        )
    }
}

fn text_size(f: HFONT, text: &[u16]) -> SIZE {
    let mut sz = SIZE::default();
    unsafe {
        let dc = GetDC(None);
        let old = SelectObject(dc, HGDIOBJ(f.0));
        let _ = GetTextExtentPoint32W(dc, text, &mut sz);
        SelectObject(dc, old);
        ReleaseDC(None, dc);
    }
    sz
}

pub fn show(title: &str, subtitle: &str, tone: Tone, position: PopupPosition) {
    close_current();
    unsafe {
        let Ok(hinst) = GetModuleHandleW(None) else {
            return;
        };
        REGISTERED.with(|r| {
            if !*r.borrow() {
                let wc = WNDCLASSW {
                    lpfnWndProc: Some(wndproc),
                    hInstance: hinst.into(),
                    lpszClassName: CLASS,
                    ..Default::default()
                };
                RegisterClassW(&wc);
                *r.borrow_mut() = true;
            }
        });

        let scale = GetDpiForSystem() as f32 / 96.0;
        let s = |px: i32| (px as f32 * scale).round() as i32;
        let title_font = font(s(18), 600);
        let sub_font = font(s(14), 400);
        let tw: Vec<u16> = title.encode_utf16().collect();
        let sw: Vec<u16> = subtitle.encode_utf16().collect();
        let (t, st) = (text_size(title_font, &tw), text_size(sub_font, &sw));
        let (pad, dot, gap) = (s(18), s(12), s(12));
        let w = (pad + dot + gap + t.cx.max(st.cx) + pad).max(s(300));
        let h = pad + t.cy + s(2) + st.cy + pad;

        let mut wa = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut wa as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        let m = s(24);
        let (x, y) = match position {
            PopupPosition::TopRight => (wa.right - w - m, wa.top + m),
            PopupPosition::BottomRight => (wa.right - w - m, wa.bottom - h - m),
            PopupPosition::BottomCenter => (
                wa.left + (wa.right - wa.left - w) / 2,
                wa.bottom - h - s(96),
            ),
            PopupPosition::TopCenter => (wa.left + (wa.right - wa.left - w) / 2, wa.top + s(40)),
        };

        let Ok(hwnd) = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            CLASS,
            w!(""),
            WS_POPUP,
            x,
            y,
            w,
            h,
            None,
            None,
            Some(hinst.into()),
            None,
        ) else {
            let _ = DeleteObject(HGDIOBJ(title_font.0));
            let _ = DeleteObject(HGDIOBJ(sub_font.0));
            return;
        };
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
        SetWindowRgn(
            hwnd,
            Some(CreateRoundRectRgn(0, 0, w + 1, h + 1, s(24), s(24))),
            false,
        );
        let (r, g, b) = tone_rgb(tone);
        CURRENT.with(|c| {
            *c.borrow_mut() = Some(Osd {
                hwnd,
                title: tw,
                sub: sw,
                accent: rgb(r, g, b),
                title_font,
                sub_font,
                scale,
                title_h: t.cy,
                phase: 0,
                alpha: 0.0,
                elapsed: 0,
            })
        });
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        );
        SetTimer(Some(hwnd), 1, TICK_MS, None);
    }
}

pub(crate) fn close_current() {
    let hwnd = CURRENT.with(|c| c.borrow().as_ref().map(|o| o.hwnd));
    if let Some(h) = hwnd {
        unsafe {
            let _ = DestroyWindow(h); // WM_DESTROY frees the rest
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_TIMER => {
            let done = CURRENT.with(|c| {
                let mut c = c.borrow_mut();
                let Some(o) = c.as_mut().filter(|o| o.hwnd == hwnd) else {
                    return true;
                };
                match o.phase {
                    0 => {
                        o.alpha = (o.alpha + 0.12).min(0.96);
                        if o.alpha >= 0.96 {
                            o.phase = 1;
                        }
                    }
                    1 => {
                        o.elapsed += TICK_MS;
                        if o.elapsed >= HOLD_MS {
                            o.phase = 2;
                        }
                    }
                    _ => o.alpha -= 0.06,
                }
                let _ = SetLayeredWindowAttributes(
                    hwnd,
                    COLORREF(0),
                    (o.alpha.max(0.0) * 255.0) as u8,
                    LWA_ALPHA,
                );
                o.phase == 2 && o.alpha <= 0.02
            });
            if done {
                let _ = KillTimer(Some(hwnd), 1);
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            CURRENT.with(|c| {
                if let Some(o) = c.borrow().as_ref().filter(|o| o.hwnd == hwnd) {
                    paint(o, dc, hwnd);
                }
            });
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            CURRENT.with(|c| {
                let mut c = c.borrow_mut();
                if c.as_ref().is_some_and(|o| o.hwnd == hwnd) {
                    let o = c.take().unwrap();
                    let _ = DeleteObject(HGDIOBJ(o.title_font.0));
                    let _ = DeleteObject(HGDIOBJ(o.sub_font.0));
                }
            });
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn paint(o: &Osd, dc: windows::Win32::Graphics::Gdi::HDC, hwnd: HWND) {
    let s = |px: i32| (px as f32 * o.scale).round() as i32;
    let mut rc = RECT::default();
    let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rc);
    let (pad, dot, gap) = (s(18), s(12), s(12));

    let bg = CreateSolidBrush(rgb(28, 28, 32));
    FillRect(dc, &rc, bg);
    let _ = DeleteObject(HGDIOBJ(bg.0));

    let accent = CreateSolidBrush(o.accent);
    let old_brush = SelectObject(dc, HGDIOBJ(accent.0));
    let old_pen = SelectObject(dc, GetStockObject(NULL_PEN));
    let dy = pad + (o.title_h - dot) / 2;
    let _ = Ellipse(dc, pad, dy, pad + dot, dy + dot); // status dot
    SelectObject(dc, old_pen);
    SelectObject(dc, old_brush);
    let _ = DeleteObject(HGDIOBJ(accent.0));

    SetBkMode(dc, TRANSPARENT);
    let x = pad + dot + gap;
    let old_font = SelectObject(dc, HGDIOBJ(o.title_font.0));
    SetTextColor(dc, rgb(255, 255, 255));
    let _ = TextOutW(dc, x, pad, &o.title);
    SelectObject(dc, HGDIOBJ(o.sub_font.0));
    SetTextColor(dc, rgb(190, 190, 198));
    let _ = TextOutW(dc, x, pad + o.title_h + s(2), &o.sub);
    SelectObject(dc, old_font);

    let pen = CreatePen(PS_SOLID, 1, rgb(70, 70, 76));
    let old_pen = SelectObject(dc, HGDIOBJ(pen.0));
    let old_brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
    let _ = RoundRect(dc, 0, 0, rc.right, rc.bottom, s(24), s(24));
    SelectObject(dc, old_brush);
    SelectObject(dc, old_pen);
    let _ = DeleteObject(HGDIOBJ(pen.0));
}
