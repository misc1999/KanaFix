//! 対象アプリが前面のとき、RDP が注入する VK_PACKET / Backspace のバーストを
//! 一定間隔で再注入し直し、メモ帳などで起きる日本語入力の二重化を防ぐ常駐フック。

#![windows_subsystem = "console"]
#![deny(unsafe_op_in_unsafe_fn)]

mod config;
mod foreground;
mod hook;
mod inject;
mod keys;

use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let config = config::load_config();
    let pace_ms = config.pace_ms;
    let apps: Vec<String> = config.apps.iter().map(|a| a.to_lowercase()).collect();
    let apps_display = apps.join(", ");
    foreground::set_target_apps(apps);

    inject::start_worker(Duration::from_millis(pace_ms));

    eprintln!(
        "kanafix 起動 (pid={}, pace={pace_ms}ms)。対象: [{apps_display}]。\n\
         設定は kanafix.toml で変更。Ctrl+C で終了。",
        std::process::id()
    );

    hook::run()
}
