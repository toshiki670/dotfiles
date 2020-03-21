" Common setting file
if &compatible
  se nocompatible
endif

" 文字コードをUTF-8に設定
se fenc=utf-8

if has('nvim')
  se sh=bash
endif

" テキストのモードを非表示
se noshowmode

" Last status
se laststatus=2

" 行番号の表示
se number

" カーソルが何行目の何列目に置かれているかを表示する
se ruler

" タイトルを表示
se title

" 括弧入力時の対応する括弧を表示
se showmatch
se ambiwidth=double

" Tabの設定
se expandtab
se tabstop=2
se softtabstop=2
se shiftwidth=2

" 複数行のクリップボードからの貼付けがおかしい時、:se paste をすると治る
se smartindent

" 不可視文字を可視化
se list
se listchars=tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%
se history=50

" フリーカーソルモード
se virtualedit=block

" カーソルを左右に動かした時に前後の行末、行頭に移動
se whichwrap=b,s,h,l,[,],<,>

" 行番号を相対的に表示する
" se relativenumber

" バックスペースの動作
se backspace=indent,eol,start

" 入力中のコマンドを表示する
se showcmd

" コマンドモードの補完
se wildmenu

" クリップボードにコピー
if has('mac')
  se clipboard+=unnamed
elseif has('unix')
  se clipboard+=unnamedplus
endif

" ビープ音を可視化
se visualbell

" 保存せずにバッファ移動
se hidden

" 検索系
" 検索文字列が小文字の場合は大文字小文字を区別なく検索
se ignorecase
" 検索文字列に大文字が含まれている場合は区別して検索する
se smartcase
" 検索文字列入力時に順次対象文字列にヒットさせる
se incsearch
" 検索時に最後まで行ったら最初に戻る
se wrapscan
" 検索語をハイライト表示
se hlsearch

" 文字置換をインタラクティブに表示
se inccommand=split

" Don't hide the double quote on Json.
autocmd Filetype json setl conceallevel=0

" スペルチェック機能の有効化
" se spell

" マウスクリック有効
if has("mouse") " Enable the use of the mouse in all modes
  se mouse=a
endif

filetype plugin indent on

