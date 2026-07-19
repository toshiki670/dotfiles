//! `dotfiles local`: named value（マシンローカル値）をストアへ設定し、設定済みの値を一覧する明示経路。
//!
//! ストアの読み書きは [`crate::locals::store`]。`set` の値は argv に載るためシェル履歴/ps に残る点は
//! 呼び出し側の選択であり、対話取得（[`crate::locals::prompt`]）は apply の TTY 経路が担う別物。
//!
//! `list` は値を verbatim で出す ― 秘匿値のマスクは、それを宣言する語彙（per-entry `sensitive`・
//! #588）が入る時に併せて入る。

use crate::locals::store::Store;
use std::path::Path;

/// `name`→`value` をストアへ設定し 0600 で保存する。
pub fn set(home: &Path, name: &str, value: &str) -> Result<(), String> {
    let mut store = Store::load(home)?;
    store.set(name, value);
    store.save()?;
    println!(
        "local set: {name} を保存しました（{}）",
        Store::path(home).display()
    );
    Ok(())
}

/// ストアの全 named value を `名前 = 値` の形で名前順に表示する。
pub fn list(home: &Path) -> Result<(), String> {
    let store = Store::load(home)?;
    let path = Store::path(home);

    let rows: Vec<_> = store.entries().collect();
    if rows.is_empty() {
        println!("local list: 対象なし（{} に値がない）", path.display());
        return Ok(());
    }

    let width = rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    println!("dotfiles local list（store: {}）", path.display());
    for (name, value) in rows {
        println!("  {name:<width$} = {value}");
    }
    Ok(())
}
