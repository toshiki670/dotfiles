# ロジックは Rust バイナリ（src/fzf_picker の fzf-gh）に移した。
# Issue/PR 混在リストを fzf で選び、種別に応じたアクションを fzf で選んで、
# 組み上げた `gh <group> <action> <number>` を stdout に出す。fish はそれを
# コマンドラインへ挿入するだけの薄い shim（実行は Enter で自分が行う）。
function _fzf_gh
    set -l cmd (command fzf-gh)
    or return
    test -n "$cmd"; and commandline -i -- "$cmd "
    commandline -f repaint
end
