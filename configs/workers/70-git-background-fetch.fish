# After each interactive command, may spawn a subprocess that runs throttled
# `git fetch` (see the git-background-fetch-worker binary, src/workers). Foreground only
# registers the event and starts that subprocess — no repo or network work here.

function __git_background_fetch_maybe --on-event fish_postexec
    status is-interactive || return
    # Subprocess does not inherit fish globals; pass throttle override when set.
    if set -q GIT_FETCH_THROTTLE_SEC
        env GIT_FETCH_THROTTLE_SEC="$GIT_FETCH_THROTTLE_SEC" command git-background-fetch-worker &
    else
        command git-background-fetch-worker &
    end
    # Do not keep the worker in fish's job table (avoids exit warning while fetch runs).
    disown
end
