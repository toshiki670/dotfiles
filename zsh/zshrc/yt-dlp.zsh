# yt-dlp config

# Browser selection via environment variable
# Set YT_BROWSER in ~/.config/mise/config.toml or export in shell
# Default: chrome:Default
YT_BROWSER="${YT_BROWSER:-chrome:Default}"

alias yt="yt-dlp --cookies-from-browser \"${YT_BROWSER}\""
alias yt-comment='yt --write-subs --write-comments'
alias yt-chat='yt --write-subs --write-comments'
alias yt-vr='yt "https://www.nhk.or.jp/radio/ondemand/detail.html?p=6N87LJL8ZM_01"'

