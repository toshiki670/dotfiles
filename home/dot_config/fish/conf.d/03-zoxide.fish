# zoxide - Smarter cd with frecency (Phase 2: match zsh)
# ^j^f = interactive directory jump with fzf preview

if not command -q zoxide
  exit 0
end

zoxide init fish | source

abbr --add zb 'z ..'
abbr --add zbb 'z ../..'
abbr --add zbbb 'z ../../..'
abbr --add zp 'z -'

abbr --add zgit 'z (git rev-parse --show-toplevel)'

