//! コマンド実行プリミティブ: step の `cmd` 内容源を spawn する。
//!
//! - **input.cmd**（[`run`]）: argv を実行し、その標準出力を内容へ畳む中身にする。補完生成
//!   （`gh completion …`）や生きた外部状態の取得（`defaults export …`）が使う。
//! - **output.cmd**（[`run_piped`]）: 合成済みの内容を子プロセスの標準入力へ渡す。生きた外部状態への
//!   書き戻し（`defaults import … -`）が使う。**毎 apply 実行**され、コマンドが冪等であることを契約
//!   とする（#531 が実機検証して出荷済みの挙動）。
//!
//! いずれも非ゼロ終了は stderr 付きでエラーにし、apply を止める。

use std::io::Write;
use std::process::{Command, Stdio};

/// `cmd`（argv）を実行し標準出力を返す。非ゼロ終了は stderr 付きでエラーにする。
/// input.cmd（標準出力を内容へ畳む中身源）が使う。
pub fn run(cmd: &[String]) -> Result<Vec<u8>, String> {
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map_err(|e| format!("{}: 実行失敗: {e}", cmd[0]))?;
    if !output.status.success() {
        return Err(format!(
            "{cmd:?} が異常終了 ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// `cmd`（argv）を実行し、`input`（合成済みの内容）を子プロセスの標準入力へ渡す。output.cmd が使う。
///
/// 標準出力は捨てる（`Stdio::null`）。書き戻し先コマンド（`defaults import … -` 等）は標準出力に
/// 意味を持たない sink であり、`null` にすることで「子が stdout へ書こうとして詰まり、親は stdin を
/// 書き終えられず待つ」デッドロックの片方を塞ぐ。
///
/// もう片方（stderr）は捨てずに捕捉するため、素朴に「stdin を全部書いてから `wait_with_output`」の
/// 順で実装すると、子が stdin を読み切る前に stderr の pipe バッファを埋めてブロックした場合に
/// 相互待ちが起こりうる（親は stdin write で止まり、子は stderr write で止まる）。これを避けるため
/// `std::process::Command` の標準パターン（[`std::process::Child::wait_with_output`] 相当の自前実装）に
/// 倣い、**stdin の書き込みを別スレッドへ出し**、メインスレッドは `wait_with_output` で stdout/stderr を
/// 同時に drain する。書き込みスレッドと `wait_with_output` は独立に進むため、上記のどちらの順でも
/// デッドロックしない。
///
/// 書き込みスレッドの失敗（例: 子が早期に stdin を閉じた `BrokenPipe`）は、プロセスの終了ステータスが
/// 非ゼロならそちらを優先して報告する（実際の失敗理由は exit status + stderr にあり、`BrokenPipe` は
/// 「子が早期に読むのをやめた」という結果を示すだけで原因ではないため）。プロセスが成功終了したのに
/// 書き込みが失敗した場合（子が入力を全部読まずに正常終了する等）は、内容の反映が不完全な可能性があり
/// `output.cmd` の「合成済みの内容をそのまま反映する」契約を破るため、黙って握りつぶさずエラーにする。
pub fn run_piped(cmd: &[String], input: &[u8]) -> Result<(), String> {
    let mut child = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: 実行失敗: {e}", cmd[0]))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| format!("{}: 標準入力を確保できません", cmd[0]))?;
    let input = input.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&input));

    // メインスレッドは stdin 書き込みと並行して stdout(null)/stderr(piped) を drain する
    // （`wait_with_output` が両方とも読み切ってから子の終了を待つ）。
    let output = child
        .wait_with_output()
        .map_err(|e| format!("{}: 実行失敗: {e}", cmd[0]))?;
    let write_result = writer
        .join()
        .map_err(|_| format!("{}: 標準入力書き込みスレッドが panic しました", cmd[0]))?;

    if !output.status.success() {
        return Err(format!(
            "{cmd:?} が異常終了 ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    // プロセスは成功終了しているが、書き込みが失敗していた場合（子が入力を全部読まずに
    // 正常終了する等）は反映不完全の可能性があるため、ここで初めて書き込みエラーを報告する。
    write_result.map_err(|e| format!("{}: 標準入力への書き込み失敗: {e}", cmd[0]))?;
    Ok(())
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn sh(script: &str) -> Vec<String> {
        vec!["sh".to_string(), "-c".to_string(), script.to_string()]
    }

    /// `run_piped` を別スレッドで実行し、`timeout` 以内に完了しなければテスト失敗にする
    /// （相互待ちのリグレッションが CI を無限にハングさせず、失敗として現れるようにする）。
    fn run_piped_with_watchdog(
        cmd: &[String],
        input: &[u8],
        timeout: Duration,
    ) -> Result<(), String> {
        let cmd = cmd.to_vec();
        let input = input.to_vec();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(run_piped(&cmd, &input));
        });
        rx.recv_timeout(timeout).unwrap_or_else(|_| {
            panic!("run_piped が {timeout:?} 以内に完了しなかった（相互待ちの疑い）")
        })
    }

    #[test]
    fn run_piped_writes_input_to_child_stdin() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("captured");
        let cmd = sh(&format!("cat > {}", out.display()));
        run_piped_with_watchdog(&cmd, b"hello\n", Duration::from_secs(5)).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"hello\n");
    }

    /// 非ゼロ終了時は、標準入力の書き込みが（子が読み切る前に終了して）`BrokenPipe` になっていても、
    /// 実際の終了ステータス・stderr が報告されること（書き込みエラーで握りつぶさない・#4）。
    #[test]
    fn run_piped_reports_real_exit_status_even_when_stdin_write_is_broken_pipe() {
        // 子は stdin を一切読まずに stderr へ出して非ゼロ終了する。入力をパイプバッファ超の
        // サイズにして、書き込み側が確実に BrokenPipe を踏む状況を作る。
        let cmd = sh("echo custom-failure-message 1>&2; exit 7");
        let input = vec![b'x'; 4 * 1024 * 1024];
        let err = run_piped_with_watchdog(&cmd, &input, Duration::from_secs(10)).unwrap_err();
        assert!(
            err.contains("custom-failure-message") && err.contains("異常終了"),
            "実際の終了理由（stderr/exit status）が報告されていない: {err}"
        );
        assert!(
            !err.contains("標準入力への書き込み失敗"),
            "書き込み側の BrokenPipe が本当の失敗理由を覆い隠している: {err}"
        );
    }

    /// 子が「stdin を読み切る前に stderr のパイプバッファを埋める」と、素朴な実装
    /// （stdin を全部書いてから stdout/stderr を drain する）は相互待ちになりうる。
    /// stdin 書き込みを別スレッドへ出し `wait_with_output` と並行させる本実装では、
    /// タイムアウトせず完了することを確認する（#4 の回帰防止）。
    #[test]
    fn run_piped_does_not_deadlock_when_child_writes_large_stderr_before_reading_stdin() {
        let big = 2 * 1024 * 1024; // 典型的な pipe バッファ（数十 KB）を大きく超えるサイズ。
        let cmd = sh(&format!(
            "yes xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx | head -c {big} 1>&2; cat > /dev/null"
        ));
        let input = vec![b'a'; big];
        run_piped_with_watchdog(&cmd, &input, Duration::from_secs(15)).unwrap();
    }
}
