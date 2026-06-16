//! `env-tools` の各 bin の **E2E テスト**（assert_cmd で実バイナリを起動して検証）。
//!
//! 内部の純ロジック（`prompt::is_yes`）のユニットテストは lib 側の `#[cfg(test)]` にある。
//! ここは「ビルドしたバイナリの振る舞い」だけを扱う。
//!
//! # 検証内容（bin 別）
//!
//! - [`upgrade_env`]: `--help`/`--version`、全 PM 存在で順に更新呼び出し、PM 不在でスキップ、
//!   `cargo` はあるが `cargo-install-update` 不在で Cargo ステップをスキップ
//! - [`cleanup_env`]: `--help`/`--version`、確認 `y` で実削除コマンド呼び出し、`--dry-run` で
//!   各コマンドに dry-run フラグ付与、確認 `n`/EOF で何も実行しない、未知オプションで失敗
//!
//! 外部コマンド（brew / mise / cargo / cargo-cache / cargo-install-update）は PATH 先頭に
//! 置く**純シェルビルトインのスタブ**で差し替える。各スタブは自分の名前＋引数を
//! `$ENV_TOOLS_LOG` に追記するだけなので、呼ばれたコマンド列をログから検証できる。
//! PM「不在」は [`EMPTY_PATH`]（実在しない PATH）で再現する。

use std::fs;
use std::path::Path;

mod cleanup_env;
mod upgrade_env;

/// 実在しない PATH（外部コマンドを「不在」にするため）。
pub(crate) const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

/// 名前 `name` のスタブ本体を作る。呼ばれると `name` ＋引数を `$ENV_TOOLS_LOG` に追記する。
///
/// `printf` と `>>` は sh のビルトインなので外部コマンド（PATH）に依存しない。これにより
/// PATH をスタブディレクトリだけにしても動き、ホストの brew/mise 等が混入しない。
pub(crate) fn stub_body(name: &str) -> String {
    format!("#!/bin/sh\nprintf '{name} %s\\n' \"$*\" >> \"$ENV_TOOLS_LOG\"\n")
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
