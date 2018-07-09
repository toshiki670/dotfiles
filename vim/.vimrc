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
" プラグインが実際にインストールされるディレクトリ
let s:dein_dir = expand('~/dotfiles/vim/plugin')

" dein.vim 本体
let s:dein_repo_dir = s:dein_dir . '/repos/github.com/Shougo/dein.vim'

" dein.vim がなければ github から落としてくる
if &runtimepath !~# '/dein.vim'
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
  let plugins_dir = '~/dotfiles/vim/config/plugin/'


  " Async Proc
  call dein#add('Shougo/vimproc.vim', {'build' : 'make'})

  " Common
  call dein#load_toml(plugins_dir . 'common.toml', {'lazy': 0})

  " Denite
  call dein#load_toml(plugins_dir . 'denite.toml', {'lazy': 0})

  " Completion
  call dein#load_toml(plugins_dir . 'completion.toml', {'lazy': 0})

  " Control
  call dein#add('kana/vim-submode')

  " Ruby and Rails
  call dein#load_toml(plugins_dir . 'ruby.toml', {'lazy': 1})
  call dein#add('tpope/vim-rails')

  " Web related
  call dein#add('mattn/emmet-vim')


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
" dein.vim ここまで -----------------------------------------------------


" Plugin key-mappings.
" Note: It must be "imap" and "smap".  It uses <Plug> mappings.
imap <C-k>     <Plug>(neosnippet_expand_or_jump)
smap <C-k>     <Plug>(neosnippet_expand_or_jump)
xmap <C-k>     <Plug>(neosnippet_expand_target)

" SuperTab like snippets behavior.
" Note: It must be "imap" and "smap".  It uses <Plug> mappings.
imap <expr><TAB>
 \ pumvisible() ? "\<C-n>" :
 \ neosnippet#expandable_or_jumpable() ?
 \    "\<Plug>(neosnippet_expand_or_jump)" : "\<TAB>"
smap <expr><TAB> neosnippet#expandable_or_jumpable() ?
\ "\<Plug>(neosnippet_expand_or_jump)" : "\<TAB>"

" For conceal markers.
if has('conceal')
  set conceallevel=2 concealcursor=niv
endif

" For Emmet
let g:user_emmet_leader_key='<C-f>'
" End of Plugin config ---------------------------------------------------
" color ------------------------------------------------------------------
" 分差時の表示を変更
highlight DiffAdd    ctermfg=10 ctermbg=22
highlight DiffDelete ctermfg=10 ctermbg=52
highlight DiffChange ctermfg=10 ctermbg=17
highlight DiffText   ctermfg=10 ctermbg=21

" バッファ外のチルダ
highlight EndOfBuffer ctermfg=8

" set series -------------------------------------------------------------

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
set clipboard+=unnamed

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

