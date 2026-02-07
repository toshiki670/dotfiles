# pbcopy-path / pbcopy-file: clipboard helpers (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
  function pbcopy-path {
    if (( $# != 1 )); then
      echo "usage: pbcopy-path <file>" 1>&2
      return 1
    fi
    readlink -f "$1" | pbcopy
  }

  function pbcopy-file {
    if (( $# != 1 )); then
      echo "usage: pbcopy-file <file>" 1>&2
      return 1
    fi
    pbcopy < "$1"
  }
fi
