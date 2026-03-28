# After each interactive command, may spawn a subprocess that runs throttled
# `git fetch` (see _git_background_fetch_worker). Foreground only registers
# the event and starts that subprocess — no repo or network work here.

function __git_background_fetch_maybe --on-event fish_postexec
    status is-interactive || return
    # Subprocess does not inherit fish globals; pass throttle override when set.
    if set -q GIT_FETCH_THROTTLE_SEC
        env GIT_FETCH_THROTTLE_SEC="$GIT_FETCH_THROTTLE_SEC" command fish -c _git_background_fetch_worker &
    else
        command fish -c _git_background_fetch_worker &
    end
    # Do not keep the worker in fish's job table (avoids exit warning while fetch runs).
    disown
end
