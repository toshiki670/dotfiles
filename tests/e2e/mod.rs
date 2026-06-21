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
//! - [`cli`]: `--help` / `--version` / 引数なし
//! - [`apply_copy`]: copy 層（S0/S1）— 実 config 配置・tilde 展開・再帰委譲・パーミッション
//! - [`list`]: 分散 manifest の集約・名前順・属性ラベル・ソース欠落
//! - [`generate`]: generate 層（S2/#456）— cmd 実行・deps gate・sibling 連結・list 表示
//! - [`overlay`]: 合成軸（S3/#471）— overlay/strategy/when/preserve と load 時検証群
//! - [`claude_settings`]: claude/settings 実 config（S3/#457）の結線確認
//! - [`locals`]: マシンローカル値（S4/#458）— ストア注入・非 TTY 警告・secret set・doctor

use assert_cmd::Command;

mod apply_copy;
mod claude_settings;
mod cli;
mod generate;
mod list;
mod locals;
mod overlay;

/// 実バイナリ `dotfiles` を起動する Command を返す（全テスト共通）。
pub(crate) fn dotfiles() -> Command {
    Command::cargo_bin("dotfiles").unwrap()
}

/// PATH に置く実行可能スタブを書き出す（固定テキストを stdout に出す）。
/// generate / overlay / claude_settings の各テストが共有する。
#[cfg(unix)]
pub(crate) fn write_stub(dir: &std::path::Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
