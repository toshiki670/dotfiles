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
//! 可視性の基準（apply 子も含めた crate 共通の SSOT）:
//! 1. 既定は **コード結合**で決める ― 親ツリー外から `use` される子だけ `pub(crate)`、それ以外は
//!    private。`store` / `resolve` / `prompt` は他ドメイン（apply / secret / doctor）のコードが
//!    横断利用するため `pub(crate)`、`inject` は `resolve` 内部からのみ使う実装詳細なので private
//!    （いずれも外部 API は持たない＝binary 内に閉じる）。
//! 2. rustdoc の横断リンク（`deny(broken_intra_doc_links)` 下）のためだけに可視性を上げる前に、まず
//!    リンクを**上位の窓口へ寄せられないか**を検討する ― overview の `inject` リンクは解決＋注入の
//!    窓口 `resolve` へ集約して private を保てた。窓口へ寄せられない深い参照（apply 側の `copy_tree`
//!    / `gate` の評価規則 / `generate` の `cmd` など、各層が概念そのもの）だけ、最後の手段として
//!    `pub(crate)` を許す。
//!
//! 再エクスポートは設けず、呼び出し側は `crate::locals::resolve` 等を直接参照する（不要な集約点を
//! 増やさない）。

mod inject;
pub(crate) mod prompt;
pub(crate) mod resolve;
pub(crate) mod store;
