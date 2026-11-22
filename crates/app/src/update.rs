use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::Duration;

use serde_json::Value;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_PROGRESS_CLASS, INITCOMMONCONTROLSEX,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetSystemMetrics,
    MessageBoxW, PeekMessageW, RegisterClassW, SendMessageW, SetWindowTextW, TranslateMessage,
    MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_TOPMOST, MB_YESNO, MSG, PM_REMOVE,
    SM_CXSCREEN,
    SM_CYSCREEN, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW, WM_CLOSE, WM_DESTROY, WS_CAPTION,
    WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
};

const API_URL: &str =
    "https://api.github.com/repos/timur-code-guy/Key-Helper/releases/latest";
const EXE_NAME: &str = "key-helper.exe";
const UPDATE_DIR: &str = "key-helper-update";
const CREATE_NO_WINDOW: u32 = 0x08000000;

const PBM_SETRANGE: u32 = 0x0401;
const PBM_SETPOS: u32 = 0x0402;
const PBS_SMOOTH: u32 = 0x0001;

pub fn spawn() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(3));
        let _ = run();
    });
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(300)))
        .build()
        .new_agent()
}

fn run() -> Result<(), String> {
    check_just_updated();
    let agent = agent();
    let body = get_text(&agent, API_URL)?;
    let json: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;

    let tag = json
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let latest = parse_version(tag).ok_or_else(|| format!("bad tag: {tag}"))?;
    let current = parse_version(env!("CARGO_PKG_VERSION")).unwrap_or((0, 0, 0));
    if latest <= current {
        return Ok(());
    }

    let (download_url, size) = find_download_url(&json).ok_or("no release asset")?;

    let text = format!(
        "A new version of Key-Helper is available.\n\nCurrent: v{}\nLatest: {}\n\nInstall the update?",
        env!("CARGO_PKG_VERSION"),
        tag
    );
    if !ask_yes_no(&text) {
        return Ok(());
    }

    let mut progress = ProgressDialog::new(size.unwrap_or(0));
    let staged = download(&agent, &download_url, progress.as_mut())?;
    drop(progress);
    apply_update(&staged)
}

fn check_just_updated() {
    let flag = std::env::temp_dir()
        .join(UPDATE_DIR)
        .join("updated.flag");
    if flag.exists() {
        let _ = fs::remove_file(&flag);
        let _ = fs::remove_dir(std::env::temp_dir().join(UPDATE_DIR));
        let text = wide("Update installed successfully!");
        let caption = wide("Key-Helper Update");
        unsafe {
            let _ = MessageBoxW(
                None,
                PCWSTR(text.as_ptr()),
                PCWSTR(caption.as_ptr()),
                MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST,
            );
        }
    }
}

fn get_text(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let mut resp = agent
        .get(url)
        .header("User-Agent", "key-helper")
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?;
    resp.body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())
}

fn find_download_url(json: &Value) -> Option<(String, Option<u64>)> {
    let assets = json.get("assets")?.as_array()?;
    let asset = assets
        .iter()
        .find(|a| a.get("name").and_then(Value::as_str) == Some(EXE_NAME))
        .or_else(|| assets.first())?;
    let url = asset
        .get("browser_download_url")?
        .as_str()?
        .to_string();
    let size = asset.get("size").and_then(Value::as_u64);
    Some((url, size))
}

fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let v = tag.trim_start_matches(['v', 'V']);
    let mut parts = v.split('.');
    let major = parse_part(parts.next()?)?;
    let minor = parse_part(parts.next().unwrap_or("0"))?;
    let patch = parse_part(parts.next().unwrap_or("0"))?;
    Some((major, minor, patch))
}

fn parse_part(s: &str) -> Option<u64> {
    let digits: String = s
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn ask_yes_no(text: &str) -> bool {
    let text = wide(text);
    let caption = wide("Key-Helper Update");
    unsafe {
        let result = MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            PCWSTR(caption.as_ptr()),
            MB_YESNO | MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST,
        );
        result.0 == 6
    }
}

struct ProgressDialog {
    hwnd: HWND,
    bar: HWND,
    percent: HWND,
    total: u64,
    last_pos: i32,
}

impl ProgressDialog {
    fn new(total: u64) -> Option<Self> {
        unsafe {
            let instance = GetModuleHandleW(None).ok()?;
            static INIT: Once = Once::new();
            INIT.call_once(|| {
                let wc = WNDCLASSW {
                    lpfnWndProc: Some(dlg_proc),
                    hInstance: HINSTANCE(instance.0),
                    lpszClassName: w!("KeyHelperUpdateWindow"),
                    ..Default::default()
                };
                let _ = RegisterClassW(&wc);
            });

            let icc = INITCOMMONCONTROLSEX {
                dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_PROGRESS_CLASS,
            };
            let _ = InitCommonControlsEx(&icc);

            let (wpx, hpx) = (400i32, 140i32);
            let x = (GetSystemMetrics(SM_CXSCREEN) - wpx) / 2;
            let y = (GetSystemMetrics(SM_CYSCREEN) - hpx) / 2;
            let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE;
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("KeyHelperUpdateWindow"),
                w!("Key-Helper Update"),
                style,
                x,
                y,
                wpx,
                hpx,
                None,
                None,
                Some(HINSTANCE(instance.0)),
                None,
            )
            .ok()?;

            let child = |class: PCWSTR, text: PCWSTR, st: WINDOW_STYLE, x, y, w, h| {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    class,
                    text,
                    WS_CHILD | WS_VISIBLE | st,
                    x,
                    y,
                    w,
                    h,
                    Some(hwnd),
                    None,
                    Some(HINSTANCE(instance.0)),
                    None,
                )
                .ok()
            };

            child(w!("Static"), w!("Downloading update..."), WINDOW_STYLE(0), 16, 14, 300, 20)?;
            let percent = child(w!("Static"), w!(""), WINDOW_STYLE(0), 320, 14, 64, 20)?;
            let bar = child(
                w!("msctls_progress32"),
                w!(""),
                WINDOW_STYLE(PBS_SMOOTH),
                16,
                48,
                368,
                24,
            )?;

            let _ = SendMessageW(
                bar,
                PBM_SETRANGE,
                Some(WPARAM(1)),
                Some(LPARAM(100isize << 16)),
            );

            let dlg = Self {
                hwnd,
                bar,
                percent,
                total,
                last_pos: -1,
            };
            dlg.pump();
            Some(dlg)
        }
    }

    fn set(&mut self, done: u64) {
        if self.total > 0 {
            let pos = ((done.saturating_mul(100)) / self.total).min(100) as i32;
            if pos != self.last_pos {
                self.last_pos = pos;
                unsafe {
                    let _ = SendMessageW(
                        self.bar,
                        PBM_SETPOS,
                        Some(WPARAM(pos as usize)),
                        Some(LPARAM(0)),
                    );
                    let text = wide(&format!("{pos}%"));
                    let _ = SetWindowTextW(self.percent, PCWSTR(text.as_ptr()));
                }
            }
        }
        self.pump();
    }

    fn pump(&self) {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for ProgressDialog {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

unsafe extern "system" fn dlg_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE | WM_DESTROY => {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn download(
    agent: &ureq::Agent,
    url: &str,
    mut progress: Option<&mut ProgressDialog>,
) -> Result<PathBuf, String> {
    let dir = update_dir()?;
    let target = dir.join(EXE_NAME);
    let _ = fs::remove_file(&target);

    let mut resp = agent.get(url).call().map_err(|e| e.to_string())?;
    let result = write_body(resp.body_mut(), &target, progress.as_deref_mut());
    if result.is_err() {
        let _ = fs::remove_file(&target);
    }
    result.map_err(|e| e.to_string())?;
    Ok(target)
}

fn write_body(
    body: &mut ureq::Body,
    path: &Path,
    mut progress: Option<&mut ProgressDialog>,
) -> io::Result<()> {
    let mut reader = body.as_reader();
    let mut file = fs::File::create(path)?;
    let mut buf = [0u8; 256 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        done += n as u64;
        if let Some(p) = progress.as_mut() {
            p.set(done);
        }
    }
    file.flush()
}

fn apply_update(staged: &Path) -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_name = current
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| EXE_NAME.to_string());
    let pid = std::process::id();
    let bat = update_dir()?.join("update.bat");
    let script = format!(
        "@echo off\r\n\
         set /a n=0\r\n\
         :loop\r\n\
         tasklist /fi \"PID eq {pid}\" /fi \"IMAGENAME eq {exe_name}\" 2>nul | find /i \"{exe_name}\" >nul\r\n\
         if errorlevel 1 goto gone\r\n\
         set /a n+=1\r\n\
         if %n% gtr 60 goto gone\r\n\
         ping 127.0.0.1 -n 2 >nul\r\n\
         goto loop\r\n\
         :gone\r\n\
         del /f \"{}\"\r\n\
         move /y \"{}\" \"{}\"\r\n\
         echo.>\"%TEMP%\\key-helper-update\\updated.flag\"\r\n\
         start \"\" \"{}\"\r\n\
         del \"%~f0\"\r\n",
        current.display(),
        staged.display(),
        current.display(),
        current.display()
    );
    fs::write(&bat, script).map_err(|e| e.to_string())?;

    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .arg("/c")
        .arg(&bat)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| e.to_string())?;

    crate::tray::quit();
    Ok(())
}

fn update_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(UPDATE_DIR);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
