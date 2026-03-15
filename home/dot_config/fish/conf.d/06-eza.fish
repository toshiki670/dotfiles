# eza - ls/la/lt (match eza.zsh)
if command -q eza
  function ls --wraps eza --description 'eza -l'
    eza --icons=always -l -gh --time-style long-iso --git $argv
  end
  function la --wraps eza --description 'eza -la'
    eza --icons=always -la -gh --time-style long-iso --git $argv
  end
  function lt --description 'eza tree with level and target'
    set -l level 2
    set -l target .

    if test (count $argv) -eq 1
      if test -e "$argv[1]"
        set target "$argv[1]"
      else if string match -qr '^[0-9]+$' -- "$argv[1]"
        set level "$argv[1]"
      else
        set target "$argv[1]"
      end
    else if test (count $argv) -eq 2
      set level "$argv[1]"
      set target "$argv[2]"
    end

    eza --icons=always -la -gh --time-style long-iso --git --tree --level=$level $target
  end
else
  function ls --wraps ls
    command ls --color=auto -lh $argv
  end
  function la --wraps ls
    command ls --color=auto -lah $argv
  end
end

abbr -a sls 'command ls'
