# CI status (GitHub PR/checks) in prompt. Simplified: no Zsh parity, no parent feedback.
# While fetching in background, keep showing the previous cached result (no ◐).
# First run (no cache yet): show nothing until the first result is ready.
# Result stored in universal var _ci_cache (entries "key|value|ts") keyed by directory + branch.
#
# Fish does not truly background a function with & (only external commands run async).
# Run the fetch in a separate fish process so the prompt returns immediately.

set -q CI_STATUS_CACHE_SECONDS || set -g CI_STATUS_CACHE_SECONDS 10

# Key = sanitized path + branch so each dir/branch has its own entry.
function _ci_status_key
	set -l out (git rev-parse --show-toplevel --abbrev-ref HEAD 2>/dev/null)
	test (count $out) -lt 2 && return 1
	set -l path_san (string replace -a "/" "_" "$out[1]")
	set -l br_san (string replace -a "/" "_" "$out[2]")
	echo (string join "_" $path_san $br_san)
end

function _ci_status_gh_available
	git ls-remote --get-url origin >/dev/null 2>&1 || return 1
	command -q gh && return 0
	return 1
end

# Look up key in _ci_cache (list of "key|value|ts"). Sets _ci_value and _ci_ts if found.
function _ci_status_get
	set -l key $argv[1]
	set -q _ci_cache || return 1
	for entry in $_ci_cache
		set -l parts (string split "|" "$entry" --max 3)
		test (count $parts) -lt 3 && continue
		test "$parts[1]" = "$key" || continue
		set -g _ci_value "$parts[2]"
		set -g _ci_ts "$parts[3]"
		return 0
	end
	return 1
end

# Output prompt segment from _ci_value (set by _ci_status_get). Return 0 if something shown.
function _ci_status_render
	set -q _ci_value || return 1
	set -l line $_ci_value
	set -l pr_state (string split "," "$line")[1]
	set -l checks_state (string split "," "$line")[2]
	set -l pr_sym ""
	switch "$pr_state"
		case ok
			set pr_sym (set_color green)"✓"(set_color normal)
		case merged
			set pr_sym (set_color green)"✔"(set_color normal)
		case waiting
			set pr_sym (set_color blue)"◐"(set_color normal)
		case ng closed
			set pr_sym (set_color red)"✗"(set_color normal)
		case draft
			set pr_sym (set_color blue)"D"(set_color normal)
		case ""
			return 1
		case "*"
			set pr_sym ""
	end
	test -z "$pr_sym" && return 1
	echo -n " $pr_sym"
	switch "$checks_state"
		case ok
			echo -n (set_color green)" ✓"(set_color normal)
		case in_progress
			echo -n (set_color yellow)" ◐"(set_color normal)
		case action_required
			echo -n (set_color magenta)" ⚠"(set_color normal)
		case ng
			echo -n (set_color red)" ✗"(set_color normal)
	end
	return 0
end

# Replace or append entry for key in _ci_cache. Value may contain commas.
function _ci_status_set
	set -l key $argv[1]
	set -l value $argv[2]
	set -l ts $argv[3]
	set -l new_cache
	for entry in $_ci_cache
		set -l k (string split "|" "$entry" --max 1)[1]
		test "$k" = "$key" && continue
		set -a new_cache $entry
	end
	set -a new_cache "$key|$value|$ts"
	set -U _ci_cache $new_cache
end

# Run from background. Writes result to _ci_cache via universal var.
function ci_status_fetch
	_ci_status_gh_available || return 0
	set -l key (_ci_status_key)
	test -z "$key" && return 0

	set -l pr_state ""
	set -l checks_state ""

	if command -q jq
		set -l pr_json (gh pr view --json state,mergedAt,closed,mergeable,mergeStateStatus,reviewDecision,isDraft 2>/dev/null)
		if test -n "$pr_json"
			set pr_state (echo "$pr_json" | jq -r '
				if . == null then ""
				elif .state == "MERGED" or (.mergedAt != null and .mergedAt != "") then "merged"
				elif .state == "CLOSED" or .closed == true then "closed"
				elif .mergeable == "CONFLICTING" or .reviewDecision == "CHANGES_REQUESTED" then "ng"
				elif .isDraft == true then "draft"
				elif .reviewDecision == "REVIEW_REQUIRED" or .mergeStateStatus == "BEHIND" then "waiting"
				else "ok"
				end
			' 2>/dev/null)
		end

		if test -n "$pr_state"
			set -l checks_json (gh pr checks --json state,bucket 2>/dev/null)
			if test -n "$checks_json"
				set checks_state (echo "$checks_json" | jq -r '
					if length == 0 then ""
					elif [.[] | select(.bucket == "fail" or .bucket == "cancel")] | length > 0 then "ng"
					elif [.[] | select(.state == "ACTION_REQUIRED")] | length > 0 then "action_required"
					elif [.[] | select(.bucket == "pending")] | length > 0 then "in_progress"
					else "ok"
					end
				' 2>/dev/null)
			end
		end
	end

	set -l now (date +%s)
	_ci_status_set "$key" "$pr_state,$checks_state" "$now"
	# Only clear loading if we were the one being loaded
	set -q _ci_loading && test "$_ci_loading" = "$key" && set -U _ci_loading ""
end

# Show cached result when available; while fetching, keep showing previous cache. First run: show nothing.
function ci_status_prompt
	set -l key (_ci_status_key)
	test -z "$key" && return 0
	_ci_status_gh_available || return 0

	set -l now (date +%s)
	set -l is_loading 0
	set -q _ci_loading && test "$_ci_loading" = "$key" && set is_loading 1

	# Currently fetching: show previous cache if any; else show nothing
	if test $is_loading -eq 1
		_ci_status_get "$key" && _ci_status_render
		return 0
	end

	# Have cached value
	if _ci_status_get "$key"
		_ci_status_render
		# If stale, trigger background refresh (keep showing current cache)
		if test (math "$now - $_ci_ts") -ge $CI_STATUS_CACHE_SECONDS
			set -U _ci_loading "$key"
			set -l config_dir "$HOME/.config/fish"
			set -q fish_config_dir && set config_dir "$fish_config_dir"
			fish -c "source $config_dir/conf.d/65-ci-status.fish; ci_status_fetch" &
		end
		return 0
	end

	# No cache (first run): start fetch, show nothing
	set -U _ci_loading "$key"
	set -l config_dir "$HOME/.config/fish"
	set -q fish_config_dir && set config_dir "$fish_config_dir"
	fish -c "source $config_dir/conf.d/65-ci-status.fish; ci_status_fetch" &
end
