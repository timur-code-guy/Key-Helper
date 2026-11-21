use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY,
};

fn vk_input(vk: VIRTUAL_KEY, key_up: bool, extended: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    if extended {
        flags |= windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn unicode_input(unit: u16, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: if key_up {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_UNICODE
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

pub fn backspaces(count: usize) -> Vec<INPUT> {
    let vk = VIRTUAL_KEY(0x08);
    (0..count)
        .flat_map(|_| [vk_input(vk, false, false), vk_input(vk, true, false)])
        .collect()
}

pub fn type_text(text: &str) -> Vec<INPUT> {
    text.encode_utf16()
        .flat_map(|unit| [unicode_input(unit, false), unicode_input(unit, true)])
        .collect()
}

pub fn tap(vk: VIRTUAL_KEY) -> Vec<INPUT> {
    vec![vk_input(vk, false, false), vk_input(vk, true, false)]
}
