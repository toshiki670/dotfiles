
" Escape terminal by ESC
if has('nvim')
  tnoremap <silent> <ESC> <C-\><C-n>
endif

" ESC連打でハイライト解除
noremap <Esc><Esc> :nohlsearch<CR><Esc>

" nNで移動する時画面中央に移動する
noremap n nzz
noremap N Nzz
noremap <Space>na *:%s///g<LEFT><LEFT>

" 自動挿入
" inoremap {<Enter> {}<Left><CR><ESC><S-o>
" inoremap [ []<LEFT>
" inoremap ( ()<LEFT>
" inoremap ' ''<LEFT>
" inoremap " ""<LEFT>


" コマンド入力用の設定
" noremap ; :
" noremap : ;


" https://qiita.com/tekkoc/items/98adcadfa4bdc8b5a6ca
" 画面分割設定
nnoremap s <Nop>
" スピリット画面移動
nnoremap sw <C-w>w
nnoremap sj <C-w>j
nnoremap sk <C-w>k
nnoremap sl <C-w>l
nnoremap sh <C-w>h
" スピリット画面そのものを移動
nnoremap sJ <C-w>J
nnoremap sK <C-w>K
nnoremap sL <C-w>L
nnoremap sH <C-w>H
" nnoremap sr <C-w>r
" タブ移動
nnoremap sm gt
nnoremap sn gT

" スピリットの大きさを整える
nnoremap s= <C-w>=
" 縦横最大化
nnoremap so <C-w>_<C-w>|

nnoremap sN :<C-u>bn<CR>
nnoremap sP :<C-u>bp<CR>
" 新規タブ
nnoremap st :<C-u>tabnew<CR>
" タブ一覧
nnoremap sT :<C-u>Unite tab<CR>
" 水平分割
nnoremap sr :<C-u>sp<CR>
" 垂直分割
nnoremap sv :<C-u>vs<CR>
" ウィンドウを閉じる
nnoremap sq :<C-u>q<CR>
" バッファを閉じる
nnoremap sQ :<C-u>bd<CR>
" バッファ移動
" nnoremap sm :bn<CR>
" nnoremap sn :bp<CR>
