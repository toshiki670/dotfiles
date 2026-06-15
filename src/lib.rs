//! toshiki670/dotfiles の CLI コマンドで共有するロジック。
//!
//! 各コマンド固有のロジックは各 `src/bin/<name>/` 配下に置く。ここには
//! 複数の bin から使う純粋なユーティリティだけを置く。

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
/// v-sync / daily-check-worker / gh-clone から使う。
pub fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}
