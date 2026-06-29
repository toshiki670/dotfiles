//! toshiki670/dotfiles の配布コマンド群を 1 クレートに束ねる lib。
//!
//! 配布物は root `dotfiles` パッケージの複数 bin（`src/bin/<name>.rs`）として並ぶ。各 bin は
//! 数行のシムで、対応する family モジュールの `run()` を呼ぶだけ。ロジックは family ごとの
//! module（[`core`] / [`clip`] / [`gcm`] / [`gh_clone`] / [`git_upstream`] / [`fzf_picker`] /
//! [`env_tools`] / [`workers`]）に置く。`cargo install --git <repo>` 一発で全 bin が入る。
//!
//! intra-doc リンク腐敗の恒久ガード（CI の `cargo doc -p dotfiles` で検出）。配布用の
//! 内部 crate なので、module doc から非公開の同居アイテムへ張る構造ナビ用リンクは許容する。
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::private_intra_doc_links)]

pub mod clip;
pub mod core;
pub mod env_tools;
pub mod fzf_picker;
pub mod gcm;
pub mod gh_clone;
pub mod git_upstream;
pub mod workers;
