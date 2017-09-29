export PATH=$PATH:/Applications/MAMP/Library/bin
PS1="\[\e[0;34m\]⌘ \[\e[0;36m\] [\w] ->\n[\t \s]\[\e[0m\]\$ "


export PYENV_ROOT="${HOME}/.pyenv"
export PATH="${PYENV_ROOT}/bin:$PATH"
eval "$(pyenv init -)"

eval `/usr/local/opt/coreutils/libexec/gnubin/dircolors ~/.dircolors-solarized/dircolors.ansi-dark`
alias ls='gls --color=auto'

# added by Anaconda3 5.0.0 installer
export PATH="/Users/tsk/anaconda3/bin:$PATH"
