mod modules;

use modules::controller::WindowController;

use modules::handler::win_proc;

use modules::win_fact::{WindowBuilder, WindowType};
use std::sync::Arc;

use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

fn main() -> Result<()> {
    unsafe {
        let controller = Arc::new(WindowController::new());

        let window = WindowBuilder::new()
            .set_window_type(WindowType::Transparent)
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
