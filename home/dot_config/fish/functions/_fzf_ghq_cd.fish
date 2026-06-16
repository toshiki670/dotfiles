# ロジックは Rust バイナリ（crates/fzf-picker の fzf-ghq-cd, ~/.cargo/bin）に移した。
# 別プロセスでは親シェルの cd を変えられないため、ここはバイナリが選択結果として
# 出力した絶対パスへ cd するだけの薄い shim（zoxide / starship と同じ定石）。
# fzf 本体はバイナリが TTY を継承して実行し、選択パスのみ stdout に出す。
function _fzf_ghq_cd
    set -l dest (command fzf-ghq-cd)
    or return
    test -n "$dest"; and cd "$dest"
    commandline -f repaint
end
