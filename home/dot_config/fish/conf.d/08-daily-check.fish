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

	set -g _daily_check_ts "$timestamp_file"
	set -g _daily_check_cache "$cache_dir"
	set -g _daily_check_result "$result_file"
	function _daily_check_bg
		set -l today (date +%Y-%m-%d)
		if test -f "$_daily_check_ts"
			set -l last_run (cat "$_daily_check_ts")
			test "$last_run" = "$today" && return 0
		end
		mkdir -p "$_daily_check_cache"
		echo -n "$today" > "$_daily_check_ts"

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
		string join "\n" $lines > "$_daily_check_result"
	end
	_daily_check_bg &
end

status is-interactive && daily-check
