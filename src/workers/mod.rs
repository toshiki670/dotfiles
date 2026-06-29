//! fish の conf.d フックからバックグラウンドで起動される worker 群。
//!
//! 引数は取らず環境変数で駆動する（clap は持たない）。各 worker は `run()` を公開し、
//! `src/bin/<name>-worker.rs` の数行シムから呼ばれる。
//!
//! - [`daily_check`] — 1 日 1 回 brew/mise の outdated を集計して結果ファイルへ書く
//! - [`git_background_fetch`] — スロットル付き `git fetch` をバックグラウンドで実行する

pub mod daily_check;
pub mod git_background_fetch;
