//! `upkeep` の各サブコマンドの **E2E テスト**（assert_cmd で実バイナリを起動して検証）。
//!
//! # 検証内容（サブコマンド別）
//!
//! - [`upgrade`]: `--help`/`--version`、全 PM 存在で順に更新呼び出し、PM 不在でスキップ、
//!   `cargo` はあるが `cargo-install-update` 不在で Cargo ステップをスキップ
//! - [`cleanup`]: `--help`/`--version`、確認 `y` で実削除コマンド呼び出し、`--dry-run` で
//!   各コマンドに dry-run フラグ付与、確認 `n`/EOF で何も実行しない、未知オプションで失敗
//! - [`doctor`]: `--help`/`--version`、全 PM 存在で順に診断呼び出し、PM 不在でスキップ、
//!   診断が問題を検出（非ゼロ exit）しても `upkeep doctor` 自体は成功しその出力を表示
//!
//! 外部コマンド（brew / mise / cargo / cargo-cache / cargo-install-update）は PATH 先頭に
//! 置く**純シェルビルトインのスタブ**（[`stub_body`]）で差し替える。PM「不在」は
//! [`EMPTY_PATH`]（実在しない PATH）で再現する。

use std::fs;
use std::path::Path;

mod cleanup;
mod doctor;
mod outdated;
mod upgrade;

/// 実在しない PATH（外部コマンドを「不在」にするため）。
pub(crate) const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

/// 名前 `name` のスタブ本体を作る。呼ばれると `name` ＋引数を `$UPKEEP_LOG` に追記する。
///
/// `printf` と `>>` は sh のビルトインなので外部コマンド（PATH）に依存しない。これにより
/// PATH をスタブディレクトリだけにしても動き、ホストの brew/mise 等が混入しない。
pub(crate) fn stub_body(name: &str) -> String {
    format!("#!/bin/sh\nprintf '{name} %s\\n' \"$*\" >> \"$UPKEEP_LOG\"\n")
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

/// `bin` ディレクトリに指定名のスタブ群を書き出す（呼び出し記録用）。
pub(crate) fn write_stubs(bin: &Path, names: &[&str]) {
    for name in names {
        write_exec(bin, name, &stub_body(name));
    }
}

/// 呼び出しを記録せず、指定した環境変数の中身をそのまま stdout に出すスタブ本体
/// （`tests/gcm/mod.rs` の `CLAUDE_STUB` と同じ手法の一般化）。
///
/// stdin は読み捨てる: 呼び出し元が `Stdio::piped()` で書き込んで閉じるケース
/// （claude 要約）でも、`Command::output()` の既定（stdin は null）のケース
/// （brew/mise/cargo/curl/gh）でも、どちらでも安全にブロックせず終わる。
pub(crate) fn stdout_stub_body(env_var: &str) -> String {
    format!("#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' \"${env_var}\"\n")
}
