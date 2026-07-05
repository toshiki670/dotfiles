//! `clip` — ファイルをクリップボードへコピーする統合コマンド（macOS）。
//!
//! サブコマンドごとに 1 ファイルへ分割している（[`obj`] / [`text`] / [`path`]。詳細は
//! 各モジュールの doc を参照）。クリップボード書き込みの低レベル手段と macOS ガード・
//! パス解決は [`clipboard`] に集約する。CLI 定義とディスパッチは [`cli`]。

use std::process::ExitCode;

mod cli;
mod clipboard;
mod obj;
mod path;
mod text;

fn main() -> ExitCode {
    cli::run()
}
