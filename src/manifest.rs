//! `manifest.toml` のスキーマと読み込み。
//!
//! 設計書（docs/dotfiles-native-design.md §6.2 / §7）のスキーマのうち、現スライスまでで
//! 必要な部分を解釈する: `dst`（必須）/ `kind`（省略時 copy）/ `private` / `executable`、
//! および generate 用の `cmd` / `deps`（S2）。merge / theme / hooks / os / secrets は
//! 後続スライスで追加する。

use serde::Deserialize;
use std::path::Path;

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// 配置先（必須）。`~` は HOME に展開する。
    /// copy は実体を置くディレクトリ、generate は生成物を書き出すファイルパス。
    pub dst: String,
    /// 配置種別（省略時 = copy）。S2 までで copy / generate に対応。
    #[serde(default)]
    pub kind: Kind,
    /// 所有者のみアクセス可（chezmoi `private_` = 0600 相当）。省略時 false。
    #[serde(default)]
    pub private: bool,
    /// 実行ビットを付与（chezmoi `executable_` 相当。0644→0755 / 0600→0700）。省略時 false。
    #[serde(default)]
    pub executable: bool,
    /// generate のとき実行するコマンド（argv）。先頭が実行ファイル名、以降が引数。
    /// 標準出力を `dst` のファイルへ書き出す。copy では未使用。
    #[serde(default)]
    pub cmd: Vec<String>,
    /// 依存バイナリ（gate, §7）。PATH に揃わないものがあれば配置/生成をスキップする。
    #[serde(default)]
    pub deps: Vec<String>,
}

/// 配置種別。S2 までで copy / generate。merge は後続スライス（S3）。
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Copy,
    Generate,
}

impl Manifest {
    /// `manifest.toml` を読み込んでパースする。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("{}: パース失敗: {e}", path.display()))
    }

    /// この単位の配置ファイルへ与える Unix パーミッション（8 進）。
    ///
    /// base は `private` で決まる（0600 / 0644）。`executable` のとき、read ビットが
    /// 立っている桁へ execute ビットを足す（0644→0755 / 0600→0700）。chezmoi の
    /// `private_` / `executable_` 属性と同じ合成規則。
    #[cfg(unix)]
    pub fn mode(&self) -> u32 {
        let base: u32 = if self.private { 0o600 } else { 0o644 };
        if self.executable {
            base | ((base & 0o444) >> 2)
        } else {
            base
        }
    }
}
