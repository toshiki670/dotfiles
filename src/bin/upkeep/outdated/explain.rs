//! `--explain` の解決フロー: 上流の特定 → リリースノート取得 → 要約。
//!
//! 上流の特定はパッケージマネージャ自身のメタデータだけを使う（[`super::brew`] /
//! [`super::mise`] / [`super::cargo`] の `upstream`）。推測や検索はしない。

use super::package::{OutdatedPackage, Source};
use super::release;

/// 1 パッケージについて `--explain` を試みた結果。
pub enum Explanation {
    /// リリースノート本文を機械的に解決できなかった。`reference_url` は、それでも
    /// 利用者が一次情報へ辿れるようメタデータから引けた URL。
    Unavailable { reference_url: Option<String> },
    /// リリースノートは取得できたが claude による要約に失敗した。
    GenerationFailed,
    /// 要約成功。`source_url` は要約元のリリースページ
    /// （誤りが疑わしいときにユーザーが自分で一次情報を確認できるようにするため）。
    Summary { text: String, source_url: String },
}

/// パッケージのリリースノートを解決し、取得できれば claude で要約する。
pub fn resolve(pkg: &OutdatedPackage) -> Explanation {
    let upstream = match pkg.source {
        Source::Brew => super::brew::upstream(&pkg.name),
        Source::Mise => super::mise::upstream(&pkg.name),
        Source::Cargo => super::cargo::upstream(&pkg.name),
    };

    let notes = upstream.repo.as_ref().and_then(release::fetch);
    let Some(notes) = notes else {
        return Explanation::Unavailable {
            reference_url: upstream.reference_url(),
        };
    };

    match super::claude::summarize(&notes.body) {
        Some(text) => Explanation::Summary {
            text,
            source_url: notes.url,
        },
        None => Explanation::GenerationFailed,
    }
}
