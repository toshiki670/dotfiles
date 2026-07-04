//! リンク worktree を fzf で選び、確認のうえ削除する。
//!
//! 削除対象 worktree の **内側にいた場合だけ**、退避先（メイン worktree）の絶対パスを
//! stdout に出力する。fish shim はそのパスへ `cd` する（自分が消えるディレクトリから
//! 抜けるため）。それ以外の出力（プロンプト・結果メッセージ・git の出力）は stderr に
//! 出し、stdout の cd チャネルを汚さない。

use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use super::format::removal_lines;
use super::launch::run_fzf;
use super::worktree::list_worktrees;

/// fzf 各行のフィールド区切り（表示は 1 列目、削除対象パスは 2 列目）。
const TAB: char = '\t';

// fish shim（`_fzf_worktree_remove.fish`）から引数なしで呼ばれるだけなので、引数パース
// （clap）は持たない。

pub fn run() -> ExitCode {
    if !inside_work_tree() {
        eprintln!("not in a git repository");
        return ExitCode::FAILURE;
    }

    let worktrees = list_worktrees(Path::new("."));
    let main_path = worktrees.iter().find(|w| w.is_main).map(|w| w.path.clone());

    let lines = removal_lines(&worktrees);
    if lines.is_empty() {
        eprintln!("No worktrees to delete");
        return ExitCode::SUCCESS;
    }

    let selection = match run_fzf(
        &lines,
        &[
            "--preview",
            "git -C {2} log --oneline -20",
            "--preview-window",
            "right:60%",
        ],
    ) {
        Ok(Some(selection)) => selection,
        // ESC / Ctrl-C（fzf キャンセル）は何もせず正常終了。
        Ok(None) => return ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("fzf-worktree-remove: failed to run fzf.");
            return ExitCode::FAILURE;
        }
    };
    let Some(wpath) = selection.split(TAB).nth(1).map(str::to_string) else {
        return ExitCode::SUCCESS;
    };

    if !confirm("WT を削除しますか? [y/N] ") {
        return ExitCode::SUCCESS;
    }

    // 現在地が削除対象 worktree の内側なら、削除前に自プロセスを退避（git は現在の
    // worktree を消せない）し、親シェル用に退避先パスを stdout へ出す。削除の成否に
    // 関わらず（内側にいた以上）親シェルも退避させる。
    let mut cd_target: Option<String> = None;
    if let Some(main) = &main_path
        && is_inside(&wpath)
    {
        let _ = std::env::set_current_dir(main);
        cd_target = Some(main.clone());
    }

    if worktree_remove(&wpath, false) {
        eprintln!("削除しました: {wpath}");
    } else if confirm("強制削除しますか? [y/N] ") && worktree_remove(&wpath, true) {
        eprintln!("強制削除しました: {wpath}");
    }

    if let Some(target) = cd_target {
        println!("{target}");
    }
    ExitCode::SUCCESS
}

/// `git rev-parse --is-inside-work-tree` 相当。
fn inside_work_tree() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// 現在地（cwd）が `wpath` と同じか、その配下にあるか。シンボリックリンクは
/// 解決して比較する。
fn is_inside(wpath: &str) -> bool {
    let cur = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok());
    let target = Path::new(wpath).canonicalize().ok();
    match (cur, target) {
        (Some(cur), Some(target)) => cur.starts_with(&target),
        _ => false,
    }
}

/// `git worktree remove [--force] <wpath>` を実行し成功可否を返す。git の出力は
/// stderr へ流して stdout（cd チャネル）を汚さない。
fn worktree_remove(wpath: &str, force: bool) -> bool {
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "remove"]);
    if force {
        cmd.arg("--force");
    }
    match cmd.arg(wpath).stderr(Stdio::inherit()).output() {
        Ok(output) => {
            let _ = io::stderr().write_all(&output.stdout);
            output.status.success()
        }
        Err(_) => false,
    }
}

/// `[y/N]` プロンプトを stderr に出し、回答が yes（`y`/`Y`）かを返す。
/// EOF・その他は no。
fn confirm(prompt: &str) -> bool {
    eprint!("{prompt}");
    let _ = io::stderr().flush();
    asked_yes()
}

/// 端末なら cbreak で **1 キー確定**（Enter 不要）、非端末（テストのパイプ等）なら 1 行読む。
///
/// fish のキーバインド経由で呼ばれると、fzf 終了後の端末は fish の行編集が使う raw モードの
/// まま戻ってくる。raw では Enter が `\n` に変換されず echo も無いため、canonical 前提の
/// `read_line` は入力を受け付けられない（[y/N] が無反応になる）。cbreak（非 canonical + echo）で
/// 1 バイトだけ読めばこれを回避できる。余分な打鍵（`yes` と続けた等）は `CbreakGuard` の Drop
/// が TCSAFLUSH で捨てるので fish の行編集に漏れない。
#[cfg(unix)]
fn asked_yes() -> bool {
    use std::os::unix::io::AsRawFd;

    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return line_is_yes();
    }
    // 端末なのに cbreak にできないときは、raw のまま read_line に落とすと再びハングし得る。
    // 破壊的操作なので安全側に倒して No 扱い（＝削除しない）にする。
    let Some(_guard) = CbreakGuard::enable(stdin.as_raw_fd()) else {
        eprintln!("端末設定に失敗したため中止します");
        return false;
    };
    let mut buf = [0u8; 1];
    let read = stdin.lock().read(&mut buf).unwrap_or(0);
    let _ = writeln!(io::stderr()); // cbreak は Enter を伴わないので押下キーの後に改行を補う。
    read > 0 && matches!(buf[0], b'y' | b'Y')
}

#[cfg(not(unix))]
fn asked_yes() -> bool {
    line_is_yes()
}

/// 非端末（テストのパイプ等）: stdin から 1 行読み、先頭の非空白が `y`/`Y` かを返す。EOF は no。
fn line_is_yes() -> bool {
    let mut line = String::new();
    if io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
        return false;
    }
    matches!(line.trim_start().as_bytes().first(), Some(b'y' | b'Y'))
}

/// 端末を cbreak（非 canonical・echo あり・1 バイト単位）にし、`Drop` で元へ戻す RAII ガード。
/// raw モードのままでも Enter を待たず 1 キーで確定でき、エラーで早期 return しても復元される。
#[cfg(unix)]
struct CbreakGuard {
    fd: i32,
    original: libc::termios,
}

#[cfg(unix)]
impl CbreakGuard {
    /// `fd`（端末）の現在属性を保存し、cbreak にした属性を設定する。触れなければ `None`。
    fn enable(fd: i32) -> Option<Self> {
        // SAFETY: term は tcgetattr で完全に初期化してから使う。fd は stdin（端末）。
        unsafe {
            let mut term: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut term) != 0 {
                return None;
            }
            let original = term; // libc::termios は Copy。復元用に保存。
            term.c_lflag &= !libc::ICANON; // 行バッファリングを止め 1 バイトずつ届かせる
            term.c_lflag |= libc::ECHO; // 押下キーは見せる
            term.c_cc[libc::VMIN] = 1; // 最低 1 バイト届くまでブロック
            term.c_cc[libc::VTIME] = 0; // タイムアウト無し
            if libc::tcsetattr(fd, libc::TCSAFLUSH, &term) != 0 {
                return None;
            }
            Some(Self { fd, original })
        }
    }
}

#[cfg(unix)]
impl Drop for CbreakGuard {
    fn drop(&mut self) {
        // SAFETY: 取得済みの original を書き戻すのみ。失敗時も他に取れる手当ては無い。
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.original);
        }
    }
}
