//! root `dotfiles`（core）bin の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! 各ファイルは実バイナリ `dotfiles` を起動し、サブコマンド／合成軸ごとの
//! 「ビルドしたバイナリの振る舞い」だけを検証する。内部の純ロジックのユニット
//! テストは各 lib モジュールの `#[cfg(test)]` 側にある。横断ヘルパー（下記）は
//! 各テストファイルから `crate::` で使い、各ファイル固有の fixture ビルダーは
//! その消費ファイル内に置く（`support.rs` は作らない）。
//!
//! # 検証内容（ファイル別）
//!
//! エンジンの契約テストは全て **架空 fixture の hermetic 群**（`foo` / `faketool` / `demo` /
//! 中立な argv）で書き、`configs/` の実体に手を伸ばさない・特定ツールを名指ししない（#488）。
//! 「出荷する実 configs が妥当に load/apply/list するか」は [`real_configs`] が **data-driven**
//! （`configs/` 配下の全ユニットを実行時走査）に確かめる。ツールが増減・改名しても両者は無変更で
//! 生き残る（エンジンとテストはツールのライフサイクルから独立）。
//!
//! - [`cli`]: `--help` / `--version` / 引数なし
//! - [`apply_copy`]: copy 層（S0/S1）— kind 既定・tilde 展開・再帰委譲・パーミッション（hermetic）
//! - [`list`]: 分散 manifest の名前順・属性ラベル・ソース欠落（hermetic）
//! - [`generate`]: generate 層（S2/#456）— cmd 実行・when.deps gate・sibling 連結・list 表示
//! - [`overlay`]: 合成軸（S3/#471）— overlay/strategy/when/preserve と load 時検証群
//! - [`secrets`]: マシンローカル値（S4/#458）— secret set / 注入 / doctor（hermetic）
//! - [`hooks`]: onchange フック（S5/#459）— 架空コマンドでエンジンの汎用実行を検証
//!   （ソースハッシュ skip/run・when.os gate・未インストール skip・非ゼロ終了エラー）
//! - [`real_configs`]: 出荷する実 configs の data-driven 検証 — 全ユニット走査で load/apply/list

use assert_cmd::Command;

mod apply_copy;
mod cli;
mod generate;
mod hooks;
mod list;
mod overlay;
mod real_configs;
mod secrets;

/// 実バイナリ `dotfiles` を起動する Command を返す（全テスト共通）。
pub(crate) fn dotfiles() -> Command {
    Command::cargo_bin("dotfiles").unwrap()
}

/// PATH に置く実行可能スタブを書き出す（固定テキストを stdout に出す）。
/// generate / overlay / hooks の各テストが共有する。
#[cfg(unix)]
pub(crate) fn write_stub(dir: &std::path::Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
