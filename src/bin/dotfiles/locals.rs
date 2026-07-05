//! named value 機構のドメインルート。
//!
//! マシンローカルな「名前→値」（git の email/name など環境ごとに異なる値）を、ソースへ直書き
//! せずストアに退避し、配置時に `@@name@@` placeholder へ注入する仕組み一式。本ファイルは
//! 子モジュールを束ねる入口に徹し、ロジックは持たない（機構の本体は各子モジュール）。
//!
//! 呼び出し側は `crate::locals::resolve` 等を直接参照する（再エクスポートの集約点は設けない）。
//!
//! 値の穴埋め（locals: 宣言した名前へ値を注入する）と断片の採否（`when.profile`: マシンクラスで
//! 断片を採るか捨てるか）は別軸。マシン差の正体が gate なら profile、値なら locals で表す。

mod inject;
pub(crate) mod prompt;
pub(crate) mod resolve;
pub(crate) mod store;
