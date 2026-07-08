//! 対話入力: TTY 判定と1行入力。
//!
//! apply はローカル値の取得経路を **TTY=対話 / 非TTY=警告のみ**に分ける（[`crate::locals::resolve`]）。
//! 本モジュールはその TTY 側を担う。プロンプトは stdout を汚さないよう stderr へ出す。

use std::io::{self, BufRead, IsTerminal, Write};

/// 標準入力が TTY か。apply の取得経路（対話 / 警告のみ）を分岐するのに使う。
pub fn is_tty() -> bool {
    io::stdin().is_terminal()
}

/// `label`（値の名前）を提示して1行入力を受け取る。プロンプトは stderr へ出し、**末尾の行終端
/// （`\n` / `\r\n`）のみ除去**して返す。前後の空白は保持する（値は verbatim ＝ `local set <name> <value>`
/// と同じ扱い）。full trim をしないのは、前後空白が有意な値を黙って壊さないため。
pub fn ask(label: &str) -> Result<String, String> {
    let mut err = io::stderr();
    let _ = write!(err, "{label} を入力してください: ");
    let _ = err.flush();

    Ok(strip_line_ending(read_line()?))
}

/// 末尾の1つの行終端（`\n`、または `\r\n`）だけを取り除く。それ以外の空白・文字は保持する。
fn strip_line_ending(mut line: String) -> String {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}

/// 標準入力から1行読む。
fn read_line() -> Result<String, String> {
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("入力の読み取りに失敗: {e}"))?;
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::strip_line_ending;

    #[test]
    fn strips_only_trailing_line_ending() {
        // 末尾の改行のみ除去（LF / CRLF）。
        assert_eq!(strip_line_ending("value\n".to_string()), "value");
        assert_eq!(strip_line_ending("value\r\n".to_string()), "value");
        // 前後の空白・内部空白は保持（verbatim ＝ local set と同じ）。
        assert_eq!(strip_line_ending("  pa ss  \n".to_string()), "  pa ss  ");
        // 改行が無ければそのまま（EOF で終端されたケース）。
        assert_eq!(strip_line_ending("value".to_string()), "value");
    }
}
