//! fzf 系ピッカー。共有ロジックと各 bin の `run()` をまとめる。
//!
//! # 設計方針
//!
//! - **役割ごとに 1 モジュール**。各ファイルは単一の責務だけを持つ:
//!   - [`worktree`] — git worktree ドメイン（型と一覧取得 IO）
//!   - [`gh`] — gh Issue/PR ピッカーのドメイン（型・純ロジック・一覧取得 IO）
//!   - [`parse`] — テキスト → 構造のパース（純ロジック）
//!   - [`mod@format`] — 構造 → fzf 候補行の成形（純ロジック）
//!   - [`launch`] — 外部コマンド（fzf 等）の起動・存在確認（IO）
//! - **純ロジックと IO を分離**。パース・成形は純関数にしてユニットテストを同居させ、
//!   git/fzf を叩く IO 層（[`worktree::list_worktrees`] / [`launch`]）はバイナリの
//!   E2E（`tests/fzf_picker/`）で検証する。
//! - 各 bin（[`cdabbr`] / [`fzf_gh`] / [`fzf_ghq_cd`] / [`fzf_worktree_remove`]）は
//!   `run()` を公開し、`src/bin/<name>.rs` の数行シムから呼ばれる。

pub mod format;
pub mod gh;
pub mod launch;
pub mod parse;
pub mod worktree;

pub mod cdabbr;
pub mod fzf_gh;
pub mod fzf_ghq_cd;
pub mod fzf_worktree_remove;
