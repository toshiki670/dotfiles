//! `fzf-picker`: fzf 系ピッカーをまとめた bin。共有ロジックと各サブコマンドの `run()`
//! をまとめる。
//!
//! # 設計方針
//!
//! - **役割ごとに 1 モジュール**。各ファイルは単一の責務だけを持つ（[`worktree`] /
//!   [`gh`] / [`parse`] / [`mod@format`] / [`launch`]。詳細は各モジュールの doc を参照）。
//! - **純ロジックと IO を分離**。パース・成形は純関数にしてユニットテストを同居させ、
//!   git/fzf を叩く IO 層（[`worktree::list_worktrees`] / [`launch`]）はバイナリの
//!   E2E（`tests/fzf_picker/`）で検証する。
//! - 各サブコマンド（`cdabbr` / `fzf-gh` / `fzf-ghq-cd` / `fzf-worktree-remove`）の
//!   `run()` は [`cli`] からのみ呼ばれる。この `main()` が [`cli::run`] を呼ぶ入口。

use std::process::ExitCode;

mod format;
mod gh;
mod launch;
mod parse;
mod worktree;

mod cdabbr;
mod cli;
mod fzf_gh;
mod fzf_ghq_cd;
mod fzf_worktree_remove;

fn main() -> ExitCode {
    cli::run()
}
