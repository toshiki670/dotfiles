# Changelog

## [0.69.2] - 2026-06-17
### Fixes
- Root の Release 名から dotfiles- 接頭辞を外す ([#431](https://github.com/toshiki670/dotfiles/pull/431)) ([`72c6cf6`](https://github.com/toshiki670/dotfiles/commit/72c6cf6b43d2890191713be43ca337aa857112d0))


## [0.69.1] - 2026-06-16
### Features
- _fzf_ghq_cd を Rust 化（B群 Rust化 PR1/3） ([#413](https://github.com/toshiki670/dotfiles/pull/413)) ([`a041941`](https://github.com/toshiki670/dotfiles/commit/a041941bdc23a195a816dd496376d504c289646a))
- _fzf_worktree_remove を Rust 化（B群 Rust化 PR2/3） ([#414](https://github.com/toshiki670/dotfiles/pull/414)) ([`0884b97`](https://github.com/toshiki670/dotfiles/commit/0884b973f5e5f4d4ea0e5b47c0ac8ad929bfae9c))
- Cdabbr を Rust 化（B群 Rust化 PR3/3・完了） ([#415](https://github.com/toshiki670/dotfiles/pull/415)) ([`ef312d0`](https://github.com/toshiki670/dotfiles/commit/ef312d08243cfdc7ee23888f4669e39378847043))
- メモリ棚卸し Skill memory-tidy を追加 ([#420](https://github.com/toshiki670/dotfiles/pull/420)) ([`4d8ce5c`](https://github.com/toshiki670/dotfiles/commit/4d8ce5ccb7b36f8aff53efcf8ff863e99e827520))
- Cleanup-env・upgrade-env を Rust 化 ([#390](https://github.com/toshiki670/dotfiles/pull/390)) ([#421](https://github.com/toshiki670/dotfiles/pull/421)) ([`876dacd`](https://github.com/toshiki670/dotfiles/commit/876dacd14aff91ca396dd12a1c5441a15f007c85))
### Fixes
- Per-package CHANGELOG.md も markdown lint から除外 ([#411](https://github.com/toshiki670/dotfiles/pull/411)) ([`336fe09`](https://github.com/toshiki670/dotfiles/commit/336fe0996ec01f0d71283c9426651ed2d167ba16))


## [0.69.0] - 2026-06-16

Rust / Cargo workspace 化のブートストラップリリース（epic #388）。シェル関数群を Rust バイナリへ移行し、release-plz による per-package バージョニングを確立した。以降のフォーマットは release-plz（git-cliff）に統一される。

### Features

- 各コマンドを Rust バイナリ化し、`cargo install` × chezmoi で配布（color / copy-obj / v-sync / gh-clone / gcm / git-upstream、背景 worker の daily-check / git-background-fetch）
- lint/format オーケストレータを Rust 化（rumdl / mise でツール供給）
- Rust edition を 2024 に更新

### Refactor

- 単一利用モジュールを lib から各 bin 配下へ移動

### Build

- リリースを release-please から release-plz（git_only・per-package タグ）へ移行
- Nix を撤去し lint を cargo + mise に一本化

## [0.68.0](https://github.com/toshiki670/dotfiles/compare/v0.67.0...v0.68.0) (2026-06-10)


### Features

* **fish:** bat + fzf のファイル選択ショートカット bf を追加 ([#377](https://github.com/toshiki670/dotfiles/issues/377)) ([3443a13](https://github.com/toshiki670/dotfiles/commit/3443a13c81fc3917cdd80becf304e3ccbf39b2f0))


### Bug Fixes

* **claude:** use rtk hook claude instead of legacy rtk-rewrite.sh ([#374](https://github.com/toshiki670/dotfiles/issues/374)) ([6760f92](https://github.com/toshiki670/dotfiles/commit/6760f927475b4b40d999cac737ef23f188351b9d))
* **fish:** gcm でフェンス付き/単一オブジェクトの JSON 応答を許容 ([#376](https://github.com/toshiki670/dotfiles/issues/376)) ([5bb5b18](https://github.com/toshiki670/dotfiles/commit/5bb5b182db650d778349832ae023001d8bfddb78))

## [0.67.0](https://github.com/toshiki670/dotfiles/compare/v0.66.1...v0.67.0) (2026-06-03)


### Features

* **fish:** nvim プラグインを同期する v-sync 関数を追加 ([#372](https://github.com/toshiki670/dotfiles/issues/372)) ([2aae33f](https://github.com/toshiki670/dotfiles/commit/2aae33fd906a64307e7d864a6d05e7ba81bc0b5f))

## [0.66.1](https://github.com/toshiki670/dotfiles/compare/v0.66.0...v0.66.1) (2026-06-03)


### Bug Fixes

* **fish:** gcm のプロンプトでコードフェンス出力を抑止する ([#366](https://github.com/toshiki670/dotfiles/issues/366)) ([6b66039](https://github.com/toshiki670/dotfiles/commit/6b66039ebd2c3310e1d6958e6db7574823ee6013))

## [0.66.0](https://github.com/toshiki670/dotfiles/compare/v0.65.0...v0.66.0) (2026-06-03)


### Features

* **fish:** gh targeted commands に Fish ネイティブ補完を追加 (alternative) ([#363](https://github.com/toshiki670/dotfiles/issues/363)) ([062a415](https://github.com/toshiki670/dotfiles/commit/062a415e564282427fcbd8dad0affa4c60817a67))
* **ghostty:** カーソルを常に block 表示にする ([#365](https://github.com/toshiki670/dotfiles/issues/365)) ([8b556b4](https://github.com/toshiki670/dotfiles/commit/8b556b475daf9d1ed343ecdb05465b1157822e53))

## [0.65.0](https://github.com/toshiki670/dotfiles/compare/v0.64.0...v0.65.0) (2026-06-02)


### Features

* **fish:** add AI-powered gcm for Conventional Commits ([#358](https://github.com/toshiki670/dotfiles/issues/358)) ([59ab0eb](https://github.com/toshiki670/dotfiles/commit/59ab0ebbb4b61093d2f5a352ef9a623e2b57d86b))


### Code Refactoring

* **fish:** gh abbreviation をグローバル略語に変更 ([#360](https://github.com/toshiki670/dotfiles/issues/360)) ([f89131f](https://github.com/toshiki670/dotfiles/commit/f89131fdc759b5ade1c2ec04a0b56163a081b178))

## [0.64.0](https://github.com/toshiki670/dotfiles/compare/v0.63.0...v0.64.0) (2026-06-02)


### Features

* Fish でディレクトリ移動後に自動で ls (eza) する ([#355](https://github.com/toshiki670/dotfiles/issues/355)) ([59918b5](https://github.com/toshiki670/dotfiles/commit/59918b59706dbad84299568e814bcd96f1113119))
* fzf のカラーをシステム外観追従（light/dark）に対応 ([#356](https://github.com/toshiki670/dotfiles/issues/356)) ([f573a97](https://github.com/toshiki670/dotfiles/commit/f573a976894a60a83c79f54a7b6891ec4740d6ab))
* カラーテーマをシステム外観追従（light:One Half Light / dark:Ayu）に統一 ([#353](https://github.com/toshiki670/dotfiles/issues/353)) ([e70c7dc](https://github.com/toshiki670/dotfiles/commit/e70c7dc872c3405347d2eec451755354544f38ba))


### Code Refactoring

* fzf fish 関数を実態に合わせてリネーム ([#357](https://github.com/toshiki670/dotfiles/issues/357)) ([1828842](https://github.com/toshiki670/dotfiles/commit/1828842af321c33b48902faac403710927772d58))

## [0.63.0](https://github.com/toshiki670/dotfiles/compare/v0.62.3...v0.63.0) (2026-05-31)


### Features

* ghq/worktree ピッカーを統合し WT 削除ショートカットを追加 ([#349](https://github.com/toshiki670/dotfiles/issues/349)) ([5378ac9](https://github.com/toshiki670/dotfiles/commit/5378ac9e1db1980b3e2801167de66a247558308f))


### Bug Fixes

* nvim checkhealth の警告・エラーを解消しプラグインを更新 ([#351](https://github.com/toshiki670/dotfiles/issues/351)) ([f68cd71](https://github.com/toshiki670/dotfiles/commit/f68cd711d7109308e9dff6df4d63d0120b9d942f))

## [0.62.3](https://github.com/toshiki670/dotfiles/compare/v0.62.2...v0.62.3) (2026-05-30)


### Bug Fixes

* fzf キャンセル時にプロンプトが消える問題を修正 ([#346](https://github.com/toshiki670/dotfiles/issues/346)) ([f19a12a](https://github.com/toshiki670/dotfiles/commit/f19a12a33adb39c1439dff09b7b97a8f40156a79))

## [0.62.2](https://github.com/toshiki670/dotfiles/compare/v0.62.1...v0.62.2) (2026-05-26)


### Bug Fixes

* mise モジュールを無効化して Starship プロンプト呼び出しを削減 ([#344](https://github.com/toshiki670/dotfiles/issues/344)) ([85af381](https://github.com/toshiki670/dotfiles/commit/85af381c31b4f1ce176fafe8244a147eec04acc2))

## [0.62.1](https://github.com/toshiki670/dotfiles/compare/v0.62.0...v0.62.1) (2026-05-23)


### Bug Fixes

* mise の二重 activate を解消し fish 起動を高速化 ([#342](https://github.com/toshiki670/dotfiles/issues/342)) ([d96a43d](https://github.com/toshiki670/dotfiles/commit/d96a43d917ad7a1c2c227b1a81785ea96ab7598e))

## [0.62.0](https://github.com/toshiki670/dotfiles/compare/v0.61.0...v0.62.0) (2026-05-20)


### Features

* merge-ready の format を複数 PR 対応の $pr_ids に変更 ([#339](https://github.com/toshiki670/dotfiles/issues/339)) ([557529b](https://github.com/toshiki670/dotfiles/commit/557529b2201c53da3708b1cc0326f91720d0ec48))

## [0.61.0](https://github.com/toshiki670/dotfiles/compare/v0.60.0...v0.61.0) (2026-05-20)


### Features

* ~/.claude/settings.json を modify_ script 化してローカル編集値を保持 ([#336](https://github.com/toshiki670/dotfiles/issues/336)) ([#337](https://github.com/toshiki670/dotfiles/issues/337)) ([1f5043f](https://github.com/toshiki670/dotfiles/commit/1f5043f81dbc8fdb65d34f65d81fa4ea582c3a59))

## [0.60.0](https://github.com/toshiki670/dotfiles/compare/v0.59.0...v0.60.0) (2026-05-17)


### Features

* bin に cleanup-env コマンドを追加 ([#334](https://github.com/toshiki670/dotfiles/issues/334)) ([6fb8f35](https://github.com/toshiki670/dotfiles/commit/6fb8f35a98d268e12a21750705c0cfe7695e6af0))

## [0.59.0](https://github.com/toshiki670/dotfiles/compare/v0.58.2...v0.59.0) (2026-05-17)


### Features

* starship の gcloud に account/domain/region 表示の format を追加 ([#332](https://github.com/toshiki670/dotfiles/issues/332)) ([781c3d4](https://github.com/toshiki670/dotfiles/commit/781c3d4b65d3c54fb0edc9f8bce0609e94af73f9))

## [0.58.2](https://github.com/toshiki670/dotfiles/compare/v0.58.1...v0.58.2) (2026-05-17)


### Code Refactoring

* ホスト側削除完了後の .chezmoiremove.tmpl を撤去 ([#270](https://github.com/toshiki670/dotfiles/issues/270), [#325](https://github.com/toshiki670/dotfiles/issues/325)) ([#328](https://github.com/toshiki670/dotfiles/issues/328)) ([f7d39f0](https://github.com/toshiki670/dotfiles/commit/f7d39f06a9b338a52868f29ee1c929747a9f9ee3))

## [0.58.1](https://github.com/toshiki670/dotfiles/compare/v0.58.0...v0.58.1) (2026-05-17)


### Code Refactoring

* Zsh 専用設定 (sheldon/zeno) を chezmoi 管理から外す (1/2) ([#326](https://github.com/toshiki670/dotfiles/issues/326)) ([bbd9fca](https://github.com/toshiki670/dotfiles/commit/bbd9fca1ad8503eb0097e374c506e423a82e914f))
* Zsh 設定を chezmoi 管理から外しホスト側削除を予告 (1/2) ([#323](https://github.com/toshiki670/dotfiles/issues/323)) ([06a8620](https://github.com/toshiki670/dotfiles/commit/06a862058be991128557b5c23154ebf18de44ca3))

## [0.58.0](https://github.com/toshiki670/dotfiles/compare/v0.57.0...v0.58.0) (2026-05-17)


### Features

* Ctrl-j Ctrl-w で worktree を fzf 切り替えできるようにする ([#318](https://github.com/toshiki670/dotfiles/issues/318)) ([#319](https://github.com/toshiki670/dotfiles/issues/319)) ([af43854](https://github.com/toshiki670/dotfiles/commit/af4385463dd4e13943015946ad7c23dbd68fde1f))
* Fish に ps-grep 関数を追加 ([#322](https://github.com/toshiki670/dotfiles/issues/322)) ([dbdfef9](https://github.com/toshiki670/dotfiles/commit/dbdfef9d3e61b3e10e455e66f6d0b87549385b87))

## [0.57.0](https://github.com/toshiki670/dotfiles/compare/v0.56.0...v0.57.0) (2026-05-16)


### Features

* chezmoi apply からパッケージ更新を分離し upgrade-env コマンドを追加 ([#317](https://github.com/toshiki670/dotfiles/issues/317)) ([19c2594](https://github.com/toshiki670/dotfiles/commit/19c259425ba35bba378d82d6ce358d2e92ae6e17))
* merge-ready の fish completion を追加 ([#313](https://github.com/toshiki670/dotfiles/issues/313)) ([5a2330d](https://github.com/toshiki670/dotfiles/commit/5a2330d063fcf4aceec2b03fcef15be606f95bd5))
* merge-ready の全ステータスの format に PR ID を追加 ([#314](https://github.com/toshiki670/dotfiles/issues/314)) ([a3dafaa](https://github.com/toshiki670/dotfiles/commit/a3dafaabc64b2eec5ec9c66eb7ab613be5d286b0))


### Bug Fixes

* browser-use の PATH を末尾追加にして mise Python を優先 ([#279](https://github.com/toshiki670/dotfiles/issues/279)) ([#316](https://github.com/toshiki670/dotfiles/issues/316)) ([f7f9e25](https://github.com/toshiki670/dotfiles/commit/f7f9e25053916a6f2d596109c96c7a1a171e1484))

## [0.56.0](https://github.com/toshiki670/dotfiles/compare/v0.55.0...v0.56.0) (2026-05-02)


### Features

* merge-ready の全ステータスに format と新規ステート追加 ([#312](https://github.com/toshiki670/dotfiles/issues/312)) ([cae016b](https://github.com/toshiki670/dotfiles/commit/cae016bd68c2f87e3b7b6386ff8e67077aca2b3e))


### Bug Fixes

* gpup abbr に git pull を追加 ([#310](https://github.com/toshiki670/dotfiles/issues/310)) ([d2bb535](https://github.com/toshiki670/dotfiles/commit/d2bb535d2b29efcca31f3310840493b23c5c3e17)), closes [#309](https://github.com/toshiki670/dotfiles/issues/309)

## [0.55.0](https://github.com/toshiki670/dotfiles/compare/v0.54.2...v0.55.0) (2026-04-29)


### Features

* add gpup abbr for gh pr update-branch ([#303](https://github.com/toshiki670/dotfiles/issues/303)) ([2a3abad](https://github.com/toshiki670/dotfiles/commit/2a3abadeff0cdcc8e6d331aa7651762e1ada6d30))
* replace manual release flow with release-please automation ([#304](https://github.com/toshiki670/dotfiles/issues/304)) ([cb59ddf](https://github.com/toshiki670/dotfiles/commit/cb59ddfda4e4d01ea8bcfe2e2aa761208ea02510))


### Bug Fixes

* add workflow_dispatch trigger to release-please workflow ([#306](https://github.com/toshiki670/dotfiles/issues/306)) ([875bedf](https://github.com/toshiki670/dotfiles/commit/875bedf769334e17cfe35d89b0cc850ce3bfb220))
* prepend Homebrew to fish PATH ([#301](https://github.com/toshiki670/dotfiles/issues/301)) ([04ed35c](https://github.com/toshiki670/dotfiles/commit/04ed35c1662d9f6a46c2c8e10cf95dcea3fc8161))
* use PAT token for release-please to trigger CI on release PRs ([#307](https://github.com/toshiki670/dotfiles/issues/307)) ([69c8e7f](https://github.com/toshiki670/dotfiles/commit/69c8e7f03b1a6006ef686f19aca9ec9a0542b1fd))
