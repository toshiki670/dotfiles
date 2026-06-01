# eza - ls/la/lt (match eza.zsh)
if command -q eza
    function ls --wraps eza --description 'eza -l'
        eza --icons=always -l -gh --time-style long-iso --git $argv
    end
    function la --wraps eza --description 'eza -la'
        eza --icons=always -la -gh --time-style long-iso --git $argv
    end
    function lt --description 'eza tree with level and target'
        set -l level 2
        set -l target .

        if test (count $argv) -eq 1
            if test -e "$argv[1]"
                set target "$argv[1]"
            else if string match -qr '^[0-9]+$' -- "$argv[1]"
                set level "$argv[1]"
            else
                set target "$argv[1]"
            end
        else if test (count $argv) -eq 2
            set level "$argv[1]"
            set target "$argv[2]"
        end

        eza --icons=always -la -gh --time-style long-iso --git --tree --level=$level $target
    end

    # EZA_COLORS: follow the terminal's light/dark theme, matching Ghostty
    # (light:One Half Light / dark:Ayu). File-type, size, permission and git
    # colors use ANSI palette indices, so Ghostty's per-theme palette already
    # renders them per appearance. Here we only adjust the brightness of grey
    # metadata (owner/group, punctuation, header, link/inode/block counts) so it
    # stays legible on each background. fish derives $fish_terminal_color_theme
    # from the terminal background (OSC 11); re-derive EZA_COLORS when it changes.
    function __apply_eza_colors --on-variable fish_terminal_color_theme
        if test "$fish_terminal_color_theme" = light
            # Light background (#fafafa): use darker greys.
            set -gx EZA_COLORS "uu=38;5;241:gu=38;5;241:un=38;5;245:gn=38;5;245:xx=38;5;247:hd=38;5;243:lc=38;5;245:in=38;5;245:bl=38;5;245"
        else
            # Dark background (#0b0e14): use lighter greys.
            set -gx EZA_COLORS "uu=38;5;247:gu=38;5;247:un=38;5;243:gn=38;5;243:xx=38;5;238:hd=38;5;240:lc=38;5;243:in=38;5;243:bl=38;5;243"
        end
    end
    # Apply immediately if the theme is already known (e.g. on config reload).
    if set -q fish_terminal_color_theme
        __apply_eza_colors
    end
else
    function ls --wraps ls
        command ls --color=auto -lh $argv
    end
    function la --wraps ls
        command ls --color=auto -lah $argv
    end
end

abbr -a sls 'command ls'
