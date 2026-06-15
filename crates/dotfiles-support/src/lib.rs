//! toshiki670/dotfiles の各コマンド（`crates/*`）で共有するロジック。
//!
//! コマンド固有のロジックは各クレート（`crates/<name>/`）に置く。ここには
//! 複数のクレートから使う純粋なユーティリティだけを置く。

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
/// v-sync / daily-check-worker / gh-clone から使う。
pub fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_command_is_not_found() {
        assert!(!command_exists("definitely-not-a-real-command-9b2f7e"));
    }
}
