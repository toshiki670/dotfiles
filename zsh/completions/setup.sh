#!/bin/bash

curl -L https://raw.githubusercontent.com/docker/compose/$(docker-compose version --short)/contrib/completion/zsh/_docker-compose > ~/dotfiles/zsh/completions/_docker-compose



curl -L https://raw.githubusercontent.com/docker/cli/master/contrib/completion/zsh/_docker > ~/dotfiles/zsh/completions/_docker
