#!/bin/bash


if [ $XDG_SESSION_TYPE = "x11" ]; then
  ln -sf ~/dotfiles/linux/.xprofile ~/.xprofile
fi
[ $? -eq 0 ] && echo "Success!"


if [ -x "$(command -v imwheel)" ]; then
  ln -sf ~/dotfiles/linux/.imwheelrc ~/.imwheelrc
fi

[ $? -eq 0 ] && echo "Success!"

