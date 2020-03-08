# Cheet file
## Ex command
`:sp    `: Split horizontally<br>
`:vs    `: Split vertically<br>


## Normal mode
`*      `: Choice word after all marked the same word.<br>
`=      `: Fix indent(and on visual mode).<br>
`C-d    `: Move corsor to down half window<br>
`C-u    `: Move corsor to up half window<br>
`C-f    `: Move corsor to next window<br>
`C-u    `: Move corsor to previous window<br>
`ga     `: Get Ascii value<br>


## Visual mode
`<      `: Indent to left<br>
`>      `: Indent to right<br>


## Window
`sw     `: Rotate spirit<br>
`sj     `: Move to down split<br>
`sk     `: Move to up split<br>
`sl     `: Move to right split<br>
`sh     `: Move to left split<br>

`sJ     `: Move a split window to down<br>
`sK     `: Move a split window to up<br>
`sL     `: Move a split window to right<br>
`sH     `: Move a split window to left<br>

`sm     `: Move to right tab<br>
`sn     `: Move to left tab<br>

`st     `: Create new tab<br>

### Adjust spirit
`s>>..  `: Move a split to right<br>
`s<<..  `: Move a split to left<br>
`s++..  `: Move a split to up<br>
`s--..  `: Move a split to down<br>


## Completion
`C-n    `:<br>
`C-p    `:<br>
`C-k    `:<br>

### Emmet
`C-f    `: Have to change this keybind<br>


## Denite
`Space+d`: Prefix key

`f      `: Search for files within Project<br>
`g      `: Search within Project<br>
`*      `: Search for word under cursor<br>
`b      `: Search for files that opened in buffer<br>
`o      `: Search for functions / classes in files<br>

### Denite rails
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

