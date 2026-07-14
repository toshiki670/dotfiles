//! 外部コマンドの存在確認とラベル付き実行。

use std::process::Command;

/// `command -v <name>` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
pub fn command_exists(name: &str) -> bool {
    which::which(name).is_ok()
}

/// `=== <label> ===` を出してコマンドを実行し、終了後に空行を入れる。
///
/// 失敗してもパニックや異常終了はせず、警告を出して続行する。
pub fn run(label: &str, program: &str, args: &[&str]) {
    println!("=== {label} ===");
    let ok = Command::new(program)
        .args(args)
        .status()
        .is_ok_and(|status| status.success());
    if !ok {
        println!("⚠️  {label} failed, continuing...");
    }
    println!();
}
