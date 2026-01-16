# Completions
# Setting completions
completions=${DOTFILES}/zsh/completions

# For docker
comp_docker=${completions}/_docker
if [[ ! -e ${comp_docker} ]] && type "docker" > /dev/null 2>&1; then
  mkdir -p ${completions}
  curl -L https://raw.githubusercontent.com/docker/cli/master/contrib/completion/zsh/_docker > ${comp_docker}
fi

# For docker-compose
comp_docker_compose=${completions}/_docker-compose
if [[ ! -e ${comp_docker_compose} ]] && type "docker-compose" > /dev/null 2>&1; then
  mkdir -p ${completions}
  curl -L https://raw.githubusercontent.com/docker/compose/$(docker-compose version --short)/contrib/completion/zsh/_docker-compose > ${comp_docker_compose}
fi

# TODO:今後、オリジナルのCompletionを実装するときに`Add directoryとComplationディレクトリの共通化を検討する`
# Add directory
if [ -e ${completions} ]; then
  fpath=(${completions} $fpath)
fi

# For docker
alias dce='docker-compose exec'
