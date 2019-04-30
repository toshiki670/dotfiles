" .vimrc
"           []
"   __    ____ __ _ _  _ __ ___
"   \ \  / /| || ' ' \| '__/ __|
"    \ \/ / | || | | || | | (__
" o   \__/  |_||_|_|_||_|  \___|
"
" Root directory
let g:vimrc_root = expand('~/dotfiles/vim/')

" config's directory
let g:config_dir = g:vimrc_root . 'config/'


let s:config_names = [
      \'setting.vim',
      \'mapping.vim',
      \'color.vim',
      \'plugin.vim',
      \]

for filename in s:config_names
  let path = g:config_dir . filename
  if filereadable(path)
    exe 'so' path
  endif
endfor

