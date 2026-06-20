//! `manifest.toml` のスキーマと読み込み。
//!
//! 設計書（docs/dotfiles-native-design.md §6.2 / §7）のスキーマのうち、現スライス（S1）
//! で必要な部分を解釈する: `dst`（必須）/ `kind`（省略時 copy）/ `private` / `executable`。
//! generate / merge / theme / deps / hooks / os / secrets は後続スライスで追加する。

use serde::Deserialize;
use std::path::Path;

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// 配置先（必須）。`~` は HOME に展開する。
    pub dst: String,
    /// 配置種別（省略時 = copy）。S1 は copy のみ対応。
    #[serde(default)]
    pub kind: Kind,
    /// 所有者のみアクセス可（chezmoi `private_` = 0600 相当）。省略時 false。
    #[serde(default)]
    pub private: bool,
    /// 実行ビットを付与（chezmoi `executable_` 相当。0644→0755 / 0600→0700）。省略時 false。
    #[serde(default)]
    pub executable: bool,
}

/// 配置種別。S1 では copy のみ。generate / merge は後続スライス（S2 / S3）。
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Copy,
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
