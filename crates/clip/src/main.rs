//! `clip` のエントリポイント。CLI 定義とディスパッチは [`clip::cli`] にある。

use std::process::ExitCode;

fn main() -> ExitCode {
    clip::cli::run()
}
