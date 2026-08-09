//! パッケージマネージャのバージョン文字列と GitHub のタグ名の対応付け。
//!
//! 同じリリースでも綴りは揃わない。brew は `3.4.14`、cargo は `v0.27.0` を返し、タグ側は
//! `release-3.4.14`（SDL）や `jq-1.8.2`（jq）のように接頭辞を持つ。接頭辞だけを落として
//! 本体どうしを突き合わせる。

/// パッケージ名に依らない接頭辞。長い方から試す（`release-` は `r` から始まらないので
/// 順序の衝突は無いが、追加するときは前方一致の包含関係に注意する）。
const FIXED_PREFIXES: [&str; 2] = ["release-", "v"];

/// `tag` が `version` と同じリリースを指すか。
///
/// `name` はパッケージ名。`jq-1.8.2` のようにタグがパッケージ名で始まる綴りを拾うために
/// 使う（大文字小文字は区別する。ここで揺らすと別パッケージのタグまで拾いうるため）。
pub fn matches(tag: &str, version: &str, name: &str) -> bool {
    !version.trim().is_empty() && strip(tag, name) == strip(version, name)
}

/// 接頭辞を落としてバージョン本体だけにする。
fn strip<'a>(text: &'a str, name: &str) -> &'a str {
    let text = text.trim();

    if !name.is_empty()
        && let Some(rest) = text.strip_prefix(name).and_then(|r| r.strip_prefix('-'))
    {
        return rest;
    }

    FIXED_PREFIXES
        .iter()
        .find_map(|prefix| text.strip_prefix(prefix))
        .unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_version_matches_bare_tag() {
        assert!(matches("1.33.7", "1.33.7", "mpg123"));
    }

    #[test]
    fn v_prefixed_tag_matches_bare_version() {
        assert!(matches("v5.4.0", "5.4.0", "docker-compose"));
    }

    #[test]
    fn v_prefixed_tag_matches_v_prefixed_version() {
        assert!(matches("v0.27.0", "v0.27.0", "cargo-modules"));
    }

    /// SDL のタグは `release-3.4.14`、brew の版は `3.4.14`。
    #[test]
    fn release_prefixed_tag_matches_bare_version() {
        assert!(matches("release-3.4.14", "3.4.14", "sdl3"));
    }

    /// jq のタグは `jq-1.8.2`、mise の版は `1.8.2`。
    #[test]
    fn name_prefixed_tag_matches_bare_version() {
        assert!(matches("jq-1.8.2", "1.8.2", "jq"));
    }

    #[test]
    fn different_versions_do_not_match() {
        assert!(!matches("v0.26.0", "0.27.0", "cargo-modules"));
    }

    /// brew の luajit は `2.1.<タイムスタンプ>` で、上流のタグ体系と無関係。
    #[test]
    fn unrelated_version_scheme_does_not_match() {
        assert!(!matches("v2.1.ROLLING", "2.1.1785763465", "luajit"));
    }

    /// 名前接頭辞は別パッケージのタグを拾わない。
    #[test]
    fn name_prefix_of_another_package_does_not_match() {
        assert!(!matches("jq-1.8.2", "1.8.2", "gojq"));
    }

    /// 接頭辞だけのタグはバージョン本体が空になる。空版と一致させない。
    #[test]
    fn empty_version_never_matches() {
        assert!(!matches("v", "", "bat"));
    }

    #[test]
    fn surrounding_whitespace_is_ignored() {
        assert!(matches(" v1.2.3 ", "1.2.3", "bat"));
    }
}
