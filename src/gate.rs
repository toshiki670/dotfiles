//! gate の評価: トップレベル `when`（ユニットスコープ）と `[[overlay]]` の `when`（断片スコープ）。
//!
//! 設計書 §5.5「評価順と不変条件」の gate 語彙を 1 か所に集約する。gate 語彙は `when`
//! （`deps` 配列・AND / `os` スカラ）に一本化されており、**書く位置でスコープが決まる**:
//! トップレベルの `when` はユニット全体 gate（満たさなければユニットごと skip）、overlay の
//! `when` はその断片だけの採否。両者は **同じ評価規則**（[`when_unsatisfied_reason`]: PATH 探索・
//! OS 正規化・複数キー AND）を共有する。ここはその共有ロジックと、PATH 上の実行ファイル探索
//! （`which`）を持つ。

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

/// トップレベル `when`（ユニットスコープ gate）を評価し、満たさないとき skip 理由を返す
/// （満たせば None）。
///
/// 不変条件①（§5.5）の短絡判定に使う。`when` 省略のユニットは常時採用（None）。判定は
/// overlay と共有の [`when_unsatisfied_reason`] に委譲する。
pub fn unit_skip_reason(manifest: &Manifest) -> Option<String> {
    manifest.when.as_ref().and_then(when_unsatisfied_reason)
}

/// overlay の `when`（断片スコープ gate）を満たすか（省略は真）。
///
/// ユニット gate と同じ [`when_unsatisfied_reason`] を共有し、理由が無い（None）＝満たす。
pub fn when_satisfied(when: &Option<When>) -> bool {
    when.as_ref()
        .is_none_or(|when| when_unsatisfied_reason(when).is_none())
}

/// `when`（`deps` 配列・AND / `os` スカラ）を評価し、満たさないとき理由を返す（満たせば None）。
///
/// ユニットスコープ（[`unit_skip_reason`]）と断片スコープ（[`when_satisfied`]）が共有する
/// 唯一の評価本体。`deps` は 1 つでも PATH に無ければ不成立、`os` は指定があり現在 OS と
/// 一致しなければ不成立。複数キーは AND（どれか 1 つでも欠ければ不成立）。
fn when_unsatisfied_reason(when: &When) -> Option<String> {
    if let Some(missing) = first_missing_dep(&when.deps) {
        return Some(format!("依存 `{missing}` が PATH にない"));
    }
    if let Some(want) = &when.os
        && want != current_os()
    {
        return Some(format!("OS `{want}` 不一致（現在 {}）", current_os()));
    }
    None
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
            deps: vec![],
            os: Some(current_os().to_string()),
        };
        assert!(when_satisfied(&Some(hit)));

        let miss = When {
            deps: vec![],
            os: Some("nonsuch-os".to_string()),
        };
        assert!(!when_satisfied(&Some(miss)));
    }

    #[cfg(unix)]
    #[test]
    fn when_deps_check_path_via_absolute_executable() {
        // パス区切りを含む名前は PATH 探索せず直接判定する（/bin/sh は実行可能）。
        let present = When {
            deps: vec!["/bin/sh".to_string()],
            os: None,
        };
        assert!(when_satisfied(&Some(present)));

        let absent = When {
            deps: vec!["/nonexistent/itic-bin".to_string()],
            os: None,
        };
        assert!(!when_satisfied(&Some(absent)));
    }

    #[cfg(unix)]
    #[test]
    fn when_deps_are_and_of_array() {
        // deps は配列 AND: 1 つでも欠ければ不成立。
        let mixed = When {
            deps: vec!["/bin/sh".to_string(), "/nonexistent/itic-bin".to_string()],
            os: None,
        };
        assert!(!when_satisfied(&Some(mixed)));
    }

    #[cfg(unix)]
    #[test]
    fn when_is_and_of_keys() {
        // deps は満たすが os が外れる → 全体は false（AND）。
        let mixed = When {
            deps: vec!["/bin/sh".to_string()],
            os: Some("nonsuch-os".to_string()),
        };
        assert!(!when_satisfied(&Some(mixed)));
    }
}
