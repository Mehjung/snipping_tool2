mod modules;

use modules::drawing::Drawing;
use modules::renderer::Render;
use std::mem::ManuallyDrop;
use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::Common::*, Direct2D::*, Direct3D::*, Direct3D11::*, DirectComposition::*,
            Dxgi::Common::*, Dxgi::*, Gdi::*,
        },
        System::LibraryLoader::*,
        System::SystemServices::*,
        UI::WindowsAndMessaging::*,
    },
};

macro_rules! get_x_lparam {
    ($lparam:expr) => {
        ($lparam & 0xFFFF) as i16 as i32 // Cast to i16 first to handle negative coordinates correctly
    };
}

macro_rules! get_y_lparam {
    ($lparam:expr) => {
        (($lparam >> 16) & 0xFFFF) as i16 as i32 // Shift right 16 bits and then cast to i16
    };
}

fn main() -> Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleA(None)?.into();
        let window_class = s!("TransparentWindowClass");

        let wc = WNDCLASSA {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: window_class,
            ..Default::default()
        };

        RegisterClassA(&wc);

        let hwnd = CreateWindowExA(
            WS_EX_NOREDIRECTIONBITMAP,
            window_class,
            s!("Transparent Window"),
            WS_POPUP,
            100,
            100,
            800,
            600,
            None,
            None,
            instance,
            None,
        );

        if hwnd.0 == 0 {
            panic!("Failed to create window");
        }

        let render = Render::new(hwnd).unwrap();

        SetWindowLongPtrA(hwnd, GWLP_USERDATA, &render as *const _ as isize);

        ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        Ok(())
    }
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let render_ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *const Render;
        if render_ptr.is_null() {
            return DefWindowProcA(hwnd, msg, wparam, lparam);
        }

        let render = &*render_ptr;
        let drawing = Drawing::new(&render);

        static mut RECT: Option<D2D_RECT_F> = None;

        match msg {
            WM_CREATE => {
                RECT = None; // Initialisieren als None
                LRESULT(0)
            }
            WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP => {
                let x = get_x_lparam!(lparam.0) as f32;
                let y = get_y_lparam!(lparam.0) as f32;

                if msg == WM_LBUTTONDOWN {
                    RECT = Some(D2D_RECT_F {
                        left: x,
                        top: y,
                        right: x,
                        bottom: y,
                    });
                } else if msg == WM_MOUSEMOVE && (wparam.0 & MK_LBUTTON.0 as usize) != 0 {
                    if let Some(ref mut rect) = RECT {
                        update_rect(rect, x, y);
                        RedrawWindow(hwnd, None, None, RDW_INTERNALPAINT);
                    }
                }
                LRESULT(0)
            }
            WM_SETCURSOR => {
                let cursor = LoadCursorW(None, IDC_ARROW);
                if let Ok(cursor_handle) = cursor {
                    SetCursor(cursor_handle);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                if let Some(rect) = RECT {
                    // Create a solid color brush
                    let brush_color = D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    };
                    drawing.draw_overlay(hwnd, Some(rect));
                }

                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}

fn update_rect(rect: &mut D2D_RECT_F, x: f32, y: f32) {
    if x == rect.left {
        std::mem::swap(&mut rect.left, &mut rect.right);
        rect.left = x;
    } else {
        rect.right = x;
    }

    if y == rect.top {
        std::mem::swap(&mut rect.top, &mut rect.bottom);
        rect.top = y;
    } else {
        rect.bottom = y;
    }
}
