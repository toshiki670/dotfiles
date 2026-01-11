# eza - ls command replacement
if type "eza" > /dev/null 2>&1; then
  alias ls='eza --icons=always -l -gh --time-style long-iso --git'
  alias la='eza --icons=always -la -gh --time-style long-iso --git'

  lt() {
    local level="${1:-2}"  # デフォルトは2
    eza --icons=always -la -gh --time-style long-iso --git --tree --level="$level"
  }
else
  alias ls='ls --color=auto -lh'
  alias la='ls --color=auto -lah'
fi

# Standard ls (always use standard ls command)
alias sls='command ls'
