# Daily package check: brew outdated / mise outdated once per calendar day.
# On startup: if result file exists, show it and remove in background.
# Otherwise start background job that checks date, runs brew/mise outdated, writes result.

function daily-check
    set -l cache_dir (string join "/" (test -n "$XDG_CACHE_HOME" && echo $XDG_CACHE_HOME || echo $HOME/.cache) fish)
    set -l timestamp_file "$cache_dir/daily-check-timestamp"
    set -l result_file "$cache_dir/daily-check-result"

    if test -f "$result_file"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "📦 Daily Package Check"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        cat "$result_file"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        rm -f "$result_file" &
        return
    end

    # Run the worker binary (Rust: src/workers, daily-check-worker) in the background
    # so brew/mise outdated do not block the prompt.
    env DAILY_CHECK_TS="$timestamp_file" \
        DAILY_CHECK_CACHE="$cache_dir" \
        DAILY_CHECK_RESULT="$result_file" \
        daily-check-worker &
end

status is-interactive && daily-check
