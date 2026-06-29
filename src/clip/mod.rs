//! `clip` — ファイルをクリップボードへコピーする統合コマンド（macOS）。
//!
//! 旧 `copy-obj`（バイナリ）と abbr `c-file` / `c-path` を統合したもの。サブコマンド
//! ごとに 1 ファイルへ分割している:
//!
//! - [`obj`]  — Finder 貼り付け用ファイルオブジェクト（osascript）
//! - [`text`] — ファイルの中身（pbcopy）
//! - [`path`] — 絶対パス（pbcopy ＋ stdout 表示）
//!
//! クリップボード書き込みの低レベル手段（pbcopy / osascript）と macOS ガード・パス
//! 解決は [`clipboard`] に集約する。CLI 定義とディスパッチは [`cli`]。

pub mod cli;

mod clipboard;
mod obj;
mod path;
mod text;
