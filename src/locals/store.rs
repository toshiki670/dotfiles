//! named value ストア（§9.1）: マシンローカル値の単一ストア `~/.config/dotfiles/local.toml`。
//!
//! 名前→値を全ツール横断で集約する **dotfiles 非管理**ファイル（repo には値を一切置かない）。
//! `locals` を宣言した単位の placeholder（`@@name@@`）注入（[`crate::locals::resolve`]）と doctor 診断
//! （[`crate::doctor`]）がここを参照する。秘匿値を含むためファイルは 0600 で書き、ドット入り
//! キー（`git.email`）は toml が自動でクォートするので round-trip する（フラットな名前→値マップ）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `~/.config/dotfiles/local.toml` の名前→値マップ。
pub struct Store {
    path: PathBuf,
    values: BTreeMap<String, String>,
}

impl Store {
    /// ストアファイルのパス（`<home>/.config/dotfiles/local.toml`）。
    pub fn path(home: &Path) -> PathBuf {
        home.join(".config/dotfiles/local.toml")
    }

    /// ストアを読み込む。ファイルが無ければ空（apply 初回・未設定状態）として扱う。
    pub fn load(home: &Path) -> Result<Self, String> {
        let path = Self::path(home);
        let values = match std::fs::read_to_string(&path) {
            Ok(text) => {
                toml::from_str(&text).map_err(|e| format!("{}: パース失敗: {e}", path.display()))?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(format!("{}: 読み込み失敗: {e}", path.display())),
        };
        Ok(Self { path, values })
    }

    /// 名前に対応する値（無ければ None）。
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    /// 名前→値を設定する（メモリ上。永続化は [`Store::save`]）。
    pub fn set(&mut self, name: &str, value: &str) {
        self.values.insert(name.to_string(), value.to_string());
    }

    /// ストアを **原子的に** 0600 で書き出す（親ディレクトリを作成）。同一ディレクトリへ
    /// 一時ファイルを 0600 で作成し、fsync してから rename で置き換える。rename は既存の
    /// `local.toml`（live ファイル）を切り詰めないため、書き込み途中のクラッシュ/kill でも
    /// 既存値は無傷で残る（空/部分書き込みにならない）。一時ファイルの後始末は `tempfile` が
    /// Drop で行う（明示削除なし）。
    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
        }
        let text = toml::to_string(&self.values)
            .map_err(|e| format!("{}: 直列化失敗: {e}", self.path.display()))?;
        write_private(&self.path, text.as_bytes())
    }
}

/// 所有者のみ（0600）で原子的に書き出す。同一ディレクトリの一時ファイル（`tempfile` が Unix では
/// 0600 で作成）へ書き、fsync 後に rename で `path` を置き換える。失敗時は一時ファイルが Drop で
/// 消える（本書き込みは未反映）。秘匿値の露出窓と、live ファイルの truncate を両方避ける。
fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;

    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| format!("{}: 一時ファイル作成失敗: {e}", dir.display()))?;
    tmp.write_all(bytes)
        .map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))?;
    tmp.as_file()
        .sync_all()
        .map_err(|e| format!("{}: fsync 失敗: {e}", path.display()))?;
    tmp.persist(path)
        .map_err(|e| format!("{}: 原子的置換失敗: {}", path.display(), e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_loads_empty() {
        let home = tempfile::tempdir().unwrap();
        let store = Store::load(home.path()).unwrap();
        assert_eq!(store.get("git.email"), None);
    }

    #[test]
    fn set_save_load_round_trips_dotted_keys() {
        // ドット入りキー（git.email）が save→load で round-trip することを確認（toml の
        // 自動クォート依存。フラットなマップとして round-trip する）。
        let home = tempfile::tempdir().unwrap();
        let mut store = Store::load(home.path()).unwrap();
        store.set("git.email", "me@example.com");
        store.set("git.name", "Toshiki");
        store.save().unwrap();

        let reloaded = Store::load(home.path()).unwrap();
        assert_eq!(reloaded.get("git.email"), Some("me@example.com"));
        assert_eq!(reloaded.get("git.name"), Some("Toshiki"));
    }

    #[cfg(unix)]
    #[test]
    fn saved_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir().unwrap();
        let mut store = Store::load(home.path()).unwrap();
        store.set("github.token", "ghp_secret");
        store.save().unwrap();

        let mode = std::fs::metadata(Store::path(home.path()))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "ストアは 0600 で書かれる");
    }

    #[cfg(unix)]
    #[test]
    fn overwriting_existing_store_keeps_owner_only() {
        // 原子的置換（rename）でも、既存ファイルの上書き後に 0600 と最新値が保たれる。
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir().unwrap();

        let mut first = Store::load(home.path()).unwrap();
        first.set("git.email", "old@example.com");
        first.save().unwrap();

        let mut second = Store::load(home.path()).unwrap();
        second.set("git.email", "new@example.com");
        second.save().unwrap();

        let path = Store::path(home.path());
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "上書き後も 0600 を保つ");
        assert_eq!(
            Store::load(home.path()).unwrap().get("git.email"),
            Some("new@example.com"),
            "最新値で置き換わる",
        );
    }
}
