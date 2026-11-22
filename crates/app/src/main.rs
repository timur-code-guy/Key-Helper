#![windows_subsystem = "windows"]

mod hook;
mod input;
mod layout;
mod state;
mod tray;
mod update;

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

const SINGLE_INSTANCE_MUTEX: &str = "Local\\Key-Helper-single-instance";

fn main() {
    if !acquire_single_instance() {
        return;
    }

    hook::install();
    tray::init();
    update::spawn();

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    tray::shutdown();
    hook::uninstall();
}

fn acquire_single_instance() -> bool {
    unsafe {
        let name: Vec<u16> = SINGLE_INSTANCE_MUTEX.encode_utf16().chain(std::iter::once(0)).collect();
        let mutex = CreateMutexW(None, true, windows::core::PCWSTR(name.as_ptr()));
        match mutex {
            Ok(handle) => {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(handle);
                    false
                } else {
                    true
                }
            }
            Err(_) => true,
        }
    }
}
