//! 対話入力: TTY 判定と1行入力。sensitive な値は端末エコーを落として読む。
//!
//! apply はローカル値の取得経路を **TTY=対話 / 非TTY=警告のみ**に分ける（[`crate::locals::resolve`]）。
//! 本モジュールはその TTY 側を担う。プロンプトは stdout を汚さないよう stderr へ出す。非エコーは
//! termios の `ECHO` を一時的に落とし、RAII ガードで必ず復元する（Unix。非 Unix は通常入力）。

use std::io::{self, BufRead, IsTerminal, Write};

/// 標準入力が TTY か。apply の取得経路（対話 / 警告のみ）を分岐するのに使う。
pub fn is_tty() -> bool {
    io::stdin().is_terminal()
}

/// `label`（値の名前）を提示して1行入力を受け取る。`sensitive` のときは端末エコーを抑制する。
/// プロンプトは stderr へ出し、**末尾の行終端（`\n` / `\r\n`）のみ除去**して返す。前後の空白は
/// 保持する（値は verbatim ＝ `local set <name> <value>` と同じ扱い）。full trim をしないのは、
/// 前後空白が有意な sensitive 値（パスフレーズ等）を黙って壊さないため。
pub fn ask(label: &str, sensitive: bool) -> Result<String, String> {
    let mut err = io::stderr();
    let _ = write!(err, "{label} を入力してください: ");
    let _ = err.flush();

    let line = if sensitive {
        read_line_no_echo()?
    } else {
        read_line()?
    };
    Ok(strip_line_ending(line))
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

/// 端末エコーを落として1行読む（秘匿値用）。読み終えたら端末設定を必ず復元する（Unix）。
#[cfg(unix)]
fn read_line_no_echo() -> Result<String, String> {
    use std::os::unix::io::AsRawFd;

    let fd = io::stdin().as_raw_fd();
    let guard = EchoGuard::disable(fd)?;
    let line = read_line();
    drop(guard); // 端末設定を復元してから改行を出す。
    // エコーを落としている間は Enter の改行も表示されないため、ここで1つ補う。
    let _ = writeln!(io::stderr());
    line
}

/// 非 Unix ではエコー抑制の termios が無いため通常入力にフォールバックする。
#[cfg(not(unix))]
fn read_line_no_echo() -> Result<String, String> {
    read_line()
}

/// termios の `ECHO` を一時的に落とし、`Drop` で元に戻す RAII ガード（Unix）。
/// エラーで早期 return しても、確保済みのガードが drop されれば端末は復元される。
#[cfg(unix)]
struct EchoGuard {
    fd: i32,
    original: libc::termios,
}

#[cfg(unix)]
impl EchoGuard {
    /// `fd`（端末）の現在属性を保存し、`ECHO` を落とした属性を設定する。
    fn disable(fd: i32) -> Result<Self, String> {
        // SAFETY: term は tcgetattr で完全に初期化してから読む。fd は stdin（端末）。
        unsafe {
            let mut term: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut term) != 0 {
                return Err("端末属性の取得に失敗（tcgetattr）".to_string());
            }
            let original = term; // libc::termios は Copy。復元用に保存。
            term.c_lflag &= !libc::ECHO;
            if libc::tcsetattr(fd, libc::TCSAFLUSH, &term) != 0 {
                return Err("端末属性の設定に失敗（tcsetattr）".to_string());
            }
            Ok(Self { fd, original })
        }
    }
}

#[cfg(unix)]
impl Drop for EchoGuard {
    fn drop(&mut self) {
        // SAFETY: 取得済みの original を書き戻すのみ。失敗時も他に取れる手当ては無い。
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.original);
        }
    }
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
