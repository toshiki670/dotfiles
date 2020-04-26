# ログインシェルとインタラクティブシェルの場合だけ読み込まれる。
# シェルスクリプトでは不要な場合に記述する。
# 
export DOTFILES="${HOME%/}/dotfiles"


function require() {
  dir="${DOTFILES}/zsh/${1}"
  if [[ ! -f $dir ]]; then
    echo "${0##*/}: LoadError (cannot load such file -- \`${dir}')." 1>&2
    return 1
  fi
  . "$dir"
  return 0
}


export PATH="/usr/local/sbin:$PATH"
export PATH="${DOTFILES}/bin:$PATH"

if type "brew" > /dev/null 2>&1; then
  export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
fi

# For pyenv
# export PYENV_ROOT="$HOME/.pyenv"
# export PATH="$PYENV_ROOT/bin:$PATH"
# eval "$(pyenv init --no-rehash -)"


# For comporser (Laravel)
# export PATH=$PATH:~/.composer/vendor/bin


# Setting completions
require 'zshrc/docker.zsh'


# Add directory
if [ -e ${completions} ]; then
  fpath=(${completions} $fpath)
fi


# Initialize and Install the Zplug
require "zshrc/zplug.zsh"

# ls or exa command config
require 'zshrc/ls.zsh'

# ruby and rails config
require 'zshrc/ruby.zsh'

# git config
require 'zshrc/git.zsh'

# zsh-users/zsh-autosuggestions
# https://github.com/zsh-users/zsh-autosuggestions
# ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=244'


# 補完機能
bindkey "^[[Z" reverse-menu-complete

# complete 普通の補完関数; approximate ミススペルを訂正した上で補完を行う。; prefixカーソルの位置で補完を行う
zstyle ':completion:*' completer _complete _prefix #_approzimate

# 多部補完時に大文字小文字を区別しない
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# タブを１回押すと、補完候補が表示され、さらにタブを押すことで、選択モードに入る
zstyle ':completion:*:default' menu select=2
if [ -n $LS_COLORS ]; then
  zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
fi

# `Command not found' hook
if type 'pkgfile' > /dev/null 2>&1; then
  source /usr/share/doc/pkgfile/command-not-found.zsh
fi

# Printable 8bit
setopt print_eight_bit
setopt auto_cd
setopt auto_pushd
setopt correct

# Reload
alias reload='exec $SHELL -l'


# グローバルIPアドレス確認
alias ipecho='curl ipecho.net/plain; echo'

# For vim
alias vim=nvim
alias v=vim
alias vim-utf8='vim -c ":e ++enc=utf8"'
alias vim-euc_jp='vim -c ":e ++enc=euc-jp"'
alias vim-shift_jis='vim -c ":e ++enc=shift_jis"'
alias vim-cheat="vim ${DOTFILES}/vim/cheatsheet/common.md"
# alias eclipse='open -a eclipse -data /User/tsk/Documents/workspace &'

# 拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s py='python'
alias -s php='php -f'

function rungcc(){
    gcc $1
    base=$1
    file=${base%.*}
    ./a.out
    rm -f a.out
}

alias -s {c,cpp}=rungcc

# Dotfiles Config
alias vimrc="vim ${DOTFILES}/vim/.vimrc"
alias zshrc="vim ${DOTFILES}/zsh/.zshrc"

# 履歴ファイルの保存先
export HISTFILE=${HOME}/.zsh_history

# メモリに保存される履歴の件数
export HISTSIZE=3072

# 履歴ファイルに保存される履歴の件数
export SAVEHIST=1000000

# 重複を記録しない
setopt hist_ignore_dups

# 開始と終了を記録
setopt EXTENDED_HISTORY

# 全履歴を一覧表示する
function history-all { history -E 1 }

# Process grep
function ps-grep {
  ps aux | grep $1 | grep -v grep
}


# ターミナル起動時に実行

# ZSHの起動した関数の時間計測 .zshenv参照
# if (which zprof > /dev/null 2>&1) ;then
#   zprof
# fi

