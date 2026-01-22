export HISTCONTROL=ignoredups

# mise config
eval "$(mise activate bash)"

# Homebrew PATH configuration (macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && [[ -d "/opt/homebrew/bin" ]]; then
  export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/opt/homebrew/opt/coreutils/libexec/gnubin:$PATH"
fi
