" プラグインが実際にインストールされるディレクトリ
let s:dein_dir = g:vimrc_root . 'plugin/'

" dein.vim 本体
let s:dein_repo_dir = s:dein_dir . 'repos/github.com/Shougo/dein.vim'

" Plugin's config directory
let s:plugin_dir = g:config_dir . 'plugin/'

" dein.vim がなければ github から落としてくる
if &runtimepath !~# 'dein.vim'
  if !isdirectory(s:dein_repo_dir)
    exe '!git clone https://github.com/Shougo/dein.vim' s:dein_repo_dir
  endif
  exe 'set runtimepath^=' . fnamemodify(s:dein_repo_dir, ':p')
endif

" Add the dein installation directory into runtimepath
if dein#load_state(s:dein_dir)
  cal dein#begin(s:dein_dir)
  cal dein#add(s:dein_dir)
  " Add or remove your plugins her e:

  " Async Proc
  cal dein#add('Shougo/vimproc.vim', {'build' : 'make'})

  " Common
  cal dein#load_toml(s:plugin_dir . 'common.toml', {'lazy': 0})
  cal dein#load_toml(s:plugin_dir . 'defx.toml', {'lazy': 0})
  cal dein#load_toml(s:plugin_dir . 'lightline.toml', {'lazy': 0})
  cal dein#add('cespare/vim-toml', {'on_ft': 'toml'})
  cal dein#add('tomtom/tcomment_vim')
  cal dein#load_toml(s:plugin_dir . 'lexima.toml', {'lazy': 0})

  " Completion
  cal dein#load_toml(s:plugin_dir . 'language_client.toml', {'lazy': 0})
  cal dein#load_toml(s:plugin_dir . 'completion.toml', {'lazy': 0})
  cal dein#load_toml(s:plugin_dir . 'completion_lazy.toml', {'lazy': 1})

  " Denite
  cal dein#load_toml(s:plugin_dir . 'denite.toml', {'lazy': 0})

  " Control
  cal dein#load_toml(s:plugin_dir . 'vim-submode.toml', {'lazy': 0})

  " Strengthen %
  cal dein#add('andymass/vim-matchup')

  " Ruby and Rails
  cal dein#load_toml(s:plugin_dir . 'ruby.toml', {'lazy': 1})
  cal dein#add('tpope/vim-rails')

  " Rust
  cal dein#add('rust-lang/rust.vim')

  " Typescript
  cal dein#load_toml(s:plugin_dir . 'typescript.toml', {'lazy': 0})


  " Install if uninstalled
  if dein#check_install()
    cal dein#install()
  endif

  " For Debug
  " cal dein#recache_runtimepath()
  cal dein#end()
  cal dein#save_state()
endif

