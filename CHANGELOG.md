# Changelog

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
