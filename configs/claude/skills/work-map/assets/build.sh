#!/usr/bin/env bash
# work-map のページを組み立て、描けていることを確かめてから書き出す。
#
# ライブラリ本体はこのスクリプトが結合するので、モデルのコンテキストを通らない。
# 生成物は外部への通信を一切行わない単一ファイルになる。
#
#   build.sh -o OUT -c CONTENT [-t TITLE]
#
#     -c CONTENT      <body> の中身（.shell 以下）を書いた断片。
#                     <pre class="mermaid"> があれば mermaid を同梱する
#
# 検査に落ちたら書き出さない。開くまで気づけない欠陥をここで止めるのが目的なので、
# headless Chrome が無ければ検査を飛ばさずに失敗する。

set -euo pipefail

lib="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="" content="" title="作業中の理解"

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
python3 - "$content" "$lib/head.html" <<'PY' || exit 1
import re, sys
from html.parser import HTMLParser

frag = open(sys.argv[1], encoding='utf-8').read()
bad = []

# 既存の部品が勝つと実測できた役目。型そのものを禁じるのではなく、その役目に mermaid を
# 使わせない。落ちるものと代わりの部品は SKILL.md の「渡さない型」の表にある
LOSES = {'kanban', 'timeline'}
# 断片は <pre class="mermaid"> で書く。他のタグに mermaid class を付けると、
# 下の走査から漏れたまま render-mermaid.js が拾って描いてしまう
for tag in re.findall(r'<(\w+)[^>]*\bclass="[^"]*\bmermaid\b[^"]*"', frag):
    if tag != 'pre':
        bad.append(f'mermaid の断片は <pre class="mermaid"> で書く（<{tag}> になっている）')

for block in re.findall(r'<pre[^>]*\bclass="[^"]*\bmermaid\b[^"]*"[^>]*>(.*?)</pre>', frag, re.S):
    # frontmatter・init ディレクティブ・コメントを落とすと、宣言はブロックの先頭に来る。
    # 全行を見ると flowchart のノード id が渡さない型と同名のときに誤って弾く
    block = re.sub(r'^\s*---\s*\n.*?\n\s*---\s*$', '', block.strip(), count=1, flags=re.S | re.M)
    block = re.sub(r'%%\{.*?\}%%', '', block, flags=re.S)
    for line in block.splitlines():
        line = line.strip()
        if not line or line.startswith('%%'):
            continue
        head = line.split()[0]
        if head in LOSES:
            bad.append(f'{head} は使わない — SKILL.md「渡さない型」の表にある部品で書く')
        break

# --mm-* は - を入れ子の区切りに使うので、ある名前が別の名前の親を兼ねられてしまう。
# 兼ねると文字列とオブジェクトが同じ場所を取り合い、宣言順によらず入れ子側が黙って消える
# （文字列へのプロパティ代入は例外にならない）。線がテーマの既定色に戻るだけで build は通る
def mm_names(css):
    return re.findall(r'--mm-([A-Za-z0-9-]+)\s*:', re.sub(r'/\*.*?\*/', '', css, flags=re.S))


names = sorted(set(mm_names(frag) + mm_names(open(sys.argv[2], encoding='utf-8').read())))
for parent in names:
    for child in names:
        if child.startswith(parent + '-'):
            bad.append(f'--mm-{parent} と --mm-{child} は同じ名前を値と入れ子の親に使っている（どちらかを改名する）')

# 外部参照は markup の構造で見る。文字列で探すと、図のラベルや本文に書いた
# url(…) や src="…" を、実際に効く属性・CSS と区別できずに弾く
def css_refs(css):
    # scheme は CSS でも属性でも大小を区別しない
    return [f'url({u})' for u in re.findall(r'url\(\s*[\'"]?(https?:[^)\'"\s]*)', css, re.I)]

class Refs(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.found = []
        self.in_style = 0

    def handle_starttag(self, tag, attrs):
        if tag == 'style':
            self.in_style += 1
        if tag == 'link':
            self.found.append('<link>')
        for name, value in attrs:
            if not value:
                continue
            if name == 'src' and not value.lower().startswith('data:'):
                self.found.append(f'src="{value[:60]}"')
            elif name == 'style':
                self.found += css_refs(value)

    def handle_endtag(self, tag):
        if tag == 'style' and self.in_style:
            self.in_style -= 1

    def handle_data(self, data):
        if self.in_style:
            if '@import' in data:
                self.found.append('@import')
            self.found += css_refs(data)


refs = Refs()
refs.feed(frag)
refs.close()
bad += [f'外部参照: {r}' for r in refs.found]

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
if grep -qE '<pre[^>]*class="[^"]*\bmermaid\b' "$content"; then
  {
    printf '<script>\n'
    cat "$lib/license-header.js"
    cat "$lib/mermaid.min.js"
    printf '\n</script>\n<script>\n'
    cat "$lib/render-mermaid.js"
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
[ -x "$chrome" ] || {
  echo "headless Chrome が見つからない（CHROME=${CHROME:-未指定}）。CHROME=<パス> で指す" >&2
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
