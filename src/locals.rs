//! named value（§9）機構のドメインルート。
//!
//! マシンローカルな「名前→値」（git の email/name など環境ごとに異なる値）を、ソースへ直書き
//! せずストアに退避し、配置時に `@@name@@` placeholder へ注入する仕組み一式。本ファイルは
//! 子モジュールを束ねる入口に徹し、ロジックは持たない（機構の本体は各子モジュール）。
//!
//! - [`store`] — ストア（`~/.config/dotfiles/local.toml`）の読み書き（0600）。
//! - [`resolve`] — apply 時のローカル値解決（ストア突き合わせ・未設定の対話取得）。
//! - [`inject`] — 解決済み値での `@@name@@` placeholder 置換。
//! - [`prompt`] — TTY 対話入力（sensitive は非エコー）。
//!
//! 呼び出し側は `crate::locals::resolve` 等を直接参照する（再エクスポートの集約点は設けない）。

mod inject;
pub(crate) mod prompt;
pub(crate) mod resolve;
pub(crate) mod store;
