# Your brain gave up thinking, So I give happiness...

## Ex command
:sp   : 横方向に分割
:vs   : 縦方向に分割


## Normal mode
*     : Choice word after all marked the same word.
=     : Fix indent(and on visual mode).
C-d   : Move corsor to down half window
C-u   : Move corsor to up half window
C-f   : Move corsor to next window
C-u   : Move corsor to previous window
ga    : Get Ascii value

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

## Denite rails
nmap     <Leader>r [rails]
nnoremap <silent> [rails]r   :<C-u>Denite<Space>rails:dwim<Return>
nnoremap <silent> [rails]m   :<C-u>Denite<Space>rails:model<Return>
nnoremap <silent> [rails]c   :<C-u>Denite<Space>rails:controller<Return>
nnoremap <silent> [rails]v   :<C-u>Denite<Space>rails:view<Return>
nnoremap <silent> [rails]a   :<C-u>Denite<Space>rails:asset<Return>
nnoremap <silent> [rails]h   :<C-u>Denite<Space>rails:helper<Return>
nnoremap <silent> [rails]t   :<C-u>Denite<Space>rails:test<Return>
nnoremap <silent> [rails]s   :<C-u>Denite<Space>rails:spec<Return>
nnoremap <silent> [rails]d   :<C-u>Denite<Space>rails:db<Return>
nnoremap <silent> [rails]co  :<C-u>Denite<Space>rails:config<Return>
Denite rails:ability
Denite rails:asset
Denite rails:attribute
Denite rails:config
Denite rails:controller
Denite rails:db
Denite rails:decorator
Denite rails:domain
Denite rails:factory
Denite rails:form
Denite rails:helper
Denite rails:job
Denite rails:loyalty
Denite rails:mailer
Denite rails:model
Denite rails:policy
Denite rails:presenter
Denite rails:query
Denite rails:serializer
Denite rails:service
Denite rails:spec
Denite rails:test
Denite rails:uploader
Denite rails:validator
Denite rails:view

