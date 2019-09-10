# Overview

# Reference

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
    `$ fdisk -l`

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
    Result
    ```
    Restore: 1GB
    Boot   : 256MB
    LVM    : Crypt
      Root : 50GB : XFS
      Home : FREE : XFS
    ```

1. Mount


```
Restore: 1GB
Boot   : 256MB
LVM    : Crypt
  Root : 50GB : XFS
  Home : FREE : XFS
```

## Install

## Setting

## After setting

# Note


