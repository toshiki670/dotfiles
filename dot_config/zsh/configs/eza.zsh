# eza - ls command replacement
if type "eza" > /dev/null 2>&1; then
  alias ls='eza --icons=always -l -gh --time-style long-iso --git'
  alias la='eza --icons=always -la -gh --time-style long-iso --git'

  lt() {
    local level="2"  # デフォルトは2
    local target="."   # デフォルトはカレントディレクトリ

    if [[ $# -eq 1 ]]; then
      # 引数が1つの場合
      if [[ -e "$1" ]]; then
        # パスとして存在する場合はパスとして扱う
        target="$1"
      elif [[ "$1" =~ ^[0-9]+$ ]]; then
        # 存在しないが数値の場合はレベルとして扱う
        level="$1"
      else
        # 存在せず、数値でもない場合はパスとして扱う（ezaがエラーを出す）
        target="$1"
      fi
    elif [[ $# -eq 2 ]]; then
      # 引数が2つの場合、第1引数をレベル、第2引数をパスとして扱う
      level="$1"
      target="$2"
    fi

    eza --icons=always -la -gh --time-style long-iso --git --tree --level="$level" "$target"
  }
else
  alias ls='ls --color=auto -lh'
  alias la='ls --color=auto -lah'
fi

# Standard ls (always use standard ls command)
alias sls='command ls'
