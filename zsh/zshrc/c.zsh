# 拡張子に応じたコマンドを実行
function rungcc(){
  output_file_name=$(md5sum $1 | awk '{print $2"."$1}')'.out'
  gcc -o $output_file_name $1
  ./$output_file_name
  rm -f $output_file_name
}
alias -s {c,cpp}=rungcc
