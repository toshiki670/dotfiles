//! `clip` のエントリポイント。CLI 定義とディスパッチは [`dotfiles::clip::cli`]。

use std::process::ExitCode;

fn main() -> ExitCode {
    dotfiles::clip::cli::run()
}
