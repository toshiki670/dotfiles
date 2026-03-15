# Prompt: path (blue) + branch + * (dirty, sakura pink) + ⇡/⇣ (ahead/behind, Pure) + ci-status. Right = time.
# Defensive: only parse ahead/behind when we have a status line; only test numeric ahead/behind.

function fish_prompt
	set -l path_str (prompt_pwd)
	set -l branch_str ""
	set -l dirty ""
	set -l ahead 0
	set -l behind 0

	set -l git_out (git rev-parse --is-inside-work-tree --abbrev-ref HEAD 2>/dev/null)
	if test (count $git_out) -ge 2 && test "$git_out[1]" = true
		set branch_str "$git_out[2]"
		if test -n "$branch_str"
			set -l sb (git status -sb 2>/dev/null)
			test (count $sb) -gt 1 && set dirty "*"
			set -l line1 ""
			test (count $sb) -ge 1 && set line1 "$sb[1]"
			if test -n "$line1"
				set -l a_raw (string replace -r '.*\[ahead ([0-9]+).*' '$1' "$line1")
				set -l a (string trim "$a_raw[1]")
				string match -rq '^[0-9]+$' "$a" && set ahead $a
				set -l b_raw (string replace -r '.*\[behind ([0-9]+).*' '$1' "$line1")
				set -l b (string trim "$b_raw[1]")
				string match -rq '^[0-9]+$' "$b" && set behind $b
			end
		end
	end

	set_color blue
	echo -n "$path_str"
	set_color normal
	if test -n "$branch_str"
		set_color 888
		echo -n " $branch_str"
		set_color normal
		if test -n "$dirty"
			set_color FFB7C5
			echo -n "*"
			set_color normal
		end
		if string match -rq '^[0-9]+$' "$ahead" && test "$ahead" -gt 0
			set_color 888
			echo -n " ⇡$ahead"
			set_color normal
		end
		if string match -rq '^[0-9]+$' "$behind" && test "$behind" -gt 0
			set_color 888
			echo -n " ⇣$behind"
			set_color normal
		end
	end
	if functions -q ci_status_prompt
		ci_status_prompt
	end
	echo ""
	test $status = 0 && set_color magenta || set_color red
	echo -n "❯ "
	set_color normal
end

function fish_right_prompt
	date +%H:%M:%S
end
