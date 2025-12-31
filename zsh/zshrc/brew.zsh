# Homebrew PATH configuration
# Ensure Homebrew paths come before mise-managed paths
# mise uses precmd hooks to reorder PATH, so we need to do the same
# This must be loaded AFTER all plugins (sheldon) to avoid being cleared

if [[ "$OSTYPE" == "darwin"* ]] && [[ -d "/opt/homebrew/bin" ]]; then
  _fix_homebrew_path() {
    if [[ "$PATH" != "/opt/homebrew/bin:"* ]]; then
      local clean_path="${PATH}"
      clean_path="${clean_path//:\/opt\/homebrew\/bin/}"
      clean_path="${clean_path//:\/opt\/homebrew\/sbin/}"
      clean_path="${clean_path//:\/opt\/homebrew\/opt\/coreutils\/libexec\/gnubin/}"
      export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/opt/homebrew/opt/coreutils/libexec/gnubin:${clean_path#:}"
    fi
  }
  precmd_functions+=(_fix_homebrew_path)
fi

