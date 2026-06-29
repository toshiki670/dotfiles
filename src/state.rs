//! 状態駆動 gate（§10）が参照する、`~/.config/dotfiles/` 配下のスカラ状態ファイル。
//!
//! `profile`（マシンクラス）・`theme`（color スライス）のような **user が一度選んでおく状態**を、
//! キー名ごとに 1 ファイル＝1 スカラ値で持つ。`when`（[`crate::manifest::When`]）の状態 gate は
//! ここの現在値と一致するときだけ断片を採用する。値は秘匿でない（commit に載りはしないが隠す
//! 対象でもない）ので平文・通常パーミッションで書く ― 秘匿値の named value ストア
//! （[`crate::locals::store`]・0600）とは別物。
//!
//! 未設定（ファイル無し / 空）は `None` を返す。profile はこれを「private ではない」既定として
//! 解釈し、新規・仕事マシンへ private 設定が漏れないようにする（明示 opt-in）。`theme` も同じ
//! read/write を後で相乗りする想定で、キー名でだけ分ける（機構を二重実装しない）。

use std::path::{Path, PathBuf};

/// マシンクラス（private / work …）の状態キー。`dotfiles profile <name>` が書き、
/// `when = { profile = … }`（[`crate::apply::gate`]）が読む。
pub const PROFILE: &str = "profile";

/// 状態ファイルのパス（`<home>/.config/dotfiles/<key>`）。named value ストアと同じディレクトリ。
pub fn path(home: &Path, key: &str) -> PathBuf {
    home.join(".config/dotfiles").join(key)
}

/// `key` の現在値を読む。ファイルが無い・空（空白のみ）なら `None`（＝未設定）。
///
/// 末尾改行や前後の空白は write が付ける整形なので trim して落とす（値そのものだけを返す）。
pub fn read(home: &Path, key: &str) -> Result<Option<String>, String> {
    let path = path(home, key);
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let value = text.trim();
            Ok((!value.is_empty()).then(|| value.to_string()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: 読み込み失敗: {e}", path.display())),
    }
}

/// `key` へ `value` を書く（親ディレクトリを作成）。末尾に改行を 1 つ付ける（read は trim する）。
pub fn write(home: &Path, key: &str, value: &str) -> Result<(), String> {
    let path = path(home, key);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    std::fs::write(&path, format!("{value}\n"))
        .map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_reads_none() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(read(home.path(), PROFILE).unwrap(), None);
    }

    #[test]
    fn write_then_read_round_trips() {
        let home = tempfile::tempdir().unwrap();
        write(home.path(), PROFILE, "private").unwrap();
        assert_eq!(
            read(home.path(), PROFILE).unwrap(),
            Some("private".to_string())
        );
    }

    #[test]
    fn empty_or_whitespace_file_reads_none() {
        // 空・空白のみのファイルは「未設定」と同じ扱い（trim 後に空なら None）。
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(path(home.path(), PROFILE).parent().unwrap()).unwrap();
        std::fs::write(path(home.path(), PROFILE), "  \n").unwrap();
        assert_eq!(read(home.path(), PROFILE).unwrap(), None);
    }

    #[test]
    fn read_trims_trailing_newline_from_write() {
        // write が付ける末尾改行は値に含めない（read が trim する）。
        let home = tempfile::tempdir().unwrap();
        write(home.path(), PROFILE, "work").unwrap();
        let raw = std::fs::read_to_string(path(home.path(), PROFILE)).unwrap();
        assert_eq!(raw, "work\n", "ファイルには改行付きで書く");
        assert_eq!(
            read(home.path(), PROFILE).unwrap(),
            Some("work".to_string())
        );
    }
}
