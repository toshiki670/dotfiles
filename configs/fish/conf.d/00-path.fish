# PATH: ~/.local/bin, ~/.cargo/bin
# Homebrew (macOS) — /etc/paths.d/homebrew appends /opt/homebrew/bin after
# /usr/bin, so prepend explicitly to override system tools.
if test -d /opt/homebrew
    fish_add_path --global --prepend --move /opt/homebrew/sbin
    fish_add_path --global --prepend --move /opt/homebrew/bin
end

# Prepend in this order so effective search order is: ~/.local/bin, then cargo, then homebrew.
fish_add_path --global --prepend --move $HOME/.cargo/bin
fish_add_path --global --prepend --move $HOME/.local/bin

# Obsidian (macOS only)
if test -d /Applications/Obsidian.app/Contents/MacOS
    fish_add_path --global --append /Applications/Obsidian.app/Contents/MacOS
end
