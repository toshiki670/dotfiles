# Common config

# for macOS
export PATH="${DOTFILES}/bin:$PATH"

# Printable 8bit
setopt print_eight_bit

# Enable command spell correction
setopt correct

# Reload
alias reload='exec $SHELL -l'

# グローバルIPアドレス確認
alias ipecho='curl ipecho.net/plain; echo'

# 拡張子に応じたコマンドを実行
alias -s txt='vim'
alias -s html='open'
alias -s php='php -f'

# Process grep
function ps-grep {
  ps aux | grep $1 | grep -v grep
}

# df config
alias df='df -h'

if [[ "$OSTYPE" != "darwin"* ]]; then
  # cp config
  alias cp='cp --verbose'
  # mv config
  alias mv='mv --verbose'
fi

# Local bin
export PATH="$HOME/.local/bin:$PATH"

# Rust env
export PATH="$HOME/.cargo/bin:$PATH"
