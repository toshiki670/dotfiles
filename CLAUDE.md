# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## リポジトリ構造

`dotfiles` CLI で管理。`configs/` がソースで `dotfiles apply` でホームディレクトリへデプロイされる（各ツールの `configs/<tool>/manifest.toml` が配置方式・配置先を宣言）。配置後の onchange hooks も manifest に宣言（bat cache 再構築・ghostty symlink 等）。バイナリの導入は apply の外で `cargo install --git`（＋ `upgrade-env`）が担う。

CLI コマンドはリポジトリルートの **Cargo workspace**。配布物は root package `dotfiles` 1 つに統合される。`dotfiles` lib は本体（`dotfiles` コマンド）のモジュールツリーそのもの（`src/apply.rs` `src/manifest.rs` 等がクレート直下、`src/cli.rs` が CLI 定義とディスパッチ）。`clip` / `gcm` / `gh-clone` / `git-upstream` / `fzf-picker` / `env-tools` / `workers` は、いずれも `src/bin/<name>/main.rs` を起点に自分専用のモジュールツリー（`cli.rs` や各サブコマンドの module）を持つ、`dotfiles` lib に属さない独立した bin。fzf-picker / env-tools / workers は複数コマンドを1 bin へサブコマンド化したもの。`dotfiles` lib に同居させない（rustdoc の crate root がシムだけの重複ページで埋まらないようにするため）。version は単一・SoT はタグ `v{version}`。root package は一度の `cargo install --git <repo>` で `~/.cargo/bin` へ配布する（apply の外・更新は `upgrade-env` も担う）。開発・保守ツールは `tools/` 配下で非配布・版なし: lint/format オーケストレータ `dotfiles-lint`（`mise run lint` / `check` → `cargo run -p dotfiles-lint`、mise 供給の shfmt / shellcheck / taplo / stylua / rumdl / ruff を呼ぶ）、Neovim プラグイン同期 `v-sync`（`mise run v-sync`）。リリースは release-plz（単一版の git タグ + GitHub Release、crates.io へは publish しない）。

- 設計・内部構造 → rustdoc（<https://toshiki670.github.io/dotfiles/>。ソースは各モジュールの doc コメント）
- セットアップ・ツール一覧・Rust コマンド → [README.md](README.md)
- lint/check・テスト・リリース手順 → [CONTRIBUTING.md](CONTRIBUTING.md)
- バージョニングルール → [CONTRIBUTING.md](CONTRIBUTING.md#リリースプロセス)
- カラーテーマ設定の一覧・変更手順 → [COLOR.md](COLOR.md)
