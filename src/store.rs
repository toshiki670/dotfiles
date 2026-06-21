//! named value ストア（§9.1）: マシンローカル値の単一ストア `~/.config/dotfiles/local.toml`。
//!
//! 名前→値を全ツール横断で集約する **dotfiles 非管理**ファイル（repo には値を一切置かない）。
//! `locals` を宣言した単位の placeholder（`@@name@@`）注入（[`crate::resolve`]）と doctor 診断
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

    /// ストアを 0600 で書き出す（親ディレクトリを作成）。秘匿値を含むため、作成時点から
    /// 所有者のみのモードで開き（新規ファイルの 0644 露出窓を作らない）、既存ファイルにも
    /// 念のため 0600 を再設定する。
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

/// 所有者のみ（0600）で書き出す（Unix）。create-mode 0600 で開いて作成窓を塞ぎ、
/// 既存ファイルにも 0600 を再設定する。
#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", path.display()))
}

/// 非 Unix ではパーミッションモデルが異なるため通常の書き込み。
#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))
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
}
