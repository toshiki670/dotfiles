# Background daily check worker. Spawned via:
#   env DAILY_CHECK_TS=... DAILY_CHECK_CACHE=... DAILY_CHECK_RESULT=... fish -c '_daily_check_worker'
# (Fish does not truly background named functions with `&`; use a subprocess.)

function _daily_check_worker
    set -l today (date +%Y-%m-%d)
    if test -f "$DAILY_CHECK_TS"
        set -l last_run (cat "$DAILY_CHECK_TS")
        test "$last_run" = "$today" && return 0
    end
    mkdir -p "$DAILY_CHECK_CACHE"
    echo -n "$today" >"$DAILY_CHECK_TS"

    set -l brew_out ""
    set -l mise_out ""
    command -q brew && set brew_out (brew outdated 2>/dev/null)
    command -q mise && set mise_out (mise outdated 2>/dev/null)

    set -l has_out 0
    test -n "$brew_out" && set has_out 1
    test -n "$mise_out" && set has_out 1
    test $has_out -eq 0 && return 0

    set -l lines "=== Homebrew Outdated Packages ===" ""
    if test -n "$brew_out"
        set lines $lines $brew_out "" ""
    end
    if test -n "$mise_out"
        set lines $lines "=== Mise Outdated Tools ===" "" $mise_out "" ""
    end
    string join "\n" $lines >"$DAILY_CHECK_RESULT"
end
