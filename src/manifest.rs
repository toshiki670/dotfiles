//! `manifest.toml` のスキーマと読み込み。
//!
//! 設計書（docs/dotfiles-native-design.md §5 / §5.5 / §6.2 / §7）の **2軸モデル**を解釈する:
//! - **生成方式 `kind`**（断片をどう実体化するか）= `copy` / `generate`（省略時 copy）。
//! - **合成 `strategy`**（複数の条件付き断片を1 dst=ファイルへどう重ねるか）= `concat` /
//!   `json-shallow`。`merge` は独立 kind ではなく合成軸の JSON 戦略（§5.5）。
//! - **条件付き overlay**（`[[overlay]]` ＋ `when`）= dst を「base ＋ gate された断片」の合成
//!   として組む。各 overlay は `src`（copy 断片）/ `cmd`（generate 断片）/ `preserve`（既存 dst
//!   を読む built-in overlay）のいずれか ＋ `when`（`dep` / `os`）。
//!
//! `deps` / `os` はユニット単位 gate（＝ ユニット全体に係る `when` の退化形, §5.5）。
//! theme / hooks / secrets は後続スライスで追加する。

use serde::Deserialize;
use std::path::Path;

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// 配置先（必須）。`~` は HOME に展開する。
    /// copy は実体を置くディレクトリ、generate / 合成は生成物を書き出すファイルパス。
    pub dst: String,
    /// 生成方式（省略時 = copy）。断片をどう実体化するか（copy / generate）。
    #[serde(default)]
    pub kind: Kind,
    /// 合成戦略（複数 overlay を1 dst=ファイルへ重ねるとき）。単一 overlay なら省略。
    /// generate の既定挙動（cmd 出力＋sibling 連結）は暗黙 `concat`。
    #[serde(default)]
    pub strategy: Option<Strategy>,
    /// 所有者のみアクセス可（chezmoi `private_` = 0600 相当）。省略時 false。
    #[serde(default)]
    pub private: bool,
    /// 実行ビットを付与（chezmoi `executable_` 相当。0644→0755 / 0600→0700）。省略時 false。
    #[serde(default)]
    pub executable: bool,
    /// generate のとき実行するコマンド（argv）。先頭が実行ファイル名、以降が引数。
    /// 標準出力を断片とする。copy では未使用。
    #[serde(default)]
    pub cmd: Vec<String>,
    /// 依存バイナリ（ユニット単位 gate, §7）。PATH に揃わないものがあればユニット全体を
    /// スキップする（＝ ユニット全体に係る `when.dep` の退化形）。
    #[serde(default)]
    pub deps: Vec<String>,
    /// OS 条件（ユニット単位 gate, §7）。chezmoi 互換表記（例 `darwin` / `linux`）。
    /// 不一致ならユニット全体をスキップする（＝ ユニット全体に係る `when.os` の退化形）。
    #[serde(default)]
    pub os: Option<String>,
    /// 合成 overlay（条件付き断片の配列, §5.5）。空 = 生成方式の既定挙動。
    /// 各 overlay は `src` / `cmd` / `preserve` のいずれか ＋ `when?` を持つ。
    #[serde(default)]
    pub overlay: Vec<Overlay>,
}

/// 生成方式（断片の実体化方法）。copy / generate。`merge` は kind ではなく `strategy`（§5.5）。
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Copy,
    Generate,
}

/// 合成戦略（複数断片を1 dst=ファイルへ重ねる方法, §5.5）。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Strategy {
    /// テキスト連結（後ろへ連結）。境目に改行を 1 つ補う。
    Concat,
    /// JSON のトップレベル shallow merge（後勝ち）。deep merge はしない。
    JsonShallow,
}

/// 1 つの overlay（条件付き断片, §5.5）。`when` を満たす時だけ合成に参加する。
/// 断片の実体化方法は `src`（copy）/ `cmd`（generate）/ `preserve`（既存 dst 読み）の択一。
#[derive(Debug, Deserialize)]
pub struct Overlay {
    /// copy 断片: 単位ディレクトリからの相対ファイル。内容をそのまま断片にする。
    #[serde(default)]
    pub src: Option<String>,
    /// generate 断片: 実行する argv。標準出力を断片にする。
    #[serde(default)]
    pub cmd: Vec<String>,
    /// 既存 dst から温存するトップレベルキー（built-in overlay の糖衣, §5.5）。
    /// `json-shallow` で常に最後に重なり、ローカル値を勝たせる。
    #[serde(default)]
    pub preserve: Vec<String>,
    /// 採用条件（省略 = 常時採用）。`dep` / `os` を AND で評価する。
    #[serde(default)]
    pub when: Option<When>,
}

/// overlay の採用条件（§5.5）。複数キーは AND（全て満たす時だけ採用）。
#[derive(Debug, Deserialize, Default)]
pub struct When {
    /// 依存バイナリが PATH にある時だけ採用（旧 `{{ if lookPath … }}`）。
    #[serde(default)]
    pub dep: Option<String>,
    /// OS 一致時だけ採用（旧 `{{ if eq .chezmoi.os … }}`）。chezmoi 互換表記。
    #[serde(default)]
    pub os: Option<String>,
}

impl Overlay {
    /// 既存 dst を読む built-in overlay（`preserve` だけを持つ）か。
    pub fn is_preserve(&self) -> bool {
        !self.preserve.is_empty()
    }
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
