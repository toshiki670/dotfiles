# fzf keybindings (Phase 2: match zsh ^j^h, ^j^t, ^j^g)
# All keybindings: Ctrl+J then second key (avoids Zellij conflicts)

bind -M default ctrl-j,ctrl-h _fzf_history
bind -M default ctrl-j,ctrl-t _fzf_file
bind -M default ctrl-j,ctrl-g _fzf_ghq_cd
bind -M default ctrl-j,ctrl-w _fzf_worktree_remove
bind -M insert ctrl-j,ctrl-h _fzf_history
bind -M insert ctrl-j,ctrl-t _fzf_file
bind -M insert ctrl-j,ctrl-g _fzf_ghq_cd
bind -M insert ctrl-j,ctrl-w _fzf_worktree_remove

# gh targeted-command completion: Tab triggers fzf picker with preview
# Activates only for: gh issue/pr/run/release <subcommand> (no ID yet)
# Use default mode (not insert) — vi mode is not enabled in this config
bind -M default tab _fzf_gh_complete
