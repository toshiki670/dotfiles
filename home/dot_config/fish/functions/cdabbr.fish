function _cdabbr_expand_recursive
    set -l base "$argv[1]"
    set -l segments $argv[2..-1]

    if test (count $segments) -eq 0
        echo "$base"
        return
    end

    set -l seg "$segments[1]"
    set -l rest $segments[2..-1]

    set -l subdirs (find "$base" -maxdepth 1 -mindepth 1 -type d 2>/dev/null)
    for d in $subdirs
        set -l name (path basename "$d")
        if string match -q "$seg*" "$name"
            _cdabbr_expand_recursive "$d" $rest
        end
    end
end

function _cdabbr_select_with_fzf
    set -l candidates $argv
    string join \n $candidates \
        | fzf --height=40% \
        --reverse \
        --select-1 \
        --exit-0 \
        --preview='eza -1 --color=always --icons {}' \
        --preview-window='right:50%:wrap' \
        --bind='ctrl-/:toggle-preview'
    # Treat ESC/Ctrl-C as "no selection" instead of command error.
    return 0
end

function _cdabbr_select_without_fzf
    set -l candidates $argv

    if test (count $candidates) -eq 1
        echo "$candidates[1]"
        return 0
    end

    echo "cdabbr: multiple matches (install fzf to choose):" >&2
    for c in $candidates
        echo "  $c" >&2
    end
    return 1
end

function cdabbr --description 'cd by expanding prompt_pwd-style abbreviated path'
    set -l abbr_path "$argv[1]"
    if test -z "$abbr_path"
        echo "usage: cdabbr <abbreviated-path>" >&2
        return 1
    end

    set -l base
    set -l segments (string split '/' "$abbr_path")

    if string match -q '~*' "$abbr_path"
        set base $HOME
        set -e segments[1]
    else if string match -q '/*' "$abbr_path"
        set base /
        set -e segments[1]
    else
        echo "cdabbr: path must start with ~ or /" >&2
        return 1
    end

    set -l segs
    for s in $segments
        if test -n "$s"
            set segs $segs $s
        end
    end

    set -l candidates (_cdabbr_expand_recursive "$base" $segs | string trim)

    if test (count $candidates) -eq 0
        echo "cdabbr: no matching path for '$abbr_path'" >&2
        return 1
    end

    set -l selector _cdabbr_select_with_fzf
    if not command -q fzf
        set selector _cdabbr_select_without_fzf
    end

    set -l result ($selector $candidates)
    if test $status -ne 0
        return 1
    end

    if test -n "$result"
        cd "$result"
    end
end
