//! 畳み込み: 現在の内容（`base`）へ新しい断片（`frag`）を 1 つ後勝ちで重ねる純ロジック。
//! [`crate::apply::pipeline`] の `fold_in` が step ごとに、現在の内容を `base`・新しい input を
//! `frag` として呼ぶ（`format` × per-step `merge` が重ね方を選ぶ）。値の格納形式ごとに子モジュール
//! （[`text`] / [`json`] / [`plist`]）へ分かれ、本ファイルは束ねる入口に徹する。
//!
//! いずれの関数も副作用のない純関数で、配置（書き込み）は [`crate::apply::pipeline`] が行う。
//! `base = None` で呼ぶと `frag` そのもの（再直列化）を返す ― これにより step 列の最初の input
//! （土台なし）と 2 つ目以降（土台あり）を同じ関数で畳める（[`crate::apply::pipeline`] の
//! `fold_in`）。パースエラーのラベル付け（どの input か）は呼び出し元が担い、各関数は `base` 由来の
//! エラーだけ `base` と前置する（`frag` 由来はそのまま返す）。

pub(crate) mod json;
pub(crate) mod plist;
pub(crate) mod text;
