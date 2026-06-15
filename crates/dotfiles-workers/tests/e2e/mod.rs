//! `dotfiles-workers`（daily-check-worker / git-background-fetch-worker）の
//! 最小 E2E。worker は引数を取らず env 駆動のバックグラウンドプロセスのため、必要な外部
//! コマンド（date / brew / mise / git）が不在でも無害に成功終了することを確認する。

use assert_cmd::Command;

const EMPTY_PATH: &str = "/nonexistent-dotfiles-e2e";

#[test]
fn daily_check_worker_exits_success_without_tools() {
    Command::cargo_bin("daily-check-worker")
        .unwrap()
        .env("PATH", EMPTY_PATH)
        .assert()
        .success();
}

#[test]
fn git_background_fetch_worker_exits_success_outside_repo() {
    Command::cargo_bin("git-background-fetch-worker")
        .unwrap()
        .env("PATH", EMPTY_PATH)
        .assert()
        .success();
}
