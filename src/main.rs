mod modules;

use modules::drawing::Drawing;
use modules::renderer::Render;
use modules::win_fact::{WindowBuilder, WindowType};

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Direct2D::Common::*, Gdi::*},
        System::LibraryLoader::*,
        System::SystemServices::*,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
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
        let window = WindowBuilder::new()
            .set_window_type(WindowType::Opaque)
            .set_window_proc(opaque_handler)
            .build()
            .expect("Failed to create main window");

        let render = Render::new(window.get_hwnd()).unwrap();
        SetWindowLongPtrA(
            window.get_hwnd(),
            GWLP_USERDATA,
            &render as *const _ as isize,
        );

        // Hier kannst du die weiteren Operationen auf dem Fenster ausführen
        window.show();

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        Ok(())
    }
}

extern "system" fn win_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
            WM_KEYDOWN => {
                if wparam.0 == VK_ESCAPE.0 as usize {
                    PostQuitMessage(0);
                }
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
                    let _brush_color = D2D1_COLOR_F {
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

pub extern "system" fn opaque_handler(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let render_ptr = GetWindowLongPtrA(window, GWLP_USERDATA) as *const Render;
        if render_ptr.is_null() {
            return DefWindowProcA(window, message, wparam, lparam);
        }

        let render = &*render_ptr;
        let drawing = Drawing::new(&render);
        match message {
            WM_KEYDOWN => {
                if wparam.0 == VK_ESCAPE.0 as usize {
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_PAINT => {
                println!("WM_PAINT");
                let _ = drawing.fill_background(
                    window,
                    D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                );

                LRESULT(0)
            }
            WM_ERASEBKGND => {
                LRESULT(1) // Return 1 to indicate that the background has been erased
            }
            WM_ERASEBKGND => {
                //if FIRST_PAINT.swap(false, Ordering::SeqCst) {

                //}
                LRESULT(0)
            }

            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
