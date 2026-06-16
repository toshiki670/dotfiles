//! 展開: cdabbr の省略パスをファイルシステム上の実ディレクトリへ再帰展開する。
//!
//! [`crate::parse::parse_abbr_path`] が出したセグメント列を、各段で前方一致する
//! サブディレクトリへ降りながら辿る（`read_dir` を伴う IO だが、結果が決定的になる
//! よう名前順に並べる）。

use std::path::{Path, PathBuf};

/// `base` から、各セグメントを「前方一致するサブディレクトリ名」として 1 段ずつ
/// 下りていき、辿り着いたディレクトリ群を返す（旧 `_cdabbr_expand_recursive`）。
/// セグメントが空なら `base` 自身を返す。順序は名前順で決定的にする。
pub fn expand_abbreviated(base: &Path, segments: &[String]) -> Vec<PathBuf> {
    let Some((seg, rest)) = segments.split_first() else {
        return vec![base.to_path_buf()];
    };

    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    dirs.sort();

    let mut out = Vec::new();
    for dir in dirs {
        if let Some(name) = dir.file_name().and_then(|n| n.to_str())
            && name.starts_with(seg.as_str())
        {
            out.extend(expand_abbreviated(&dir, rest));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descends_by_prefix() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path();
        std::fs::create_dir_all(base.join("dev/project")).unwrap();
        std::fs::create_dir_all(base.join("documents/photos")).unwrap();
        std::fs::create_dir_all(base.join("downloads")).unwrap();

        // "d" は dev/documents/downloads に一致、その下で "p" 始まりへ降りる。
        let segs = vec!["d".to_string(), "p".to_string()];
        let mut got = expand_abbreviated(base, &segs);
        got.sort();
        assert_eq!(
            got,
            vec![base.join("dev/project"), base.join("documents/photos")]
        );
    }

    #[test]
    fn empty_segments_returns_base() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(
            expand_abbreviated(tmp.path(), &[]),
            vec![tmp.path().to_path_buf()]
        );
    }

    #[test]
    fn no_match_is_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("alpha")).unwrap();
        assert!(expand_abbreviated(tmp.path(), &["z".to_string()]).is_empty());
    }
}
