//! gate の評価: ユニット単位 gate（`deps` / `os`）と overlay の `when`。
//!
//! 設計書 §5.5「評価順と不変条件」の gate 語彙を 1 か所に集約する。`deps` / `os` は
//! ユニット全体に係る gate（満たさなければユニットごと skip）、overlay の `when`（`dep` /
//! `os`）はその断片だけの採否で、**同じ評価規則**（PATH 探索・OS 正規化・複数キー AND）を
//! 共有する。ここはその共有ロジックと、PATH 上の実行ファイル探索（`which`）を持つ。

use crate::manifest::{Manifest, When};
use std::path::{Path, PathBuf};

/// 現在の OS を chezmoi 互換表記で返す（macOS は `darwin`）。
///
/// manifest の `os` / `when.os` は chezmoi の `.chezmoi.os` と同じ表記（`darwin` / `linux`）で
/// 書くため、Rust の `std::env::consts::OS`（macOS では `macos`）を `darwin` に正規化して比較する。
pub fn current_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    }
}

/// ユニット単位 gate（`deps` / `os`）を評価し、満たさないとき skip 理由を返す（満たせば None）。
///
/// 不変条件①（§5.5）の短絡判定に使う。`deps` は 1 つでも PATH に無ければ skip、`os` は
/// 指定があり現在 OS と一致しなければ skip。複数条件は AND（どれか 1 つでも欠ければ skip）。
pub fn unit_skip_reason(manifest: &Manifest) -> Option<String> {
    if let Some(missing) = first_missing_dep(&manifest.deps) {
        return Some(format!("依存 `{missing}` が PATH にない"));
    }
    if let Some(want) = &manifest.os {
        if want != current_os() {
            return Some(format!("OS `{want}` 不一致（現在 {}）", current_os()));
        }
    }
    None
}

/// overlay の `when` を満たすか（省略キーは真、複数キーは AND）。
///
/// `dep` は PATH に在れば真、`os` は現在 OS と一致すれば真。`when` 省略（None）は常時採用。
pub fn when_satisfied(when: &Option<When>) -> bool {
    let Some(when) = when else {
        return true;
    };
    if let Some(dep) = &when.dep {
        if which(dep).is_none() {
            return false;
        }
    }
    if let Some(os) = &when.os {
        if os != current_os() {
            return false;
        }
    }
    true
}

/// `deps` のうち最初に PATH 上で見つからないものを返す（全て揃えば None）。
fn first_missing_dep(deps: &[String]) -> Option<&str> {
    deps.iter()
        .map(String::as_str)
        .find(|dep| which(dep).is_none())
}

/// `name` の実行ファイルを `$PATH` から探す（簡易 which）。
pub fn which(name: &str) -> Option<PathBuf> {
    // パス区切りを含む名前は PATH 探索せず、そのまま実行ファイルとして扱う。
    if name.contains('/') {
        let p = PathBuf::from(name);
        return is_executable(&p).then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

/// 実ファイルかつ実行ビットが立っているか（Unix）。
#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// 非 Unix では実ファイルの存在のみで判定する。
#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_os_normalizes_macos_to_darwin() {
        // ビルドした OS に応じて chezmoi 互換表記を返す。
        let expected = if cfg!(target_os = "macos") {
            "darwin"
        } else {
            std::env::consts::OS
        };
        assert_eq!(current_os(), expected);
    }

    #[test]
    fn when_none_is_always_satisfied() {
        assert!(when_satisfied(&None));
    }

    #[test]
    fn when_os_matches_current_only() {
        let hit = When {
            dep: None,
            os: Some(current_os().to_string()),
        };
        assert!(when_satisfied(&Some(hit)));

        let miss = When {
            dep: None,
            os: Some("nonsuch-os".to_string()),
        };
        assert!(!when_satisfied(&Some(miss)));
    }

    #[cfg(unix)]
    #[test]
    fn when_dep_checks_path_via_absolute_executable() {
        // パス区切りを含む名前は PATH 探索せず直接判定する（/bin/sh は実行可能）。
        let present = When {
            dep: Some("/bin/sh".to_string()),
            os: None,
        };
        assert!(when_satisfied(&Some(present)));

        let absent = When {
            dep: Some("/nonexistent/itic-bin".to_string()),
            os: None,
        };
        assert!(!when_satisfied(&Some(absent)));
    }

    #[cfg(unix)]
    #[test]
    fn when_is_and_of_keys() {
        // dep は満たすが os が外れる → 全体は false（AND）。
        let mixed = When {
            dep: Some("/bin/sh".to_string()),
            os: Some("nonsuch-os".to_string()),
        };
        assert!(!when_satisfied(&Some(mixed)));
    }
}
