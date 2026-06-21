//! named value の対話取得（§9.1 step3）: apply が TTY のとき未設定値をその場で尋ねる。
//!
//! [`stdin_is_tty`] で TTY を判定し（非 TTY は呼び出し側が警告のみで継続）、[`prompt`] で 1 値を
//! 読む。`sensitive` 指定の値はエコーを抑制する（unix は termios の `ECHO` を一時 off。非 unix は
//! 抑制非対応のため通常読みにフォールバック）。プロンプト・改行は stdout を汚さないよう stderr へ出す。

use std::io::{self, IsTerminal, Write};

/// 標準入力が TTY か（対話取得が可能か）。非 TTY ではフック実行等とみなし呼び出し側が警告のみで継続する。
pub fn stdin_is_tty() -> bool {
    io::stdin().is_terminal()
}

/// `name` の値を 1 行読む。`sensitive` ならエコーを抑制する。末尾の改行は除去する。
pub fn prompt(name: &str, sensitive: bool) -> io::Result<String> {
    let hint = if sensitive {
        "machine-local, hidden"
    } else {
        "machine-local"
    };
    eprint!("dotfiles: {name} を入力してください ({hint}): ");
    io::stderr().flush()?;
    let line = if sensitive {
        read_line_no_echo()?
    } else {
        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        s
    };
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}

/// エコーを抑制して 1 行読む（unix: termios の `ECHO` を一時 off にして復元）。
#[cfg(unix)]
fn read_line_no_echo() -> io::Result<String> {
    use std::os::unix::io::AsRawFd;

    let fd = io::stdin().as_raw_fd();
    let mut term: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(fd, &mut term) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let original = term;
    term.c_lflag &= !(libc::ECHO as libc::tcflag_t);
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &term) } != 0 {
        return Err(io::Error::last_os_error());
    }

    let mut s = String::new();
    let read = io::stdin().read_line(&mut s);

    // 入力中に何があっても端末設定は必ず元へ戻す。
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original) };
    eprintln!(); // エコーされない改行を補い、後続出力が同じ行に続かないようにする。

    read?;
    Ok(s)
}

/// 非 unix ではエコー抑制をサポートせず通常読みにフォールバックする。
#[cfg(not(unix))]
fn read_line_no_echo() -> io::Result<String> {
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s)
}
