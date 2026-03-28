# After each interactive command, run `git fetch` in the background when inside a
# repository. Throttled per repository (default 20s) via timestamps under
# $XDG_CACHE_HOME/fish/git-fetch-last/. Override interval with GIT_FETCH_THROTTLE_SEC.

# Foreground (runs in the postexec handler, blocks only briefly): detect repo,
# apply throttle, update stamp. Prints repo root to stdout when a fetch should run;
# prints nothing otherwise.
function __git_background_fetch_repo_if_ready
    status is-interactive || return

    set -l top (command git -C "$PWD" rev-parse --show-toplevel 2>/dev/null)
    test -z "$top" && return

    set -l interval 20
    set -q GIT_FETCH_THROTTLE_SEC && set interval $GIT_FETCH_THROTTLE_SEC

    set -l cache_root (string join / (test -n "$XDG_CACHE_HOME" && echo $XDG_CACHE_HOME || echo $HOME/.cache) fish git-fetch-last)
    command mkdir -p "$cache_root" || return

    set -l id (echo -n $top | command git hash-object --stdin | string sub -l 12)
    set -l stamp_file "$cache_root/$id"

    set -l now (command date +%s)
    if test -f "$stamp_file"
        set -l last (string trim (cat "$stamp_file"))
        if test -n "$last" && test (math "$now - $last") -lt $interval
            return
        end
    end

    echo $now >"$stamp_file"
    echo $top
end

# Background: network and git work; does not block the next prompt.
function __git_background_fetch_spawn -a repo_root
    env GIT_TERMINAL_PROMPT=0 command git -C "$repo_root" fetch --quiet >/dev/null 2>&1 &
end

function __git_background_fetch_maybe --on-event fish_postexec
    set -l top (__git_background_fetch_repo_if_ready)
    test -z "$top" && return
    __git_background_fetch_spawn "$top"
end
