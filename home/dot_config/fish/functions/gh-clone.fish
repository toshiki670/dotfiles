# gh-clone のロジックは Rust バイナリ（crates/gh-clone, ~/.cargo/bin）に移した。
# 別プロセスでは親シェルの cd を変えられないため、ここはバイナリが出力した
# 移設先パスへ cd するだけの薄い shim（zoxide / starship と同じ定石）。
function gh-clone --description 'Clone repository via gh and migrate with ghq'
    set -l migrated_path (command gh-clone $argv)
    or return
    test -n "$migrated_path"; and cd "$migrated_path"
end
