# PATH: DOTFILES/bin, ~/.local/bin, ~/.cargo/bin (match common.zsh)
# Prepend in order: cargo, local, dotfiles so PATH has dotfiles first
contains -- $HOME/.cargo/bin $fish_user_paths || set -gx fish_user_paths $HOME/.cargo/bin $fish_user_paths
contains -- $HOME/.local/bin $fish_user_paths || set -gx fish_user_paths $HOME/.local/bin $fish_user_paths
if set -q DOTFILES
  contains -- $DOTFILES/bin $fish_user_paths || set -gx fish_user_paths $DOTFILES/bin $fish_user_paths
end
