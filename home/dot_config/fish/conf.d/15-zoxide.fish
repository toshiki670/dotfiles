# zoxide - Smarter cd with frecency (Phase 2: match zsh)
# ^j^f = interactive directory jump with fzf preview
if command -q zoxide
    zoxide init fish | source
end
