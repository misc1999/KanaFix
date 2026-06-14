//! 横取りしたキーイベントを一定間隔で再注入するワーカーと送信チャネル。

use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

use crate::keys::{MAGIC, VK_PACKET};

#[derive(Clone, Copy)]
pub(crate) struct InjectEvent {
    pub(crate) vk: u32,
    pub(crate) scan: u32,
    pub(crate) up: bool,
}

fn sender() -> &'static Mutex<Option<Sender<InjectEvent>>> {
    static TX: OnceLock<Mutex<Option<Sender<InjectEvent>>>> = OnceLock::new();
    TX.get_or_init(|| Mutex::new(None))
}

/// 再注入ワーカーを起動し、送信チャネルを登録する。
/// 以降 [`send_event`] で積んだイベントが `pace` 間隔で再注入される。
pub(crate) fn start_worker(pace: Duration) {
    let (tx, rx) = mpsc::channel::<InjectEvent>();
    *sender().lock().expect("sender mutex poisoned") = Some(tx);

    thread::spawn(move || {
        for event in rx {
            inject(&event);
            thread::sleep(pace);
        }
    });
}

/// イベントをワーカーへ積む。チャネル未登録なら黙って捨てる。
pub(crate) fn send_event(event: InjectEvent) {
    if let Ok(guard) = sender().lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(event);
        }
    }
}

/// 1 件のキーボードイベントを表す INPUT を組み立てる。
fn build_input(event: &InjectEvent) -> INPUT {
    let ki = if event.vk == VK_PACKET {
        KEYBDINPUT {
            wVk: VIRTUAL_KEY(0),
            wScan: event.scan as u16,
            dwFlags: if event.up {
                KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
            } else {
                KEYEVENTF_UNICODE
            },
            time: 0,
            dwExtraInfo: MAGIC,
        }
    } else {
        KEYBDINPUT {
            wVk: VIRTUAL_KEY(event.vk as u16),
            wScan: 0,
            dwFlags: if event.up {
                KEYEVENTF_KEYUP
            } else {
                KEYBD_EVENT_FLAGS(0)
            },
            time: 0,
            dwExtraInfo: MAGIC,
        }
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki },
    }
}

/// イベントを 1 件再注入する。MAGIC を付与してフック側で除外できるようにする。
fn inject(event: &InjectEvent) {
    let input = build_input(event);
    // SAFETY: `input` は 1 件分の有効な INPUT。スライス長と要素サイズは整合し、
    // SendInput はこの値を読むだけで書き換えない。
    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
}
