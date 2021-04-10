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

# Setting completions
require 'zshrc/docker.zsh'

# History config
require 'zshrc/history.zsh'

# Completion config
require 'zshrc/completion.zsh'

# Initialize and Install the Zplug
require 'zshrc/zplug.zsh'

# ls or exa command config
require 'zshrc/ls.zsh'

# ruby and rails config
require 'zshrc/ruby.zsh'

# python config
require 'zshrc/python.zsh'

# C language config
require 'zshrc/c.zsh'

# git config
require 'zshrc/git.zsh'

# vim config
require 'zshrc/vim.zsh'


### Common config

# for macOS
export PATH="${DOTFILES}/bin:$PATH"

if type 'brew' > /dev/null 2>&1; then
  export PATH="$(brew --prefix coreutils)/libexec/gnubin:$PATH"
fi


# zsh-users/zsh-autosuggestions
# https://github.com/zsh-users/zsh-autosuggestions
# ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=244'


# Printable 8bit
setopt print_eight_bit
setopt auto_cd
setopt auto_pushd
setopt correct


# Reload
alias reload='exec $SHELL -l'


# グローバルIPアドレス確認
alias ipecho='curl ipecho.net/plain; echo'


# 拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s php='php -f'


# Dotfiles Config
alias zshrc="vim ${DOTFILES}/zsh/.zshrc"


# Process grep
function ps-grep {
  ps aux | grep $1 | grep -v grep
}


# df config
alias df='df -h'

### end


# ターミナル起動時に実行

# ZSHの起動した関数の時間計測 .zshenv参照
# if (which zprof > /dev/null 2>&1) ;then
#   zprof
# fi
