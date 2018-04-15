# export PATH=/usr/local/bin:/usr/bin
export LANG=ja_JP.UTF-8
export KCODE=u
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/sbin:$PATH"
export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
#For pyenv
export PYENV_ROOT="$HOME/.pyenv"
export PATH="$PYENV_ROOT/bin:$PATH"
eval "$(pyenv init -)"

#For rbenv
export PATH="$HOME/.rbenv/bin:$PATH"
eval "$(rbenv init -)";

#For Bundle
export PATH="$HOME/.rbenv/versions/2.4.2/bin:$PATH"

#For comporser (Laravel)
export PATH=$PATH:~/.composer/vendor/bin:/usr/local/sbin
#export PATH=$PATH:~/.composer/vendor/bin
#for zsh-completions
if [ -e /usr/local/share/zsh-completions ]; then
    fpath=(/usr/local/share/zsh-completions $fpath)
fi

autoload -Uz add-zsh-hook
#Color
autoload -Uz colors && colors

#補完機能
autoload -Uz compinit && compinit -u
bindkey "^[[Z" reverse-menu-complete

#complete 普通の補完関数; approximate ミススペルを訂正した上で補完を行う。; prefixカーソルの位置で補完を行う
zstyle ':completion:*' completer _complete _approzimate _prefix
#多部補完時に大文字小文字を区別しない
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'
#タブを１回押すと、補完候補が表示され、さらにタブを押すことで、選択モードに入る
zstyle ':completion:*:default' menu select=2
if [ -n "$LS_COLORS" ]; then
  zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
fi
#Printable 8bit
setopt print_eight_bit
setopt auto_cd
setopt auto_pushd
setopt correct


#PROMPT='%F{cyan}[%#%n : %~]%f'$'\n''>> '
PROMPT=%(?@'%F{cyan}[%m%#%n %~]%f'$'\n''>> '@'%F{red}[%m%#%n %~]%f'$'\n''>> ')
PROMPT2='>> '
SPROMPT="%F{red}Correct '%R' to '%r'?%f"$'\n''[nyae]>> '


#Theme configure
#eval `/usr/local/opt/coreutils/libexec/gnubin/dircolors ~/.dircolors-solarized/dircolors.ansi-dark`
eval $(gdircolors ~/.dircolors-solarized)
eval $(dircolors ~/dircolors-solarized/dircolors.ansi-universal)
alias ls='gls --color=auto'
#For PHP
alias xam='cd /Applications/XAMPP/xamppfiles/htdocs/php/'
#For Rails
alias ror='cd ~/dev/RailsProject/'
#For Note
alias note='cd ~/Documents/Note'
#グローバルIPアドレス確認
alias ipecho='curl ipecho.net/plain; echo'
#For vim
alias vim-utf8='vim -c ":e ++enc=utf8"'
alias vim-euc-jp='vim -c ":e ++enc=euc-jp"'
alias vim-shift-jis='vim -c ":e ++enc=shift_jis"'
alias eclipse='open -a eclipse -data /User/tsk/Documents/workspace &'
alias ll='ls -l'
#拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s rb='ruby'
alias -s py='python'
alias -s php='php -f'



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
