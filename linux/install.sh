#!/bin/bash


if [ $XDG_SESSION_TYPE = "x11" ]; then
  ln -sf ~/dotfiles/linux/.xprofile ~/.xprofile
fi
[ $? -eq 0 ] && echo "Success!"


# if [ -x "$(command -v imwheel)" ]; then
#   ln -sf ~/dotfiles/linux/.imwheelrc ~/.imwheelrc
# fi

cp ~/dotfiles/linux/X11/xorg.conf.d/20-nvidia.conf /etc/X11/xorg.conf.d 

[ $? -eq 0 ] && echo "Success!"

