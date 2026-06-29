//! 起動: 外部コマンド（主に fzf）の起動と存在確認の IO。
//!
//! - [`run_fzf`]: 候補を渡して fzf を起動し選択行を得る（全 bin 共通）
//! - [`command_exists`]: PATH 上にコマンドがあるか（起動前チェック）

use std::io::Write;
use std::process::{Command, Stdio};

/// 候補行を stdin から流して fzf を起動し、選択行（あれば）を返す。
///
/// 各 bin 共通の `--delimiter <tab> --with-nth 1` に `extra_args`（`--preview` 等）を
/// 足して実行する。fzf は UI を `/dev/tty` に描くため stderr は継承で良い。stdout は
/// 捕捉して選択行を読む。キャンセル（exit code != 0）や空選択は `None`。
pub fn run_fzf(lines: &[String], extra_args: &[&str]) -> std::io::Result<Option<String>> {
    let mut child = Command::new("fzf")
        .args(["--delimiter", "\t", "--with-nth", "1"])
        .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        for line in lines {
            writeln!(stdin, "{line}")?;
        }
        // stdin を drop して EOF を送る。
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let selection = String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string();
    Ok((!selection.is_empty()).then_some(selection))
}

/// `command -q` 相当: PATH 上に指定コマンドの実行ファイルがあるか判定する。
pub fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
}
