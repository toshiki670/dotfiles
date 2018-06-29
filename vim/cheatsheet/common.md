# Your brain gave up thinking, So I give happiness...
## Normal mode
*     : Choice word after all marked the same word.
=     : Fix indent(and on visual mode).
C-d   : Move corsor to down half window
C-u   : Move corsor to up half window
C-f   : Move corsor to next window
C-u   : Move corsor to previous window

## Window
sw    : Rotate spirit
sj    : Move to down split
sk    : Move to up split
sl    : Move to right split
sh    : Move to left split

sJ    : Move a split window to down
sK    : Move a split window to up
sL    : Move a split window to right
sH    : Move a split window to left

sm    : Move to right tab
sn    : Move to left tab

st    : Create new tab

### Adjust spirit
s>>.. : Move a split to right
s<<.. : Move a split to left
s++.. : Move a split to up
s--.. : Move a split to down


## Completion
C-n
C-p
C-k

### Emmet
C-f   : Have to change this keybind

## Unite rails
nnoremap <C-H><C-H><C-H>  rails/view
nnoremap <C-H><C-H>       rails/model
nnoremap <C-H>            rails/controller

nnoremap <C-H>c           rails/config
nnoremap <C-H>s           rails/spec
nnoremap <C-H>m           rails/db -input=migrate
nnoremap <C-H>l           rails/lib
nnoremap <expr><C-H>g     Gemfile
nnoremap <expr><C-H>r     config/routes.rb
nnoremap <expr><C-H>se    db/seeds.rb
nnoremap <C-H>ra          rails/rake
nnoremap <C-H>h           rails/heroku

