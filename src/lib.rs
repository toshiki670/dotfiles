//! 個人用 dotfiles を管理する `dotfiles` コマンドと、日常の Git/GitHub・クリップボード操作を
//! 担う小さな CLI 群のホーム。
//!
//! 中心は [`core`]（`dotfiles` コマンド本体）: リポジトリの `configs/` に置いた設定を、
//! 各設定単位の `manifest.toml` の宣言に従ってホームディレクトリへ配置する。
//! 配置モデルの用語と全体像は [`core`] のモジュール doc が入口。
//!
//! 配布物は root `dotfiles` パッケージの複数 bin（`src/bin/<name>.rs`）として並ぶ。各 bin は
//! 数行のシムで、対応する family モジュールの `run()` を呼ぶだけ。ロジックは family ごとの
//! module（[`core`] / [`clip`] / [`gcm`] / [`gh_clone`] / [`git_upstream`] / [`fzf_picker`] /
//! [`env_tools`] / [`workers`]）に置く。`cargo install --git <repo>` 一発で全 bin が入る。

// `deny(broken_intra_doc_links)`: doc コメントのリンク切れを CI の `cargo doc -p dotfiles` で
// 検出するガード。`allow(private_intra_doc_links)`: 既定では公開アイテムの doc から非公開
// アイテムへのリンクは警告になる（外部利用者はリンク先を読めない前提のため）。`dotfiles` は
// ライブラリとして公開しない内部 crate（bin だけを配布し、crates.io へは publish しない）で、
// 公開 rustdoc も `--document-private-items` 付きでビルドするため、module doc から非公開の
// 子モジュールへの構造ナビ用リンクは実害が無く許容する。
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
