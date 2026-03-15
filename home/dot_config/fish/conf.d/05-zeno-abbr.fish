# Abbreviations matching zeno config.yml + common aliases
# Fish abbr expand on space; no context, so use prefix (g for git, v for nvim)

# ========== shell / misc ==========
abbr -a reload 'exec fish -l'

# ========== docker ==========
abbr -a d 'docker'
abbr -a dc 'docker compose'
abbr -a dce 'docker compose exec'

# ========== git ==========
abbr -a g 'git'

abbr -a gs 'git status'
abbr -a gb 'git branch'
abbr -a gd 'git diff'
abbr -a gds 'git diff --staged'
abbr -a gl 'git log --graph --all --pretty=format:\'%C(yellow)%h%C(cyan)%d%Creset %s %C(white)- %an, %ar%Creset\''
abbr -a ga 'git add'
abbr -a gap 'git add -p'
abbr -a gc 'git commit'
abbr -a gcm 'git commit -m '
abbr -a gsw 'git switch'
abbr -a gnew 'git switch -c'
abbr -a gpre 'git switch -'
abbr -a gpull 'git pull'
abbr -a gpush 'git push'
abbr -a gpullre 'git pull --rebase'
abbr -a greset 'git reset --hard HEAD'
abbr -a grebase 'git rebase -i HEAD~'
abbr -a gclean 'git branch --merged | egrep -v \'(^[*+]|master|main)\' | xargs git branch -d; git fetch --prune'
abbr -a gpr 'gh pr'
abbr -a gweb 'gh pr view --web'

# ========== bat ==========
abbr -a b 'bat'

# ========== claude ==========
abbr -a c 'claude'

# ========== nvim ==========
abbr -a v 'nvim'
abbr -a vr 'nvim -R'
# encoding (expand then type path): vcu / vce / vcs
abbr -a vcu 'nvim -c ":e ++enc=utf8" '
abbr -a vce 'nvim -c ":e ++enc=euc-jp" '
abbr -a vcs 'nvim -c ":e ++enc=shift_jis" '

# ========== yt-dlp (only when YT_BROWSER is set) ==========
if set -q YT_BROWSER
  abbr -a yt 'yt-dlp --cookies-from-browser $YT_BROWSER'
end
