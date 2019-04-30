
" Escape terminal by ESC
if has('nvim')
  tno <silent> <ESC> <C-\><C-n>
endif

" ESC連打でハイライト解除
no <Esc><Esc> :nohlsearch<CR><Esc>

" nNで移動する時画面中央に移動する
no n nzz
no N Nzz
no <Space>na *:%s///g<LEFT><LEFT>

" 自動挿入
" ino {<Enter> {}<Left><CR><ESC><S-o>
" ino [ []<LEFT>
" ino ( ()<LEFT>
" ino ' ''<LEFT>
" ino " ""<LEFT>


" コマンド入力用の設定
" no ; :
" no : ;


" https://qiita.com/tekkoc/items/98adcadfa4bdc8b5a6ca
" 画面分割設定
nn s <Nop>
" スピリット画面移動
nn sw <C-w>w
nn sj <C-w>j
nn sk <C-w>k
nn sl <C-w>l
nn sh <C-w>h
" スピリット画面そのものを移動
nn sJ <C-w>J
nn sK <C-w>K
nn sL <C-w>L
nn sH <C-w>H
" nn sr <C-w>r
" タブ移動
nn sm gt
nn sn gT

" スピリットの大きさを整える
nn s= <C-w>=
" 縦横最大化
nn so <C-w>_<C-w>|

nn sN :<C-u>bn<CR>
nn sP :<C-u>bp<CR>
" 新規タブ
nn st :<C-u>tabnew<CR>
" タブ一覧
nn sT :<C-u>Unite tab<CR>
" 水平分割
nn sr :<C-u>sp<CR>
" 垂直分割
nn sv :<C-u>vs<CR>
" ウィンドウを閉じる
nn sq :<C-u>q<CR>
" バッファを閉じる
nn sQ :<C-u>bd<CR>
" バッファ移動
" nn sm :bn<CR>
" nn sn :bp<CR>

