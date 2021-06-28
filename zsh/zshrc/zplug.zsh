# Initialize and Install the Zplug
if type "git" > /dev/null 2>&1; then

  # Install location
  ZPLUG_HOME=${DOTFILES}/zsh/bundle/zplug

  # Zplug's installation
  if [[ ! -d $ZPLUG_HOME ]]; then
    git clone https://github.com/zplug/zplug $ZPLUG_HOME
  fi

  # Zplug's activation
  if [[ -d $ZPLUG_HOME ]]; then
    export ZPLUG_HOME
    export ZPLUG_BIN=$ZPLUG_HOME/bin
    export ZPLUG_CACHE_DIR=$ZPLUG_HOME/cache
    export ZPLUG_REPOS=$ZPLUG_HOME/repos
    source $ZPLUG_HOME/init.zsh

    zplug "zsh-users/zsh-completions"
    zplug "zsh-users/zsh-syntax-highlighting"
    zplug "zsh-users/zsh-autosuggestions"
    zplug "zsh-users/zaw"
    zplug "mafredri/zsh-async", from:github
    zplug "sindresorhus/pure", use:pure.zsh, from:github, as:theme
    zplug "mollifier/cd-gitroot"
    zplug "Tarrasch/zsh-bd"
    zplug "supercrabtree/k"
    # zplug "docker/cli", use:"contrib/completion/zsh/_docker"
    # zplug "starcraftman/zsh-git-prompt"

    zplug check || zplug install
    zplug load
  fi
fi
