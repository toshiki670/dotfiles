//! `cdabbr` のエントリポイント。ロジックは [`dotfiles::fzf_picker::cdabbr`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::fzf_picker::cdabbr::run()
}
