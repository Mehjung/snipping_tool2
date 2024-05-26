mod modules;

use modules::controller::{self, Command, WindowController};
use modules::drawing::Drawing;
use modules::handler::{opaque_handler, win_proc};
use modules::renderer::Render;
use modules::resource_manager::ResourceManager;
use modules::win_fact::{WindowBuilder, WindowType};
use std::sync::{Arc, Mutex, MutexGuard};

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

fn main() -> Result<()> {
    unsafe {
        let controller = Arc::new(WindowController::new());

        let window = WindowBuilder::new()
            .set_window_type(WindowType::Main)
            .set_window_proc(win_proc)
            .build()
            .expect("Failed to create main window");

        controller
            .add_window(window.clone())
            .expect("Failed to add window");

        window.show();

        let mut msg = MSG::default();
        while GetMessageA(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        Ok(())
    }
}
