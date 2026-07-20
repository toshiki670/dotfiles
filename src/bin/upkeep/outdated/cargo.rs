//! `cargo install-update --list` の検出。
//!
//! cargo は JSON 非対応なのでテーブル出力をパースする。ヘッダー行（先頭トークンが
//! `"Package"`）より前に `Polling registry '...'.......` という進捗行が混ざる。

use std::process::Command;

use super::package::{OutdatedPackage, Source};

/// 2 個以上の連続空白で分割する。
///
/// `str::split_whitespace` だと `Latest` 列内の単一空白トークン
/// （例: `"v0.21.14 (v0.22.0-beta.1 available)"`）まで割れてしまうため専用実装する。
fn split_on_multi_space(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut fields = Vec::new();
    let mut field_start = 0usize;
    let mut i = 0usize;

    while i < len {
        if bytes[i] == b' ' {
            let run_start = i;
            while i < len && bytes[i] == b' ' {
                i += 1;
            }
            if i - run_start >= 2 {
                fields.push(line[field_start..run_start].trim());
                field_start = i;
            }
        } else {
            i += 1;
        }
    }
    fields.push(line[field_start..].trim());

    fields.into_iter().filter(|s| !s.is_empty()).collect()
}

/// テーブルの1データ行から [`OutdatedPackage`] を作る。`Needs update` が `Yes` の行だけ返す。
fn parse_row(line: &str) -> Option<OutdatedPackage> {
    let fields = split_on_multi_space(line);
    if fields.len() != 4 || fields[3] != "Yes" {
        return None;
    }
    Some(OutdatedPackage {
        source: Source::Cargo,
        name: fields[0].to_string(),
        current: fields[1].to_string(),
        latest: fields[2].to_string(),
    })
}

/// `cargo install-update --list` の stdout から [`OutdatedPackage`] の一覧を作る。
///
/// ヘッダー行（先頭トークンが `"Package"`）が現れるまでの行（進捗表示等）は無視する。
fn parse(raw: &str) -> Vec<OutdatedPackage> {
    let mut in_table = false;
    let mut out = Vec::new();

    for line in raw.lines() {
        if !in_table {
            if line.split_whitespace().next() == Some("Package") {
                in_table = true;
            }
            continue;
        }
        if let Some(pkg) = parse_row(line) {
            out.push(pkg);
        }
    }

    out
}

/// `cargo install-update --list` を実行してアップデート可能なバイナリを集める。
///
/// 実行失敗は警告して空の結果を返す（呼び出し元は続行する）。
pub fn detect() -> Vec<OutdatedPackage> {
    let output = match Command::new("cargo")
        .args(["install-update", "--list"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => {
            eprintln!("⚠️  cargo install-update failed, skipping...");
            return Vec::new();
        }
    };

    parse(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = "    Polling registry 'https://index.crates.io/'.......................\n\nPackage              Installed  Latest                               Needs update\nsea-orm-cli          v1.1.20    v2.0.0                               Yes\ncargo-audit          v0.22.2    v0.22.2                              No\ntrunk                v0.21.14   v0.21.14 (v0.22.0-beta.1 available)  No\n";

    #[test]
    fn ignores_progress_line_before_header() {
        let got = parse(TABLE);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "sea-orm-cli");
    }

    #[test]
    fn excludes_needs_update_no() {
        let got = parse(TABLE);
        assert!(got.iter().all(|p| p.name != "cargo-audit"));
        assert!(got.iter().all(|p| p.name != "trunk"));
    }

    #[test]
    fn parenthetical_note_in_latest_does_not_break_parsing() {
        let raw = "Package  Installed  Latest                               Needs update\ntrunk    v0.21.14   v0.21.14 (v0.22.0-beta.1 available)  Yes\n";
        let got = parse(raw);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].latest, "v0.21.14 (v0.22.0-beta.1 available)");
    }

    #[test]
    fn no_rows_after_header_yields_empty() {
        let raw = "Package  Installed  Latest  Needs update\n";
        assert_eq!(parse(raw), Vec::new());
    }

    #[test]
    fn no_header_yields_empty() {
        assert_eq!(parse("some unrelated output\n"), Vec::new());
    }

    #[test]
    fn extracts_name_current_latest() {
        let got = parse(TABLE);
        assert_eq!(got[0].current, "v1.1.20");
        assert_eq!(got[0].latest, "v2.0.0");
    }
}
