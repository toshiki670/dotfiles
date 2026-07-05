//! `claude -p` 呼び出し。

use std::io::Write;
use std::process::{Command, Stdio};

use super::proposals::{Commit, parse_proposals};

const SYSTEM_PROMPT: &str = "You are a git commit message generator using Conventional Commits.

Analyze the staged diff and split into the minimum number of semantically independent commits.
- Single concern -> one entry
- Multiple independent concerns (e.g. feat + fix, or unrelated files) -> multiple entries

Output a JSON array of commit objects:

[{\"message\": \"<type>[scope]: <description>\", \"files\": [\"path/to/file\"]}, ...]

Rules:
- type: feat, fix, docs, style, refactor, test, chore, perf, ci, build
- description: English, imperative mood (add, fix, update, remove, ...)
- Every staged file must appear in exactly one entry's files array";

/// claude を呼び出して commit 提案を得る。
pub(crate) fn call_claude(conversation: &str, model: &str) -> Option<Vec<Commit>> {
    let mut child = match Command::new("claude")
        .args(["-p", "--model", model, "--system-prompt", SYSTEM_PROMPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!("生成に失敗しました。claude コマンドが利用可能か確認してください。");
            return None;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(conversation.as_bytes());
        // stdin はここで drop され閉じる。
    }

    let output = child.wait_with_output().ok()?;
    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    if raw.trim().is_empty() {
        eprintln!("生成に失敗しました。claude コマンドが利用可能か確認してください。");
        return None;
    }

    match parse_proposals(&raw) {
        Ok(proposals) if !proposals.is_empty() => Some(proposals),
        Ok(_) => {
            eprintln!("コミット提案が空です。");
            None
        }
        Err(_) => {
            eprintln!("不正なJSON出力を受け取りました:");
            eprintln!("{raw}");
            None
        }
    }
}
