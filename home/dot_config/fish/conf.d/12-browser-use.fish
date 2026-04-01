# browser-use
for dir in $HOME/.browser-use/bin $HOME/.browser-use-env/bin
    test -d $dir && fish_add_path $dir
end
