# ログインシェルとインタラクティブシェルの場合だけ読み込まれる。
# シェルスクリプトでは不要な場合に記述する。
# 
export DOTFILES=~/dotfiles

export PATH="/usr/local/sbin:$PATH"
export PATH="$HOME/dotfiles/bin:$PATH"

if type "brew" > /dev/null 2>&1; then
  export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
fi

# For pyenv
# export PYENV_ROOT="$HOME/.pyenv"
# export PATH="$PYENV_ROOT/bin:$PATH"
# eval "$(pyenv init --no-rehash -)"


# For comporser (Laravel)
# export PATH=$PATH:~/.composer/vendor/bin


# For rbenv
if type "rbenv" > /dev/null 2>&1; then
  eval "$(rbenv init --no-rehash -)";
  export PATH="$HOME/.rbenv/shims:$PATH"
fi

# For gem
if type "gem" > /dev/null 2>&1; then
  PATH="$(ruby -e 'print Gem.user_dir')/bin:$PATH"
fi

if [ -e $DOTFILES/zsh/completions ]; then
  fpath=($DOTFILES/zsh/completions $fpath)
fi

# aotoload設定一覧 (Zplugが入っている場合無効)
# export ZPLUG_HOME=/usr/local/opt/zplug
export ZPLUG_HOME=$DOTFILES/zsh/plugin/zplug
export ZPLUG_BIN=$ZPLUG_HOME/bin
export ZPLUG_CACHE_DIR=$ZPLUG_HOME/cache
export ZPLUG_REPOS=$ZPLUG_HOME/repos

# Initialize and Install the Zplug
if [ -e $ZPLUG_HOME ]; then

  # Zplug の有効化
  source $ZPLUG_HOME/init.zsh
  zplug "zsh-users/zsh-completions"
  zplug "zsh-users/zsh-syntax-highlighting"
  zplug "zsh-users/zsh-autosuggestions"
  zplug "mafredri/zsh-async", from:github
  zplug "sindresorhus/pure", use:pure.zsh, from:github, as:theme
  zplug "mollifier/cd-gitroot"
  zplug "Tarrasch/zsh-bd"
  zplug "supercrabtree/k"
  # zplug "docker/cli", use:"contrib/completion/zsh/_docker"
  # zplug "starcraftman/zsh-git-prompt"

  # プラグイン追加後、下記を実行する
  zplug check || zplug install
  zplug load

elif type "git" > /dev/null 2>&1; then

  # Install the Zplug
  git clone https://github.com/zplug/zplug $ZPLUG_HOME
fi


ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=244'


# Theme configure
# eval `/usr/local/opt/coreutils/libexec/gnubin/dircolors ~/.dircolors-solarized/dircolors.ansi-dark`
eval $(dircolors $DOTFILES/zsh/bundle/color/dircolors-solarized)
eval $(dircolors $DOTFILES/zsh/bundle/color/dircolors-solarized/dircolors.ansi-universal)

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

# Printable 8bit
setopt print_eight_bit
setopt auto_cd
setopt auto_pushd
setopt correct

alias relogin='exec $SHELL -l'
alias ls='ls --color=auto'
alias ll='ls -l'
alias la='ls -a'
alias lla='ls -la'

# For git
alias g='git'
alias gad='git add'
alias gcm='git commit'
alias gb='git branch'
alias gch='git checkout'
alias gd='git diff'
alias gs='git status'
alias gpull='git pull'
alias gpullre='git pull --rebase'
alias gpush='git push'
alias glog="git log --graph --all --pretty=format:'%C(yellow)%h%C(cyan)%d%Creset %s %C(white)- %an, %ar%Creset'"
alias g-reset-hard='git reset --hard HEAD'


# For PHP
alias xam='cd /Applications/XAMPP/xamppfiles/htdocs/php/'

# For Rails
alias be='bundle exec'

# For docker
alias dce='docker-compose exec'

# For Note
alias note='cd ~/Documents/Note'

# グローバルIPアドレス確認
alias ipecho='curl ipecho.net/plain; echo'

# For vim
alias vim=nvim
alias v=vim
alias vim-utf8='vim -c ":e ++enc=utf8"'
alias vim-euc_jp='vim -c ":e ++enc=euc-jp"'
alias vim-shift_jis='vim -c ":e ++enc=shift_jis"'
alias vim-cheat='vim $DOTFILES/vim/cheatsheet/common.md'
# alias eclipse='open -a eclipse -data /User/tsk/Documents/workspace &'

# 拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s rb='ruby'
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
alias vimrc='vim $DOTFILES/vim/.vimrc'
alias zshrc='vim $DOTFILES/zsh/.zshrc'

# 履歴ファイルの保存先
export HISTFILE=${HOME}/.zsh_history

# メモリに保存される履歴の件数
export HISTSIZE=1000

# 履歴ファイルに保存される履歴の件数
export SAVEHIST=100000

# 重複を記録しない
setopt hist_ignore_dups

# 開始と終了を記録
setopt EXTENDED_HISTORY

# 全履歴を一覧表示する
function history-all { history -E 1 }

function ps-grep {
  ps aux | grep $1 | grep -v grep
}


# ターミナル起動時に実行

# ZSHの起動した関数の時間計測 .zshenv参照
# if (which zprof > /dev/null 2>&1) ;then
#   zprof
# fi

