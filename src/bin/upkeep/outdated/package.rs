//! 検出元パッケージマネージャと、アップデート可能な1パッケージの共通型。

/// 検出元パッケージマネージャ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Brew,
    Mise,
    Cargo,
}

impl Source {
    /// 表示用ラベル（`[brew]` 等の角括弧の中身）。
    pub fn label(self) -> &'static str {
        match self {
            Source::Brew => "brew",
            Source::Mise => "mise",
            Source::Cargo => "cargo",
        }
    }
}

/// アップデート可能な1パッケージ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutdatedPackage {
    pub source: Source,
    pub name: String,
    pub current: String,
    pub latest: String,
}
