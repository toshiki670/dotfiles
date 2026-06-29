//! `git-upstream` のエントリポイント。ロジックは [`dotfiles::git_upstream`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::git_upstream::run()
}
