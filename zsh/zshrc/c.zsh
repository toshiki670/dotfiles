# 拡張子に応じたコマンドを実行
function rungcc(){
  gcc $1
  base=$1
  file=${base%.*}
  ./a.out
  rm -f a.out
}
alias -s {c,cpp}=rungcc
