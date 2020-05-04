<a href="LICENSE" alt="MIT License"><img alt="GitHub" src="https://img.shields.io/github/license/toshiki670/dotfiles?style=flat-square"></a>

# Overview
- Simplification of environment construction
- unification of environment

# Install
`$ ./install`

# Tools
## arch-update
- Archlinux を最新化

## chown-current-user
- 指定したディレクトリ/ファイルを現在のユーザに変更

## gcm
- `git commit -m`のエイリアス
- Gitのコミットメッセージにブランチ名を付加

## git-upstream
- fork したプロジェクトでupstreamの変更の取込み

# Version History
## 1.6.0
- Archlinuxのインストール時に利用するファイルを分離
- `chown-current-user` をアップデート
- Bash のエイリアス追加
- `arch-update`のアップデート処理を個別で実行できるように変更
  * Neovim のアップデート機能を追加
- Bugfix: Nvimdiff を有効化

## 1.5.2
- Deoplete の非推奨設定を修正

## 1.5.1
- 誤記修正

## 1.5.0
- git flow のショートカットを追加
- .zshrc の整理
- zsh に `Command not found` ホック機能を追加
- Archlinuxを最新にする `arch-update` コマンドを追加
- `git difftool` と `git mergetool` で Neovim を使うように変更

## 1.4.0
- git-upstream に初期化機能を追加

## 1.3.0
- Git flow の導入
