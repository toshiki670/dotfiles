#!/bin/bash

# iptables initialze

## reference
## https://wiki.archlinux.jp/index.php/Iptables
## https://wiki.archlinux.org/index.php/Simple_stateful_firewall
## https://wiki.archlinux.jp/index.php/%E3%82%B7%E3%83%B3%E3%83%97%E3%83%AB%E3%81%AA%E3%82%B9%E3%83%86%E3%83%BC%E3%83%88%E3%83%95%E3%83%AB%E3%83%95%E3%82%A1%E3%82%A4%E3%82%A2%E3%82%A6%E3%82%A9%E3%83%BC%E3%83%AB

# Cheking as root
if [[ $EUID -ne 0 ]]; then
  echo "Please exec as root."
  exit 1
fi


# Initialize
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X
iptables -t raw -F
iptables -t raw -X
iptables -t security -F
iptables -t security -X
iptables -P INPUT ACCEPT
iptables -P FORWARD ACCEPT
iptables -P OUTPUT ACCEPT


