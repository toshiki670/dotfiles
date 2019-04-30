" .vimrc
"           []
"   __    ____ __ _ _  _ __ ___
"   \ \  / /| || ' ' \| '__/ __|
"    \ \/ / | || | | || | | (__
" o   \__/  |_||_|_|_||_|  \___|
"

let g:vimrc_root = expand('~/dotfiles/vim/')

" config's directory
let g:config_dir = g:vimrc_root . 'config/'


source ~/dotfiles/vim/config/setting.vim
source ~/dotfiles/vim/config/mapping.vim
source ~/dotfiles/vim/config/color.vim
source ~/dotfiles/vim/config/plugin.vim

