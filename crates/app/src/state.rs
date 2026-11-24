use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct Stroke {
    pub vk: u16,
    pub shift: bool,
    pub caps: bool,
}

pub struct TypedChar {
    pub ch: char,
    pub stroke: Stroke,
}

#[derive(Clone, Copy)]
pub enum Boundary {
    Char(char),
    Space,
    Enter,
    Tab,
}

struct State {
    word: Vec<TypedChar>,
    shift: bool,
    ctrl: bool,
    alt: bool,
    caps: bool,
    suppressed: HashSet<u16>,
    foreground: isize,
}

impl State {
    fn new() -> Self {
        Self {
            word: Vec::with_capacity(64),
            shift: false,
            ctrl: false,
            alt: false,
            caps: false,
            suppressed: HashSet::new(),
            foreground: 0,
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub fn set_foreground(hwnd: isize) {
    with(|s| {
        if s.foreground != hwnd {
            s.foreground = hwnd;
            s.word.clear();
        }
    });
}

pub fn push_char(ch: char, stroke: Stroke) {
    with(|s| {
        if s.word.len() < 128 {
            s.word.push(TypedChar { ch, stroke });
        }
    });
}

pub fn take_word() -> Vec<TypedChar> {
    with(|s| std::mem::take(&mut s.word))
}

pub fn clear_word() {
    with(|s| s.word.clear());
}

pub fn set_shift(down: bool) {
    with(|s| s.shift = down);
}

pub fn set_ctrl(down: bool) {
    with(|s| s.ctrl = down);
}

pub fn set_alt(down: bool) {
    with(|s| s.alt = down);
}

pub fn toggle_caps() {
    with(|s| s.caps = !s.caps);
}

pub fn shift_held() -> bool {
    with(|s| s.shift)
}

pub fn caps_on() -> bool {
    with(|s| s.caps)
}

pub fn ctrl_or_alt_held() -> bool {
    with(|s| s.ctrl || s.alt)
}

pub fn suppress_key(vk: u16) {
    with(|s| {
        s.suppressed.insert(vk);
    });
}

pub fn take_suppressed(vk: u16) -> bool {
    with(|s| s.suppressed.remove(&vk))
}
