" .vimrc
"           []
"   __    ____ __ _ _  _ __ ___
"   \ \  / /| || ' ' \| '__/ __|
"    \ \/ / | || | | || | | (__
" o   \__/  |_||_|_|_||_|  \___|
"
if &compatible
  set nocompatible
endif

" dein.vim ここから -----------------------------------------------------
let s:vimrc_root = '~/dotfiles/vim/'

" config's directory
let s:config_dir = s:vimrc_root . 'config/'

" プラグインが実際にインストールされるディレクトリ
let s:dein_dir = expand(s:vimrc_root . 'plugin/')

" dein.vim 本体
let s:dein_repo_dir = s:dein_dir . 'repos/github.com/Shougo/dein.vim'

" dein.vim がなければ github から落としてくる
if &runtimepath !~# 'dein.vim'
  if !isdirectory(s:dein_repo_dir)
    execute '!git clone https://github.com/Shougo/dein.vim' s:dein_repo_dir
  endif
  execute 'set runtimepath^=' . fnamemodify(s:dein_repo_dir, ':p')
endif

" Add the dein installation directory into runtimepath
if dein#load_state(s:dein_dir)
  call dein#begin(s:dein_dir)
  call dein#add(s:dein_dir)
  " Add or remove your plugins her e:
  let s:plugin_dir = s:config_dir . 'plugin/'


  " Async Proc
  call dein#add('Shougo/vimproc.vim', {'build' : 'make'})

  " Common
  call dein#load_toml(s:plugin_dir . 'common.toml', {'lazy': 0})

  " Completion
  call dein#load_toml(s:plugin_dir . 'completion.toml', {'lazy': 0})
  call dein#load_toml(s:plugin_dir . 'completion_lazy.toml', {'lazy': 1})

  " Denite
  call dein#load_toml(s:plugin_dir . 'denite.toml', {'lazy': 0})

  " Control
  call dein#add('kana/vim-submode')

  " Ruby and Rails
  call dein#load_toml(s:plugin_dir . 'ruby.toml', {'lazy': 1})
  call dein#add('tpope/vim-rails')


  " Install if uninstalled
  if dein#check_install()
    call dein#install()
  endif

  " For Debug
  " call dein#recache_runtimepath()
  call dein#end()
  call dein#save_state()
endif

filetype plugin indent on
syntax enable

" color ------------------------------------------------------------------
" 分差時の表示を変更
highlight DiffAdd    ctermfg=10 ctermbg=22
highlight DiffDelete ctermfg=10 ctermbg=52
highlight DiffChange ctermfg=10 ctermbg=17
highlight DiffText   ctermfg=10 ctermbg=21

" バッファ外のチルダ
highlight EndOfBuffer ctermfg=8

" set series -------------------------------------------------------------

if has('nvim')
  set sh=bash
endif

" テキストのモードを非表示
set noshowmode

" Last status
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

" フリーカーソルモード
set virtualedit=block

" カーソルを左右に動かした時に前後の行末、行頭に移動
set whichwrap=b,s,h,l,[,],<,>

" バックスペースの動作
set backspace=indent,eol,start

" 入力中のコマンドを表示する
set showcmd

" コマンドモードの補完
set wildmenu

" クリップボードにコピー
if has('mac')
  set clipboard+=unnamed
elseif has('unix')
  set clipboard+=unnamedplus
endif

" ビープ音を可視化
set visualbell

" 保存せずにバッファ移動
set hidden

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

" スペルチェック機能の有効化
set spell

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

