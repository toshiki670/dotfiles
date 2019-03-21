#!/bin/bash

ln -sf ~/dotfiles/linux/.xprofile ~/.xprofile

[ $? -eq 0 ] && echo "Success!"

if [ -x "$(command -v imwheel)" ]; then
  ln -sf ~/dotfiles/linux/.imwheelrc ~/.imwheelrc
fi

[ $? -eq 0 ] && echo "Success!"

