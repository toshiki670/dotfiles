# Theme configure
# Install location
DIRCOLORS=${DOTFILES}/zsh/bundle/dircolors-solarized

# Dircolors installation
if $(type "git" > /dev/null 2>&1) && [[ ! -d $DIRCOLORS ]]; then
  git clone https://github.com/seebi/dircolors-solarized.git $DIRCOLORS
fi

# Dircolors activation
if [[ -d $DIRCOLORS ]]; then
  eval $(dircolors $DIRCOLORS)
  eval $(dircolors $DIRCOLORS/dircolors.ansi-universal)
fi


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
