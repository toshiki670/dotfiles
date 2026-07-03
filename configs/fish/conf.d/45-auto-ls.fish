# Auto-ls: after a directory change (cd / z / pushd / fzf pickers ...), re-run
# `ls` so the listing appears *below the destination prompt* — the same look the
# ghq/worktree pickers produce.
#
# The mechanism is to queue ` ls` as the next command:
#     commandline --replace -- ' ls'; commandline -f repaint; commandline -f execute
# `repaint` before `execute` is required; without it the queued ls corrupts the
# input buffer. The leading space keeps it out of history.
#
# WHERE we queue it matters, so there are two handlers split by an in-command
# flag:
#
#  - Command typed at the prompt (cd / z / pushd, incl. compound forms like
#    `cd dir && make`): queue it from fish_postexec, i.e. AFTER the whole command
#    line finished. Replacing the buffer mid-run would drop the rest of the line
#    (`&& make` would never run).
#
#  - cd inside a key binding (ghq/worktree pickers): never reaches postexec, but
#    the binding runs in editing context where execute is valid right away, so
#    __auto_ls_on_pwd handles it from the PWD variable handler.
#
# The flag routes each change to exactly one handler: during a command the PWD
# handler defers (postexec will fire); outside a command postexec never fires so
# the PWD handler takes over. Comparing $PWD avoids listing when the command did
# not actually move (e.g. a plain `echo`).
function __auto_ls_preexec --on-event fish_preexec
    set -g __auto_ls_in_cmd 1
    set -g __auto_ls_pwd_before $PWD
end
function __auto_ls_postexec --on-event fish_postexec
    set -e __auto_ls_in_cmd
    status is-interactive || return
    set -q __auto_ls_pwd_before; or return
    test "$PWD" = "$__auto_ls_pwd_before"; and return
    commandline --replace -- ' ls'
    commandline -f repaint
    commandline -f execute
end
function __auto_ls_on_pwd --on-variable PWD
    status is-interactive || return
    set -q __auto_ls_in_cmd; and return
    commandline --replace -- ' ls'
    commandline -f repaint
    commandline -f execute
end
