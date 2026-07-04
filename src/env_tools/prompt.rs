//! 対話 y/N 確認。純判定と stdin の IO ラッパに分ける。

use std::io::{self, Write};

/// 応答が肯定（`y` または `Y` の 1 文字のみ）かどうか。
///
/// `yes` などは肯定として扱わない。
pub fn is_yes(reply: &str) -> bool {
    matches!(reply.trim(), "y" | "Y")
}

/// `question [y/N] ` を出して 1 行読み、[`is_yes`] が真なら `true`。
///
/// EOF（非対話・Ctrl-D）や読み取り失敗は `false`（= No）として扱う。
pub fn confirm(question: &str) -> bool {
    print!("{question} [y/N] ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) | Err(_) => false,
        Ok(_) => is_yes(&line),
    }
}

#[cfg(test)]
mod tests {
    use super::is_yes;

    #[test]
    fn accepts_only_single_y() {
        assert!(is_yes("y"));
        assert!(is_yes("Y"));
        assert!(is_yes("y\n"));
        assert!(is_yes("  Y  "));
    }

    #[test]
    fn rejects_everything_else() {
        assert!(!is_yes("yes"));
        assert!(!is_yes("n"));
        assert!(!is_yes("N"));
        assert!(!is_yes(""));
        assert!(!is_yes("\n"));
    }
}
