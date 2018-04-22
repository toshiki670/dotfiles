# ログインシェルとインタラクティブシェルの場合だけ読み込まれる。
# シェルスクリプトでは不要な場合に記述する。
# 言語を日本語に設定KCODEにUTF-8を設定
export LANG=ja_JP.UTF-8
export KCODE=u

# 基本パス設定
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/sbin:$PATH"
export PARH="/usr/local/sbin:$PATH"

#ZSHの起動した関数の時間計測 .zshrc参照
zmodload zsh/zprof && zprof
