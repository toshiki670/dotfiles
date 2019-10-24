#!/bin/bash

# Zsh Setup
ln -sf ~/dotfiles/zsh/.zshenv ~/.zshenv
ln -sf ~/dotfiles/zsh/.zlogin ~/.zlogin
ln -sf ~/dotfiles/zsh/.zshrc ~/.zshrc

# Vim Setup
mkdir -p ~/.config/nvim/
ln -sf ~/dotfiles/vim/.vimrc ~/.config/nvim/init.vim

# Tmux Setup
ln -sf ~/dotfiles/tmux/.tmux.conf ~/.tmux.conf

