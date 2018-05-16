".vimrc
"         []
"   __    ____ __ _ _  _ __ ___
"   \ \  / /| || ' ' \| '__/ __|
"    \ \/ / | || | | || | | (__
" o   \__/  |_||_|_|_||_|  \___|
"

"Vundleの記述はここから
set nocompatible              " be iMproved, required
filetype off                  " required

" set the runtime path to include Vundle and initialize
set rtp+=~/.vim/bundle/Vundle.vim
call vundle#begin()
" alternatively, pass a path where Vundle should install plugins
"call vundle#begin('~/some/path/here')

" let Vundle manage Vundle, required
Plugin 'VundleVim/Vundle.vim'

" The following are examples of different formats supported.
" Keep Plugin commands between vundle#begin/end.
" plugin on GitHub repo
Plugin 'tpope/vim-fugitive'
" plugin from http://vim-scripts.org/vim/scripts.html
" Plugin 'L9'
" Git plugin not hosted on GitHub
Plugin 'git://git.wincent.com/command-t.git'
" git repos on your local machine (i.e. when working on your own plugin)
"Plugin 'file:///home/gmarik/path/to/plugin'
" The sparkup vim script is in a subdirectory of this repo called vim.
" Pass the path to set the runtimepath properly.
Plugin 'rstacruz/sparkup', {'rtp': 'vim/'}
" Install L9 and avoid a Naming conflict if you've already installed a
" different version somewhere else.
" Plugin 'ascenator/L9', {'name': 'newL9'}

"---------Ruby on Rails--------------
" 導入したいプラグインを以下に列挙
" Plugin '[Github Author]/[Github repo]' の形式で記入
"Plugin 'airblade/vim-gitgutter'
Plugin 'tpope/vim-rails'
" Ruby向けにendを自動挿入してくれる
Plugin 'tpope/vim-endwise'


"--------End---------------

"--------html-------------
Plugin 'mattn/emmet-vim'
"--------------------------



"lightline - https://github.com/itchyny/lightline.vim
Plugin 'itchyny/lightline.vim'
"Solarized
Plugin 'altercation/vim-colors-solarized'

"Folder Tree
Plugin 'scrooloose/nerdtree'
Plugin 'jistr/vim-nerdtree-tabs'
"ERDTreeToggleoggle の設定
"autocmd vimenter * NERDTree
"noremap tree :NERDTreeToggle<Enter>
noremap tree :NERDTreeTabsToggle<Enter>


"For keybind
Plugin 'kana/vim-submode'

" All of your Plugins must be added before the following line
call vundle#end()            " required
filetype plugin indent on    " required
" To ignore plugin indent changes, instead use:
"filetype plugin on
"
" Brief help
" :PluginList       - lists configured plugins
" :PluginInstall    - installs plugins; append `!` to update or just :PluginUpdate
" :PluginSearch foo - searches for foo; append `!` to refresh local cache
" :PluginClean      - confirms removal of unused plugins; append `!` to auto-approve removal
"
" see :h vundle for more details or wiki for FAQ
" Put your non-Plugin stuff after this line

"ここまで



"Emmetの設定
let g:user_emmet_leader_key='<C-f>'

" Solarized
syntax enable
set background=dark
colorscheme solarized
highlight LineNr ctermfg=darkcyan
"個人設定
set laststatus=2
"文字コードをUTF-8に設定
set fenc=utf-8

set number
" カーソルが何行目の何列目に置かれているかを表示する
set ruler
"現在の行を強調
"set cursorline

set title
set showmatch "括弧入力時の対応する括弧を表示
set ambiwidth=double
set tabstop=2
set expandtab
set smarttab
set shiftwidth=2
"複数行のクリップボードからの貼付けがおかしい時、:set paste をすると治る
set smartindent
"不可視文字を可視化
set list
set listchars=tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%
set history=50
set virtualedit=block
set whichwrap=b,s,[,],<,>
set backspace=indent,eol,start
" 入力中のコマンドを表示する
set showcmd
"クリップボードにコピー
set clipboard+=unnamed
"ビープ音を可視化
set visualbell

"検索系
"検索文字列が小文字の場合は大文字小文字を区別なく検索
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


"自動挿入
inoremap {<Enter> {}<Left><CR><ESC><S-o>
"inoremap [<Enter> []<Left><CR><ESC><S-o>
inoremap [ []<LEFT>
"inoremap (<Enter> ()<Left><CR><ESC><S-o>
inoremap ( ()<LEFT>
inoremap ' ''<LEFT>
inoremap " ""<LEFT>

"簡単にノーマルモード移動
"inoremap <C-a> <Esc>

"noremap zz :w<Enter>
noremap ; :
noremap QQ :q<Enter>
"折り返し時に表示行単位での移動出来るようにする
nnoremap j gj
nnoremap k gk

"インサートモードでも移動
inoremap <C-j>  <down>
inoremap <C-k>  <up>
inoremap <C-h>  <left>
inoremap <C-l>  <right>


"https://qiita.com/tekkoc/items/98adcadfa4bdc8b5a6ca
"画面分割設定
nnoremap s <Nop>
"スピリット画面移動
nnoremap sw <C-w>w
nnoremap sj <C-w>j
nnoremap sk <C-w>k
nnoremap sl <C-w>l
nnoremap sh <C-w>h
"スピリット画面そのものを移動
nnoremap sJ <C-w>J
nnoremap sK <C-w>K
nnoremap sL <C-w>L
nnoremap sH <C-w>H
"nnoremap sr <C-w>r
"タブ移動
nnoremap sm gt
nnoremap sn gT

"スピリットの大きさを整える
nnoremap s= <C-w>=
"縦横最大化
nnoremap so <C-w>_<C-w>|
"大きさを揃える
nnoremap sO <C-w>=
nnoremap sN :<C-u>bn<CR>
nnoremap sP :<C-u>bp<CR>
"新規タブ
nnoremap st :<C-u>tabnew<CR>
"タブ一覧
nnoremap sT :<C-u>Unite tab<CR>
"水平分割
nnoremap sr :<C-u>sp<CR>
"垂直分割
nnoremap sv :<C-u>vs<CR>
"ウィンドウを閉じる
nnoremap sq :<C-u>q<CR>
"バッファを閉じる
nnoremap sQ :<C-u>bd<CR>
"現在のタブで開いたバッファ一覧
nnoremap sb :<C-u>Unite buffer_tab -buffer-name=file<CR>
"バッファ一覧
nnoremap sB :<C-u>Unite buffer -buffer-name=file<CR>


call submode#enter_with('bufmove', 'n', '', 's>', '<C-w>>')
call submode#enter_with('bufmove', 'n', '', 's<', '<C-w><')
call submode#enter_with('bufmove', 'n', '', 's+', '<C-w>+')
call submode#enter_with('bufmove', 'n', '', 's-', '<C-w>-')
call submode#map('bufmove', 'n', '', '>', '<C-w>>')
call submode#map('bufmove', 'n', '', '<', '<C-w><')
call submode#map('bufmove', 'n', '', '+', '<C-w>+')
call submode#map('bufmove', 'n', '', '-', '<C-w>-')

"マウスクリック有効
if has("mouse") " Enable the use of the mouse in all modes
  set mouse=a
endif



