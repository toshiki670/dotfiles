//! 外部コマンドの存在確認とラベル付き実行（旧 bash の `command -v` / `run()` 相当）。

use std::process::Command;

/// `command -v <name>` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
pub fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
}

/// `=== <label> ===` を出してコマンドを実行し、終了後に空行を入れる。
///
/// 旧 bash の `run()` と同じく、失敗してもパニックや異常終了はせず警告を出して続行する
/// （`"$@" || echo "⚠️  $label failed, continuing..."` 相当）。
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
