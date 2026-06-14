//! toshiki670/dotfiles の CLI コマンドで共有するロジック。
//!
//! 各コマンドは `src/bin/<name>/main.rs` に置き、純粋に検証可能なロジックは
//! ここへ切り出してユニットテスト対象にする。

pub mod lint;

/// `git remote` の出力（1 行 1 リモート名）に指定したリモートが含まれるか判定する。
pub fn remote_exists(remotes_output: &str, name: &str) -> bool {
    remotes_output.lines().any(|line| line.trim() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_exists_finds_named_remote() {
        let output = "origin\nupstream\n";
        assert!(remote_exists(output, "upstream"));
        assert!(remote_exists(output, "origin"));
    }

    #[test]
    fn remote_exists_returns_false_when_absent() {
        assert!(!remote_exists("origin\n", "upstream"));
    }

    #[test]
    fn remote_exists_ignores_surrounding_whitespace() {
        assert!(remote_exists("  upstream  \n", "upstream"));
    }

    #[test]
    fn remote_exists_is_exact_match() {
        assert!(!remote_exists("upstream-mirror\n", "upstream"));
    }
}
