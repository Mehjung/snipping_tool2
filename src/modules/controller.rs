use crate::modules::drawing::Drawing;

use crate::modules::resource_manager::ResourceManager;
use crate::modules::win_fact::{Window, WindowType};
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex, MutexGuard};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub enum Command {
    Show,
    DrawOverlay(Option<D2D_RECT_F>),
    FillBackground(D2D1_COLOR_F),
}

pub struct WindowController {
    transparent_window: Mutex<Option<Window>>,
    opaque_window: Mutex<Option<Window>>,
    main_window: Mutex<Option<Window>>,
    resource_manager: Arc<ResourceManager>,
}

impl WindowController {
    pub fn new() -> Self {
        let resource_manager = ResourceManager::new().expect("Failed to create resource manager");
        WindowController {
            transparent_window: Mutex::new(None),
            opaque_window: Mutex::new(None),
            main_window: Mutex::new(None),
            resource_manager: resource_manager,
        }
    }

    fn window_ref(&self, window_type: WindowType) -> Option<&Mutex<Option<Window>>> {
        match window_type {
            WindowType::Transparent => Some(&self.transparent_window),
            WindowType::Opaque => Some(&self.opaque_window),
            WindowType::Main => Some(&self.main_window),
            _ => None,
        }
    }

    fn locked_window(&self, window_type: WindowType) -> Result<MutexGuard<Option<Window>>> {
        self.window_ref(window_type)
            .ok_or_else(|| anyhow!("Invalid window type"))
            .and_then(|mutex| {
                mutex
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock window mutex"))
            })
    }

    pub fn add_window(&self, mut window: Window) -> Result<()> {
        let hwnd = window.get_hwnd();
        let drawing = Drawing::new(hwnd, &self.resource_manager);

        window.set_drawing(drawing);

        let mut locked_window = self.locked_window(window.window_type)?;
        *locked_window = Some(window);

        // Set the WindowController pointer to the window
        unsafe {
            SetWindowLongPtrA(hwnd, GWLP_USERDATA, self as *const _ as isize);
        }

        Ok(())
    }

    pub fn dispatch(&self, window_type: WindowType, command: Command) -> Result<(), anyhow::Error> {
        if let Some(window) = &*self.locked_window(window_type)? {
            match command {
                Command::Show => window.show(),
                Command::DrawOverlay(rect) => window.draw_overlay(rect)?,
                Command::FillBackground(color) => window.fill_background(color)?,
            }
        }
        Ok(())
    }
}
