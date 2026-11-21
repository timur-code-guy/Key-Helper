use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    NIM_ADD, NIM_DELETE, NIF_ICON, NIF_MESSAGE, NIF_TIP, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    GetCursorPos, HWND_MESSAGE, IDI_APPLICATION, LoadIconW, MF_STRING, PostQuitMessage,
    RegisterClassW, SetForegroundWindow, TPM_RETURNCMD, TrackPopupMenu, WINDOW_EX_STYLE,
    WINDOW_STYLE, WNDCLASSW, WM_COMMAND, WM_DESTROY, WM_RBUTTONUP, WM_USER,
};
use windows::core::w;

const WM_TRAYICON: u32 = WM_USER + 1;
const TRAY_ID: u32 = 1;
const IDM_QUIT: usize = 1;

static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn init() {
    unsafe {
        let Ok(instance) = GetModuleHandleW(None) else {
            return;
        };

        let class_name = w!("KeyHelperTrayWindow");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(tray_proc),
            hInstance: HINSTANCE(instance.0),
            lpszClassName: class_name,
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return;
        }

        let Ok(hwnd) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Key-Helper"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(HINSTANCE(instance.0)),
            None,
        ) else {
            return;
        };

        let Ok(icon) = LoadIconW(None, IDI_APPLICATION) else {
            let _ = DestroyWindow(hwnd);
            return;
        };

        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ID;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = icon;
        let tip: Vec<u16> = "Key-Helper".encode_utf16().chain(std::iter::once(0)).collect();
        let len = tip.len().min(nid.szTip.len());
        nid.szTip[..len].copy_from_slice(&tip[..len]);

        if !Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
            let _ = DestroyWindow(hwnd);
            return;
        }

        TRAY_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
    }
}

pub fn shutdown() {
    let raw = TRAY_HWND.swap(0, Ordering::SeqCst);
    if raw == 0 {
        return;
    }
    unsafe {
        let hwnd = HWND(raw as *mut _);
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ID;
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        let _ = DestroyWindow(hwnd);
    }
}

unsafe extern "system" fn tray_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            if lparam.0 as u32 == WM_RBUTTONUP {
                show_menu(hwnd);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            if wparam.0 as usize & 0xFFFF == IDM_QUIT {
                PostQuitMessage(0);
                LRESULT(0)
            } else {
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn show_menu(hwnd: HWND) {
    unsafe {
        let Ok(menu) = CreatePopupMenu() else {
            return;
        };

        let _ = AppendMenuW(menu, MF_STRING, IDM_QUIT, w!("Quit"));

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(menu, TPM_RETURNCMD, pt.x, pt.y, Some(0), hwnd, None);
        if cmd.as_bool() {
            let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(hwnd),
                WM_COMMAND,
                WPARAM(cmd.0 as usize),
                LPARAM(0),
            );
        }
        let _ = DestroyMenu(menu);
    }
}
