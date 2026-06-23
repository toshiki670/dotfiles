//! `dotfiles secret set`（§9・§11）: named value をストアへ設定する明示経路。
//!
//! ストアの読み書きは [`crate::store`]。コマンド名 `secret` は仮称（email/name 等の非秘匿値も
//! 扱うため概念とズレる。§16 で最終命名）。値は argv に載るためシェル履歴/ps に残る点は呼び出し側
//! の選択であり、対話取得の非エコー（[`crate::prompt`]）は apply の TTY 経路が担う別物。

use crate::store::Store;
use std::path::Path;

/// `name`→`value` をストアへ設定し 0600 で保存する。
pub fn set(home: &Path, name: &str, value: &str) -> Result<(), String> {
    let mut store = Store::load(home)?;
    store.set(name, value);
    store.save()?;
    println!(
        "secret set: {name} を保存しました（{}）",
        Store::path(home).display()
    );
    Ok(())
}
