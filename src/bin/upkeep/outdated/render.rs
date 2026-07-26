//! 一覧表示のフォーマット（純粋関数、IO はここに持ち込まない）。

use super::explain::Explanation;
use super::package::OutdatedPackage;

/// パッケージ行にぶら下がる解説の字下げ。
const INDENT: &str = "    ";

/// 要約本文の2行目以降を1行目と同じ深さへ揃える。カラム 0 に来るのをパッケージ行だけに
/// 保つため（空行には余白を残さない）。
fn align(text: &str) -> String {
    let mut lines = text.split('\n');
    let mut out = lines.next().unwrap_or_default().to_string();
    for line in lines {
        out.push('\n');
        if !line.trim().is_empty() {
            out.push_str(INDENT);
            out.push_str(line);
        }
    }
    out
}

/// `[source] name: current -> latest` の1行。`--explain` 時は解説を続く行に足す。
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
            let text = align(text);
            format!("{base}\n{INDENT}要約: {text}\n{INDENT}出典: {source_url}")
        }
        Some(Explanation::Unavailable { reference_url }) => match reference_url {
            Some(url) => format!("{base}\n{INDENT}変更内容不明\n{INDENT}参考: {url}"),
            None => format!("{base}\n{INDENT}変更内容不明"),
        },
        Some(Explanation::GenerationFailed { reason }) => {
            format!("{base}\n{INDENT}要約失敗: {reason}")
        }
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
    fn multiline_summary_is_aligned_and_leaves_blank_lines_bare() {
        let explanation = Explanation::Summary {
            text: "1行目\n\n2段落目".into(),
            source_url: "https://example.com/r/v1".into(),
        };
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約: 1行目\n\n    2段落目\n    出典: https://example.com/r/v1"
        );
    }

    #[test]
    fn with_unavailable_and_reference() {
        let explanation = Explanation::Unavailable {
            reference_url: Some("https://ghostty.org/".into()),
        };
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    変更内容不明\n    参考: https://ghostty.org/"
        );
    }

    #[test]
    fn with_unavailable_and_no_reference() {
        let explanation = Explanation::Unavailable {
            reference_url: None,
        };
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(got, "[brew] bat: 0.24.0 -> 0.25.0\n    変更内容不明");
    }

    #[test]
    fn with_generation_failed() {
        let explanation = Explanation::GenerationFailed {
            reason: "claude の出力が空でした".into(),
        };
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約失敗: claude の出力が空でした"
        );
    }
}
