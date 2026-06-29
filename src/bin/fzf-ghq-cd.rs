//! `fzf-ghq-cd` のエントリポイント。ロジックは [`dotfiles::fzf_picker::fzf_ghq_cd`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::fzf_picker::fzf_ghq_cd::run()
}
