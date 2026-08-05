//! 一覧表示のフォーマット（純粋関数、IO はここに持ち込まない）。

use super::explain::Explanation;
use super::package::OutdatedPackage;
use super::release::Coverage;

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

/// 文字列の表示幅（カラム数）。全角を2カラムとして数える。
fn columns(text: &str) -> usize {
    text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// 要約に含めたリリースを列挙する。
///
/// 件数を先に出すのは、1件のときに「1件だった」のか「1件しか見ていない」のかが URL の
/// 行数だけでは読み取れないため。2件目以降は1件目の左端へ揃える。
fn sources(urls: &[String]) -> String {
    let label = format!("出典 {}件: ", urls.len());
    let separator = format!("\n{INDENT}{}", " ".repeat(columns(&label)));
    format!("{label}{}", urls.join(&separator))
}

/// 範囲を確定できなかったことを添える。確定できたなら何も足さない（1件だけの要約が
/// 「1件しか無かった」のか「起点を見失った」のかを、この一行の有無で見分けられる）。
fn coverage_note(coverage: Coverage, current: &str) -> String {
    match coverage {
        Coverage::Complete => String::new(),
        Coverage::LatestOnly => {
            format!("\n{INDENT}※ {current} 以降の範囲を特定できず、最新1件のみ要約")
        }
    }
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
        Some(Explanation::Summary {
            text,
            source_urls,
            coverage,
        }) => {
            let text = align(text);
            let sources = sources(source_urls);
            let note = coverage_note(*coverage, &pkg.current);
            format!("{base}\n{INDENT}要約: {text}\n{INDENT}{sources}{note}")
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

    fn summary(text: &str, urls: &[&str], coverage: Coverage) -> Explanation {
        Explanation::Summary {
            text: text.to_string(),
            source_urls: urls.iter().map(|url| url.to_string()).collect(),
            coverage,
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
        let explanation = summary(
            "新機能追加",
            &["https://github.com/sharkdp/bat/releases/tag/v0.25.0"],
            Coverage::Complete,
        );
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約: 新機能追加\n    出典 1件: https://github.com/sharkdp/bat/releases/tag/v0.25.0"
        );
    }

    /// 複数リリースを要約したときは、含めた分だけ出典を並べる。
    #[test]
    fn lists_every_source_it_summarized() {
        let explanation = summary(
            "新機能追加",
            &[
                "https://github.com/o/r/releases/tag/v3",
                "https://github.com/o/r/releases/tag/v2",
                "https://github.com/o/r/releases/tag/v1",
            ],
            Coverage::Complete,
        );
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約: 新機能追加\n    出典 3件: https://github.com/o/r/releases/tag/v3\n              https://github.com/o/r/releases/tag/v2\n              https://github.com/o/r/releases/tag/v1"
        );
    }

    /// 件数が2桁になっても、2件目以降はラベルの右端へ揃い続ける。
    #[test]
    fn continuation_follows_the_label_width() {
        let urls: Vec<String> = (0..10).map(|index| format!("https://e/{index}")).collect();
        let refs: Vec<&str> = urls.iter().map(String::as_str).collect();
        let explanation = summary("新機能追加", &refs, Coverage::Complete);
        let got = format_package_line(&sample(), Some(&explanation));

        assert!(got.contains("出典 10件: https://e/0"), "label:\n{got}");
        // INDENT の4カラム ＋ `出典 10件: ` の11カラム。
        assert!(
            got.contains("\n               https://e/1"),
            "misaligned:\n{got}"
        );
    }

    /// 範囲を確定できなかったときだけ注記が付く。
    #[test]
    fn marks_an_unresolved_range() {
        let explanation = summary(
            "新機能追加",
            &["https://github.com/o/r/releases/tag/v0.25.0"],
            Coverage::LatestOnly,
        );
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約: 新機能追加\n    出典 1件: https://github.com/o/r/releases/tag/v0.25.0\n    ※ 0.24.0 以降の範囲を特定できず、最新1件のみ要約"
        );
    }

    /// 確定した範囲がたまたま1件でも注記は出さない。上の未確定と見分けられること。
    #[test]
    fn a_resolved_single_release_carries_no_note() {
        let explanation = summary(
            "新機能追加",
            &["https://github.com/o/r/releases/tag/v0.25.0"],
            Coverage::Complete,
        );
        let got = format_package_line(&sample(), Some(&explanation));
        assert!(!got.contains('※'), "unexpected note:\n{got}");
    }

    #[test]
    fn multiline_summary_is_aligned_and_leaves_blank_lines_bare() {
        let explanation = summary(
            "1行目\n\n2段落目",
            &["https://example.com/r/v1"],
            Coverage::Complete,
        );
        let got = format_package_line(&sample(), Some(&explanation));
        assert_eq!(
            got,
            "[brew] bat: 0.24.0 -> 0.25.0\n    要約: 1行目\n\n    2段落目\n    出典 1件: https://example.com/r/v1"
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
