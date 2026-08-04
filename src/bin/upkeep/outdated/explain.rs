//! `--explain` の解決フロー: 上流の特定 → リリースノート取得 → 要約。
//!
//! 上流の特定はパッケージマネージャ自身のメタデータだけを使う（[`super::brew`] /
//! [`super::mise`] / [`super::cargo`] の `upstream`）。推測や検索はしない。

use super::package::{OutdatedPackage, Source};
use super::release::{self, Coverage};

/// 1 パッケージについて `--explain` を試みた結果。
pub enum Explanation {
    /// リリースノート本文を機械的に解決できなかった。`reference_url` は、それでも
    /// 利用者が一次情報へ辿れるようメタデータから引けた URL。
    Unavailable { reference_url: Option<String> },
    /// リリースノートは取得できたが claude による要約に失敗した。`reason` は失敗の内訳。
    GenerationFailed { reason: String },
    /// 要約成功。`source_urls` は要約に含めたリリースのページ（誤りが疑わしいときに
    /// ユーザーが自分で一次情報を確認できるようにするため）で、要約した順に並ぶ。
    Summary {
        text: String,
        source_urls: Vec<String>,
        coverage: Coverage,
    },
}

/// パッケージのリリースノートを解決し、取得できれば claude で要約する。
pub fn resolve(pkg: &OutdatedPackage) -> Explanation {
    let upstream = match pkg.source {
        Source::Brew => super::brew::upstream(&pkg.name),
        Source::Mise => super::mise::upstream(&pkg.name),
        Source::Cargo => super::cargo::upstream(&pkg.name),
    };

    let span = upstream
        .repo
        .as_ref()
        .and_then(|repo| release::fetch(repo, &pkg.name, &pkg.current, &pkg.latest));
    let Some(span) = span else {
        return Explanation::Unavailable {
            reference_url: upstream.reference_url(),
        };
    };

    match super::claude::summarize(&span.notes) {
        Ok(text) => Explanation::Summary {
            text,
            source_urls: span.notes.iter().map(|note| note.url.clone()).collect(),
            coverage: span.coverage,
        },
        Err(reason) => Explanation::GenerationFailed { reason },
    }
}
