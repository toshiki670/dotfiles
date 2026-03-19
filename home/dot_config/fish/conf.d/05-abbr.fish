# Abbreviations (shell, docker, git, bat, claude, nvim, yt-dlp)

# ========== shell / misc ==========
abbr -a reload 'exec fish -l'

# ========== docker ==========
abbr -a d 'docker'
abbr -a dc 'docker compose'
abbr -a dce 'docker compose exec'

# ========== git ==========
abbr -a g 'git'

# --command git: git の直後のトークンのみ展開 (g s → git status)。-- 必須。
abbr -a --command git -- s 'status'
abbr -a --command git -- br 'branch'
abbr -a --command git -- d 'diff'
abbr -a --command git -- ds 'diff --staged'
abbr -a --command git -- l 'log --graph --all --pretty=format:\'%C(yellow)%h%C(cyan)%d%Creset %s %C(white)- %an, %ar%Creset\''
abbr -a --command git -- a 'add'
abbr -a --command git -- ap 'add -p'
abbr -a --command git -- ci 'commit'
abbr -a --command git -- cm 'commit -m '
abbr -a --command git -- sw 'switch'
abbr -a --command git -- new 'switch -c'
abbr -a --command git -- pre 'switch -'
abbr -a --command git -- pull 'pull'
abbr -a --command git -- push 'push'
abbr -a --command git -- pullre 'pull --rebase'
abbr -a --command git -- reset 'reset --hard HEAD'
abbr -a --command git -- rebase 'rebase -i HEAD~'
abbr -a --command git -- clean 'branch --merged | egrep -v \'(^[*+]|master|main)\' | xargs git branch -d; git fetch --prune'
# rebase は git で使用済みのため prrebase に (gh prrebase → gh pr merge --delete-branch --rebase)
abbr -a --command gh -- merge 'pr merge --delete-branch --merge'
abbr -a --command gh -- rebase 'pr merge --delete-branch --rebase'
abbr -a --command gh -- web 'pr view --web'
abbr -a --command gh -- switch 'pr checkout'
abbr -a --command gh -- b 'browse'
abbr -a --command gh -- 'switch' 'pr checkout'

# ========== bat ==========
abbr -a b 'bat'

# ========== claude ==========
abbr -a c 'claude'

# ========== nvim ==========
abbr -a --position anywhere v 'nvim'
abbr -a --position anywhere vr 'nvim -R'
# encoding: nvim の引数として cu / ce / cs を展開 (続けて path を入力)
# -- 必須: オプション終了を示し NAME と EXPANSION を渡す (無いと "Requires at least two arguments")
abbr -a --command nvim -- cu '-c ":e ++enc=utf8" '
abbr -a --command nvim -- ce '-c ":e ++enc=euc-jp" '
abbr -a --command nvim -- cs '-c ":e ++enc=shift_jis" '

# ========== yt-dlp (only when YT_BROWSER is set) ==========
if set -q YT_BROWSER
  abbr -a yt 'yt-dlp --cookies-from-browser $YT_BROWSER'
end
