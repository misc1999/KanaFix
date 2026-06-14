//! `kanafix.toml` の読み込みと、初回起動時の既定設定生成。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    /// 補正対象とするプロセスの実行ファイル名（大小無視）。
    pub(crate) apps: Vec<String>,
    /// 再注入イベント間の間隔（ミリ秒）。
    pub(crate) pace_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            apps: vec!["notepad.exe".to_string()],
            pace_ms: 6,
        }
    }
}

fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("kanafix.toml")))
        .unwrap_or_else(|| PathBuf::from("kanafix.toml"))
}

/// 設定を読み込む。ファイルが無ければ既定値を書き出してそれを使う。
pub(crate) fn load_config() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
            eprintln!(
                "設定ファイル {} が不正です: {e}（既定値を使用）",
                path.display()
            );
            Config::default()
        }),
        Err(_) => {
            let config = Config::default();
            write_default_config(&path, &config);
            config
        }
    }
}

/// 既定設定をファイルへ書き出す。失敗しても続行するが、原因は通知する。
fn write_default_config(path: &Path, config: &Config) {
    let text = match toml::to_string_pretty(config) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("既定設定の生成に失敗しました: {e}");
            return;
        }
    };
    match std::fs::write(path, text) {
        Ok(()) => eprintln!("既定の設定ファイルを生成しました: {}", path.display()),
        Err(e) => eprintln!(
            "設定ファイル {} を書き出せませんでした: {e}",
            path.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_targets_notepad() {
        let config = Config::default();
        assert_eq!(config.apps, vec!["notepad.exe".to_string()]);
        assert_eq!(config.pace_ms, 6);
    }

    #[test]
    fn config_round_trips_through_toml() {
        let original = Config::default();
        let text = toml::to_string_pretty(&original).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.apps, original.apps);
        assert_eq!(parsed.pace_ms, original.pace_ms);
    }

    #[test]
    fn invalid_toml_is_rejected() {
        let result: std::result::Result<Config, _> = toml::from_str("apps = 42");
        assert!(result.is_err());
    }
}
