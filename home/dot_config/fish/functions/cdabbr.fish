# ロジックは Rust バイナリ（src/fzf_picker の cdabbr, ~/.cargo/bin）に移した。
# 別プロセスでは親シェルの cd を変えられないため、ここはバイナリが選択結果として
# 出力した絶対パスへ cd するだけの薄い shim。展開・fzf 選択はバイナリが行う。
function cdabbr --description 'cd by expanding prompt_pwd-style abbreviated path'
    set -l dest (command cdabbr $argv)
    or return
    test -n "$dest"; and cd "$dest"
end
