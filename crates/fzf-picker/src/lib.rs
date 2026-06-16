//! fzf 系ピッカー（旧 fish B 群）の共有ロジック。
//!
//! # 設計方針
//!
//! - **役割ごとに 1 モジュール**。各ファイルは単一の責務だけを持つ:
//!   - [`worktree`] — git worktree ドメイン（型と一覧取得 IO）
//!   - [`parse`] — テキスト → 構造のパース（純ロジック）
//!   - [`format`] — 構造 → fzf 候補行の成形（純ロジック）
//!   - [`launch`] — 外部コマンド（fzf 等）の起動・存在確認（IO）
//! - **純ロジックと IO を分離**。パース・成形は純関数にしてユニットテストを同居させ、
//!   git/fzf を叩く IO 層（[`worktree::list_worktrees`] / [`launch`]）はバイナリの
//!   E2E（`tests/e2e/`）で検証する。
//! - **この lib はクレート内部限定**。未公開（crates.io 非掲載）なので他の配布クレート
//!   から path 依存してはいけない（release-plz `git_only` で `cargo package` が壊れる）。
//!   共有はクレート内の bin（`src/bin/*.rs`）からのみ行う。

pub mod format;
pub mod launch;
pub mod parse;
pub mod worktree;
