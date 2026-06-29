//! `daily-check-worker` のエントリポイント。ロジックは [`dotfiles::workers::daily_check`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::workers::daily_check::run()
}
