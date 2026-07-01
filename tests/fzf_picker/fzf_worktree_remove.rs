//! `fzf-worktree-remove` の E2E（実バイナリ + 実 git worktree で検証）。
//!
//! 検証: 非 git repo で失敗、削除候補なしで `No worktrees to
//! delete`、確認 `y` で worktree 削除、確認 `n` で残置、fzf キャンセルで残置、削除対象
//! の内側から実行したときに退避先（メイン worktree）パスを stdout 出力。加えて、fish の
//! キーバインド経由と同じ raw tty 上で `y` 1 キーだけで確定できること（pty で再現）。

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

use crate::{EMPTY_PATH, FZF_STUB, git, path_with, write_exec};

fn fzf_worktree_remove() -> Command {
    Command::cargo_bin("fzf-worktree-remove").unwrap()
}

#[test]
fn outside_git_repo_fails() {
    let dir = TempDir::new().unwrap();
    fzf_worktree_remove()
        .current_dir(dir.path())
        .env("PATH", EMPTY_PATH) // git も見えない環境
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in a git repository"));
}

/// リンク worktree を 1 本持つ実 git repo と fzf スタブを用意する。
struct RemoveFixture {
    _root: TempDir,
    repo: PathBuf,
    worktree: PathBuf,
    bin: PathBuf,
    dump: PathBuf,
}

fn remove_fixture() -> RemoveFixture {
    let root = TempDir::new().unwrap();
    let repo = root.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(
        &repo,
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=test",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ],
    );
    let worktree = root.path().join("wt-feature");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature/x",
            worktree.to_str().unwrap(),
        ],
    );

    let bin = root.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let dump = bin.join("fzf-input.txt");
    write_exec(&bin, "fzf", FZF_STUB);
    RemoveFixture {
        _root: root,
        repo,
        worktree,
        bin,
        dump,
    }
}

#[test]
fn reports_when_no_linked_worktrees() {
    // メインのみの repo（リンク worktree を消しておく）。
    let fx = remove_fixture();
    git(
        &fx.repo,
        &["worktree", "remove", fx.worktree.to_str().unwrap()],
    );

    fzf_worktree_remove()
        .current_dir(&fx.repo)
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .assert()
        .success()
        .stdout("")
        .stderr(predicate::str::contains("No worktrees to delete"));
}

#[test]
fn confirm_yes_deletes_worktree() {
    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());

    fzf_worktree_remove()
        .current_dir(&fx.repo) // メイン側（対象の内側ではない）→ cd 不要
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout("") // 内側でないので退避パスは出さない
        .stderr(predicate::str::contains("削除しました"));

    assert!(!fx.worktree.exists(), "worktree dir should be gone");
}

/// fish のキーバインド経由と同じ状況を **pty で再現**する回帰テスト。
///
/// キーバインド実行中は端末が raw モード（非 canonical・echo なし）のまま。fzf 終了後も
/// その状態が戻るため、canonical 前提の `read_line` は Enter(`\n`) を待ってハングし、
/// `[y/N]` が無反応になる（本バグ）。ここでは stdin を raw な pty slave にして、`\n` を一切
/// 送らず `y` だけを流す。cbreak 化された confirm は 1 キーで確定して削除するが、旧
/// `read_line` 実装は `\n` を待ち続けてタイムアウトする（＝このテストが RED になる）。
///
/// パイプ stdin（他テストの `write_stdin("y\n")`）では raw tty を経ないため本バグを捕らえ
/// られない。実端末状態の再現が要る。
#[cfg(unix)]
#[test]
fn confirm_yes_over_raw_tty_deletes_worktree() {
    use std::process::{Command as StdCommand, Stdio};
    use std::time::{Duration, Instant};

    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());
    let (master, slave) = open_raw_pty();

    let exe = assert_cmd::cargo::cargo_bin("fzf-worktree-remove");
    let mut child = StdCommand::new(exe)
        .current_dir(&fx.repo) // メイン側（対象の内側ではない）→ cd 不要
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .stdin(unsafe { <Stdio as std::os::unix::io::FromRawFd>::from_raw_fd(slave) }) // 子の stdin = raw tty
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn fzf-worktree-remove");

    // 端末エミュレータのように master を読み捨て（プロンプト等の出力をドレイン）しつつ、`y` を
    // 送る（Enter は決して送らない）。master をドレインしないと confirm 側の cbreak 設定
    // （tcsetattr TCSAFLUSH）が出力転送待ちでブロックする。cbreak なら最初の `y` で確定する
    // が、旧 read_line 実装は `\n` を待ち続けるので下のタイムアウトで RED になる。
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut scratch = [0u8; 256];
    let status = loop {
        while unsafe { libc::read(master, scratch.as_mut_ptr().cast(), scratch.len()) } > 0 {}
        let _ = unsafe { libc::write(master, b"y".as_ptr().cast(), 1) };
        if let Some(status) = child.try_wait().expect("try_wait") {
            break Some(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    unsafe { libc::close(master) };

    assert!(
        status.is_some(),
        "raw tty で confirm がハングした（Y/N を受け付けない）: \
         read_line が raw モードでは決して届かない改行を待っている",
    );
    assert!(
        status.unwrap().success(),
        "process should exit successfully"
    );
    assert!(
        !fx.worktree.exists(),
        "worktree dir should be gone after 'y'"
    );
}

/// pty を開き、slave 側を raw モード（fish のキーバインド実行中と同じ端末状態）にして
/// `(master, slave)` の生 fd を返す。master は非ブロッキングにし、読み捨てドレインや `y`
/// 送出が詰まらないようにする（ハング再現時に送り続けても閉塞しない）。
#[cfg(unix)]
fn open_raw_pty() -> (i32, i32) {
    let (mut master, mut slave) = (0, 0);
    // SAFETY: 出力 fd を受け取るだけ。name/termp/winp は既定でよいので null。
    let rc = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, 0, "openpty failed");
    // SAFETY: term は tcgetattr で初期化してから使う。fd は openpty が返した slave。
    unsafe {
        let mut term: libc::termios = std::mem::zeroed();
        assert_eq!(libc::tcgetattr(slave, &mut term), 0, "tcgetattr");
        libc::cfmakeraw(&mut term);
        assert_eq!(libc::tcsetattr(slave, libc::TCSANOW, &term), 0, "tcsetattr");
        let flags = libc::fcntl(master, libc::F_GETFL);
        libc::fcntl(master, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
    (master, slave)
}

#[test]
fn confirm_no_keeps_worktree() {
    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());

    fzf_worktree_remove()
        .current_dir(&fx.repo)
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout("");

    assert!(fx.worktree.exists(), "worktree dir should remain");
}

#[test]
fn cancel_fzf_keeps_worktree() {
    let fx = remove_fixture();

    fzf_worktree_remove()
        .current_dir(&fx.repo)
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_EXIT", "1") // ESC 相当
        .assert()
        .success()
        .stdout("");

    assert!(fx.worktree.exists(), "worktree dir should remain");
    // 候補にはメインを含めず、リンク 1 本だけ並ぶ。
    let candidates = fs::read_to_string(&fx.dump).unwrap_or_default();
    assert!(
        candidates.lines().any(|l| l.starts_with("feature/x\t")),
        "linked worktree candidate missing:\n{candidates}"
    );
    assert_eq!(
        candidates.lines().count(),
        1,
        "only the linked worktree expected"
    );
}

#[test]
fn from_inside_target_prints_cd_path() {
    let fx = remove_fixture();
    let pick = format!("feature/x\t{}", fx.worktree.display());

    let assert = fzf_worktree_remove()
        .current_dir(&fx.worktree) // 削除対象の内側にいる
        .env("PATH", path_with(&fx.bin))
        .env("FZF_DUMP", &fx.dump)
        .env("FZF_PICK", &pick)
        .write_stdin("y\n")
        .assert()
        .success();

    // 退避先（メイン worktree）パスが stdout に出る。
    let out = assert.get_output();
    let printed = String::from_utf8_lossy(&out.stdout);
    let printed = printed.trim();
    assert!(!printed.is_empty(), "expected a cd target path on stdout");
    assert_eq!(
        Path::new(printed).canonicalize().ok(),
        fx.repo.canonicalize().ok(),
        "cd target should be the main worktree"
    );
    assert!(!fx.worktree.exists(), "worktree dir should be gone");
}
