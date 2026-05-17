function ps-grep --description 'grep running processes'
    ps aux | grep $argv[1] | grep -v grep
end
