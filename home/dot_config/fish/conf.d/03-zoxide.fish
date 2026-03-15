# zoxide - Smarter cd with frecency (Phase 2: match zsh)
# ^j^f = interactive directory jump with fzf preview

if not command -q zoxide
  exit 0
end

zoxide init fish | source

abbr -a bz 'z ..'

# Interactive zoxide + fzf (same as zsh __zoxide_zi_widget)
# Key: Ctrl+J then Ctrl+F (avoids Zellij conflicts)
function _fzf_zoxide_cd
  set -l result (
    zoxide query --list --score 2>/dev/null \
    | fzf --height=40% \
          --reverse \
          --no-sort \
          --preview='eza -1 --color=always --icons {2}' \
          --preview-window='right:50%:wrap' \
          --bind='ctrl-/:toggle-preview' \
    | awk '{print $2}'
  )
  if test -n "$result"
    cd "$result"
    commandline -f repaint
  end
end

bind -M default ctrl-j,ctrl-f '_fzf_zoxide_cd' repaint
bind -M insert ctrl-j,ctrl-f '_fzf_zoxide_cd' repaint
