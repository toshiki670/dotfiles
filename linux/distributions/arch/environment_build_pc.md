# Overview

# Reference
- [Installation guide - Arch Wiki](https://wiki.archlinux.org/index.php/Installation_guide)
- [dm-crypt/Encrypting an entire system - Arch Wiki](https://wiki.archlinux.org/index.php/Dm-crypt/Encrypting_an_entire_system)

# Composition
- Motherboard : ASRock Intel Z370 Extreme4 ATX
- CPU         : Intel i5-8500 (6) @ 4.100GHz
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
    Restore: 1GB: ???
    Boot   : 256MB: EF00
    LVM    : FREE:  8E00

    $ mkfs.fat -F 32 /dev/nvme*n*p*
    ```

1. Make LVM.
    ```
    $ cryptsetup luksFormat /dev/nvme*n*p*
    $ cryptsetup open /dev/nvme*n*p* cryptolvm
    $ pvcreate /dev/mapper/cryptolvm
    $ vgcreate cryptolvm /dev/mapper/cryptolvm
    $ lvcreate -L 50G cryptolvm -n root
    $ lvcreate -l 100%FREE cryptolvm -n home

    $ mkfs.xfs /dev/mapper/cryptolvm-root
    $ mkfs.xfs /dev/mapper/cryptolvm-home
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
    $ mount /dev/mapper/cryptolvm-root /mnt
    $ mkdir /mnt/boot
    $ mkdir /mnt/home
    $ mount /dev/nvme*n*p* /mnt/boot
    $ mount /dev/mapper/cryptolvm-home /mnt/home
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
    $ pacstrap /mnt base base-devel
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

1. Network setting.

1. Initramfs.<br>
    1. Add the keyboard, encrypt and lvm2 hooks to mkinitcpio.conf:
    ```
    HOOKS=(base udev autodetect keyboard keymap consolefont modconf block encrypt lvm2 filesystems fsck)
    ```
    2. Create.
    ```
    $ mkinitcpio -p linux
    ```

1. Boot loader.
    1. Note the PARTUUID
    ```
    blkid -s PARTUUID -o value /dev/nvme*n*
    ```
    2. The following kernel parameter needs to be set by the boot loader:
    ```
    cryptdevice=PARTUUID=device-PARTUUID:cryptolvm root=/dev/mapper/cryptolvm-root
    ```

1. Set the root password.
    ```
    $ passwd
    ```

## After setting
1. Desktop's enviroment.
    ```
    $ pacman -S plasma
    $ systemctl enable sddm
    ```

1. Necessary packages
    ```
    $ pacman -S zsh git vim neovim
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


