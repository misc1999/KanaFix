//! 前面ウィンドウのプロセスが補正対象かどうかの判定。
//!
//! プロセス解決はフォーカス変更時のみ行い、同じウィンドウが前面の間は
//! 前回結果を再利用する。

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::OnceLock;

use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// プロセスパス取得バッファの長さ（UTF-16 単位）。
const PATH_BUF_LEN: usize = 260;

fn target_apps() -> &'static OnceLock<Vec<String>> {
    static APPS: OnceLock<Vec<String>> = OnceLock::new();
    &APPS
}

/// 補正対象アプリ名（小文字）を一度だけ登録する。
pub(crate) fn set_target_apps(apps: Vec<String>) {
    let _ = target_apps().set(apps);
}

/// フルパスから実行ファイル名のみを取り出し、小文字化する。
fn basename_lower(path: &str) -> String {
    path.rsplit(['\\', '/'])
        .next()
        .unwrap_or(path)
        .to_lowercase()
}

/// 前面プロセス名が補正対象一覧に含まれるか。`apps` は小文字で渡す前提。
fn is_target_app(apps: &[String], name: &str) -> bool {
    apps.iter().any(|app| app == name)
}

/// PID から実行ファイル名（小文字）を取得する。
fn process_name(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    let mut buf = [0u16; PATH_BUF_LEN];
    let mut size = buf.len() as u32;
    // SAFETY: 有効な PID に対する OpenProcess は有効なハンドルかエラーを返す。
    // QueryFullProcessImageNameW は最大 `size` 個の UTF-16 を `buf` に書き込み、
    // `size` を実際の書き込み数に更新する。ハンドルは返る前に必ず閉じる。
    let full = unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        result.ok()?;
        String::from_utf16_lossy(&buf[..size as usize])
    };
    Some(basename_lower(&full))
}

/// 前面プロセスが補正対象一覧に含まれるか。
pub(crate) fn foreground_is_target() -> bool {
    static LAST_HWND: AtomicIsize = AtomicIsize::new(0);
    static LAST_RESULT: AtomicBool = AtomicBool::new(false);

    // SAFETY: GetForegroundWindow は引数にポインタを取らず常に呼び出す。
    let hwnd = unsafe { GetForegroundWindow() };
    let key = hwnd.0 as isize;
    if key == 0 {
        return false;
    }
    if LAST_HWND.load(Ordering::Relaxed) == key {
        return LAST_RESULT.load(Ordering::Relaxed);
    }

    let mut pid = 0u32;
    // SAFETY: `hwnd` は現在の前面ウィンドウ、`pid` は有効なローカル out 引数。
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let name = process_name(pid).unwrap_or_default();
    let result = target_apps()
        .get()
        .map(|apps| is_target_app(apps, &name))
        .unwrap_or(false);

    LAST_HWND.store(key, Ordering::Relaxed);
    LAST_RESULT.store(result, Ordering::Relaxed);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basename_lower_strips_windows_path() {
        assert_eq!(
            basename_lower(r"C:\Windows\System32\notepad.exe"),
            "notepad.exe"
        );
    }

    #[test]
    fn basename_lower_strips_forward_slashes_and_lowercases() {
        assert_eq!(basename_lower("/usr/local/bin/Foo.EXE"), "foo.exe");
    }

    #[test]
    fn basename_lower_handles_bare_name() {
        assert_eq!(basename_lower("App.exe"), "app.exe");
    }

    #[test]
    fn is_target_app_matches_known_name() {
        let apps = vec!["notepad.exe".to_string(), "code.exe".to_string()];
        assert!(is_target_app(&apps, "notepad.exe"));
        assert!(is_target_app(&apps, "code.exe"));
    }

    #[test]
    fn is_target_app_rejects_unknown_name() {
        let apps = vec!["notepad.exe".to_string()];
        assert!(!is_target_app(&apps, "chrome.exe"));
        assert!(!is_target_app(&[], "notepad.exe"));
    }
}
