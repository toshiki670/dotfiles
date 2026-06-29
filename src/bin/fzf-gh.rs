//! `fzf-gh` のエントリポイント。ロジックは [`dotfiles::fzf_picker::fzf_gh`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::fzf_picker::fzf_gh::run()
}
