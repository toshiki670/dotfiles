PS1="\[\e[0;36m\]\u@\H \W\[\e[0m\]\$ "

# ls command series
if type "exa" > /dev/null 2>&1; then
  # pacman -S exa
  alias ls='exa'
  alias ll='exa  -l  -gh --time-style long-iso --git'
  alias la='exa  -a'
  alias lla='exa -la -gh --time-style long-iso --git'
else
  alias ls='ls  --color=auto'
  alias ll='ls  -l'
  alias la='ls  -a'
  alias lla='ls -la'
fi


export HISTCONTROL=ignoredups
