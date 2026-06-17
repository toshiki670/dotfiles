//! `clip` の E2E テスト（assert_cmd で実バイナリを起動して検証）。
//!
//! 内部ロジックは外部コマンド（pbcopy / osascript）呼び出しが主体のため、ユニットでは
//! なくここで「ビルドしたバイナリの振る舞い」を検証する。
//!
//! # 検証内容（対象別ファイル）
//!
//! - [`cli`]:  `--help`/`--version`、サブコマンド無し/未知サブコマンド/ファイル引数欠落で
//!   exit code 2、`completions fish` が補完スクリプトを出力（OS 非依存）
//! - [`obj`]:  macOS で osascript を `set the clipboard to POSIX file …` で呼ぶ／非 macOS で失敗
//! - [`text`]: macOS で pbcopy にファイルの中身をそのまま渡す／非 macOS で失敗
//! - [`path`]: macOS で絶対パスを pbcopy へ渡し stdout にも出力する／非 macOS で失敗
//!
//! 外部コマンド（pbcopy / osascript）は PATH 先頭に置くスタブで差し替える。pbcopy スタブは
//! stdin を一時ファイルへ退避し、osascript スタブは引数を退避して、呼び出し内容を検証する。

use std::ffi::OsString;
use std::fs;
use std::path::Path;

use assert_cmd::Command;

mod cli;
mod obj;
mod path;
mod text;

/// 実バイナリ `clip` の Command。
pub(crate) fn clip() -> Command {
    Command::cargo_bin("clip").unwrap()
}

/// 実行可能なスタブスクリプトを `dir/name` に書き出す。
pub(crate) fn write_exec(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// `bin` を先頭に追加した PATH を返す（スタブを実コマンドより優先させる）。
pub(crate) fn path_with(bin: &Path) -> OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    std::env::join_paths(std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(&existing)))
        .unwrap()
}
