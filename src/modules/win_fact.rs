use crate::modules::drawing::Drawing;
use crate::modules::renderer::Render;
use std::{any::Any, default, os::raw::c_void, time::Instant};
use windows::{
    core::{w, Error, PCWSTR},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{
            Direct2D::Common::{D2D1_COLOR_F, D2D_POINT_2F, D2D_RECT_F},
            Dwm::DwmExtendFrameIntoClientArea,
            Gdi::{CreateSolidBrush, RedrawWindow, RDW_ERASE, RDW_INVALIDATE, RDW_NOINTERNALPAINT},
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::MARGINS,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetSystemMetrics,
                LoadCursorW, RegisterClassW, SetForegroundWindow, SetLayeredWindowAttributes,
                SetWindowPos, ShowWindow, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, HMENU,
                HWND_TOPMOST, IDC_ARROW, IDC_CROSS, LWA_ALPHA, LWA_COLORKEY, SM_CXVIRTUALSCREEN,
                SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SWP_NOMOVE, SWP_NOSIZE,
                SW_HIDE, SW_SHOW, ULW_COLORKEY, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
                WS_EX_COMPOSITED, WS_EX_LAYERED, WS_EX_NOREDIRECTIONBITMAP, WS_POPUP, WS_VISIBLE,
            },
        },
    },
};
#[derive(Clone)]
pub struct Window {
    hwnd: HWND,
    pub window_type: WindowType,
    drawing: Option<Drawing>,
}

impl Window {
    pub fn new(hwnd: HWND, window_type: WindowType) -> Self {
        Window {
            hwnd,
            window_type,
            drawing: None,
        }
    }
    pub fn draw_overlay(&self, rect: Option<D2D_RECT_F>) -> Result<(), Error> {
        match &self.drawing {
            Some(drawing) => drawing.draw_overlay(self.hwnd, rect),
            None => Err(Error::from_win32()),
        }
    }

    pub fn get_hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn set_drawing(&mut self, drawing: Drawing) {
        self.drawing = Some(drawing);
    }

    pub fn fill_background(&self, color: D2D1_COLOR_F) -> Result<(), Error> {
        match &self.drawing {
            Some(drawing) => drawing.fill_background(self.hwnd, color),
            None => Err(Error::from_win32()),
        }
    }

    pub fn set_foreground(&self) {
        unsafe {
            SetForegroundWindow(self.hwnd);
        }
    }

    pub fn set_position(&self) {
        unsafe {
            SetWindowPos(self.hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn reload(&self) {
        self.hide();
        self.show();
    }

    pub fn trigger_screenshot(&self) {
        unsafe {
            println!("Trigger Screenshot");
            let res = RedrawWindow(
                self.hwnd,
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_NOINTERNALPAINT,
            );
            match res.as_bool() {
                false => eprintln!("Error in RedrawWindow"),
                _ => println!("RedrawWindow successful"),
            }
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = DestroyWindow(self.hwnd) {
                let error = anyhow::anyhow!("Error destroying window: {:?}", e);
                eprintln!("{}", error);
            }
        }
    }
}

#[derive(Debug)]
pub struct WINDOWPROPS {
    pub dwexstyle: WINDOW_EX_STYLE,
    pub lpclassname: PCWSTR,
    pub lpwindowname: PCWSTR,
    pub dwstyle: WINDOW_STYLE,
    pub x: i32,
    pub y: i32,
    pub nwidth: i32,
    pub nheight: i32,
    pub hwndparent: HWND,
    pub hmenu: HMENU,
    pub hinstance: HINSTANCE,
    pub lpparam: Option<*const c_void>,
}

impl Default for WINDOWPROPS {
    fn default() -> Self {
        WINDOWPROPS {
            dwexstyle: WINDOW_EX_STYLE(0),
            lpclassname: w!("win_template_class"),
            lpwindowname: w!("win_template_window"),
            dwstyle: WINDOW_STYLE(0),
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            nwidth: CW_USEDEFAULT,
            nheight: CW_USEDEFAULT,
            hwndparent: HWND::default(),
            hmenu: HMENU::default(),
            hinstance: HINSTANCE::default(),
            lpparam: None,
        }
    }
}

pub struct WindowTemplate {
    pub windowprops: WINDOWPROPS,
    pub classprops: WNDCLASSW,
}

impl WindowTemplate {
    fn new() -> Self {
        WindowTemplate {
            windowprops: WINDOWPROPS::default(),
            classprops: WNDCLASSW::default(),
        }
    }

    fn create_window(&self, builder: &WindowBuilder) -> Result<Window, anyhow::Error> {
        let hwnd: HWND;
        unsafe {
            let instance = GetModuleHandleW(None)?;

            let wc = WNDCLASSW {
                hInstance: instance.into(),
                lpszClassName: self.windowprops.lpclassname,
                lpfnWndProc: Some(builder.window_proc),
                ..self.classprops
            };

            if RegisterClassW(&wc) == 0 {
                return Err(anyhow::anyhow!("Failed to register window class"));
            }

            hwnd = CreateWindowExW(
                self.windowprops.dwexstyle,
                self.windowprops.lpclassname,
                self.windowprops.lpwindowname,
                self.windowprops.dwstyle,
                self.windowprops.x,
                self.windowprops.y,
                self.windowprops.nwidth,
                self.windowprops.nheight,
                self.windowprops.hwndparent,
                self.windowprops.hmenu,
                self.windowprops.hinstance,
                self.windowprops.lpparam,
            );
        };
        Ok(Window {
            hwnd,
            window_type: builder.window_type,
            drawing: None,
        })
    }
}

pub trait WindowFactory {
    fn create_window(&self, builder: &WindowBuilder) -> Result<Window, anyhow::Error>;
}

pub struct TransparentWindowFactory;
pub struct OpaqueWindowFactory;
pub struct MainWindowFactory;

impl WindowFactory for TransparentWindowFactory {
    fn create_window(&self, builder: &WindowBuilder) -> Result<Window, anyhow::Error> {
        let window;
        unsafe {
            let mut template = WindowTemplate::new();

            template.windowprops = WINDOWPROPS {
                lpclassname: w!("TransparentWindowClass"),
                lpwindowname: w!("TransparentWindow"),
                dwexstyle: WS_EX_COMPOSITED | WS_EX_LAYERED,
                dwstyle: WS_POPUP,
                x: GetSystemMetrics(SM_XVIRTUALSCREEN),
                y: GetSystemMetrics(SM_YVIRTUALSCREEN),
                nwidth: GetSystemMetrics(SM_CXVIRTUALSCREEN),
                nheight: GetSystemMetrics(SM_CYVIRTUALSCREEN),
                ..Default::default()
            };

            template.classprops = WNDCLASSW {
                style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
                hbrBackground: CreateSolidBrush(COLORREF(0x00000000)),
                hCursor: LoadCursorW(None, IDC_CROSS)?,
                lpfnWndProc: Some(builder.window_proc),
                lpszClassName: template.windowprops.lpclassname,
                ..Default::default()
            };

            window = template.create_window(builder)?;
        }
        Ok(window)
    }
}

impl WindowFactory for OpaqueWindowFactory {
    fn create_window(&self, builder: &WindowBuilder) -> Result<Window, anyhow::Error> {
        let res;
        unsafe {
            let mut template = WindowTemplate::new();

            template.windowprops = WINDOWPROPS {
                lpclassname: w!("OpaqueWindowClass"),
                lpwindowname: w!("OpaqueWindow"),
                dwexstyle: WS_EX_COMPOSITED,
                dwstyle: WS_POPUP,
                x: GetSystemMetrics(SM_XVIRTUALSCREEN),
                y: GetSystemMetrics(SM_YVIRTUALSCREEN),
                nwidth: GetSystemMetrics(SM_CXVIRTUALSCREEN),
                nheight: GetSystemMetrics(SM_CYVIRTUALSCREEN),
                ..Default::default()
            };

            template.classprops = WNDCLASSW {
                style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                lpszClassName: template.windowprops.lpclassname,
                lpfnWndProc: Some(builder.window_proc),
                ..Default::default()
            };

            res = template.create_window(builder)?;
        }
        Ok(res)
    }
}

impl WindowFactory for MainWindowFactory {
    fn create_window(&self, builder: &WindowBuilder) -> Result<Window, anyhow::Error> {
        let window;
        unsafe {
            let instance = GetModuleHandleW(None)?;

            let window_class = w!("MainWindowClass");

            let wc = WNDCLASSW {
                lpfnWndProc: Some(builder.window_proc),
                lpszClassName: window_class,
                style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                ..Default::default()
            };

            if RegisterClassW(&wc) == 0 {
                return Err(anyhow::anyhow!("Failed to register window class"));
            }

            let hwnd = CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP,
                window_class,
                w!("Main Window"),
                WS_POPUP,
                100,
                100,
                800,
                600,
                HWND::default(),
                HMENU::default(),
                instance,
                None,
            );

            if hwnd.0 == 0 {
                return Err(anyhow::anyhow!("Failed to create window"));
            }

            window = Window {
                hwnd,
                window_type: builder.window_type,
                drawing: None,
            };
        }
        Ok(window)
    }
}

pub struct WindowBuilder {
    window_proc: unsafe extern "system" fn(
        param0: HWND,
        param1: u32,
        param2: WPARAM,
        param3: LPARAM,
    ) -> LRESULT,

    window_type: WindowType,
}

#[derive(Clone, Copy)]
pub enum WindowType {
    Transparent,
    Opaque,
    Main,
    None,
}

unsafe extern "system" fn default_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Hier werden keine spezifischen Nachrichten behandelt. Stattdessen wird alles an DefWindowProcW weitergeleitet.
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            window_proc: default_window_proc,
            window_type: WindowType::None,
        }
    }

    pub fn set_window_type(&mut self, window_type: WindowType) -> &mut Self {
        self.window_type = window_type;
        self
    }

    pub fn set_window_proc(
        &mut self,
        window_proc: unsafe extern "system" fn(
            param0: HWND,
            param1: u32,
            param2: WPARAM,
            param3: LPARAM,
        ) -> LRESULT,
    ) -> &mut Self {
        self.window_proc = window_proc;
        self
    }

    pub fn build(&self) -> Result<Window, anyhow::Error> {
        let factory: Box<dyn WindowFactory> = match self.window_type {
            WindowType::Transparent => Box::new(TransparentWindowFactory),
            WindowType::Opaque => Box::new(OpaqueWindowFactory),
            WindowType::Main => Box::new(MainWindowFactory),
            _ => return anyhow::Result::Err(anyhow::anyhow!("Invalid window type")),
        };

        let window = factory.create_window(self)?;

        Ok(window)
    }
}
