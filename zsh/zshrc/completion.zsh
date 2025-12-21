# 補完機能の初期化
autoload -Uz compinit

# Initialize completion system (first time - makes compdef available for plugins)
compinit

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
