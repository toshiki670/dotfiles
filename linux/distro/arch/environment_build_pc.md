# Overview

# Reference
- [Installation guide - Arch Wiki](https://wiki.archlinux.org/index.php/Installation_guide)
- [dm-crypt/Encrypting an entire system - Arch Wiki](https://wiki.archlinux.org/index.php/Dm-crypt/Encrypting_an_entire_system)
- [Arch dm-cryptでデバイスを透過的に暗号化する - u+のブログ](http://u10e10.hatenablog.com/entry/dm-crypt-usage)

# Composition
- Motherboard : ASRock Intel Z370 Extreme4 ATX
- CPU         : Intel i9-9900KF (8 core / 16 thread) @ 3.6GHz
- GPU         : NVIDIA GeForce RTX 2080
- Memory      : Corsair DDR4-2666MHz 8GB * 4
- Drive       : SSD 250GB
- Network
  * Wired     : Normal
  * WI-FI     : Intel Wireless-AC 9260NGW & Bluetooth card 802.11ac wave 2(1.733Mbps) Bluetooth 5.0
- Device
  * Keyboard  : HHKB(US)
  * Mouse     : Steelseries rival 110

# Setup
## Preparation
1. Create install media.
1. Check drive to install.
    ```
    $ fdisk -l
    ```

1. Make restore, boot and LVM.
    ```
    $ gdisk /dev/nvme*n*
    Boot      : 255MB: EF00
    Restore   : 1GB: 8300
    Encrypted : FREE:  8E00

    1. o  :  Create a new empty GPT.

    Boot
    $ mkfs.fat -vcF 32 -n boot /dev/nvme*n*p*
    ```

1. Make boot key
    ```
    $ dd bs=512 count=4 if=/dev/urandom of=/path/to/key_file
    ```

1. Make LVM.
    ```
    $ cryptsetup -v -c serpent-xts-plain64 -s 512 -h sha512 luksFormat /dev/nvme*n*p* /path/to/key_file
    $ cryptsetup --key-file=/path/to/key_file luksOpen /dev/nvme*n*p* Decrypted
    $ pvcreate /dev/mapper/Decrypted
    $ vgcreate system /dev/mapper/Decrypted
    $ lvcreate -L 50G system -n root
    $ lvcreate -l 100%FREE system -n home

    $ mkfs.xfs /dev/mapper/system-root
    $ mkfs.xfs /dev/mapper/system-home
    ```
    ``` 
    Result partition
    Restore : 1GB
    Boot    : 256MB
    LVM     : Crypt
      Root  : 50GB : XFS
      Home  : FREE : XFS
    ```

1. Mount.
    ```
    $ mount /dev/mapper/system-root /mnt
    $ mkdir /mnt/boot /mnt/home
    $ mount /dev/nvme*n*p* /mnt/boot
    $ mount /dev/mapper/system-home /mnt/home
    ```

1. Check to connect network.
    ```
    $ ping archlinux.jp
    ```

1. Update system clock.
    ```
    $ timedatectl set-ntp true
    ```

## Install
1. Choose mirror.
    ```
    $ vim /etc/pacman.d/mirrorlist
    ```

1. Install base system.
    ```
    $ pacstrap /mnt base base-devel linux linux-firmware pacman-contrib
    ```

## Setting
1. Generate fstab.
    ```
    $ genfstab -U /mnt >> /mnt/etc/fstab
    ```

1. Chroot.
    ```
    $ arch-chroot /mnt
    ```

1. Timezone.
    ```
    $ ln -sf /usr/share/zoneinfo/Asia/Tokyo /etc/localtime
    $ hwclock --systohc --utc
    ```

1. Locale.<br>
    Uncomment `en_US.UTF-8 UTF-8` and other needed locales in `/etc/locale.gen`, and generate them with:
    ```
    $ locale-gen
    ```

1. Host name.
    ```
    $ cat /dev/urandom | tr -dc "[:alnum:]" | fold -w 8 | head -n 1 | sed "s/^/Linux-/" > /etc/hostname
    $ vi /etc/hosts
    127.0.0.1 localhost
    ::1       localhost
    127.0.1.1 myhostname.localdomain  myhostname
    ```

1. Network setting.
    ```
    $ systemctl enable dhcpcd
    ```
    ```
    Add Google DNS
    $ vi /etc/resolv.conf
    nameserver 8.8.8.8
    nameserver 8.8.4.4
    ```

1. Initramfs.<br>
    1. Write to /etc/mkinitcpio.conf:
    ``` mkinitcpio.conf
    MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)
    HOOKS=(base udev autodetect modconf keyboard keymap block encrypt lvm2 filesystems fsck)
    ```

    1. Create.
    ```
    $ mkinitcpio -p linux
    ```

1. Boot loader.
    1. Install systemd-boot
    ```
    $ bootctl --path=/boot install
    ```

    1. Add an Archlinux loader file to /boot/loader/entries/arch.conf.
    ```
    title  Arch Linux
    linux /vmlinuz-linux
    initrd /initramfs-linux.img
    options cryptdevice=UUID=device-UUID:Decrypted cryptkey=/dev/disk/by-uuid/USB-UUID:xfs:/path/to/key_file root=/dev/mapper/system-root nvidia-drm.modeset=1 rw
    ```
    Append the UUID of Decrypted storage.
    ```
    blkid -s UUID -o value /dev/nvme*n* >> /boot/loader/entries/arch.conf
    ```

1. Set the root password.
    ```
    $ passwd
    ```

1. Add User
    ```
    $ useradd -m -G wheel username
    $ passwd username
    ```

1. Enable wheel group as sudoer.
    ```
    $ visudo
    Uncomment the below:
    # %wheel ALL=(ALL) ALL
    ```

1. Finally
    ```
    $ exit
    $ umount -R /mnt
    $ reboot
    ```

## After setting
1. Desktop's enviroment.
    ```
    $ pacman -S --noconfirm plasma konsole dolphin filelight ark gwenview vlc openssh
    $ systemctl enable sddm
    ```
    [x] Choose the phonon-qt5-vlc.

1. Necessary packages
    ```
    $ pacman -S --noconfirm zsh git vim neovim words cryfs
    ```
    
1. Add pacman.conf
    ```
    [quarry]
    Server = https://pkgbuild.com/~anatolik/quarry/x86_64/
    ```

1. Packages for Neovim
    ```
    $ pacman -S --noconfirm ruby ruby-neovim ruby-solargraph python python-pip python-neovim yarn npm
    $ yarn global add neovim
    $ npm i javascript-typescript-langserver -g
    ```

1. For ruby
    ```
    $ pacman -S ruby-rdoc ruby-docs
    ```

1. Yay
    ```
    $ git clone https://aur.archlinux.org/yay.git
    $ cd yay
    $ makepkg -si
    $ cd ..
    $ rm -rf yay
    ```

# Note
## Bluetooth
```
$ pacman -S pulseaudio-alsa pulseaudio-bluetooth bluez-utils
$ systemctl enable bluetooth
$ systemctl start bluetooth
```
## Japanese input
```
$ pacman -S fcitx fcitx-mozc fcitx-gtk3 fcitx-qt5 kcm-fcitx
```
