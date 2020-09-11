# For git
alias g='git'
alias gad='git add'
alias gap='git add -p'
alias gb='git branch'
alias gch='git checkout'
alias gd='git diff'
alias gds='git diff --staged'
alias gs='git status'
alias gpull='git pull'
alias gpullre='git pull --rebase'
alias gpush='git push'
alias glog="git log --graph --pretty=format:'%C(yellow)%h%C(cyan)%d%Creset %s %C(white)- %an, %ar%Creset'"
alias g-reset-hard='git reset --hard HEAD'


# Git flow
# yay -S gitflow-avh
# apt install git-flow
readonly TRUE=1
readonly FALSE=0

git_flow_exists=$FALSE

if type "git-flow" > /dev/null 2>&1; then
  # for Arch Linux
  git_flow_exists=$TRUE
elif dpkg -l | grep git-flow > /dev/null 2>&1; then
  # for Ubuntu
  git_flow_exists=$TRUE
fi

if [[ $git_flow_exists == $TRUE ]]; then
  alias @feature='git flow feature'
  alias @hotfix='git flow hotfix'
  alias @init='git flow init'
  alias @release='git flow release'
  alias @support='git flow support'
  alias @version='git flow version'
fi
