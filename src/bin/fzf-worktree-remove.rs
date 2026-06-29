//! `fzf-worktree-remove` のエントリポイント。ロジックは [`dotfiles::fzf_picker::fzf_worktree_remove`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::fzf_picker::fzf_worktree_remove::run()
}
