//! `dotfiles` は `configs/` に置いた設定を、各設定単位の `manifest.toml` の宣言に従って
//! ホームディレクトリへ配置・管理する。バージョンの一次情報はタグ `v{version}`。
//! `--version` / `--help` で確認できる。
//!
//! ほか、いくつかの小さな CLI も同じパッケージで配布する。`cargo install --git <repo>`
//! で全コマンドが入る。
//!
//! ソースは `configs/` に「中身の帰属」（どのツールの設定か）で並べ、配置先はツリー上の
//! 位置でなく manifest の `[[steps]]` の `output` で宣言する。配置先が同じでも出所のツールが
//! 違えば単位は分かれ、複数ツールの断片が 1 つの配置先へ合流できる（fish の `conf.d/` 等）。
//!
//! # 配置モデルの用語
//!
//! 以下の doc は、ここで定義する用語を前提に書く。
//!
//! - **設定単位（ユニット）**: `manifest.toml` を持つ `configs/` 配下のディレクトリ。
//!   配置宣言とソースをひとまとめにした、走査・gate・フックの単位。
//! - **文書 D**: 1 ユニットが `[[steps]]` を上から評価しながら組み立てる内容。空から始まり、
//!   `input` step で内容を畳み、`output` step で書き出される。
//! - **step**: `input`（読む）か `output`（書く）の択一。どちらも「パス文字列」か
//!   「`cmd`（argv・標準入出力）」の択一（[`manifest::StepSource`]）。特例としてパス `"."` は
//!   単位ディレクトリ丸ごと（ツリー）を表す。
//! - **`merge`**: 2 つ目以降の `input` に必須の重ね方注釈（`shallow`＝トップレベルキー単位の
//!   後勝ち / `append`＝テキスト連結）。実行時の畳み込みはユニット単位の `format` だけで駆動し、
//!   `merge` は load 時の整合検証のためだけに存在する（[`apply::pipeline`]）。
//! - **`format`**: 文書 D の内容型（`json` / `plist` / `text`）。`merge` を使うユニットに必須。
//! - **gate / `when`**: 採用条件（`deps`＝コマンドの有無 / `os` / `profile`）。ユニット直下に
//!   書けばユニット全体を、step 内に書けばその step だけを gate する。false の意味は階層で
//!   異なる: ユニット gate=false は配置先ごと作らない、step の when=false はその step だけ脱落
//!   （D は不変のまま次の step へ進む）。
//! - **`optional`**: パス `input` が存在しなくてもエラーにせず D を素通しする（次の input が
//!   土台になる）。既定は「無ければエラー」。
//! - **profile**: user が一度選んでおくマシンクラス状態（例 `private`）。`when.profile` が読む。
//! - **locals（named value）**: マシンローカル値。manifest が名前を宣言し、apply が配置時に
//!   ストアの値を `@@name@@` placeholder へ注入する。
//! - **hooks**: ユニット配置後に実行するコマンド列（onchange＝ソース変化時のみ実行）。生きた
//!   外部状態への反映は hooks でなく `output.cmd` step が担う（毎 apply・冪等が契約）。
//!
//! # apply の流れ
//!
//! ソースを二段構えで解決し（作業ツリー検出 → バイナリ埋め込み・[`source`]）、ユニットを
//! 走査（[`discover`]）→ ユニット gate を評価（[`apply::gate`]）→ `[[steps]]` を実行して
//! 文書 D を組み立て配置（[`apply::pipeline`]、cmd 実行は [`apply::cmd`]）→ locals を解決・注入
//! （[`locals`]）→ 配置後フックを実行（[`hooks`]）、の順に進む。

// `deny(broken_intra_doc_links)`: doc コメントのリンク切れを CI の `cargo doc -p dotfiles` で
// 検出するガード。`allow(private_intra_doc_links)`: 既定では公開アイテムの doc から非公開
// アイテムへのリンクは警告になる（外部利用者はリンク先を読めない前提のため）。`dotfiles` は
// ライブラリとして公開しない内部 crate（bin だけを配布し、crates.io へは publish しない）で、
// 公開 rustdoc も `--document-private-items` 付きでビルドするため、module doc から非公開の
// 子モジュールへの構造ナビ用リンクは実害が無く許容する。
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::private_intra_doc_links)]

// 共有核（葉。多くの群が片方向で依存する契約・基盤）。
mod discover;
mod manifest;
mod source;
mod state;

// 配置エンジン。子モジュール（copy / pipeline / cmd / strategy / gate）は apply.rs が束ねる。
mod apply;

// named value。子モジュール（store / resolve / inject / prompt）は locals.rs が束ねる。
mod locals;

// 配置後フック。子モジュール（exec / onchange）は hooks.rs が束ねる。
mod hooks;

// 単独ビュー / コマンド。
mod color;
mod doctor;
mod list;
mod local;
mod profile;

// CLI 定義とディスパッチ。
mod cli;

fn main() {
    cli::run();
}
