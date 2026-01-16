# Dotfiles version management
# Checkout the latest version tag in the dotfiles repository

dotfiles-latest() {
  # Check if DOTFILES environment variable is set
  if [[ -z "$DOTFILES" ]]; then
    echo "Error: DOTFILES environment variable is not set" >&2
    return 1
  fi

  # Check if the directory exists
  if [[ ! -d "$DOTFILES" ]]; then
    echo "Error: Directory $DOTFILES does not exist" >&2
    return 1
  fi

  # Check if it's a git repository
  if ! git -C "$DOTFILES" rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: $DOTFILES is not a git repository" >&2
    return 1
  fi

  # Fetch the latest tags from remote
  echo "Fetching latest tags from remote..."
  if ! git -C "$DOTFILES" fetch --tags 2>&1; then
    echo "Error: Failed to fetch tags from remote" >&2
    return 1
  fi

  # Get the latest version tag
  local latest_tag
  latest_tag=$(git -C "$DOTFILES" tag --sort=-v:refname | head -1)

  if [[ -z "$latest_tag" ]]; then
    echo "Error: No version tags found in the repository" >&2
    return 1
  fi

  # Checkout the latest tag
  echo "Checking out latest version: $latest_tag"
  if git -C "$DOTFILES" -c advice.detachedHead=false checkout "$latest_tag" 2>&1; then
    echo "Successfully checked out $latest_tag"
    return 0
  else
    echo "Error: Failed to checkout $latest_tag" >&2
    return 1
  fi
}
