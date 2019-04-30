" Common setting file
if &compatible
  set nocompatible
endif

" 文字コードをUTF-8に設定
set fenc=utf-8

if has('nvim')
  set sh=bash
endif

" テキストのモードを非表示
set noshowmode

" Last status
set laststatus=2

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

" マウスクリック有効
if has("mouse") " Enable the use of the mouse in all modes
  set mouse=a
endif

filetype plugin indent on
syntax enable

