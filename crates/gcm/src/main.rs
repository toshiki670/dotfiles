//! AI-powered git commit with Conventional Commits。旧 `gcm.fish` の移植。
//!
//! ステージ済み diff を `claude -p` に渡して Conventional Commits の JSON 提案を
//! 生成し、対話的に承認/修正して 1 つ以上のコミットを実行する。JSON 解析は
//! （fish 版の jq の代わりに）serde_json で行うため jq 依存は不要。

mod proposals;

use std::io::{self, Write};
use std::process::{Command, ExitCode, Stdio};

use proposals::{Commit, parse_proposals};

const SYSTEM_PROMPT: &str = "You are a git commit message generator using Conventional Commits.

Analyze the staged diff and split into the minimum number of semantically independent commits.
- Single concern → one entry
- Multiple independent concerns (e.g. feat + fix, or unrelated files) → multiple entries

Output a JSON array of commit objects:

[{\"message\": \"<type>[scope]: <description>\", \"files\": [\"path/to/file\"]}, ...]

Rules:
- type: feat, fix, docs, style, refactor, test, chore, perf, ci, build
- description: English, imperative mood (add, fix, update, remove, ...)
- Every staged file must appear in exactly one entry's files array";

fn main() -> ExitCode {
    let staged = staged_files();
    if staged.is_empty() {
        eprintln!("ステージされた変更がありません。");
        eprintln!("git add でファイルをステージしてから実行してください。");
        return ExitCode::FAILURE;
    }

    let staged_str = staged.join(", ");
    let diff = git_capture(&["diff", "--cached"]).unwrap_or_default();

    let mut conversation = format!(
        "Propose commits for the following staged changes.\n\nStaged files: {staged_str}\n\ngit diff --staged:\n{diff}"
    );

    // 初回提案は haiku で生成。
    eprintln!("コミットを生成中...");
    let mut proposals = match call_claude(&conversation, "haiku") {
        Some(p) => p,
        None => return ExitCode::FAILURE,
    };

    // 対話承認ループ（Ctrl-C はプロセスを終了 = 中止）。
    loop {
        display(&proposals);

        print!("追加指示 (Enter でコミット実行 / Ctrl-C で中止): ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => {
                // EOF (Ctrl-D) などは中止扱い。
                println!();
                println!("中止しました。");
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
        }

        let instruction = line.trim();
        if instruction.is_empty() {
            return execute(&proposals);
        }

        let proposal_json = serde_json::to_string(&proposals).unwrap_or_default();
        conversation = format!(
            "{conversation}\n\nPrevious proposal (JSON): {proposal_json}\nRevision instruction: {instruction}\nRevise accordingly. Output ONLY the JSON array."
        );

        // 修正はより高品質な sonnet で。
        eprintln!("修正中...");
        proposals = match call_claude(&conversation, "sonnet") {
            Some(p) => p,
            None => return ExitCode::FAILURE,
        };
    }
}

/// `git diff --cached --name-only` のステージ済みファイル一覧。
fn staged_files() -> Vec<String> {
    git_capture(&["diff", "--cached", "--name-only"])
        .map(|out| {
            out.lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// `git <args>` の stdout を取得する。
fn git_capture(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// claude を呼び出して commit 提案を得る。
fn call_claude(conversation: &str, model: &str) -> Option<Vec<Commit>> {
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

/// 提案コミットを表示する。
fn display(proposals: &[Commit]) {
    let count = proposals.len();
    println!();
    if count == 1 {
        println!("提案されたコミット:");
    } else {
        println!("提案されたコミット ({count} 件):");
    }
    for (i, commit) in proposals.iter().enumerate() {
        println!();
        // bold cyan
        println!("\x1b[1;36m  {}. {}\x1b[0m", i + 1, commit.message);
        for f in &commit.files {
            println!("     {f}");
        }
    }
    println!();
}

/// 提案コミットを実行する。
fn execute(proposals: &[Commit]) -> ExitCode {
    if proposals.len() == 1 {
        return run_status(&["commit", "-m", &proposals[0].message]);
    }

    // 複数コミット: 全 unstage してからエントリごとに stage + commit。
    let _ = git_status(&["restore", "--staged", "."]);
    for (i, commit) in proposals.iter().enumerate() {
        let mut add_args = vec!["add"];
        add_args.extend(commit.files.iter().map(String::as_str));
        let _ = git_status(&add_args);

        if !git_status(&["commit", "-m", &commit.message]) {
            eprintln!("コミット {i} 失敗。残りのファイルはステージされたままです。");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

/// `git <args>` を stdio 継承で実行し、終了コードを ExitCode で返す。
fn run_status(args: &[&str]) -> ExitCode {
    match Command::new("git").args(args).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(_) => ExitCode::FAILURE,
    }
}

/// `git <args>` を stdio 継承で実行し、成功したかを返す。
fn git_status(args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
