# Abbreviations matching zeno.zsh config.yml (git, bat, claude, nvim)
# Fish abbr expand on space; no context, so use prefix (g for git, v for nvim)

# ========== git ==========
abbr -a g 'git'

abbr -a g-s 'git status'
abbr -a g-b 'git branch'
abbr -a g-d 'git diff'
abbr -a g-ds 'git diff --staged'
abbr -a g-l 'git log --graph --all --pretty=format:\'%C(yellow)%h%C(cyan)%d%Creset %s %C(white)- %an, %ar%Creset\''
abbr -a g-a 'git add'
abbr -a g-ap 'git add -p'
abbr -a g-c 'git commit'
abbr -a g-cm 'git commit -m '
abbr -a g-ch 'git checkout'
abbr -a g-sw 'git switch'
abbr -a g-new 'git switch -c'
abbr -a g-pre 'git switch -'
abbr -a g-pullre 'git pull --rebase'
abbr -a g-reset 'git reset --hard HEAD'
abbr -a g-rebase 'git rebase -i HEAD~'
abbr -a g-clean 'git branch --merged | egrep -v \'(^[*+]|master|main)\' | xargs git branch -d; git fetch --prune'
abbr -a g-web 'gh pr view --web'

# ========== bat ==========
abbr -a b 'bat'

# ========== claude ==========
abbr -a c 'claude'

# ========== nvim ==========
abbr -a v 'nvim'
abbr -a v-r 'nvim -R'
# encoding (expand then type path): v-cu / v-ce / v-cs
abbr -a v-cu 'nvim -c ":e ++enc=utf8" '
abbr -a v-ce 'nvim -c ":e ++enc=euc-jp" '
abbr -a v-cs 'nvim -c ":e ++enc=shift_jis" '
# cheat sheet (DOTFILES set by env)
abbr -a v-cheet 'nvim $DOTFILES/vim/cheatsheet/common.md'
