# ロジックは Rust バイナリ（crates/fzf-picker の fzf-worktree-remove）に移した。
# 選択・確認・削除はバイナリが行う。削除対象 worktree の内側にいた場合だけ、退避先
# （メイン worktree）パスを stdout に出すので、fish はそこへ cd するだけの薄い shim。
function _fzf_worktree_remove
    set -l dest (command fzf-worktree-remove)
    or return
    test -n "$dest"; and cd "$dest"
    commandline -f repaint
end
