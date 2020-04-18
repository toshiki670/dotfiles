echo -n 'Writing ...'
{
  echo "### Dotfile's config ###"
  echo -e "[include]\n  path = ~/dotfiles/git/git.config\n"
  echo -e "[include]\n  path = ~/dotfiles/git/user.config\n"
  echo "### End ###"
} >> ~/.gitconfig

echo ' ok'
