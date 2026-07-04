//! `dotfiles` は `configs/` に置いた設定を、各設定単位の `manifest.toml` の宣言に従って
//! ホームディレクトリへ配置する（配置モデルは [`core`] を参照）。
//!
//! ほか、いくつかの小さな CLI も同じパッケージで配布する。
//!
//! `cargo install --git <repo>` で全コマンドが入る。

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
