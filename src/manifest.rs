//! `manifest.toml` の最小スキーマと読み込み（S0）。
//!
//! 設計書（docs/dotfiles-native-design.md §6.2）のスキーマのうち、S0 で必要な
//! 最小部分だけを解釈する。`dst`（必須）と `kind`（省略時 copy）のみ。
//! generate / merge / theme / deps / hooks / os / secrets は後続スライスで追加する。

use serde::Deserialize;
use std::path::Path;

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// 配置先（必須）。`~` は HOME に展開する。
    pub dst: String,
    /// 配置種別（省略時 = copy）。S0 は copy のみ対応。
    #[serde(default)]
    pub kind: Kind,
}

/// 配置種別。S0 では copy のみ。generate / merge は後続スライス（S2 / S3）。
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
}
