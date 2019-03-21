#!/bin/bash

service=imwheel.service
users_systemd=~/.config/systemd/user

if [ ! -e $users_systemd ]; then
  mkdir -p $users_systemd
  echo "Made path that user's systemd"
fi

cp ./$service $users_systemd/$service


[ $? -eq "0" ] && echo "Success!"

