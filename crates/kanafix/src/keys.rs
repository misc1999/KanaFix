//! RDP が送る合成入力と、自分で再注入したイベントを見分けるための定数。

/// SendInput + KEYEVENTF_UNICODE で注入された文字が持つ仮想キー。
pub(crate) const VK_PACKET: u32 = 0xE7;
/// Backspace。
pub(crate) const VK_BACK: u32 = 0x08;
/// 自分で再注入したイベントを見分けるための dwExtraInfo マーカー。
pub(crate) const MAGIC: usize = 0x4B414E41;
