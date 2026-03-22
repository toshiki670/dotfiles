# fzf keybindings (Phase 2: match zsh ^j^h, ^j^t, ^j^g)
# All keybindings: Ctrl+J then second key (avoids Zellij conflicts)

# ^j^h - Search command history
function _fzf_history
  set -l line (history | fzf +s --tiebreak=index)
  if test -n "$line"
    commandline -r "$line"
  end
  commandline -f repaint
end

# ^j^t - Search files (insert path at cursor)
function _fzf_file
  set -l selected (
    find . -path '*/\.*' -prune -o -type f -print 2>/dev/null | fzf
  )
  if test -n "$selected"
    commandline -i "$selected"
  end
  commandline -f repaint
end

# Preview helper for ^j^g (run via: fish -c "__fzf_ghq_readme_preview {}")
function __fzf_ghq_readme_preview
  set -l rel (string trim -- $argv[1])
  test -n "$rel"; or return 1
  set -l r (ghq root)/$rel
  for n in README.md readme.md README Readme.md
    if test -f "$r/$n"
      head -n 400 "$r/$n"
      return 0
    end
  end
  echo "No README"
end

# ^j^g - Pick ghq-managed repo (relative paths; README preview on the right)
function _fzf_ghq_repo
  if not command -v ghq >/dev/null 2>&1
    echo "ghq: command not found"
    commandline -f repaint
    return 1
  end
  set -l rel (
    ghq list 2>/dev/null | fzf \
      --preview 'fish -c "__fzf_ghq_readme_preview {}"' \
      --preview-window=right:60%
  )
  if test -n "$rel"
    set rel (string trim "$rel")
    set -l root (ghq root)
    cd "$root/$rel"
    commandline -f repaint
  end
end

bind -M default ctrl-j,ctrl-h '_fzf_history'
bind -M default ctrl-j,ctrl-t '_fzf_file'
bind -M default ctrl-j,ctrl-g '_fzf_ghq_repo'
bind -M insert ctrl-j,ctrl-h '_fzf_history'
bind -M insert ctrl-j,ctrl-t '_fzf_file'
bind -M insert ctrl-j,ctrl-g '_fzf_ghq_repo'
