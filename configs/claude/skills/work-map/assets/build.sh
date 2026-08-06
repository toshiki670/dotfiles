#!/usr/bin/env bash
# work-map のページを組み立て、描けていることを確かめてから書き出す。
#
# ライブラリ本体はこのスクリプトが結合するので、モデルのコンテキストを通らない。
# 生成物は外部への通信を一切行わない単一ファイルになる。
#
#   build.sh -o OUT -c CONTENT [-t TITLE] [-s SERIES_DATA]
#
#     -c CONTENT      <body> の中身（.shell 以下）を書いた断片。
#                     <pre class="mermaid"> があれば mermaid を同梱する
#     -s SERIES_DATA  const SERIES = {...} を定義した .js
#
# 検査に落ちたら書き出さない。開くまで気づけない欠陥をここで止めるのが目的なので、
# headless Chrome が無ければ検査を飛ばさずに失敗する。

set -euo pipefail

lib="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="" content="" title="作業中の理解" series=""

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

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
tmp="$work/page.html"

# 断片が持ち込む欠陥を先に弾く。ブラウザを起こすより桁で速い。
python3 - "$content" <<'PY' || exit 1
import re, sys

frag = open(sys.argv[1], encoding='utf-8').read()
bad = []

# 既存の部品が勝つと実測できた3つの役目。型そのものを禁じるのではなく、
# その役目に mermaid を使わせない（状態表・.track・自前チャートが載せられるものが載らない）
LOSES = {
    'kanban': '個々の状態は状態表＋チップで書く（kanban のカードには「いま何が起きているか」が載らない）',
    'timeline': '順序は .track で書く（timeline は理由の1文が入らない）',
    'xychart-beta': '量は -s の自前チャートで描く（xychart は日付軸のラベルが重なる）',
}
for block in re.findall(r'<pre class="[^"]*\bmermaid\b[^"]*">(.*?)</pre>', frag, re.S):
    for line in block.strip().splitlines():
        head = line.strip().split()[0] if line.strip() else ''
        if head in LOSES:
            bad.append(f'{head} は使わない — {LOSES[head]}')
        if head:
            break

for pat, why in [
    (r'<link\b', '<link>'),
    (r'@import\b', '@import'),
    (r'\bsrc="(?!data:)', 'src= の外部参照'),
    (r'url\((?![\'"]?data:)[\'"]?https?:', 'url() の外部参照'),
]:
    if re.search(pat, frag):
        bad.append(f'外部参照: {why}')

for b in dict.fromkeys(bad):
    print(f'  {b}', file=sys.stderr)
sys.exit(1 if bad else 0)
PY

python3 - "$lib/head.html" "$title" >"$tmp" <<'PY'
import html, sys

head = open(sys.argv[1], encoding='utf-8').read()
sys.stdout.write(head.replace('@TITLE@', html.escape(sys.argv[2], quote=False)))
PY

cat "$content" >>"$tmp"

# mermaid は図を描くときだけ埋める。ライセンス表示は MIT の義務。
if grep -qE '<pre class="[^"]*\bmermaid\b' "$content"; then
  {
    printf '<script>\n'
    cat "$lib/license-header.js"
    cat "$lib/mermaid.min.js"
    printf '\n</script>\n<script>\n'
    cat "$lib/render-mermaid.js"
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

# 描いてみないと分からないぶんを確かめる。検査スクリプトは複製にだけ足すので生成物に残らない。
probe="$work/probe.html"
{
  cat "$tmp"
  printf '<script>\n'
  cat "$lib/check.js"
  printf '</script>\n'
} >"$probe"

chrome="${CHROME:-}"
if [ -z "$chrome" ]; then
  for c in "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
    "/Applications/Chromium.app/Contents/MacOS/Chromium" \
    "$(command -v google-chrome || true)" \
    "$(command -v chromium || true)"; do
    if [ -x "$c" ]; then
      chrome="$c"
      break
    fi
  done
fi
[ -n "$chrome" ] || {
  echo "headless Chrome が見つからない。CHROME=<パス> で指す" >&2
  exit 1
}

# 使い捨てのプロファイルを渡すと Chrome は DOM を吐いたあとも終了しない（初回起動の後始末が残る）。
# 待つのは印が出るまでにして、出たら止める。
render() {
  local dom="$work/dom.html"
  : >"$dom"
  "$chrome" --headless=new --disable-gpu --no-first-run --no-default-browser-check \
    --disable-background-networking --disable-component-update --disable-sync \
    --user-data-dir="$work/profile" --window-size="$1,900" \
    --virtual-time-budget=20000 --dump-dom "file://$probe" >"$dom" 2>/dev/null &
  local pid=$! i=0
  # </title> まで出ていれば印は途中で切れていない
  until grep -q 'WMCHECK.*</title>' "$dom" 2>/dev/null; do
    kill -0 "$pid" 2>/dev/null || break
    [ "$i" -lt 600 ] || break
    sleep 0.1
    i=$((i + 1))
  done
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
}

# 広いときと、レールが畳まれる狭いとき。横あふれは狭い側でしか出ない
for w in 1440 480; do
  render "$w"

  python3 - "$w" "$work/dom.html" <<'PY' || exit 1
import html, json, re, sys

dom = open(sys.argv[2], encoding='utf-8').read()
m = re.search(r'<title>WMCHECK (.*?)</title>', dom, re.S)
if not m:
    print(f'  幅 {sys.argv[1]}: 検査が結果を返さなかった（描画が終わっていない）', file=sys.stderr)
    sys.exit(1)

r = json.loads(html.unescape(m.group(1)))
for f in r['fail']:
    print(f'  幅 {sys.argv[1]}: {f}', file=sys.stderr)
sys.exit(1 if r['fail'] else 0)
PY
done

mkdir -p "$(dirname "$out")"
cp "$tmp" "$out"

printf 'できた: %s (%s KB)\n' "$out" "$(($(wc -c <"$out") / 1024))"
