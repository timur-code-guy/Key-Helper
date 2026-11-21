use key_helper_core::{evaluate, Language, Verdict};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED, SetWindowsHookExW,
    UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::input;
use crate::layout;
use crate::state::{self, Boundary, Stroke, TypedChar};

const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0D;
const VK_CAPITAL: u16 = 0x14;
const VK_ESCAPE: u16 = 0x1B;
const VK_SPACE: u16 = 0x20;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_INSERT: u16 = 0x2D;
const VK_DELETE: u16 = 0x2E;
const VK_LWIN: u16 = 0x5B;
const VK_RWIN: u16 = 0x5C;
const VK_APPS: u16 = 0x5D;

const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;
const VK_LMENU: u16 = 0xA4;
const VK_RMENU: u16 = 0xA5;

static HOOK_HANDLE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn install() {
    unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)
            .expect("failed to install keyboard hook");
        HOOK_HANDLE.store(hook.0 as usize, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn uninstall() {
    let raw = HOOK_HANDLE.swap(0, std::sync::atomic::Ordering::SeqCst);
    if raw != 0 {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(raw as *mut _));
        }
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    if (kb.flags.0 & LLKHF_INJECTED.0) != 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let vk = kb.vkCode as u16;
    let suppress = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => on_keydown(vk),
        WM_KEYUP | WM_SYSKEYUP => on_keyup(vk),
        _ => false,
    };

    if suppress {
        LRESULT(1)
    } else {
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}

fn on_keydown(vk: u16) -> bool {
    track_foreground();

    match vk {
        VK_LSHIFT | VK_RSHIFT | 0x10 => {
            state::set_shift(true);
            return false;
        }
        VK_LCONTROL | VK_RCONTROL | 0x11 => {
            state::set_ctrl(true);
            return false;
        }
        VK_LMENU | VK_RMENU | 0x12 => {
            state::set_alt(true);
            return false;
        }
        VK_CAPITAL => {
            state::toggle_caps();
            return false;
        }
        _ => {}
    }

    if state::ctrl_or_alt_held() || is_win_key(vk) {
        state::clear_word();
        return false;
    }

    if is_edit_key(vk) {
        state::clear_word();
        return false;
    }

    match vk {
        VK_SPACE => return commit_word(vk, Boundary::Space),
        VK_RETURN => return commit_word(vk, Boundary::Enter),
        VK_TAB => return commit_word(vk, Boundary::Tab),
        _ => {}
    }

    match decode_char(vk) {
        Some(ch) if is_word_char(ch) => {
            push_stroke(ch, vk);
            false
        }
        Some(ch) if is_letter_in_other_layout(vk) => {
            push_stroke(ch, vk);
            false
        }
        Some(ch) => commit_word_char(vk, ch),
        None => {
            state::clear_word();
            false
        }
    }
}

fn push_stroke(ch: char, vk: u16) {
    state::push_char(
        ch,
        Stroke {
            vk,
            shift: state::shift_held(),
            caps: state::caps_on(),
        },
    );
}

fn is_letter_in_other_layout(vk: u16) -> bool {
    candidate_layouts().iter().any(|(hkl, _)| {
        layout::char_for_key(*hkl, vk, state::shift_held(), state::caps_on())
            .map(is_word_char)
            .unwrap_or(false)
    })
}

fn on_keyup(vk: u16) -> bool {
    match vk {
        VK_LSHIFT | VK_RSHIFT | 0x10 => state::set_shift(false),
        VK_LCONTROL | VK_RCONTROL | 0x11 => state::set_ctrl(false),
        VK_LMENU | VK_RMENU | 0x12 => state::set_alt(false),
        _ => {}
    }
    state::take_suppressed(vk)
}

fn track_foreground() {
    let fg = unsafe { GetForegroundWindow() };
    let hwnd = if fg == HWND::default() { 0 } else { fg.0 as isize };
    state::set_foreground(hwnd);
}

fn is_win_key(vk: u16) -> bool {
    matches!(vk, VK_LWIN | VK_RWIN | VK_APPS | 0x5E)
}

fn is_edit_key(vk: u16) -> bool {
    matches!(
        vk,
        VK_BACK
            | VK_DELETE
            | VK_INSERT
            | VK_PRIOR
            | VK_NEXT
            | VK_END
            | VK_HOME
            | VK_LEFT
            | VK_UP
            | VK_RIGHT
            | VK_DOWN
            | VK_ESCAPE
    )
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphabetic() || ('\u{0400}'..='\u{04FF}').contains(&c)
}

fn decode_char(vk: u16) -> Option<char> {
    let hkl = layout::current_layout();
    let lang = layout::primary_lang(hkl);
    if lang != layout::LANG_EN && lang != layout::LANG_RU && lang != layout::LANG_UK {
        return None;
    }
    layout::char_for_key(hkl, vk, state::shift_held(), state::caps_on())
}

fn commit_word(vk: u16, boundary: Boundary) -> bool {
    commit_inner(vk, boundary)
}

fn commit_word_char(vk: u16, ch: char) -> bool {
    commit_inner(vk, Boundary::Char(ch))
}

fn commit_inner(vk: u16, boundary: Boundary) -> bool {
    let word = state::take_word();
    if word.is_empty() {
        return false;
    }
    if word.len() == 1 && matches!(boundary, Boundary::Char(_)) {
        return false;
    }

    let current = layout::current_layout();
    let Some(source) = language_of(layout::primary_lang(current)) else {
        return false;
    };
    let typed: String = word.iter().map(|t| t.ch).collect();

    let mut chosen = None;
    for (hkl, target_lang) in candidate_layouts() {
        let Some(converted) = convert_strokes(&word, hkl) else {
            continue;
        };
        if evaluate(&typed, &converted, source, target_lang) == Verdict::Convert {
            chosen = Some((hkl, converted));
            break;
        }
    }
    let Some((target, converted)) = chosen else {
        return false;
    };

    state::suppress_key(vk);

    let mut text = converted;
    match boundary {
        Boundary::Char(c) => text.push(c),
        Boundary::Space => text.push(' '),
        Boundary::Enter | Boundary::Tab => {}
    }

    let mut inputs = input::backspaces(typed.chars().count());
    inputs.extend(input::type_text(&text));
    match boundary {
        Boundary::Enter => inputs.extend(input::tap(VIRTUAL_KEY(VK_RETURN))),
        Boundary::Tab => inputs.extend(input::tap(VIRTUAL_KEY(VK_TAB))),
        _ => {}
    }
    input::send(&inputs);
    layout::switch_layout(target);
    true
}

fn convert_strokes(word: &[TypedChar], target: layout::Hkl) -> Option<String> {
    let mut out = String::with_capacity(word.len());
    for t in word {
        out.push(layout::char_for_stroke(target, t.stroke)?);
    }
    Some(out)
}

fn language_of(lang: u16) -> Option<Language> {
    match lang {
        layout::LANG_EN => Some(Language::English),
        layout::LANG_RU => Some(Language::Russian),
        layout::LANG_UK => Some(Language::Ukrainian),
        _ => None,
    }
}

fn lang_id(lang: Language) -> u16 {
    match lang {
        Language::English => layout::LANG_EN,
        Language::Russian => layout::LANG_RU,
        Language::Ukrainian => layout::LANG_UK,
    }
}

fn candidate_layouts() -> Vec<(layout::Hkl, Language)> {
    let current = layout::current_layout();
    let Some(source) = language_of(layout::primary_lang(current)) else {
        return Vec::new();
    };
    let order: &[Language] = match source {
        Language::English => &[Language::Russian, Language::Ukrainian],
        Language::Russian => &[Language::English, Language::Ukrainian],
        Language::Ukrainian => &[Language::English, Language::Russian],
    };
    order
        .iter()
        .filter_map(|&lang| layout::find_layout(lang_id(lang)).map(|hkl| (hkl, lang)))
        .collect()
}
