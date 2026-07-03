# テーマをシステムの light/dark に追従させる（light=One Half Light, dark=Ayu）。
#
# fish 4.3+ は最初の対話プロンプト描画時に OSC 11 でターミナルの背景色を問い合わせ、
# $fish_terminal_color_theme を light / dark / unknown に設定する（以後の変化も検出）。
# color-theme-aware な themes/OneHalfLight-Ayu.theme を choose しておくと、登録される
# __fish_apply_theme ハンドラがその変数変化に追従してバリアントを自動適用する。
# Ghostty 側を `theme = light:One Half Light,dark:Ayu` にしてあるので、システム外観の
# Light/Dark 切り替えに fish のシンタックスハイライトも追従する。
#
# 注: `fish_config theme save` は universal 変数へ固定保存され追従しないため使わない。
status is-interactive || exit 0
fish_config theme choose OneHalfLight-Ayu
