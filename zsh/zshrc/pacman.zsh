# For pacman
function swap-pacman-mirrorlist {
  pacman_conf='/etc/pacman.conf'
  old_list='/etc/pacman.d/mirrorlist.old'
  current_list='/etc/pacman.d/mirrorlist'
  new_list='/etc/pacman.d/mirrorlist.pacnew'

  if [[ ! -e $pacman_conf ]]; then
    echo "${0##*/}: No archlinux distribution" 1>&2
    return 2
  fi

  if [[ ! -e $new_list ]]; then
    echo "${0##*/}: Not found the new mirrorlist." 1>&2
    return 1
  fi

  if [[ -e $old_list ]]; then
    sudo rm --verbose $old_list
  fi

  sudo mv --verbose $current_list $old_list
  sudo mv --verbose $new_list $current_list

  return 0
}

