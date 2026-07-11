//! root `dotfiles`（core）bin の E2E テスト（assert_cmd + predicates + rstest）。
//!
//! 各ファイルは実バイナリ `dotfiles` を起動し、サブコマンド／合成軸ごとの
//! 「ビルドしたバイナリの振る舞い」だけを検証する。内部の純ロジックのユニット
//! テストは各 lib モジュールの `#[cfg(test)]` 側にある。横断ヘルパー（下記）は
//! 各テストファイルから `crate::` で使い、各ファイル固有の fixture ビルダーは
//! その消費ファイル内に置く（`support.rs` は作らない）。
//!
//! # テスト方針
//!
//! テスト方針（エンジンはツールのライフサイクルから独立）は CONTRIBUTING.md の「テスト方針」節が
//! 唯一の出所。本スイートでの層の割り当てだけ示す:
//!
//! - **契約テスト**: hermetic な架空 fixture（`foo` / `faketool` / `demo` / `app` /
//!   中立な argv）で書く下記の大半。実 configs を名指ししない。
//! - **実 configs の妥当性確認**: [`real_configs`] **1 ファイル**が `configs/` を走査し
//!   data-driven に確かめる（ツール名をハードコードしない）。
//!
//! # 検証内容（ファイル別）
//!
//! 各ファイルの doc はその**ローカルな検証意図だけ**を述べる。
//!
//! - [`cli`]: `--help` / `--version` / 引数なし（契約）
//! - [`apply_copy`]: ツリー配置層（S0/S1）— `input = "."` 既定・tilde 展開・再帰委譲・パーミッション（契約）
//! - [`list`]: 分散 manifest の名前順・属性ラベル（契約）
//! - [`generate`]: cmd input/output（旧 generate 層。S2/#456/#560）— `input.cmd` 実行・when.deps gate・
//!   明示 append step・list 表示（契約）
//! - [`steps`]: `[[steps]]` パイプライン（#588 スライス1）— input/merge/output・format・optional・
//!   when と load 時検証群。`input.cmd` 断片＋plist ＋ `output.cmd` 反映の一気通貫（#531/#560）も含む（契約）
//! - [`local`]: マシンローカル値（S4/#458）— local set / 注入 / doctor（契約）
//! - [`placements`]: 期待配置集合・ユニット間 output 衝突検出（#593）— ツリー共有ディレクトリの
//!   非衝突・同一パス衝突・gate 評価前の宣言ベース検出を検証（契約）
//! - [`profile`]: マシンクラス状態 gate（#467）— profile set/show・`when.profile` の配置/skip・冪等（契約）
//! - [`hooks`]: onchange フック（S5/#459）— 架空コマンドでエンジンの汎用実行を検証（契約）
//! - [`color`]: `color sample`（S6/#460）— ANSI 確認表の見出し・エスケープシーケンス（契約）
//! - [`source`]: ソース二段構え（S8/#462）— 作業ツリー検出 / 埋め込み / `--source` の解決切替（契約＋実 configs）
//! - [`real_configs`]: 出荷 configs の妥当性（実 configs 層）— 全ユニット走査で load/apply/list

use assert_cmd::Command;

mod apply_copy;
mod cli;
mod color;
mod generate;
mod hooks;
mod list;
mod local;
mod placements;
mod profile;
mod real_configs;
mod source;
mod steps;

/// 実バイナリ `dotfiles` を起動する Command を返す（全テスト共通）。
pub(crate) fn dotfiles() -> Command {
    Command::cargo_bin("dotfiles").unwrap()
}

/// PATH に置く実行可能スタブを書き出す（固定テキストを stdout に出す）。
/// generate / steps / hooks の各テストが共有する。
#[cfg(unix)]
pub(crate) fn write_stub(dir: &std::path::Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
