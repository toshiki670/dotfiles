//! `dotfiles` 本体（core）。設定の管理・配置を担う。
//!
//! `--version` / `--help` はバージョンの source of truth（タグ `v{version}`）を担う。
//!
//! `apply` / `list` サブコマンドは dotfiles ネイティブ化（Epic #453）の一部：
//! ソース（`configs/` の実体）を二段構えで解決し（作業ツリー検出 → 埋め込み・[`source`]、§8）、
//! `manifest.toml` に従って配置する。配置は **2軸**
//! （生成方式 `kind`=copy/generate × 合成 `strategy`=concat/json-shallow）＋条件付き overlay
//! （`when` gate）で捉える（設計書 §5 / §5.5）。copy はツリー配置、generate / overlay 明示は
//! ファイル合成（[`crate::core::apply::compose`]）を通り、トップレベル `when`（`deps` / `os`）はユニット単位 gate（[`crate::core::apply::gate`]）。配置の直前に
//! `locals`（named value）を解決・注入する（解決＋注入の窓口 [`crate::core::locals::resolve`] / ストア [`crate::core::locals::store`] / 対話入力 [`crate::core::locals::prompt`]、§9）。
//! 配置後は `hooks`（onchange フック）をユニットのソースハッシュ変化時だけ実行する
//! （[`hooks`] / [`onchange`]、§13）。`apply` は配置＋フック、`list` は配置先一覧、`secret set` は
//! named value 設定、`profile` はマシンクラスの状態 gate 設定／表示（[`state`]、§10）、`color sample` は
//! ANSI 確認表（旧 `crates/color` を吸収、§10）、`doctor` は診断（雛形）。

// 共有核（葉。多くの群が片方向で依存する契約・基盤）。
mod discover; // §6.3 走査（apply / list / doctor 共有）
mod manifest; // §6 manifest.toml スキーマ
mod source; // §8 ソース二段構え
mod state; // §10 状態駆動 gate のスカラ状態ファイル（profile / 将来の theme）

// 配置エンジン（§5 / §5.5）。子モジュール（copy / compose / generate / strategy / gate）は apply.rs が束ねる。
mod apply;

// named value（§9）。子モジュール（store / resolve / inject / prompt）は locals.rs が束ねる。
mod locals;

// onchange フック（§13。2 ファイルなので直下据え置き）。
mod hooks;
mod onchange;

// 単独ビュー / コマンド。
mod color;
mod doctor;
mod list;
mod profile;
mod secret;

// CLI 定義とディスパッチ。`src/bin/dotfiles.rs` の数行シムから [`cli::run`] が呼ばれる。
pub mod cli;
