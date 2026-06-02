# Auto-ls: list the directory after an interactive command that changed it
# (cd / z / pushd / abbr-expanded cd, ...).
#
# We capture $PWD at fish_preexec (just before the command runs) and compare it
# at fish_postexec (just after). Listing only when the command *itself* moved us
# has two effects:
#  - cd / z / pushd at the prompt, and the ghq jump (which runs its cd through
#    `commandline -f execute`), all list, because the command changed $PWD;
#  - a cd performed *inside* a key binding (e.g. the worktree-delete safety cd)
#    does not desync any "last pwd" state, so the next unrelated command no
#    longer triggers a stray listing. A simpler `--on-variable PWD` /
#    last-pwd-snapshot hook gets this wrong: the in-binding cd updates $PWD
#    without firing postexec, leaving the snapshot stale so the *following*
#    command spuriously lists.
#
# Pickers that DO want a listing after an in-binding cd (worktree delete, when
# it evacuates to main) run ` ls` themselves via `commandline -f execute`.
function __auto_ls_preexec --on-event fish_preexec
    set -g __auto_ls_pwd_before $PWD
end
function __auto_ls_postexec --on-event fish_postexec
    status is-interactive || return
    set -q __auto_ls_pwd_before; or return
    test "$PWD" = "$__auto_ls_pwd_before"; and return
    ls
end
