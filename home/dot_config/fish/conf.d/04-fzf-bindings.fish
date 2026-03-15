# fzf keybindings (Phase 2: match zsh ^j^h, ^j^t, ^j^g)
# All keybindings: Ctrl+J then second key (avoids Zellij conflicts)

# ^j^h - Search command history
function _fzf_history
  set -l line (history | fzf --height=40% --reverse +s --tiebreak=index)
  if test -n "$line"
    commandline -r "$line"
  end
  commandline -f repaint
end

# ^j^t - Search files (insert path at cursor)
function _fzf_file
  set -l selected (
    if set -q FZF_CTRL_T_COMMAND
      eval $FZF_CTRL_T_COMMAND
    else
      find . -path '*/\.*' -prune -o -type f -print 2>/dev/null
    end \
    | fzf --height=40% --reverse
  )
  if test -n "$selected"
    commandline -i "$selected"
  end
  commandline -f repaint
end

# ^j^g - Git branch checkout
function _fzf_git_branch
  if not git rev-parse --git-dir >/dev/null 2>&1
    echo "Not a git repository"
    commandline -f repaint
    return 1
  end
  set -l branch (
    git branch -a 2>/dev/null \
    | grep -v HEAD \
    | string replace -r '^\*?\s+' '' \
    | string replace 'remotes/origin/' '' \
    | sort -u \
    | fzf --height=40% --reverse --preview 'git log --oneline --graph --color=always --decorate {}' --preview-window=right:60%
  )
  if test -n "$branch"
    set branch (string trim "$branch")
    git checkout "$branch"
    commandline -f repaint
  end
end

bind -M default ctrl-j,ctrl-h '_fzf_history'
bind -M default ctrl-j,ctrl-t '_fzf_file'
bind -M default ctrl-j,ctrl-g '_fzf_git_branch'
bind -M insert ctrl-j,ctrl-h '_fzf_history'
bind -M insert ctrl-j,ctrl-t '_fzf_file'
bind -M insert ctrl-j,ctrl-g '_fzf_git_branch'
