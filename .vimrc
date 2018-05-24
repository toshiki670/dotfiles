" .vimrc
"           []
"   __    ____ __ _ _  _ __ ___
"   \ \  / /| || ' ' \| '__/ __|
"    \ \/ / | || | | || | | (__
" o   \__/  |_||_|_|_||_|  \___|
"

" dein.vim ここから -----------------------------------------------------
if &compatible
 set nocompatible
endif
" Add the dein installation directory into runtimepath
set runtimepath+=~/.cache/dein/repos/github.com/Shougo/dein.vim

if dein#load_state('~/.cache/dein')
 call dein#begin('~/.cache/dein')
 call dein#add('~/.cache/dein')
 " Add or remove your plugins here:
 call dein#add('Shougo/deoplete.nvim')

 " ここから
 " plugin on GitHub repo
 call dein#add('tpope/vim-fugitive')
 call dein#add('airblade/vim-gitgutter')

 "---------Ruby on Rails--------------
 call dein#add('tpope/vim-rails')
 " Ruby向けにendを自動挿入してくれる
 call dein#add('tpope/vim-endwise')
 " Rubyのシンタックス表示 
 call dein#add('vim-ruby/vim-ruby')
 "--------End---------------

 "--------html-------------
 call dein#add('mattn/emmet-vim')
 "--------------------------

 " lightline - https://github.com/itchyny/lightline.vim
 call dein#add('itchyny/lightline.vim')
 " Solarized
 call dein#add('altercation/vim-colors-solarized')

 " NERDTree
 call dein#add('scrooloose/nerdtree')
 call dein#add('jistr/vim-nerdtree-tabs')
 " NERDTreeToggleoggle の設定
 " autocmd vimenter * NERDTree
 " noremap tree :NERDTreeToggle<Enter>

 " For keybind
 call dein#add('kana/vim-submode')
 " ここまで

 if !has('nvim')
   call dein#add('roxma/nvim-yarp')
   call dein#add('roxma/vim-hug-neovim-rpc')
 endif

 call dein#end()
 call dein#save_state()
endif

filetype plugin indent on
syntax enable
" dein.vim ここまで -----------------------------------------------------


" Pluginの設定 ここから -------------------------------------------------
" Gitgutterの設定
set updatetime=250

" Emmetの設定
let g:user_emmet_leader_key='<C-f>'

" NERDTreeの設定
noremap tree :NERDTreeTabsToggle<Enter>

" Pluginの設定 ここまで -------------------------------------------------


" Solarized
syntax enable
set background=dark
colorscheme solarized
highlight LineNr ctermfg=darkcyan


" 個人設定
set laststatus=2

" 文字コードをUTF-8に設定
set fenc=utf-8

" 行番号の表示
set number

" カーソルが何行目の何列目に置かれているかを表示する
set ruler

" タイトルを表示
set title

" 括弧入力時の対応する括弧を表示
set showmatch
set ambiwidth=double

" Tabの設定
set expandtab
set tabstop=2
set softtabstop=2
set shiftwidth=2

" 複数行のクリップボードからの貼付けがおかしい時、:set paste をすると治る
set smartindent

" 不可視文字を可視化
set list
set listchars=tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%
set history=50
set virtualedit=block
set whichwrap=b,s,[,],<,>
set backspace=indent,eol,start

" 入力中のコマンドを表示する
set showcmd

" クリップボードにコピー
set clipboard+=unnamed

" ビープ音を可視化
set visualbell

" 検索系
" 検索文字列が小文字の場合は大文字小文字を区別なく検索
set ignorecase
" 検索文字列に大文字が含まれている場合は区別して検索する
set smartcase
" 検索文字列入力時に順次対象文字列にヒットさせる
set incsearch
" 検索時に最後まで行ったら最初に戻る
set wrapscan
" 検索語をハイライト表示
set hlsearch
" ESC連打でハイライト解除
nmap <Esc><Esc> :nohlsearch<CR><Esc>
" nNで移動する時画面中央に移動する
noremap n nzz
noremap N Nzz


" 自動挿入
inoremap {<Enter> {}<Left><CR><ESC><S-o>
" inoremap [<Enter> []<Left><CR><ESC><S-o>
inoremap [ []<LEFT>
" inoremap (<Enter> ()<Left><CR><ESC><S-o>
inoremap ( ()<LEFT>
inoremap ' ''<LEFT>
inoremap " ""<LEFT>


" コマンド入力用の設定
noremap ; :

" 編集されていない時に終了する
noremap QQ :q<Enter>

" インサートモードでも移動
inoremap <C-j>  <down>
inoremap <C-k>  <up>
inoremap <C-h>  <left>
inoremap <C-l>  <right>


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
" 大きさを揃える
nnoremap sO <C-w>=
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
" 現在のタブで開いたバッファ一覧
nnoremap sb :<C-u>Unite buffer_tab -buffer-name=file<CR>
" バッファ一覧
nnoremap sB :<C-u>Unite buffer -buffer-name=file<CR>

" Splitの調節
call submode#enter_with('bufmove', 'n', '', 's>', '<C-w>>')
call submode#enter_with('bufmove', 'n', '', 's<', '<C-w><')
call submode#enter_with('bufmove', 'n', '', 's+', '<C-w>+')
call submode#enter_with('bufmove', 'n', '', 's-', '<C-w>-')
call submode#map('bufmove', 'n', '', '>', '<C-w>>')
call submode#map('bufmove', 'n', '', '<', '<C-w><')
call submode#map('bufmove', 'n', '', '+', '<C-w>+')
call submode#map('bufmove', 'n', '', '-', '<C-w>-')

" マウスクリック有効
if has("mouse") " Enable the use of the mouse in all modes
  set mouse=a
endif

