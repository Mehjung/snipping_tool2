use crate::modules::controller::{Command, WindowController};
use crate::modules::win_fact::WindowType;

use windows::Win32::{
    Foundation::*,
    Graphics::{Direct2D::Common::*, Gdi::*},
    System::SystemServices::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

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

pub extern "system" fn win_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let controll_ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *const WindowController;
        if controll_ptr.is_null() {
            return DefWindowProcA(hwnd, msg, wparam, lparam);
        }

        let controller = &*controll_ptr;

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

            WM_PAINT => {
                controller.dispatch(WindowType::Transparent, Command::DrawOverlay(RECT));
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

pub extern "system" fn opaque_handler(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let controll_ptr = GetWindowLongPtrA(window, GWLP_USERDATA) as *const WindowController;
        if controll_ptr.is_null() {
            return DefWindowProcA(window, message, wparam, lparam);
        }

        let controller = &*controll_ptr;
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
                let _ = controller.dispatch(
                    WindowType::Opaque,
                    Command::FillBackground(D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
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
