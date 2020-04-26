# 履歴ファイルの保存先
export HISTFILE=${HOME}/.zsh_history

# メモリに保存される履歴の件数
export HISTSIZE=3072

# 履歴ファイルに保存される履歴の件数
export SAVEHIST=1000000

# 重複を記録しない
setopt hist_ignore_dups

# 開始と終了を記録
setopt EXTENDED_HISTORY

# 全履歴を一覧表示する
function history-all { history -E 1 }
