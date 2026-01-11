# zoxide - Smarter cd command with frecency-based directory jumping
if type "zoxide" > /dev/null 2>&1; then
  eval "$(zoxide init zsh)"
fi

# Enable directory stack with cd
setopt auto_pushd
