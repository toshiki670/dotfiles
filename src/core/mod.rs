//! `dotfiles` 本体（core）。リポジトリの `configs/` に置いた設定を、各設定単位の
//! `manifest.toml` の宣言に従ってホームディレクトリへ配置・管理する。
//! バージョンの一次情報はタグ `v{version}`。`--version` / `--help` で確認できる。
//!
//! ソースは `configs/` に「中身の帰属」（どのツールの設定か）で並べ、配置先はツリー上の
//! 位置でなく manifest の属性（`dst`）で宣言する。配置先が同じでも出所のツールが違えば
//! 単位は分かれ、複数ツールの断片が 1 つの配置先へ合流できる（fish の `conf.d/` 等）。
//!
//! # 配置モデルの用語
//!
//! core 以下の doc は、ここで定義する用語を前提に書く。
//!
//! - **設定単位（ユニット）**: `manifest.toml` を持つ `configs/` 配下のディレクトリ。
//!   配置宣言とソースをひとまとめにした、走査・gate・フックの単位。
//! - **断片**: 配置内容のもと（ソースファイル、またはコマンド出力）。
//! - **生成方式 `kind`**: 1 つの断片をどう実体化するか（`copy`＝ファイルをそのまま書き出す /
//!   `generate`＝コマンド出力を書き出す）。
//! - **合成 `strategy`**: 複数の断片を 1 つの配置先へどう重ねるか（`concat`＝連結 /
//!   `json-shallow`＝JSON トップレベルキー単位の後勝ちマージ / `plist-shallow`＝その plist 版）。
//!   生成方式と合成は独立に選べる。この独立な 2 つを「**2軸**」と呼ぶ。
//! - **overlay**: `when` 条件付きの断片。配置先は「土台＋条件を満たした overlay の重なり」
//!   として合成される。
//! - **gate / `when`**: 採用条件（`deps`＝コマンドの有無 / `os` / `profile`）。ユニット直下に
//!   書けばユニット全体を、overlay 内に書けばその断片だけを gate する。false の意味は階層で
//!   異なる: ユニット gate=false は配置先ごと作らない、overlay=false はその断片だけ脱落。
//! - **土台 / preserve**: 合成の最下層。`preserve = true` は既存の配置先ファイルを土台として
//!   温存し、dotfiles が所有するキーだけを上書きする（ローカルなキーは残る）。
//! - **profile**: user が一度選んでおくマシンクラス状態（例 `private`）。`when.profile` が読む。
//! - **locals（named value）**: マシンローカル値。manifest が名前を宣言し、apply が配置時に
//!   ストアの値を `@@name@@` placeholder へ注入する。
//! - **hooks**: ユニット配置後に実行するコマンド列。頻度は `onchange`（ソース変化時のみ）か
//!   `always`（毎 apply・冪等＝何度実行しても同じ結果になるコマンドが前提）。
//!
//! # apply の流れ
//!
//! ソースを二段構えで解決し（作業ツリー検出 → バイナリ埋め込み・[`source`]）、ユニットを
//! 走査（[`discover`]）→ ユニット gate を評価（[`apply::gate`]）→ 断片を生成・合成
//! （[`apply`] / [`apply::compose`]）→ locals を解決・注入（[`locals`]）→ 配置 → 配置後
//! フックを実行（[`hooks`]）、の順に進む。

// 共有核（葉。多くの群が片方向で依存する契約・基盤）。
mod discover;
mod manifest;
mod source;
mod state;

// 配置エンジン。子モジュール（copy / compose / generate / strategy / gate）は apply.rs が束ねる。
mod apply;

// named value。子モジュール（store / resolve / inject / prompt）は locals.rs が束ねる。
mod locals;

// 配置後フック（#546）。子モジュール（exec / onchange）は hooks.rs が束ねる。
mod hooks;

// 単独ビュー / コマンド。
mod color;
mod doctor;
mod list;
mod local;
mod profile;

// CLI 定義とディスパッチ。`src/bin/dotfiles.rs` の数行シムから [`cli::run`] が呼ばれる。
pub mod cli;
