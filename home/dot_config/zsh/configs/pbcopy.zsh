# pbcopy-path: copy absolute path of a file to clipboard (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
  function pbcopy-path {
    if (( $# != 1 )); then
      echo "usage: pbcopy-path <file>" 1>&2
      return 1
    fi
    readlink -f "$1" | pbcopy
  }
fi
