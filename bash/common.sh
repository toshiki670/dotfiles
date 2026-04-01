# shellcheck shell=bash
export HISTCONTROL=ignoredups

# mise config
eval "$(mise activate bash)"

# Homebrew PATH configuration (macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && [[ -d "/opt/homebrew/bin" ]]; then
  export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/opt/homebrew/opt/coreutils/libexec/gnubin:$PATH"
fi

# browser-use
for _browser_use_dir in "$HOME/.browser-use/bin" "$HOME/.browser-use-env/bin"; do
  [[ -d "$_browser_use_dir" ]] && export PATH="$_browser_use_dir:$PATH"
done
unset _browser_use_dir
