# git-delta: follow the terminal's light/dark theme (light:One Half Light / dark:Ayu).
#
# delta is git's pager, spawned fresh per command, so switching DELTA_FEATURES is
# picked up on the next git invocation (startup-time follow, like nvim). The
# feature definitions live in ~/.config/git/config as [delta "theme-light"] and
# [delta "theme-dark"]. fish derives $fish_terminal_color_theme from the terminal
# background (OSC 11); re-select the feature whenever it changes.
#
# The dark variant uses the custom "ayu-dark" bat theme
# (~/.config/bat/themes/ayu-dark.tmTheme, registered via `bat cache --build`).
if command -q delta
    function __apply_delta_features --on-variable fish_terminal_color_theme
        if test "$fish_terminal_color_theme" = light
            set -gx DELTA_FEATURES theme-light
        else
            set -gx DELTA_FEATURES theme-dark
        end
    end
    # Apply immediately if the theme is already known (e.g. on config reload).
    if set -q fish_terminal_color_theme
        __apply_delta_features
    end
end
