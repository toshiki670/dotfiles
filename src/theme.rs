//! テーマ状態ファイル（§10.2）: 手動固定／OS 追従の状態を持つ単一ファイル `~/.config/dotfiles/theme`。
//!
//! `dark` / `light` / `auto` を**平文 1 語**で書く（`local.toml` / `hooks.toml` と同じ
//! `~/.config/dotfiles/` 配下だが、単一スカラなので toml にせず greppable な平文にする）。
//! 書き手は `dotfiles color`（[`crate::color`]）、読み手は apply 時の `when.theme` 評価
//! （[`crate::gate`] 経由）。dotfiles 内部のテーマ状態の SSOT で、秘匿値ではないため通常
//! パーミッションで書く。値の語彙（auto/dark/light）は [`Theme`] が SSOT（Display / FromStr 共有）。

use crate::manifest::Theme;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// 状態ファイルのパス（`<home>/.config/dotfiles/theme`）。
pub fn path(home: &Path) -> PathBuf {
    home.join(".config/dotfiles/theme")
}

/// 現在のテーマ状態を読む。ファイルが無い・読めない・未知の値はいずれも [`Theme::Auto`]
/// （OS 追従＝現状の挙動・§10.1）に倒す。状態は disposable で、壊れていても apply を止めずに
/// 既定（追従）へフォールバックする（[`crate::onchange`] の状態と同じ方針）。
pub fn current(home: &Path) -> Theme {
    match std::fs::read_to_string(path(home)) {
        Ok(text) => Theme::from_str(text.trim()).unwrap_or(Theme::Auto),
        Err(_) => Theme::Auto,
    }
}

/// テーマ状態を書き出す（親ディレクトリを作成。末尾に改行 1 つ）。`Display` の正規表記
/// （auto/dark/light）をそのまま書くので [`current`] の `FromStr` と round-trip する。
pub fn set(home: &Path, theme: Theme) -> Result<(), String> {
    let path = path(home);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::write(&path, format!("{theme}\n"))
        .map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_auto() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(current(home.path()), Theme::Auto);
    }

    #[test]
    fn set_then_current_round_trips() {
        for theme in [Theme::Auto, Theme::Dark, Theme::Light] {
            let home = tempfile::tempdir().unwrap();
            set(home.path(), theme).unwrap();
            assert_eq!(current(home.path()), theme);
        }
    }

    #[test]
    fn unknown_value_falls_back_to_auto() {
        // 壊れた状態（手書き typo など）は追従へ倒す（fail safe。apply を止めない）。
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(path(home.path()).parent().unwrap()).unwrap();
        std::fs::write(path(home.path()), "darkk\n").unwrap();
        assert_eq!(current(home.path()), Theme::Auto);
    }

    #[test]
    fn current_trims_whitespace() {
        // set は改行付きで書くので、読み戻しは trim 前提。手書きの前後空白も許容する。
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(path(home.path()).parent().unwrap()).unwrap();
        std::fs::write(path(home.path()), "  dark \n").unwrap();
        assert_eq!(current(home.path()), Theme::Dark);
    }
}
