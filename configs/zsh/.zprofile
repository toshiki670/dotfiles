# rustup (cargo / rustc proxies) is keg-only, so it is not linked into /opt/homebrew/bin.
[[ -d /opt/homebrew/opt/rustup/bin ]] && export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
