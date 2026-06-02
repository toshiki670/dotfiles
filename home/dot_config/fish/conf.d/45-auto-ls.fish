# Auto-ls: list the directory after an interactive command changes it
# (cd / z / pushd / abbr-expanded cd, ...).
#
# We hook `fish_postexec` (fires after an interactive command line runs) rather
# than `--on-variable PWD` (fires on *any* cd, synchronously). A cd performed
# inside a key binding fires --on-variable PWD mid-binding, so the listing is
# printed just before fish repaints the prompt and gets clobbered. With
# fish_postexec, in-binding cd's stay quiet by default; pickers that *should*
# list (ghq jump in _fzf_ghq_repo) run their cd via `commandline -f execute`,
# so they pass through postexec too, while ones that must stay silent (the
# worktree-delete safety cd in _fzf_worktree) simply don't.
set -g __auto_ls_last_pwd $PWD
function __auto_ls_on_pwd --on-event fish_postexec
    status is-interactive || return
    test "$PWD" = "$__auto_ls_last_pwd"; and return
    set -g __auto_ls_last_pwd $PWD
    ls
end
