use crate::state::Stroke;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardLayoutList, HKL, MapVirtualKeyExW, MAPVK_VK_TO_VSC, ToUnicodeEx,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, PostMessageW, WM_INPUTLANGCHANGEREQUEST};

pub type Hkl = HKL;

pub const LANG_EN: u16 = 0x09;
pub const LANG_RU: u16 = 0x19;
pub const LANG_UK: u16 = 0x22;

const VK_SHIFT: u16 = 0x10;
const VK_CAPITAL: u16 = 0x14;
const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;

pub fn primary_lang(hkl: HKL) -> u16 {
    ((hkl.0 as usize) & 0xFFFF) as u16 & 0x03FF
}

pub fn current_layout() -> HKL {
    unsafe {
        let fg = GetForegroundWindow();
        let tid = GetWindowThreadProcessId(fg, None);
        GetKeyboardLayout(tid)
    }
}

pub fn installed_layouts() -> Vec<HKL> {
    unsafe {
        let count = GetKeyboardLayoutList(None);
        if count == 0 {
            return Vec::new();
        }
        let mut hkls = vec![HKL::default(); count as usize];
        let actual = GetKeyboardLayoutList(Some(&mut hkls));
        hkls.truncate(actual as usize);
        hkls
    }
}

pub fn find_layout(lang: u16) -> Option<HKL> {
    installed_layouts()
        .into_iter()
        .find(|hkl| primary_lang(*hkl) == lang)
}

pub fn switch_layout(hkl: HKL) {
    unsafe {
        let fg = GetForegroundWindow();
        if fg == windows::Win32::Foundation::HWND::default() {
            return;
        }
        let _ = PostMessageW(
            Some(fg),
            WM_INPUTLANGCHANGEREQUEST,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(hkl.0 as isize),
        );
    }
}

pub fn char_for_key(hkl: HKL, vk: u16, shift: bool, caps: bool) -> Option<char> {
    char_for_stroke(hkl, Stroke { vk, shift, caps })
}

pub fn char_for_stroke(hkl: HKL, stroke: Stroke) -> Option<char> {
    unsafe {
        let mut state = [0u8; 256];
        if stroke.shift {
            state[VK_SHIFT as usize] = 0x80;
            state[VK_LSHIFT as usize] = 0x80;
            state[VK_RSHIFT as usize] = 0x80;
        }
        if stroke.caps {
            state[VK_CAPITAL as usize] = 0x01;
        }

        let scan = MapVirtualKeyExW(stroke.vk as u32, MAPVK_VK_TO_VSC, Some(hkl));
        let mut buf = [0u16; 8];
        let ret = ToUnicodeEx(
            stroke.vk as u32,
            scan,
            &state,
            &mut buf,
            0,
            Some(hkl),
        );

        if ret == 1 {
            char::from_u32(buf[0] as u32)
        } else {
            None
        }
    }
}
