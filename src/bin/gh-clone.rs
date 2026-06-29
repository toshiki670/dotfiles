//! `gh-clone` のエントリポイント。ロジックは [`dotfiles::gh_clone`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::gh_clone::run()
}
