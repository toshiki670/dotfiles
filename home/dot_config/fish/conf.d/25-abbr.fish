# Abbreviations (shell, docker, git, bat, claude, nvim, yt-dlp)

# ========== shell / misc ==========
abbr --add reload 'exec fish -l'

# ========== pbcopy (macOS only; one-liner + --set-cursor, see abbr -h) ==========
# Default % marker is removed; cursor lands there after expand.
if test (uname -s) = Darwin
    abbr --add p-path --set-cursor 'path resolve % | pbcopy; echo (pbpaste)'
    abbr --add p-file --set-cursor 'pbcopy < %'
end

# ========== docker ==========
abbr --add d docker
abbr --add dc 'docker compose'
abbr --add dce 'docker compose exec'

# ========== git ==========
abbr --add g git

# 一軍: 単体で使う頻度が高いものは --command git を使わずに展開
abbr --add gs 'git status'
abbr --add gd 'git diff'
abbr --add gds 'git diff --staged'
abbr --add ga 'git add'
abbr --add gap 'git add -p'
abbr --add gc 'git commit'
abbr --add gcm 'git commit -m '
abbr --add gl 'git log --graph --all --pretty=format:\'%C(yellow)%h%C(cyan)%d%Creset %s %C(white)- %an, %ar%Creset\''
abbr --add gsw 'git switch'
abbr --add gnew 'git switch -c'
abbr --add gpre 'git switch -'
abbr --add gpl 'git pull'
abbr --add gph 'git push'

# 二軍: 誤爆しやすい/用途が限定的なものは git 配下に限定
# --command git: git の直後のトークンのみ展開 (g rebase → git rebase ...)
abbr --add --command git -- br branch
abbr --add --command git -- pullre 'pull --rebase'
abbr --add --command git -- reset 'reset --hard HEAD'
abbr --add --command git -- rebase 'rebase -i HEAD~'
abbr --add --command git -- clean 'branch --merged | egrep -v \'(^[*+]|master|main)\' | xargs git branch -d; git fetch --prune'
abbr --add --command git -- tags "for-each-ref --sort=-taggerdate --format='%(taggerdate:short) %(tag) %(taggername) %(subject)' refs/tags"
# rebase は git で使用済みのため prrebase に (gh prrebase → gh pr merge --delete-branch --rebase)
# abbr --add --command gh -- show 'pr view'
# abbr --add --command gh -- diff 'pr diff'
# abbr --add --command gh -- merge 'pr merge --delete-branch --merge'
# abbr --add --command gh -- rebase 'pr merge --delete-branch --rebase'
# abbr --add --command gh -- web 'pr view --web'
# Stacked PRs: open the parent PR (head = current PR base)
# abbr --add --command gh -- pweb 'pr view "$(gh pr view --json baseRefName --jq \'.baseRefName\')" --web'
# abbr --add --command gh -- switch 'pr checkout'
abbr --add --command gh -- b browse
abbr --add --command gh -- i issue
# Pull Request
abbr --add gp 'gh pr'
abbr --add gpv 'gh pr view'
abbr --add gpshow 'gh pr view'
abbr --add gpd 'gh pr diff'
abbr --add gpmerge 'gh pr merge --delete-branch --merge'
abbr --add gprebase 'gh pr merge --delete-branch --rebase'
abbr --add gpsquash 'gh pr merge --delete-branch --squash'
abbr --add gpw 'gh pr view --web'
abbr --add gppw 'gh pr view "$(gh pr view --json baseRefName --jq \'.baseRefName\')" --web'
abbr --add gpc 'gh pr checkout'
abbr --add gpswitch 'gh pr checkout'
abbr --add gpci 'gh pr checks'
abbr --add gpciw 'gh pr checks --watch --fail-fast'

# ========== bat ==========
abbr --add b bat

# ========== claude ==========
abbr --add c claude

# ========== nvim ==========
abbr --add --position anywhere v nvim
abbr --add --position anywhere vr 'nvim -R'
abbr --add vf 'nvim (fzf)'
abbr --add vrf 'nvim -R (fzf)'

# encoding: nvim の引数として cu / ce / cs を展開 (続けて path を入力)
# -- 必須: オプション終了を示し NAME と EXPANSION を渡す (無いと "Requires at least two arguments")
abbr --add --command nvim -- cu '-c ":e ++enc=utf8" '
abbr --add --command nvim -- ce '-c ":e ++enc=euc-jp" '
abbr --add --command nvim -- cs '-c ":e ++enc=shift_jis" '

# ========== zoxied ==========
if command -q zoxide
    abbr --add zb 'z ..'
    abbr --add zbb 'z ../..'
    abbr --add zbbb 'z ../../..'
    abbr --add zp 'z -'

    abbr --add zgit 'z (git rev-parse --show-toplevel)'
end

# ========== yt-dlp (only when YT_BROWSER is set) ==========
if set -q YT_BROWSER
    abbr --add yt 'yt-dlp --cookies-from-browser $YT_BROWSER'
end
