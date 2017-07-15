export PATH=$PATH:/Applications/MAMP/Library/bin
PS1="\[\e[0;31m\]⌘ \[\e[0;36m\] [\w]\n\[\e[0;35m\]\t \s\[\e[0m\]\$ "


export PYENV_ROOT="${HOME}/.pyenv"
export PATH="${PYENV_ROOT}/bin:$PATH"
eval "$(pyenv init -)"
