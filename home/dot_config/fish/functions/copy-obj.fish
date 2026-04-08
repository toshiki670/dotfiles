function copy-obj --description 'Copy file as file object to clipboard (Finder-pasteable, macOS only)'
    if test (uname -s) != Darwin
        echo "copy-obj: macOS only" >&2
        return 1
    end
    if test (count $argv) -ne 1
        echo "usage: copy-obj <file>" >&2
        return 1
    end
    set -l abspath (path resolve $argv[1])
    if not test -e $abspath
        echo "copy-obj: not found: $abspath" >&2
        return 1
    end
    osascript -e "set the clipboard to POSIX file \"$abspath\""
end
