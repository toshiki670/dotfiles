# fzf keybindings
# All keybindings: Ctrl+J then second key (avoids Zellij conflicts)

bind -M default ctrl-j,ctrl-h _fzf_history
bind -M default ctrl-j,ctrl-t _fzf_file
bind -M default ctrl-j,ctrl-g _fzf_ghq_cd
bind -M default ctrl-j,ctrl-w _fzf_worktree_remove
bind -M default ctrl-j,ctrl-i _fzf_gh
bind -M insert ctrl-j,ctrl-h _fzf_history
bind -M insert ctrl-j,ctrl-t _fzf_file
bind -M insert ctrl-j,ctrl-g _fzf_ghq_cd
bind -M insert ctrl-j,ctrl-w _fzf_worktree_remove
bind -M insert ctrl-j,ctrl-i _fzf_gh
