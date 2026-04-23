# Background worker for git fetch throttling. Spawned only from:
#   command fish -c _git_background_fetch_worker &
# Optional: parent may pass GIT_FETCH_THROTTLE_SEC via env (see conf.d hook).

function _git_background_fetch_worker
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

    env GIT_TERMINAL_PROMPT=0 command git -C "$top" fetch --quiet --no-write-fetch-head >/dev/null 2>&1
end
