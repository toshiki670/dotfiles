#!/usr/bin/env bash
# work-map のページを組み立てる。
#
# ライブラリ本体はこのスクリプトが結合するので、モデルのコンテキストを通らない。
# 生成物は外部への通信を一切行わない単一ファイルになる。
#
#   build.sh -o OUT -c CONTENT [-t TITLE] [-g GRAPH_DATA] [-s SERIES_DATA]
#
#     -c CONTENT      <body> の中身（.shell 以下）を書いた断片
#     -g GRAPH_DATA   const GRAPH = {...} を定義した .js。指定すると dagre を同梱する
#     -s SERIES_DATA  const SERIES = {...} を定義した .js
#
# 図を使わないページでは -g / -s を省く。省いた分のライブラリは埋め込まれない。

set -euo pipefail

lib="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="" content="" title="作業中の理解" graph="" series=""

while [ $# -gt 0 ]; do
	case "$1" in
	-o)
		out="$2"
		shift 2
		;;
	-c)
		content="$2"
		shift 2
		;;
	-t)
		title="$2"
		shift 2
		;;
	-g)
		graph="$2"
		shift 2
		;;
	-s)
		series="$2"
		shift 2
		;;
	*)
		echo "不明な引数: $1" >&2
		exit 2
		;;
	esac
done

[ -n "$out" ] || {
	echo "-o が必要" >&2
	exit 2
}
[ -n "$content" ] || {
	echo "-c が必要" >&2
	exit 2
}
[ -f "$content" ] || {
	echo "見つからない: $content" >&2
	exit 2
}

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

sed "s|@TITLE@|${title//|/\\|}|" "$lib/head.html" >"$tmp"
cat "$content" >>"$tmp"

# dagre は有向グラフを描くときだけ埋める。ライセンス表示は MIT の義務。
if [ -n "$graph" ]; then
	{
		printf '<script>\n'
		cat "$lib/license-header.js"
		cat "$lib/dagre.min.js"
		printf '\n</script>\n<script>\n'
		cat "$graph"
		cat "$lib/render-graph.js"
		printf '</script>\n'
	} >>"$tmp"
fi

if [ -n "$series" ]; then
	{
		printf '<script>\n'
		cat "$series"
		cat "$lib/render-chart.js"
		printf '</script>\n'
	} >>"$tmp"
fi

cat "$lib/tail.html" >>"$tmp"
mv "$tmp" "$out"
trap - EXIT

printf 'できた: %s (%s KB)\n' "$out" "$(($(wc -c <"$out") / 1024))"
