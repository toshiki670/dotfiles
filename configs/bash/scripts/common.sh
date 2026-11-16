# shellcheck shell=bash
export HISTCONTROL=ignoredups

# mise config
eval "$(mise activate bash)"

# Homebrew PATH configuration (macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && [[ -d "/opt/homebrew/bin" ]]; then
  export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/opt/homebrew/opt/coreutils/libexec/gnubin:$PATH"
  # rustup (cargo / rustc proxies) is keg-only, so it is not linked into /opt/homebrew/bin.
  [[ -d "/opt/homebrew/opt/rustup/bin" ]] && export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
fi

# Binaries installed by `cargo install`
[[ -d "$HOME/.cargo/bin" ]] && export PATH="$HOME/.cargo/bin:$PATH"

# Obsidian (macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && [[ -d "/Applications/Obsidian.app/Contents/MacOS" ]]; then
  export PATH="$PATH:/Applications/Obsidian.app/Contents/MacOS"
fi

# browser-use
for _browser_use_dir in "$HOME/.browser-use/bin" "$HOME/.browser-use-env/bin"; do
  [[ -d "$_browser_use_dir" ]] && export PATH="$PATH:$_browser_use_dir"
done
unset _browser_use_dir
