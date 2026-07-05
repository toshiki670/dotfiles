# ロジックは Rust バイナリ（src/bin/fzf-picker の cdabbr サブコマンド。fzf-picker として
# ~/.cargo/bin に入る）に移した。別プロセスでは親シェルの cd を変えられないため、
# ここはバイナリが選択結果として出力した絶対パスへ cd するだけの薄い shim。
# 展開・fzf 選択はバイナリが行う。
function cdabbr --description 'cd by expanding prompt_pwd-style abbreviated path'
    set -l dest (command fzf-picker cdabbr $argv)
    or return
    test -n "$dest"; and cd "$dest"
end
