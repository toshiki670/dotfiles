//! `git-background-fetch-worker` のエントリポイント。ロジックは [`dotfiles::workers::git_background_fetch`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::workers::git_background_fetch::run()
}
