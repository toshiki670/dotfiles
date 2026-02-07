# Add clipboard (first line) as a completion candidate when pressing Tab (macOS only)
if [[ "$OSTYPE" == "darwin"* ]] && command -v pbpaste &>/dev/null; then
  _clipboard_candidate() {
    # Only add when completing arguments, not the command name
    (( CURRENT >= 2 )) || return 1
    local clip
    clip=$(pbpaste 2>/dev/null)
    # Only add when clipboard is exactly one line (safer than using first line only)
    [[ -n "$clip" && "$clip" != *$'\n'* ]] || return 1
    compadd -X 'clipboard' -S '' -- "$clip"
    return 1
  }
  zstyle ':completion:*' completer _clipboard_candidate _complete _prefix
fi
