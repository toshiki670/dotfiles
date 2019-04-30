" プラグインが実際にインストールされるディレクトリ
let s:dein_dir = g:vimrc_root . 'plugin/'

" dein.vim 本体
let s:dein_repo_dir = s:dein_dir . 'repos/github.com/Shougo/dein.vim'

" Plugin's config directory
let s:plugin_dir = g:config_dir . 'plugin/'

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

  " Async Proc
  call dein#add('Shougo/vimproc.vim', {'build' : 'make'})

  " Common
  call dein#load_toml(s:plugin_dir . 'common.toml', {'lazy': 0})
  call dein#add('cespare/vim-toml', {'on_ft': 'toml'})
  call dein#add('tomtom/tcomment_vim')

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


" Splitの調節
call submode#enter_with('bufmove', 'n', '', 's>', '<C-w>>')
call submode#enter_with('bufmove', 'n', '', 's<', '<C-w><')
call submode#enter_with('bufmove', 'n', '', 's+', '<C-w>+')
call submode#enter_with('bufmove', 'n', '', 's-', '<C-w>-')
call submode#map('bufmove', 'n', '', '>', '<C-w>>')
call submode#map('bufmove', 'n', '', '<', '<C-w><')
call submode#map('bufmove', 'n', '', '+', '<C-w>+')
call submode#map('bufmove', 'n', '', '-', '<C-w>-')
