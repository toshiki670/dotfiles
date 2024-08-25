# For git
alias g='git'

# Git flow
# yay -S gitflow-avh
# apt install git-flow
readonly TRUE=1
readonly FALSE=0

git_flow_exists=$FALSE

if type "git-flow" > /dev/null 2>&1; then
  # for Arch Linux
  git_flow_exists=$TRUE
elif type "dpkg" > /dev/null 2>&1; then
  if dpkg -l | grep git-flow > /dev/null 2>&1; then
    # for Ubuntu
    git_flow_exists=$TRUE
  fi
fi

if [[ $git_flow_exists == $TRUE ]]; then
  alias Gfeature='git flow feature'
  alias Ghotfix='git flow hotfix'
  alias Ginit='git flow init'
  alias Grelease='git flow release'
  alias Gsupport='git flow support'
  alias Gversion='git flow version'
fi
