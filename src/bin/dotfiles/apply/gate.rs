//! gate の評価: トップレベル `when`（ユニットスコープ）と step の `when`（step スコープ）。
//!
//! gate 語彙を 1 か所に集約する。gate 語彙は `when`
//! （`deps` 配列・AND / `os` スカラ / `profile` スカラ）に一本化されており、**書く位置でスコープが
//! 決まる**: トップレベルの `when` はユニット全体 gate（満たさなければユニットごと skip）、step の
//! `when` はその step だけの採否。両者は **同じ評価規則**（[`when_unsatisfied_reason`]: PATH 探索・
//! OS 正規化・profile 状態一致・複数キー AND）を共有する。ここはその共有ロジックと、PATH 上の
//! 実行ファイル探索（`which`）を持つ。
//!
//! `deps`/`os` は環境（PATH・OS）からその場で判る条件だが、`profile` は user が選んで
//! おく状態（[`crate::state`]）を読む。状態は apply 開始時に 1 回 [`GateState`] へ解決し、全ユニット
//! ・全 step の評価で共有する（評価ごとにファイルを読み直さない）。`theme`（color スライス）も
//! 同じ snapshot にフィールドを足して同じ機構を使い回す想定（状態駆動 gate 族）。

use crate::manifest::{Manifest, Os, When};
use crate::state;
use std::path::Path;

/// apply 開始時に 1 回解決した、状態駆動 gate（`profile` / 将来の `theme`）の現在状態スナップショット。
///
/// `profile` は現状唯一の状態 gate。`None` ＝ 未設定で、profile gate は「private ではない」既定として
/// 解釈する（安全側 opt-in）。`deps`/`os` は環境から都度判るため snapshot に持たない。
/// 全ユニット・全 step の評価で使い回す（apply で 1 回だけ解決）。
pub struct GateState {
    profile: Option<String>,
}

impl GateState {
    /// 状態ファイル（[`crate::state`]）から現在状態を読む。apply で 1 回だけ呼ぶ。
    pub fn load(home: &Path) -> Result<Self, String> {
        Ok(Self {
            profile: state::read(home, state::PROFILE)?,
        })
    }

    /// テスト用に状態を直接組み立てる（profile のみ）。
    #[cfg(test)]
    fn with_profile(profile: Option<&str>) -> Self {
        Self {
            profile: profile.map(str::to_string),
        }
    }

    /// 現在の profile 状態（未設定は None）。gate 評価を経由せず生の値が要る呼び出し元向け
    /// （[`crate::doctor`] の宣言値集合との突合）。
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }
}

/// 現在の OS を manifest の語彙（[`Os`]）で返す。
///
/// `darwin` / `linux` 以外のビルドターゲットは None ＝ どの `os` gate とも一致しない（`os` を
/// 書いたユニット / step は skip）。
pub fn current_os() -> Option<Os> {
    match std::env::consts::OS {
        "macos" => Some(Os::Darwin),
        "linux" => Some(Os::Linux),
        _ => None,
    }
}

/// トップレベル `when`（ユニットスコープ gate）を評価し、満たさないとき skip 理由を返す
/// （満たせば None）。
///
/// 不変条件①の短絡判定に使う。`when` 省略のユニットは常時採用（None）。判定は
/// step と共有の [`when_unsatisfied_reason`] に委譲する。
pub fn unit_skip_reason(manifest: &Manifest, state: &GateState) -> Option<String> {
    manifest
        .when
        .as_ref()
        .and_then(|when| when_unsatisfied_reason(when, state))
}

/// step の `when`（step スコープ gate）を満たすか（省略は真）。
///
/// ユニット gate と同じ [`when_unsatisfied_reason`] を共有し、理由が無い（None）＝満たす。
pub fn when_satisfied(when: &Option<When>, state: &GateState) -> bool {
    when.as_ref()
        .is_none_or(|when| when_unsatisfied_reason(when, state).is_none())
}

/// `when`（`deps` 配列・AND / `os` スカラ / `profile` スカラ）を評価し、満たさないとき理由を返す
/// （満たせば None）。
///
/// ユニットスコープ（[`unit_skip_reason`]）と step スコープ（[`when_satisfied`]）が共有する
/// 唯一の評価本体。`deps` は 1 つでも PATH に無ければ不成立、`os` は指定があり現在 OS と
/// 一致しなければ不成立、`profile` は指定があり現在の profile 状態（`state`）と一致しなければ
/// 不成立（未設定 ＝ not-private 既定なので指定があれば不成立）。複数キーは AND（どれか 1 つでも
/// 欠ければ不成立）。
fn when_unsatisfied_reason(when: &When, state: &GateState) -> Option<String> {
    if let Some(missing) = first_missing_dep(&when.deps) {
        return Some(format!("依存 `{missing}` が PATH にない"));
    }
    if let Some(want) = when.os
        && Some(want) != current_os()
    {
        // 未対応 OS（None）は生の OS 名で出す ― 不一致の相手が読めないと直せない。
        let current =
            current_os().map_or_else(|| std::env::consts::OS.to_string(), |os| os.to_string());
        return Some(format!("OS `{want}` 不一致（現在 {current}）"));
    }
    if let Some(want) = &when.profile
        && state.profile.as_deref() != Some(want.as_str())
    {
        let current = state.profile.as_deref().unwrap_or("未設定");
        return Some(format!("profile `{want}` 不一致（現在 {current}）"));
    }
    None
}

/// `deps` のうち最初に PATH 上で見つからないものを返す（全て揃えば None）。
fn first_missing_dep(deps: &[String]) -> Option<&str> {
    deps.iter()
        .map(String::as_str)
        .find(|dep| which::which(dep).is_err())
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn current_os_maps_build_target_to_manifest_vocabulary() {
        let expected = if cfg!(target_os = "macos") {
            Some(Os::Darwin)
        } else if cfg!(target_os = "linux") {
            Some(Os::Linux)
        } else {
            None
        };
        assert_eq!(current_os(), expected);
    }

    /// profile 状態を持たない既定スナップショット（deps/os テストは profile に無依存）。
    fn no_profile() -> GateState {
        GateState::with_profile(None)
    }

    #[test]
    fn when_none_is_always_satisfied() {
        assert!(when_satisfied(&None, &no_profile()));
    }

    #[test]
    fn when_os_matches_current_only() {
        // 不正値（`macos` / typo）は型で構築できない ― load で弾く検証は manifest 側にある。
        for os in Os::iter() {
            let want = When {
                deps: vec![],
                os: Some(os),
                profile: None,
            };
            assert_eq!(
                when_satisfied(&Some(want), &no_profile()),
                Some(os) == current_os(),
                "os gate が現在 OS 一致のときだけ成立していない: {os}"
            );
        }
    }

    #[test]
    fn when_profile_matches_current_state_only() {
        let private_gate = || {
            Some(When {
                deps: vec![],
                os: None,
                profile: Some("private".to_string()),
            })
        };
        // 現在 profile = private → 採用。
        assert!(when_satisfied(
            &private_gate(),
            &GateState::with_profile(Some("private"))
        ));
        // 現在 profile = work（不一致）→ 不採用。
        assert!(!when_satisfied(
            &private_gate(),
            &GateState::with_profile(Some("work"))
        ));
    }

    #[test]
    fn when_profile_unset_defaults_to_not_private() {
        // profile 未設定（未 opt-in）では profile gate 付き断片は不採用 ＝ 安全側の既定。
        let want = When {
            deps: vec![],
            os: None,
            profile: Some("private".to_string()),
        };
        assert!(!when_satisfied(&Some(want), &no_profile()));
    }

    #[cfg(unix)]
    #[test]
    fn when_deps_check_path_via_absolute_executable() {
        // パス区切りを含む名前は PATH 探索せず直接判定する（/bin/sh は実行可能）。
        let present = When {
            deps: vec!["/bin/sh".to_string()],
            os: None,
            profile: None,
        };
        assert!(when_satisfied(&Some(present), &no_profile()));

        let absent = When {
            deps: vec!["/nonexistent/itic-bin".to_string()],
            os: None,
            profile: None,
        };
        assert!(!when_satisfied(&Some(absent), &no_profile()));
    }

    #[cfg(unix)]
    #[test]
    fn when_deps_are_and_of_array() {
        // deps は配列 AND: 1 つでも欠ければ不成立。
        let mixed = When {
            deps: vec!["/bin/sh".to_string(), "/nonexistent/itic-bin".to_string()],
            os: None,
            profile: None,
        };
        assert!(!when_satisfied(&Some(mixed), &no_profile()));
    }

    #[cfg(unix)]
    #[test]
    fn when_is_and_of_keys() {
        // deps・os は満たすが profile が外れる → 全体は false（AND）。
        let mixed = When {
            deps: vec!["/bin/sh".to_string()],
            os: current_os(),
            profile: Some("private".to_string()),
        };
        assert!(!when_satisfied(&Some(mixed), &no_profile()));
        // 全キー一致なら採用。
        assert!(when_satisfied(
            &Some(When {
                deps: vec!["/bin/sh".to_string()],
                os: current_os(),
                profile: Some("private".to_string()),
            }),
            &GateState::with_profile(Some("private")),
        ));
    }
}
