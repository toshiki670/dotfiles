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
  zplug "olivierverdier/zsh-git-prompt"
  # プラグイン追加後、下記を実行する
  # zplug install
  zplug load
fi

ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=230'
source $ZPLUG_HOME/repos/olivierverdier/zsh-git-prompt/zshrc.sh

# utoload -Uz add-zsh-hook
# Color
# utoload -Uz colors && colors
# 補完関連
# utoload -U compinit && compinit

# Git のステータスを表示
# autoload -Uz vcs_info
# setopt prompt_subst
# zstyle ':vcs_info:git:*' check-for-changes true
# zstyle ':vcs_info:git:*' stagedstr "|%F{yellow}staged%F{cyan}"
# zstyle ':vcs_info:git:*' unstagedstr "|%F{red}unstaged%F{cyan}"
# zstyle ':vcs_info:*' formats "%F{cyan}[%b%c%u]%f"
# zstyle ':vcs_info:*' actionformats "%F{red}[%b|%a]%f"
# precmd () { vcs_info }



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

pri_clr='green'
pri_fore='022'
pri_set="%K{${pri_clr}}%F{${pri_fore}}"

sec_clr='240'
sec_fore='255'
sec_set="%K{${sec_clr}}%F{${sec_fore}}"

fail_clr='207'
fail_fore='088'
fail_set="%K{${fail_clr}}%F{${fail_fore}}"

ZSH_THEME_GIT_PROMPT_PREFIX="${sec_set} ⭠ "
ZSH_THEME_GIT_PROMPT_SUFFIX="%K{${sec_clr}}%F{${sec_clr}} %k${sep}"
ZSH_THEME_GIT_PROMPT_SEPARATOR="${sec_set} ${sub_sep} "
ZSH_THEME_GIT_PROMPT_BRANCH="${sec_set}"
ZSH_THEME_GIT_PROMPT_STAGED="%K{${sec_clr}}%F{green}%{!%G%}"
ZSH_THEME_GIT_PROMPT_CONFLICTS="%K{${sec_clr}}%F{magenta}%{x%G%}"
ZSH_THEME_GIT_PROMPT_CHANGED="%K{${sec_clr}}%F{219}%{+%G%}"
ZSH_THEME_GIT_PROMPT_BEHIND="%K{${sec_clr}}%F{219}%{-%G%}"
ZSH_THEME_GIT_PROMPT_AHEAD="%K{${sec_clr}}%F{green}%{+%G%}"
ZSH_THEME_GIT_PROMPT_CLEAN="%K{${sec_clr}}%F{green}%{✔ %G%}"


my_prompt='%#%~ '
my_prompt2="${pri_set}⌘ %k%F{${pri_clr}}${sep}%f "

pass_status="${pri_set}${my_prompt}%K{${sec_clr}}%F{${pri_clr}}${sep}%k%f"
fail_status="${fail_set}${my_prompt}%K{${sec_clr}}%F{${fail_clr}}${sep}%k%f"

PROMPT=%(?.$pass_status.$fail_status)'$(git_super_status)'$'\n'$my_prompt2
PROMPT2=$my_prompt2

my_correct="${fail_set}Correct%K{${sec_clr}}%F{${fail_clr}}${sep}%f"
my_correct2="${pri_set}[nyae] %k%F{${pri_clr}}${sep}%f "
SPROMPT="${my_correct}%F{${sec_fore}}'%R' to '%r'? %k%F{${sec_clr}}${sep}%f"$'\n'$my_correct2

# ---------------------------------------------------------

alias relogin='exec $SHELL -l'
alias ls='gls --color=auto'
alias ll='ls -l'
alias la='ls -a'
alias lla='ls -la'

# For git
alias g='git'
alias ga='git add'
alias gau='git add -u'
alias ga.='git add .'
alias gc='git commit'
alias gcm='git commit -m'
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
alias vim-euc-jp='vim -c ":e ++enc=euc-jp"'
alias vim-shift-jis='vim -c ":e ++enc=shift_jis"'
# alias eclipse='open -a eclipse -data /User/tsk/Documents/workspace &'

# 拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s rb='ruby'
alias -s py='python'
alias -s php='php -f'

# Dotfiles Config
alias vimrc='vim ~/.vimrc'
alias zshrc='vim ~/.zshrc'

# 履歴ファイルの保存先
export HISTFILE=${HOME}/.zsh_history

# メモリに保存される履歴の件数
export HISTSIZE=1000

# 履歴ファイルに保存される履歴の件数
export SAVEHIST=100000

# 重複を記録しない
setopt hist_ignore_dups

# 開始と終了を記録
# setopt EXTENDED_HISTORY


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

# Tmux起動
if [ $SHLVL = 1 ]; then
  tmux
else
  cat ~/dotfiles/screenfetch
fi

# ターミナル起動時に実行


# ZSHの起動した関数の時間計測 .zshenv参照
# if (which zprof > /dev/null 2>&1) ;then
#   zprof
# fi
