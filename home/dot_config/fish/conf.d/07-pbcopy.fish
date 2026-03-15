# pbcopy-path / pbcopy-file (macOS only, match pbcopy.zsh)
# abbr: p-path, p-file

if not string match -q "darwin*" (uname -s)
  exit 0
end

function pbcopy-path --description 'Copy absolute path of file to clipboard'
  if test (count $argv) -ne 1
    echo "usage: pbcopy-path <file>" >&2
    return 1
  end
  path resolve "$argv[1]" | pbcopy
end

function pbcopy-file --description 'Copy file contents to clipboard'
  if test (count $argv) -ne 1
    echo "usage: pbcopy-file <file>" >&2
    return 1
  end
  pbcopy < "$argv[1]"
end

abbr -a p-path 'pbcopy-path '
abbr -a p-file 'pbcopy-file '
