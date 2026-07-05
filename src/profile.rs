//! `dotfiles profile [<name>]`: マシンクラスの状態 gate を設定／表示する。
//!
//! 状態の読み書きは [`crate::state`]（`~/.config/dotfiles/profile`）。`when = { profile = … }`
//! （[`crate::apply::gate`]）はここで書いた現在値と一致する断片だけを採用する。profile 名は固定
//! 集合にしない（任意文字列・状態一致）― エンジンを特定 profile 名へ結合させない。
//! 未設定の既定は not-private（新規・仕事マシンへ private 設定が漏れないよう明示 opt-in）。

use crate::state;
use std::path::Path;

/// `name` を profile 状態として保存する。
pub fn set(home: &Path, name: &str) -> Result<(), String> {
    state::write(home, state::PROFILE, name)?;
    println!(
        "profile: {name} に設定しました（{}）",
        state::path(home, state::PROFILE).display()
    );
    Ok(())
}

/// 現在の profile を表示する。未設定では、どの `when.profile` 値とも一致しない（＝ profile gate を
/// 持つ設定はどれも配置されない）旨を、特定 profile 名に依らない表現で示す。
pub fn show(home: &Path) -> Result<(), String> {
    match state::read(home, state::PROFILE)? {
        Some(name) => println!("profile: {name}"),
        None => println!("profile: 未設定（profile gate 付きの設定は配置されない）"),
    }
    Ok(())
}
