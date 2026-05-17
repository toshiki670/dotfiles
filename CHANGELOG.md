# Changelog

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
