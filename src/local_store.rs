//! マシンローカル値ストア（§9.1）: `~/.config/dotfiles/local.toml` の read/write。
//!
//! named value（`git.email` 等）を全ツール横断で 1 ファイルに集約する。dotfiles 非管理
//! （gitignore 相当）で、apply の `@@name@@` 置換（[`crate::inject`]）・`secret set`・`doctor` が
//! 共有する唯一のストア。**repo には値を一切置かない**（設計書 §9.2）。
//!
//! 名前はドット区切り（`git.email`）。ファイル上はネストテーブル（`[git] email = ".."`）で
//! 持ち、read 時にドット区切りキーへ flatten、write 時に再ネストする。人が手で開いて読み書き
//! しやすい自然な TOML 形を保ちつつ、エンジン内部はフラットな名前→値で扱う。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// ストアファイルのパス（`<home>/.config/dotfiles/local.toml`）。
///
/// XDG 正規化（`$XDG_CONFIG_HOME` 等）は設計書 §14 の未決事項のため、S4 では設計書 §9.2 の
/// 表記どおり素直に `~/.config` 直下に置く。
pub fn path(home: &Path) -> PathBuf {
    home.join(".config/dotfiles/local.toml")
}

/// ストアを読み、名前（ドット区切り）→ 値のフラットな map にして返す。
/// ファイルが無ければ空 map（未設定 ＝ エラーではない）。
pub fn load(home: &Path) -> Result<BTreeMap<String, String>, String> {
    let p = path(home);
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(format!("{}: 読み込み失敗: {e}", p.display())),
    };
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("{}: パース失敗: {e}", p.display()))?;
    let mut out = BTreeMap::new();
    flatten(&value, String::new(), &mut out);
    Ok(out)
}

/// 名前 `name` に `value` を設定する（load → 上書き → write の read-modify-write）。
/// 既存の他の値は保持する。親ディレクトリ・ファイルのパーミッションは [`write`] が整える。
pub fn set(home: &Path, name: &str, value: &str) -> Result<(), String> {
    let mut map = load(home)?;
    map.insert(name.to_string(), value.to_string());
    write(home, &map)
}

/// フラットな名前→値 map をネストテーブルへ再構成し、0600 で書き出す（親 dir は 0700 で作成）。
pub fn write(home: &Path, map: &BTreeMap<String, String>) -> Result<(), String> {
    let p = path(home);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
        set_dir_mode(parent)?;
    }
    let text = toml::to_string(&unflatten(map)).map_err(|e| format!("TOML 直列化失敗: {e}"))?;
    std::fs::write(&p, text).map_err(|e| format!("{}: 書き込み失敗: {e}", p.display()))?;
    set_file_mode(&p)?;
    Ok(())
}

/// ネストした TOML テーブルを、ドット区切りキー → 文字列値のフラット map へ畳む。
/// 文字列以外の leaf（配列・数値等）は S4 の named value 形式外として無視する。
fn flatten(value: &toml::Value, prefix: String, out: &mut BTreeMap<String, String>) {
    match value {
        toml::Value::Table(table) => {
            for (k, v) in table {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten(v, key, out);
            }
        }
        toml::Value::String(s) if !prefix.is_empty() => {
            out.insert(prefix, s.clone());
        }
        _ => {}
    }
}

/// フラットなドット区切りキー map を、ネストした TOML テーブルへ再構成する。
fn unflatten(map: &BTreeMap<String, String>) -> toml::Value {
    let mut root = toml::value::Table::new();
    for (name, value) in map {
        let mut table = &mut root;
        let parts: Vec<&str> = name.split('.').collect();
        for seg in &parts[..parts.len() - 1] {
            table = table
                .entry(seg.to_string())
                .or_insert_with(|| toml::Value::Table(toml::value::Table::new()))
                .as_table_mut()
                .expect("nested key segment is a table");
        }
        table.insert(
            parts[parts.len() - 1].to_string(),
            toml::Value::String(value.clone()),
        );
    }
    toml::Value::Table(root)
}

/// ストアファイルを所有者のみ読み書き可（0600）にする。秘匿値を含みうるため。
#[cfg(unix)]
fn set_file_mode(p: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", p.display()))
}

/// 親ディレクトリを所有者のみ（0700）にする。
#[cfg(unix)]
fn set_dir_mode(p: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", p.display()))
}

#[cfg(not(unix))]
fn set_file_mode(_p: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(not(unix))]
fn set_dir_mode(_p: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_nested_tables_to_dotted_keys() {
        let value: toml::Value =
            toml::from_str("[git]\nemail = \"me@x\"\nname = \"Me\"\n[github]\ntoken = \"t\"\n")
                .unwrap();
        let mut out = BTreeMap::new();
        flatten(&value, String::new(), &mut out);
        assert_eq!(out.get("git.email").map(String::as_str), Some("me@x"));
        assert_eq!(out.get("git.name").map(String::as_str), Some("Me"));
        assert_eq!(out.get("github.token").map(String::as_str), Some("t"));
    }

    #[test]
    fn flatten_ignores_non_string_leaves() {
        let value: toml::Value =
            toml::from_str("[a]\nn = 1\narr = [1, 2]\ns = \"keep\"\n").unwrap();
        let mut out = BTreeMap::new();
        flatten(&value, String::new(), &mut out);
        assert_eq!(out.get("a.s").map(String::as_str), Some("keep"));
        assert!(!out.contains_key("a.n"));
        assert!(!out.contains_key("a.arr"));
    }

    #[test]
    fn unflatten_round_trips_through_toml() {
        let mut map = BTreeMap::new();
        map.insert("git.email".to_string(), "me@x".to_string());
        map.insert("git.name".to_string(), "Me".to_string());
        map.insert("flat".to_string(), "v".to_string());
        // 再ネスト → 直列化 → 再パース → flatten で同一に戻る。
        let text = toml::to_string(&unflatten(&map)).unwrap();
        let value: toml::Value = toml::from_str(&text).unwrap();
        let mut back = BTreeMap::new();
        flatten(&value, String::new(), &mut back);
        assert_eq!(map, back);
    }

    #[test]
    fn load_missing_store_is_empty() {
        let home = tempfile::tempdir().unwrap();
        assert!(load(home.path()).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn set_creates_store_0600_and_persists() {
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir().unwrap();
        set(home.path(), "git.email", "me@x").unwrap();
        // 再 load で読める。
        assert_eq!(
            load(home.path())
                .unwrap()
                .get("git.email")
                .map(String::as_str),
            Some("me@x")
        );
        // ファイルは 0600。
        let mode = std::fs::metadata(path(home.path()))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "ストアは 0600 であるべき");
        // 既存値を保ったまま追記できる。
        set(home.path(), "git.name", "Me").unwrap();
        let map = load(home.path()).unwrap();
        assert_eq!(map.get("git.email").map(String::as_str), Some("me@x"));
        assert_eq!(map.get("git.name").map(String::as_str), Some("Me"));
    }
}
