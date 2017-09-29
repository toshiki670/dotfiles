# export PATH=/usr/local/bin:/usr/bin
export LANG=ja_JP.UTF-8
export KCODE=u
export PATH=/usr/local/bin:$PATH
export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
#For rbenv
eval "$(rbenv init -)"
#for zsh-completions
fpath=(/usr/local/share/zsh-completions $fpath)

autoload -Uz add-zsh-hook
#Color
autoload -Uz colors
colors

#補完機能
autoload -Uz compinit
compinit
bindkey "^[[Z" reverse-menu-complete

#多部補完時に大文字小文字を区別しない
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'
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
PROMPT=%(?@'%F{cyan}[%#%n : %~]%f'$'\n''>> '@'%F{red}~[%#%n : %~]%f'$'\n''>> ')
PROMPT2='>> '
SPROMPT="%F{red}~Correct '%R' to '%r'?%f"$'\n''[nyae]>> '


#zsh-completions
if [ -e /usr/local/share/zsh-completions ]; then
    fpath=(/usr/local/share/zsh-completions $fpath)
fi


#Theme configure
#eval `/usr/local/opt/coreutils/libexec/gnubin/dircolors ~/.dircolors-solarized/dircolors.ansi-dark`
eval $(gdircolors ~/.dircolors-solarized)
eval $(dircolors ~/dircolors-solarized/dircolors.ansi-universal)
alias ls='gls --color=auto'
#Currently Directory List
alias cdl='a=(`ls -1`) ; ls -1 | cat -n ; read b ; cd ${a[$b]}'
#For hermes
alias hermes='echo "Hermes Command List\nher\nhtsk"'
alias her='cd ~/Documents/Hermes/repos/hs2018-trainee/'
alias htsk='cd ~/Documents/Hermes/repos/hs2018-trainee/01_研修課題/川添\ 寿樹/'
#For PHP
alias xam='cd /Applications/XAMPP/xamppfiles/htdocs/php/'
#For vim
alias vim-utf8='vim -c ":e ++enc=utf8"'
alias vim-euc-jp='vim -c ":e ++enc=euc-jp"'
alias vim-shift-jis='vim -c ":e ++enc=shift_jis"'
alias eclipse='open -a eclipse -data /User/tsk/Documents/workspace &'

# Google Search By Safari
google() {
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
