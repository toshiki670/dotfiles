# ロジックは Rust バイナリ（src/bin/fzf-picker の fzf-ghq-cd サブコマンド。fzf-picker として
# ~/.cargo/bin に入る）に移した。別プロセスでは親シェルの cd を変えられないため、
# ここはバイナリが選択結果として出力した絶対パスへ cd するだけの薄い shim（zoxide /
# starship と同じ定石）。fzf 本体はバイナリが TTY を継承して実行し、選択パスのみ
# stdout に出す。
function _fzf_ghq_cd
    set -l dest (command fzf-picker fzf-ghq-cd)
    or return
    test -n "$dest"; and cd "$dest"
    commandline -f repaint
end
