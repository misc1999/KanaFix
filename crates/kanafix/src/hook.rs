//! WH_KEYBOARD_LL フックの設置と、対象イベントの横取り。

use anyhow::Context as _;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYUP, WM_SYSKEYUP,
};

use crate::foreground::foreground_is_target;
use crate::inject::{send_event, InjectEvent};
use crate::keys::{MAGIC, VK_BACK, VK_PACKET};

unsafe extern "system" fn ll_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: HC_ACTION のとき lParam は OS が渡す有効な KBDLLHOOKSTRUCT を指す。
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if kb.dwExtraInfo != MAGIC {
            let vk = kb.vkCode;
            if (vk == VK_PACKET || vk == VK_BACK) && foreground_is_target() {
                let up = matches!(wparam.0 as u32, WM_KEYUP | WM_SYSKEYUP);
                send_event(InjectEvent {
                    vk,
                    scan: kb.scanCode,
                    up,
                });
                // 元イベントは止め、ワーカーが間隔を空けて再注入する。
                return LRESULT(1);
            }
        }
    }
    // SAFETY: 元の引数をそのまま次フックへ転送するのは常に有効。
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// フックを設置し、メッセージループを回す。終了時にフックを解除する。
pub(crate) fn run() -> anyhow::Result<()> {
    // SAFETY: 以降はいずれも Win32 FFI を契約どおりに使う。モジュールハンドルは
    // 現在のプロセス、ll_proc はプロセス存続中に有効な関数ポインタ、メッセージ
    // ループの変数は有効なローカル。フックは戻る前に解除する。
    unsafe {
        let hinstance = GetModuleHandleW(None).context("GetModuleHandleW に失敗")?;
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(ll_proc), Some(hinstance.into()), 0)
            .context("SetWindowsHookExW に失敗")?;

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        let _ = UnhookWindowsHookEx(hook);
    }
    Ok(())
}
