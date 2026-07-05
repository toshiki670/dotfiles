# ロジックは Rust バイナリ（src/bin/fzf-picker の fzf-gh サブコマンド。fzf-picker として
# ~/.cargo/bin に入る）に移した。
# Issue/PR 混在リストを fzf で選び、種別に応じたアクションを fzf で選んで、
# 組み上げた `gh <group> <action> <number>` を stdout に出す。fish はそれを
# コマンドラインへ挿入するだけの薄い shim（実行は Enter で自分が行う）。
function _fzf_gh
    set -l cmd (command fzf-picker fzf-gh)
    or return
    test -n "$cmd"; and commandline -i -- "$cmd "
    commandline -f repaint
end
