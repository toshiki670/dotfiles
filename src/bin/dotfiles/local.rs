//! `dotfiles local set`: named value（マシンローカル値）をストアへ設定する明示経路。
//!
//! ストアの読み書きは [`crate::locals::store`]。値は argv に載るためシェル履歴/ps に残る点は呼び出し側
//! の選択であり、対話取得（[`crate::locals::prompt`]）は apply の TTY 経路が担う別物。

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
