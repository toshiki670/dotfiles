# yt-dlp config

# Browser selection via environment variable
# Set YT_BROWSER in ~/.config/mise/config.toml or export in shell
alias yt="yt-dlp --cookies-from-browser \"${YT_BROWSER}\""
alias yt-chat='yt --write-subs --write-comments'
