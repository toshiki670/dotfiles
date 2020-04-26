# For rbenv
if type "rbenv" > /dev/null 2>&1; then
  eval "$(rbenv init --no-rehash -)";
  export PATH="$HOME/.rbenv/shims:$PATH"
fi

# For gem
if type "gem" > /dev/null 2>&1; then
  PATH="$(ruby -e 'print Gem.user_dir')/bin:$PATH"
fi


# For Rails
alias be='bundle exec'

# 拡張子に応じたコマンドを実行
alias -s rb='ruby'
