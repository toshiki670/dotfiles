# Initialize and Load Sheldon
if type "sheldon" > /dev/null 2>&1; then
  eval "$(sheldon source)"

  # Re-initialize completion system after sheldon adds plugins to fpath
  # This ensures zsh-completions and other completion plugins are recognized
  compinit
else
  echo "${0##*/}: sheldon isn't found. Please install sheldon." 1>&2
fi
