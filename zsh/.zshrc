# ログインシェルとインタラクティブシェルの場合だけ読み込まれる。
# シェルスクリプトでは不要な場合に記述する。
# 
export PATH="$HOME/dotfiles/bin:$PATH"

export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
# For pyenv
# export PYENV_ROOT="$HOME/.pyenv"
# export PATH="$PYENV_ROOT/bin:$PATH"
# eval "$(pyenv init --no-rehash -)"


# For comporser (Laravel)
# export PATH=$PATH:~/.composer/vendor/bin


# For rbenv
eval "$(rbenv init --no-rehash -)";
export PATH="$HOME/.rbenv/shims:$PATH"


# aotoload設定一覧 (Zplugが入っている場合無効)
export ZPLUG_HOME=/usr/local/opt/zplug
if [ -e $ZPLUG_HOME ]; then
  # Zplug の有効化
  source $ZPLUG_HOME/init.zsh
  zplug "zsh-users/zsh-completions"
  zplug "zsh-users/zsh-syntax-highlighting"
  zplug "zsh-users/zsh-autosuggestions"
  zplug "toshiki670/zsh-git-prompt"
  # プラグイン追加後、下記を実行する
  # zplug install
  zplug load
fi

ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=244'


# Theme configure
# eval `/usr/local/opt/coreutils/libexec/gnubin/dircolors ~/.dircolors-solarized/dircolors.ansi-dark`
eval $(gdircolors ~/dotfiles/zsh/bundle/color/dircolors-solarized)
eval $(dircolors ~/dotfiles/zsh/bundle/color/dircolors-solarized/dircolors.ansi-universal)

# 補完機能
bindkey "^[[Z" reverse-menu-complete

# complete 普通の補完関数; approximate ミススペルを訂正した上で補完を行う。; prefixカーソルの位置で補完を行う
zstyle ':completion:*' completer _complete _prefix #_approzimate

# 多部補完時に大文字小文字を区別しない
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# タブを１回押すと、補完候補が表示され、さらにタブを押すことで、選択モードに入る
zstyle ':completion:*:default' menu select=2
if [ -n "$LS_COLORS" ]; then
  zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
fi

# Printable 8bit
setopt print_eight_bit
setopt auto_cd
setopt auto_pushd
setopt correct

# Prompt comfig -------------------------------------------
sep='⮀'
sub_sep='⮁'

pri_clr='226'
pri_fore='022'
pri_set="%K{${pri_clr}}%F{${pri_fore}}"

sec_clr='240'
sec_fore='255'
sec_set="%K{${sec_clr}}%F{${sec_fore}}"

fail_clr='214'
fail_fore='088'
fail_set="%K{${fail_clr}}%F{${fail_fore}}"

source $ZPLUG_HOME/repos/toshiki670/zsh-git-prompt/zshrc.sh
ZSH_THEME_GIT_PROMPT_PREFIX="${sec_set} ⭠ "
ZSH_THEME_GIT_PROMPT_SUFFIX="%K{${sec_clr}} %k"
ZSH_THEME_GIT_PROMPT_HASH_PREFIX=":"
ZSH_THEME_GIT_PROMPT_SEPARATOR="${sec_set} ${sub_sep} "
ZSH_THEME_GIT_PROMPT_BRANCH_AFTER="${sec_set} "
ZSH_THEME_GIT_PROMPT_BRANCH="${sec_set}"
ZSH_THEME_GIT_PROMPT_STAGED="%K{${sec_clr}}%F{green}%{!%G%}"
ZSH_THEME_GIT_PROMPT_CONFLICTS="%K{${sec_clr}}%F{magenta}%{x%G%}"
ZSH_THEME_GIT_PROMPT_CHANGED="%K{${sec_clr}}%F{219}%{+%G%}"
ZSH_THEME_GIT_PROMPT_BEHIND="%K{${sec_clr}}%F{219}%{-%2G%}"
ZSH_THEME_GIT_PROMPT_AHEAD="%K{${sec_clr}}%F{green}%{+%2G%}"
ZSH_THEME_GIT_PROMPT_STASHED="${sec_set}%{⚑%G%}"
ZSH_THEME_GIT_PROMPT_UNTRACKED="%K{${sec_clr}}%{… %G%}"
ZSH_THEME_GIT_PROMPT_CLEAN="%K{${sec_clr}}%F{green}%{✔ %G%}"
ZSH_THEME_GIT_PROMPT_LOCAL="${sec_set} L"
# The remote branch will be shown between these two
ZSH_THEME_GIT_PROMPT_UPSTREAM_FRONT="{%{$fg[blue]%}"
ZSH_THEME_GIT_PROMPT_UPSTREAM_END="%{${reset_color}%}}"
ZSH_THEME_GIT_PROMPT_MERGING="%{$fg_bold[magenta]%}|MERGING%{${reset_color}%}"
ZSH_THEME_GIT_PROMPT_REBASE="%{$fg_bold[magenta]%}|REBASE%{${reset_color}%}"


my_prompt=' %#%~ '
my_prompt2="${sec_set}⌘ %k%F{${sec_clr}}${sep}%f "

pass_status="${pri_set}${my_prompt}%K{${sec_clr}}%F{${pri_clr}}${sep}%k%f"
fail_status="${fail_set}${my_prompt}%K{${sec_clr}}%F{${fail_clr}}${sep}%k%f"

PROMPT=%(?.$pass_status.$fail_status)'$(git_super_status)'"%F{${sec_clr}}${sep}"$'\n'$my_prompt2
PROMPT2=$my_prompt2

my_sprompt="${fail_set}Correct%K{${sec_clr}}%F{${fail_clr}}${sep}%f"
my_sprompt2="${sec_set}[nyae] %k%F{${sec_clr}}${sep}%f "
SPROMPT="${my_sprompt}%F{${sec_fore}}'%R' to '%r'? %k%F{${sec_clr}}${sep}%f"$'\n'$my_sprompt2

# ---------------------------------------------------------

alias relogin='exec $SHELL -l'
alias ls='gls --color=auto'
alias ll='ls -l'
alias la='ls -a'
alias lla='ls -la'
alias rm='rm -i'

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

# For Note
alias note='cd ~/Documents/Note'

# グローバルIPアドレス確認
alias ipecho='curl ipecho.net/plain; echo'

# For vim
alias v='vim'
alias vim-utf8='vim -c ":e ++enc=utf8"'
alias vim-euc_jp='vim -c ":e ++enc=euc-jp"'
alias vim-shift_jis='vim -c ":e ++enc=shift_jis"'
alias vim-cheat='vim ~/dotfiles/vim/cheatsheet/common.md'
# alias eclipse='open -a eclipse -data /User/tsk/Documents/workspace &'

# 拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s rb='ruby'
alias -s py='python'
alias -s php='php -f'

# Dotfiles Config
alias vimrc='vim ~/dotfiles/vim/.vimrc'
alias zshrc='vim ~/dotfiles/zsh/.zshrc'

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


# Google Search By Safari
goo() {
  local str opt
  if [ $# != 0 ]; then
    for i in $*; do
      str="$str${str:++}$i"
    done
    opt='search?num=100'
    opt="${opt}&q=${str}"
  fi
  open http://www.google.com/$opt
}


function ps-grep {
  ps aux | grep $1 | grep -v grep
}


# Tmux起動
# if [ $SHLVL = 1 ]; then
#   tmux
#   exit
# else
#   cat ~/dotfiles/screenfetch
# fi

# ターミナル起動時に実行


# ZSHの起動した関数の時間計測 .zshenv参照
# if (which zprof > /dev/null 2>&1) ;then
#   zprof
# fi

