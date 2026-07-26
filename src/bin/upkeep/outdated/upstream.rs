//! パッケージの上流（リリースノートの所在）。
//!
//! リポジトリと配布元が名指しするページは独立に取れる。aqua backend のようにリポジトリ
//! しか分からないものも、GNU 系 formula のようにページしか分からないものもある。

/// GitHub の1リポジトリ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    pub owner: String,
    pub name: String,
}

impl Repo {
    /// `<owner>/<name>` を分解して作る。どちらかが空なら `None`。
    pub fn from_slug(slug: &str) -> Option<Self> {
        let (owner, name) = slug.trim_end_matches('/').split_once('/')?;
        let name = name.split('/').next()?;
        (!owner.is_empty() && !name.is_empty()).then(|| Repo {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }

    /// `github.com/<owner>/<name>` を含む URL から作る。`.git` 接尾とそれ以降のパスは
    /// 落とす。GitHub 以外のホストは `None`。
    pub fn from_url(url: &str) -> Option<Self> {
        let slug = url.split("github.com/").nth(1)?;
        let mut repo = Self::from_slug(slug)?;
        repo.name = repo.name.trim_end_matches(".git").to_string();
        (!repo.name.is_empty()).then_some(repo)
    }

    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub fn url(&self) -> String {
        format!("https://github.com/{}", self.slug())
    }
}

/// パッケージマネージャのメタデータから引けた上流。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Upstream {
    /// リリースノートを取りに行ける GitHub リポジトリ。
    pub repo: Option<Repo>,
    /// 配布元が名指ししたプロジェクトページ。
    pub homepage: Option<String>,
}

impl Upstream {
    /// 要約を出せなかったときに利用者へ示す URL。
    ///
    /// 配布元が名指しした `homepage` を優先する。リポジトリのページはこちらで組み立てた
    /// ものなので、名指しされた URL がある限りそちらを立てる。
    pub fn reference_url(&self) -> Option<String> {
        self.homepage
            .clone()
            .or_else(|| self.repo.as_ref().map(Repo::url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(owner: &str, name: &str) -> Repo {
        Repo {
            owner: owner.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn parses_plain_github_url() {
        assert_eq!(
            Repo::from_url("https://github.com/rustsec/rustsec"),
            Some(repo("rustsec", "rustsec"))
        );
    }

    #[test]
    fn parses_git_suffixed_url() {
        assert_eq!(
            Repo::from_url("https://github.com/kbknapp/cargo-outdated.git"),
            Some(repo("kbknapp", "cargo-outdated"))
        );
    }

    #[test]
    fn parses_npm_git_prefixed_url() {
        assert_eq!(
            Repo::from_url("git+https://github.com/textlint/textlint.git"),
            Some(repo("textlint", "textlint"))
        );
    }

    #[test]
    fn parses_monorepo_subpath_url() {
        assert_eq!(
            Repo::from_url("https://github.com/owner/monorepo/tree/main/crates/pkg"),
            Some(repo("owner", "monorepo"))
        );
    }

    #[test]
    fn parses_release_asset_url() {
        assert_eq!(
            Repo::from_url(
                "https://github.com/steipete/CodexBar/releases/download/v0.45.2/CodexBar.zip"
            ),
            Some(repo("steipete", "CodexBar"))
        );
    }

    #[test]
    fn non_github_host_is_none() {
        assert_eq!(Repo::from_url("https://gitlab.com/owner/repo"), None);
    }

    #[test]
    fn malformed_url_is_none() {
        assert_eq!(Repo::from_url("https://github.com/owner-only"), None);
    }

    #[test]
    fn bare_git_suffix_is_none() {
        assert_eq!(Repo::from_url("https://github.com/owner/.git"), None);
    }

    #[test]
    fn parses_slug() {
        assert_eq!(Repo::from_slug("jqlang/jq"), Some(repo("jqlang", "jq")));
    }

    #[test]
    fn slug_without_separator_is_none() {
        assert_eq!(Repo::from_slug("acli"), None);
    }

    #[test]
    fn builds_repo_url() {
        assert_eq!(repo("jqlang", "jq").url(), "https://github.com/jqlang/jq");
    }

    #[test]
    fn reference_url_prefers_named_homepage() {
        let upstream = Upstream {
            repo: Some(repo("steipete", "CodexBar")),
            homepage: Some("https://codexbar.app/".to_string()),
        };
        assert_eq!(
            upstream.reference_url(),
            Some("https://codexbar.app/".to_string())
        );
    }

    #[test]
    fn reference_url_falls_back_to_repo_page() {
        let upstream = Upstream {
            repo: Some(repo("jqlang", "jq")),
            homepage: None,
        };
        assert_eq!(
            upstream.reference_url(),
            Some("https://github.com/jqlang/jq".to_string())
        );
    }

    #[test]
    fn reference_url_is_none_when_nothing_resolved() {
        assert_eq!(Upstream::default().reference_url(), None);
    }
}
