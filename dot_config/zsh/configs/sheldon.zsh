# Initialize and Load Sheldon
if type "sheldon" > /dev/null 2>&1; then
  eval "$(sheldon source)"
else
  echo "${0##*/}: sheldon isn't found. Please install sheldon." 1>&2
fi
