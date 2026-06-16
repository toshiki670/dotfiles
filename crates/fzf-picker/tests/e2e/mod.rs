//! `fzf-picker` の各 bin の **E2E テスト**（assert_cmd で実バイナリを起動して検証）。
//!
//! 内部の純ロジック（パース・行成形・展開）のユニットテストは各 lib モジュールの
//! `#[cfg(test)]` 側にある。ここは「ビルドしたバイナリの振る舞い」だけを扱う。
//!
//! # 検証内容（bin 別）
//!
//! - [`fzf_ghq_cd`]: 引数・`ghq` 不在(127)・選択→パス出力・fzf キャンセル・空 list・
//!   リンク worktree のツリー構築（メイン除外）
//! - [`fzf_worktree_remove`]: 引数・非 git repo・削除候補なし・確認 y で削除・確認 n で
//!   残置・fzf キャンセルで残置・削除対象の内側からの実行で退避パス出力
//! - [`cdabbr`]: 引数・相対パス拒否・該当なし・fzf 不在で単一/再帰/複数・fzf 選択
//!
//! 外部コマンド `ghq` / `fzf` は PATH 先頭に置くスタブで差し替え、`git` は実物を使う
//! （PATH に実 PATH も残す）。スタブは `$FZF_PICK`/`$FZF_EXIT`/`$GHQ_*` 環境変数で挙動を
//! 制御する。共有ヘルパー（下記）は各 bin のテストファイルから `crate::` で使う。

use std::fs;
use std::path::Path;

mod cdabbr;
mod fzf_ghq_cd;
mod fzf_worktree_remove;

/// 実在しない PATH（外部コマンドを「不在」にするため）。
pub(crate) const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

/// `ghq` スタブ: `root` は `$GHQ_ROOT` を、`list` は `$GHQ_LIST_FILE` の中身を返す。
pub(crate) const GHQ_STUB: &str = "#!/bin/sh\ncase \"$1\" in\n  root) printf '%s\\n' \"$GHQ_ROOT\" ;;\n  list) cat \"$GHQ_LIST_FILE\" ;;\nesac\n";

/// `fzf` スタブ: 候補(stdin)を `$FZF_DUMP` に保存し、`$FZF_PICK` を選択行として出力、
/// `$FZF_EXIT`（既定 0）で終了する。
pub(crate) const FZF_STUB: &str = "#!/bin/sh\ncat > \"${FZF_DUMP:-/dev/null}\"\n[ -n \"$FZF_PICK\" ] && printf '%s\\n' \"$FZF_PICK\"\nexit \"${FZF_EXIT:-0}\"\n";

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

/// `prefix` を先頭に足した PATH を作る（実 PATH を残すので `git` は実物を使う）。
pub(crate) fn path_with(prefix: &Path) -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![prefix.to_path_buf()];
    paths.extend(std::env::split_paths(&existing));
    std::env::join_paths(paths).unwrap()
}

/// `git -C dir <args>` を実行し成功を要求する（テスト用 repo の構築に使う）。
pub(crate) fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}
