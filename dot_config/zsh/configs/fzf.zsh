# fzf configuration
# https://github.com/junegunn/fzf

# fzf default options
export FZF_DEFAULT_OPTS='
  --height 40%
  --layout=reverse
  --border
  --inline-info
  --color=fg:#d0d0d0,bg:#121212,hl:#5f87af
  --color=fg+:#d0d0d0,bg+:#262626,hl+:#5fd7ff
  --color=info:#afaf87,prompt:#d7005f,pointer:#af5fff
  --color=marker:#87ff00,spinner:#af5fff,header:#87afaf
'

# fzf default command (if fd or rg is available)
if command -v fd &>/dev/null; then
  export FZF_DEFAULT_COMMAND='fd --type f --hidden --follow --exclude .git'
  export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
elif command -v rg &>/dev/null; then
  export FZF_DEFAULT_COMMAND='rg --files --hidden --follow --glob "!.git/*"'
  export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
fi

# fzf keybindings
# All keybindings start with ^j to avoid conflicts with Zellij
# ^j^h - Search command history (fzf-history-widget)
# ^j^t - Search files (fzf-file-widget)
# ^j^f - cd into directory (uses zoxide with fzf, see zoxide.zsh)
# ^j^g^b - Git branch checkout (fzf-git-branch-widget)

# Custom keybindings with ^j prefix
bindkey '^j^h' fzf-history-widget
bindkey '^j^t' fzf-file-widget

# Git branch checkout widget
fzf-git-branch-widget() {
  # Check if we're in a git repository
  if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Not a git repository"
    zle reset-prompt
    return 1
  fi
  
  local branch
  branch=$(git branch -a | grep -v HEAD | sed 's/^[* ] //' | sed 's|remotes/origin/||' | sort -u | fzf --height=40% --reverse --preview 'git log --oneline --graph --color=always --decorate {}' --preview-window=right:60%)
  
  if [[ -n "$branch" ]]; then
    # Remove any leading/trailing whitespace
    branch=$(echo "$branch" | xargs)
    git checkout "$branch"
    zle reset-prompt
  fi
}
zle -N fzf-git-branch-widget
bindkey '^j^g' fzf-git-branch-widget

