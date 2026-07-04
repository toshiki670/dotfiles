//! ソースの二段構え: 配置エンジンが読む `configs/` の実体をどこから得るかを解決する。
//!
//! 解決優先順位は **`--source` 明示 ＞ 作業ツリー検出 ＞ 埋め込みフォールバック**。
//!
//! - **作業ツリー直読み**（dev / 移行期の主役）: 設定を編集して即 apply で検証できる。CWD から
//!   上へ辿り、ユニット（`manifest.toml`）を持つ最初の `configs/` を使う。
//! - **埋め込み**（配布の完成形）: `cargo install dotfiles` だけ（clone 無し）で自己完結させるため、
//!   コンパイル時に `configs/` をバイナリへ焼き込み（[`mod@include_dir`]）、解決時に temp dir へ展開する。
//!
//! 配置エンジン（[`crate::core::discover`] / [`crate::core::apply::copy`] / [`crate::core::apply::compose`] / [`crate::core::apply::generate`] /
//! [`crate::core::hooks`]）は全て実 path で `std::fs` を読む。埋め込みを temp dir へ実体化して**実 path**を
//! 渡すことで、エンジンは「ソースが埋め込みか実 FS か」を知らずに済む（安定な core を揮発に依存させない）。

use crate::core::discover;
use include_dir::{Dir, include_dir};
use std::fmt;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// コンパイル時に焼き込む `configs/`（埋め込みフォールバックの実体）。
static EMBEDDED: Dir = include_dir!("$CARGO_MANIFEST_DIR/configs");

/// 解決元（どの段で解決したか）。ユーザー向け表示（apply ヘッダ / list）に使う。
///
/// 表示ラベルは英語にする ― 周囲の技術ラベル（`copy` / `generate` / `overlay` 等）が英語で、
/// そこへ日本語語句を混ぜると不揃いになるため（出力の体裁を 1 系統に揃える）。
pub enum Origin {
    /// `--source` 明示（上級オプション）。
    Explicit(PathBuf),
    /// 作業ツリー検出（移行期の主役）。
    WorkingTree(PathBuf),
    /// 埋め込みフォールバック（clone 無しの配布）。
    Embedded,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Origin::Explicit(p) => write!(f, "--source ({})", p.display()),
            Origin::WorkingTree(p) => write!(f, "working tree ({})", p.display()),
            Origin::Embedded => write!(f, "embedded"),
        }
    }
}

/// 解決済みソース。エンジンへ渡す実 path（[`Resolved::root`]）と解決元（[`Resolved::origin`]）を持つ。
///
/// `_temp` は埋め込み展開先の生存保証: [`TempDir`] は drop で中身を削除するため、apply / list /
/// doctor が `root` を読み終えるまで `Resolved` を保持する必要がある（明示・作業ツリー時は `None`）。
pub struct Resolved {
    root: PathBuf,
    origin: Origin,
    _temp: Option<TempDir>,
}

impl Resolved {
    /// エンジンへ渡すソースルートの実 path。
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 解決元（表示用）。
    pub fn origin(&self) -> &Origin {
        &self.origin
    }
}

/// ソースを二段構えで解決する（上記の優先順位）。
///
/// 1. `explicit`（`--source`）あり → そのまま使う（dir でなければエラー）。
/// 2. なし → 作業ツリー検出（[`detect_working_tree`]）。
/// 3. 見つからねば → 埋め込みを temp dir へ展開して使う。
pub fn resolve(explicit: Option<&Path>) -> Result<Resolved, String> {
    if let Some(path) = explicit {
        if !path.is_dir() {
            return Err(format!(
                "--source {} がディレクトリとして見つかりません",
                path.display()
            ));
        }
        return Ok(Resolved {
            root: path.to_path_buf(),
            origin: Origin::Explicit(path.to_path_buf()),
            _temp: None,
        });
    }

    if let Some(root) = detect_working_tree() {
        return Ok(Resolved {
            origin: Origin::WorkingTree(root.clone()),
            root,
            _temp: None,
        });
    }

    extract_embedded()
}

/// CWD から祖先を上へ辿り、ユニットを持つ最初の `<ancestor>/configs` を作業ツリーとみなす。
///
/// 「ユニットを持つ」判定は [`discover::collect`] を再利用する（「ユニットとは `manifest.toml` を
/// 持つ dir」の出所を一本化）。これにより、たまたま同名の空 `configs/` を誤検出せず埋め込みへ落ちる。
fn detect_working_tree() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        let candidate = ancestor.join("configs");
        if candidate.is_dir() && has_units(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// `dir` がユニット（`manifest.toml` を持つ dir）を1つ以上含むか。走査エラーは「ソースでない」扱い。
fn has_units(dir: &Path) -> bool {
    discover::collect(dir)
        .map(|u| !u.is_empty())
        .unwrap_or(false)
}

/// 埋め込んだ `configs/` を temp dir へ展開し、その実 path を解決結果にする。
fn extract_embedded() -> Result<Resolved, String> {
    let temp = tempfile::Builder::new()
        .prefix("dotfiles-configs-")
        .tempdir()
        .map_err(|e| format!("埋め込みソースの展開先 temp dir 作成に失敗: {e}"))?;
    EMBEDDED
        .extract(temp.path())
        .map_err(|e| format!("埋め込みソースの展開に失敗: {e}"))?;
    Ok(Resolved {
        root: temp.path().to_path_buf(),
        origin: Origin::Embedded,
        _temp: Some(temp),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// `--source` 明示は dir 検証して Explicit を返す。
    #[test]
    fn explicit_source_uses_given_dir() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve(Some(dir.path())).unwrap();
        assert_eq!(resolved.root(), dir.path());
        assert!(matches!(resolved.origin(), Origin::Explicit(_)));
    }

    /// `--source` が dir でなければエラー。
    #[test]
    fn explicit_source_missing_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope");
        assert!(resolve(Some(&missing)).is_err());
    }

    /// ユニットを持つ `configs/` だけを作業ツリーとして検出し、空 dir は弾く。
    #[test]
    fn has_units_requires_a_manifest() {
        let root = tempfile::tempdir().unwrap();
        let configs = root.path().join("configs");
        fs::create_dir_all(configs.join("empty")).unwrap();
        assert!(
            !has_units(&configs),
            "manifest 無しの configs はユニット無し"
        );

        let unit = configs.join("foo");
        fs::create_dir_all(&unit).unwrap();
        fs::write(unit.join("manifest.toml"), "dst = \"~/x\"\n").unwrap();
        assert!(has_units(&configs), "manifest を持つ dir はユニット有り");
    }

    /// 埋め込みフォールバックは temp dir へ実体化し、ユニットを伴う実 path を返す。
    #[test]
    fn embedded_extracts_units_to_real_path() {
        let resolved = extract_embedded().unwrap();
        assert!(matches!(resolved.origin(), Origin::Embedded));
        assert!(resolved.root().is_dir());
        // 出荷 configs は必ずユニットを持つ（埋め込みが実体化された証跡）。
        assert!(has_units(resolved.root()), "埋め込み展開先にユニットが無い");
    }
}
