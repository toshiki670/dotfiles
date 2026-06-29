//! `gcm` のエントリポイント。ロジックは [`dotfiles::gcm`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::gcm::run()
}
