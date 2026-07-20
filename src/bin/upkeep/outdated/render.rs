//! 一覧表示のフォーマット（純粋関数、IO はここに持ち込まない）。

use super::explain::Explanation;
use super::package::OutdatedPackage;

/// `[source] name: current -> latest` の1行。`--explain` 時は解説を2行目に足す。
pub fn format_package_line(pkg: &OutdatedPackage, explanation: Option<&Explanation>) -> String {
    let base = format!(
        "[{}] {}: {} -> {}",
        pkg.source.label(),
        pkg.name,
        pkg.current,
        pkg.latest
    );

    match explanation {
        None => base,
        Some(Explanation::Summary { text, source_url }) => {
            format!("{base}\n    要約: {text}\n    出典: {source_url}")
        }
        Some(Explanation::Unavailable) => format!("{base}\n    変更内容不明"),
        Some(Explanation::GenerationFailed) => format!("{base}\n    要約失敗（claude 生成エラー）"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::package::Source;
    use super::*;

    fn sample() -> OutdatedPackage {
        OutdatedPackage {
            source: Source::Brew,
            name: "bat".to_string(),
            current: "0.24.0".to_string(),
            latest: "0.25.0".to_string(),
        }
    }

    #[test]
    fn without_explanation() {
        assert_eq!(
            format_package_line(&sample(), None),
            "[brew] bat: 0.24.0 -> 0.25.0"
        );
    }

    #[test]
    fn with_summary() {
        let explanation = Explanation::Summary {
            text: "新機能追加".into(),
            source_url: "https://github.com/sharkdp/bat/releases/tag/v0.25.0".into(),
        };
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約: 新機能追加\n    出典: https://github.com/sharkdp/bat/releases/tag/v0.25.0"
        );
    }

    #[test]
    fn with_unavailable() {
        let got = format_package_line(&sample(), Some(&Explanation::Unavailable));
        assert_eq!(got, "[brew] bat: 0.24.0 -> 0.25.0\n    変更内容不明");
    }

    #[test]
    fn with_generation_failed() {
        let got = format_package_line(&sample(), Some(&Explanation::GenerationFailed));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約失敗（claude 生成エラー）"
        );
    }
}
