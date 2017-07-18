# export PATH=/usr/local/bin:/usr/bin
export LANG=ja_JP.UTF-8
export KCODE=u
export PATH=/usr/local/bin:$PATH
export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
#for zsh-completions
fpath=(/usr/local/share/zsh-completions $fpath)

autoload -Uz add-zsh-hook
#Color
autoload -Uz colors
colors

#補完機能
autoload -Uz compinit
compinit

#多部補完時に大文字小文字を区別しない
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'
zstyle ':completion:*:default' menu select=2
if [ -n "$LS_COLORS" ]; then
  zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
fi
#Printable 8bit
setopt print_eight_bit

##vcs_info setting

autoload -Uz vcs_info

zstyle ':vcs_info:*' enable git svn
zstyle ':vcs_info:*' formats '(%s)-[%b]'
zstyle ':vcs_info:*' actionformats '(%s)-[%b|%a]'
zstyle ':vcs_info:svn:*' branchformat '%b:r%r'

autoload -Uz is-at-least
if is-at-least 4.3.10; then
  zstyle ':vcs_info:git:*' check-for-changes true
  zstyle ':vcs_info:git:*' stagedstr "+"
  zstyle ':vcs_info:git:*' unstagedstr "-"
  zstyle ':vcs_info:git:*' formats '(%s)-[%b] %c%u'
  zstyle ':vcs_info:git:*' actionformats '(%s)-[%b|%a] %c%u'
fi

function _update_vcs_info_msg() {
  psvar=()
  LANG=en_US.UTF-8 vcs_info
  [[ -n "$vcs_info_msg_0_" ]] && psvar[1]="$vcs_info_msg_0_"
}
add-zsh-hook precmd _update_vcs_info_msg

PROMPT='%F{cyan}[%#%n : %~]%f'$'\n''>> '
RPROMPT="%1(v|%F{green}%1v%f|)"

#zsh-completions
if [ -e /usr/local/share/zsh-completions ]; then
    fpath=(/usr/local/share/zsh-completions $fpath)
fi


#Theme configure
#eval `/usr/local/opt/coreutils/libexec/gnubin/dircolors ~/.dircolors-solarized/dircolors.ansi-dark`
eval $(gdircolors ~/.dircolors-solarized)
eval $(dircolors ~/dircolors-solarized/dircolors.ansi-universal)
alias ls='gls --color=auto'

