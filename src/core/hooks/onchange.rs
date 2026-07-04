//! onchange 検知: フックを「ユニットのソースが前回適用時から変わったか」で gate する。
//!
//! 2 つの責務を持つ:
//! - **ソース指紋の計算**（[`hash_dir`]）: ユニットのデプロイ対象ファイル（[`crate::core::discover::unit_files`]）
//!   を相対パス＋内容で畳んだ指紋。mtime ではなくソース内容を見る（mtime は touch/clone で
//!   揺れるため）。前回保存したソース内容との等値比較で変化を検出する。**用途は前回値との
//!   等値比較だけ**なので暗号学的ハッシュは要らず、`std::hash`（非暗号学的）で `u64` 指紋を取り
//!   16 進文字列にする ― これで sha2 等の依存も hex 化の小細工も不要。指紋ロジック変更時は
//!   全フックが一度再実行されるが、再実行は冪等なので無害。
//! - **状態の永続化**（[`State`]）: フックごとの前回指紋を `~/.config/dotfiles/hooks.toml` に
//!   持つ。秘匿値ではない（[`crate::core::locals::store`] と違い 0600 不要）ため平文で書き、破損時は warn して
//!   空（全フック再実行）扱いにする ― 再実行は冪等なので、致命的エラーで apply を止めるより
//!   作り直す方が安全（disposable な状態）。
//!
//! ハッシュはユニットのソースだけを入力にし、locals 注入値を含まない（locals は apply 時に
//! 配置先へ注入される値でありソースではない）。注入値の変化で再実行したくなったら仕様の
//! 確定が要る（#560）。

use crate::core::discover;
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hasher};
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

/// 1 単位（`dir`）のソースツリーを畳んだ 16 進指紋（`u64` を `{:016x}`）。
///
/// [`discover::unit_files`] が返すデプロイ対象ファイルを（既にソート済み）順に、
/// **`dir` からの相対パス → 内容**の順で hasher に与える。相対パスも混ぜることで、
/// ファイルの追加・改名・移動も検知する（内容だけの concat より厳密）。
///
/// **`manifest.toml` 自体は指紋対象外**（`unit_files` が除外）。これは意図的で、指紋は
/// 「フックが消費するデプロイ内容（例: bat の theme・ghostty の config）が変わったか」を測るもの
/// だから ― `dst` 変更やパーミッション属性の変更は配置先を変えるだけでフックの入力には影響しない。
/// なお **hook の追加・コマンド変更**は取りこぼさない: onchange 状態は `<unit>::<コマンド短指紋>`
/// キーで持つので（[`crate::core::hooks`]）、新しい/変更後のコマンドは未記録（`None`）= 初回として必ず走る
/// （manifest が指紋対象外でも、キー側にコマンド内容が入るため）。
pub fn hash_dir(dir: &Path) -> Result<String, String> {
    let mut hasher = DefaultHasher::new();
    for file in discover::unit_files(dir)? {
        let rel = file.strip_prefix(dir).unwrap_or(&file);
        hasher.write(rel.to_string_lossy().as_bytes());
        hasher.write_u8(0); // パス↔内容の境界（"a"+"bc" と "ab"+"c" を区別する）。
        let bytes =
            std::fs::read(&file).map_err(|e| format!("{}: 読み込み失敗: {e}", file.display()))?;
        hasher.write(&bytes);
        hasher.write_u8(0);
    }
    Ok(format!("{:016x}", hasher.finish()))
}

/// 文字列を畳んだ 16 桁（`u64` を `{:016x}`）の安定指紋。フックの状態キー suffix（コマンド内容の
/// 同一性判定）に使う ― 生のコマンド文字列をキーにすると空白/クォートで toml キーが壊れうるため、
/// 短い 16 進へ畳む。64bit なので、1 ユニット内の数個のコマンド間の衝突は無視できる。
pub fn short_hash(data: &str) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(data.as_bytes());
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_state_loads_empty() {
        let home = tempfile::tempdir().unwrap();
        let state = State::load(home.path());
        assert_eq!(state.get("demo::abc123"), None);
    }

    #[test]
    fn set_save_load_round_trips() {
        let home = tempfile::tempdir().unwrap();
        let mut state = State::load(home.path());
        state.set("demo::abc123", "deadbeef");
        state.save().unwrap();

        let reloaded = State::load(home.path());
        assert_eq!(reloaded.get("demo::abc123"), Some("deadbeef"));
    }

    #[test]
    fn short_hash_is_stable_and_distinguishes_commands() {
        // 同じ入力は同じ短ハッシュ、違う入力は違う短ハッシュ（状態キーの衝突防止）。
        assert_eq!(short_hash("cmd-a\u{0}x"), short_hash("cmd-a\u{0}x"));
        assert_ne!(short_hash("cmd-a\u{0}x"), short_hash("cmd-b\u{0}y"));
        assert_eq!(short_hash("x").len(), 16);
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
