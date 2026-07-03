# fzf configuration
# https://github.com/junegunn/fzf

# --color uses ANSI palette indices (-1 = terminal default, 0-15 = ANSI) so the
# colors follow Ghostty's per-appearance palette (light:One Half Light /
# dark:Ayu), matching the eza approach. No shell wiring needed.
# Exception: the selected line uses fg+:15 (white), not -1. palette 8 (bg+) is a
# mid grey in both themes (#686868 / #4f525e), too close to the default fg
# (#bfbdb6 / #383a42) to read; white keeps the highlighted row legible on both.
set -gx FZF_DEFAULT_OPTS '
  --height 50%
  --layout=reverse
  --inline-info
  --color=fg:-1,bg:-1,hl:4
  --color=fg+:15,bg+:8,hl+:12
  --color=info:3,prompt:5,pointer:13
  --color=marker:2,spinner:13,header:6
'

if command -q fd
    set -gx FZF_DEFAULT_COMMAND 'fd --type f --hidden --follow --exclude .git'
else if command -q rg
    set -gx FZF_DEFAULT_COMMAND 'rg --files --hidden --follow --glob "!.git/*"'
end
