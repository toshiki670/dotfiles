# zeno.zsh keybindings
# https://github.com/yuki-yano/zeno.zsh
# Requires: deno, fzf (loaded by sheldon after fzf)
#
# Key bindings (when ZENO_LOADED is set):
#   Space         - abbrev snippet expand
#   Enter         - abbrev snippet expand and run
#   Tab           - zeno completion (fzf)
#   Ctrl+J Ctrl+H - zeno-history-selection (fzf)
#   Ctrl+J Ctrl+G - zeno-ghq-cd (ghq repo list)
#   Ctrl+X Ctrl+X - zeno-insert-snippet (snippet picker)
#   Ctrl+X Space  - zeno-insert-space (literal space)
#   Ctrl+X Enter  - accept-line
#   Ctrl+X Ctrl+Z - zeno-toggle-auto-snippet
#   Ctrl+X P      - zeno-preprompt
#   Ctrl+X S      - zeno-preprompt-snippet

# User config: ~/.config/zeno/ or $ZENO_HOME
# export ZENO_HOME=~/.config/zeno

if [[ -n $ZENO_LOADED ]]; then
  # Abbrev snippet
  bindkey ' '   zeno-auto-snippet
  bindkey '^m'  zeno-auto-snippet-and-accept-line

  # Completion (fzf)
  bindkey '^i'   zeno-completion

  # ghq: change to ghq-managed repo
  bindkey '^j^g' zeno-ghq-cd

  # History selection (fzf)
  bindkey '^j^h' zeno-history-selection

  # Insert snippet and Ctrl+X submap
  bindkey '^xx' zeno-insert-snippet
  bindkey '^x ' zeno-insert-space
  bindkey '^x^m' accept-line
  bindkey '^x^z' zeno-toggle-auto-snippet

  # Preprompt (prefix for next line)
  bindkey '^xp' zeno-preprompt
  bindkey '^xs' zeno-preprompt-snippet
fi
