# fzf keybindings (Phase 2: match zsh ^j^h, ^j^t, ^j^g)
# All keybindings: Ctrl+J then second key (avoids Zellij conflicts)

bind -M default ctrl-j,ctrl-h '_fzf_history'
bind -M default ctrl-j,ctrl-t '_fzf_file'
bind -M default ctrl-j,ctrl-g '_fzf_ghq_repo'
bind -M insert ctrl-j,ctrl-h '_fzf_history'
bind -M insert ctrl-j,ctrl-t '_fzf_file'
bind -M insert ctrl-j,ctrl-g '_fzf_ghq_repo'
