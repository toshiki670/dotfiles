//! onchange 検知（§13, S5）: フックを「ユニットのソースが前回適用時から変わったか」で gate する。
//!
//! 2 つの責務を持つ:
//! - **ソースハッシュ計算**（[`hash_dir`]）: ユニットのデプロイ対象ファイル（[`crate::discover::unit_files`]）
//!   を相対パス＋内容で SHA-256 に畳む。設計書 §14 の未決項目「mtime vs ソースハッシュ」をソース
//!   ハッシュで確定する（mtime は touch/clone で揺れるため内容ハッシュを採る）。chezmoi の
//!   `run_onchange_*` が埋め込みソースの `sha256sum` を比較していたのと同じ役割。
//! - **状態の永続化**（[`State`]）: フックごとの前回ハッシュを `~/.config/dotfiles/hooks.toml` に
//!   持つ。秘匿値ではない（[`crate::store`] と違い 0600 不要）ため平文で書き、破損時は warn して
//!   空（全フック再実行）扱いにする ― 再実行は冪等なので、致命的エラーで apply を止めるより
//!   作り直す方が安全（disposable な状態）。

use crate::discover;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `~/.config/dotfiles/hooks.toml` の「フックキー → 前回適用ハッシュ」マップ。
/// キーは `<unit-rel>::<hook>`（同一ユニットが複数フックを宣言しても衝突しない）。
pub struct State {
    path: PathBuf,
    hashes: BTreeMap<String, String>,
}

impl State {
    /// 状態ファイルのパス（`<home>/.config/dotfiles/hooks.toml`）。
    pub fn path(home: &Path) -> PathBuf {
        home.join(".config/dotfiles/hooks.toml")
    }

    /// 状態を読み込む。ファイルが無ければ空（初回 apply）。パース失敗は **警告して空**扱いに
    /// する（破損した onchange 状態で apply 全体を止めない。フック再実行は冪等）。
    pub fn load(home: &Path) -> Self {
        let path = Self::path(home);
        let hashes = match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                eprintln!(
                    "apply: {} の解析に失敗（空として続行）: {e}",
                    path.display()
                );
                BTreeMap::new()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => {
                eprintln!(
                    "apply: {} の読み込みに失敗（空として続行）: {e}",
                    path.display()
                );
                BTreeMap::new()
            }
        };
        Self { path, hashes }
    }

    /// キーに対応する前回ハッシュ（無ければ None）。
    pub fn get(&self, key: &str) -> Option<&str> {
        self.hashes.get(key).map(String::as_str)
    }

    /// キー → ハッシュを設定する（メモリ上。永続化は [`State::save`]）。
    pub fn set(&mut self, key: &str, hash: &str) {
        self.hashes.insert(key.to_string(), hash.to_string());
    }

    /// 状態を書き出す（親ディレクトリを作成）。秘匿値でないので平文・通常パーミッション。
    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
        }
        let text = toml::to_string(&self.hashes)
            .map_err(|e| format!("{}: 直列化失敗: {e}", self.path.display()))?;
        std::fs::write(&self.path, text)
            .map_err(|e| format!("{}: 書き込み失敗: {e}", self.path.display()))
    }
}

/// 1 単位（`dir`）のソースツリーを SHA-256 で畳んで 16 進文字列にする。
///
/// [`discover::unit_files`] が返すデプロイ対象ファイルを（既にソート済み）順に、
/// **`dir` からの相対パス → 内容**の順で hasher に与える。相対パスも混ぜることで、
/// ファイルの追加・改名・移動も検知する（内容だけの concat より厳密）。
pub fn hash_dir(dir: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    for file in discover::unit_files(dir)? {
        let rel = file.strip_prefix(dir).unwrap_or(&file);
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update([0u8]); // パス↔内容の境界（"a"+"bc" と "ab"+"c" を区別する）。
        let bytes =
            std::fs::read(&file).map_err(|e| format!("{}: 読み込み失敗: {e}", file.display()))?;
        hasher.update(&bytes);
        hasher.update([0u8]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_state_loads_empty() {
        let home = tempfile::tempdir().unwrap();
        let state = State::load(home.path());
        assert_eq!(state.get("bat::bat-cache"), None);
    }

    #[test]
    fn set_save_load_round_trips() {
        let home = tempfile::tempdir().unwrap();
        let mut state = State::load(home.path());
        state.set("bat::bat-cache", "deadbeef");
        state.save().unwrap();

        let reloaded = State::load(home.path());
        assert_eq!(reloaded.get("bat::bat-cache"), Some("deadbeef"));
    }

    #[test]
    fn corrupt_state_is_treated_as_empty() {
        // 破損した hooks.toml は致命的にせず空扱い（フック再実行は冪等）。
        let home = tempfile::tempdir().unwrap();
        let path = State::path(home.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "this is = = not toml").unwrap();
        let state = State::load(home.path());
        assert_eq!(state.get("anything"), None);
    }

    #[test]
    fn hash_changes_when_a_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manifest.toml"), "dst = \"~/x\"\n").unwrap();
        std::fs::write(dir.path().join("theme.txt"), "v1").unwrap();
        let before = hash_dir(dir.path()).unwrap();

        std::fs::write(dir.path().join("theme.txt"), "v2").unwrap();
        let after = hash_dir(dir.path()).unwrap();
        assert_ne!(
            before, after,
            "ファイル内容が変われば onchange ハッシュも変わる"
        );
    }

    #[test]
    fn hash_ignores_manifest_only_edits() {
        // manifest.toml はデプロイ対象外（unit_files が除外）。dst 等の変更ではハッシュ不変。
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manifest.toml"), "dst = \"~/a\"\n").unwrap();
        std::fs::write(dir.path().join("config"), "body").unwrap();
        let before = hash_dir(dir.path()).unwrap();

        std::fs::write(dir.path().join("manifest.toml"), "dst = \"~/b\"\n").unwrap();
        let after = hash_dir(dir.path()).unwrap();
        assert_eq!(
            before, after,
            "manifest だけの変更はソースハッシュに影響しない"
        );
    }
}
