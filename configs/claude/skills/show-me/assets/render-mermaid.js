// mermaid の図を描く。断片は本文側に <pre class="mermaid"> で書く。
//
// 色は head.html の :root にある --mm-* を読んで渡す。値の出所はページのトークン1箇所で、
// 型を増やすときも `--mm-*` を CSS へ足すだけで済む。残りの - は入れ子に開くので、
// --mm-pie1 は pie へ、--mm-xyChart-plotColorPalette は xyChart.plotColorPalette へ届く。

(function () {
  const root = document.documentElement;
  const computed = getComputedStyle(root);
  const vars = {};

  for (const sheet of document.styleSheets) {
    for (const rule of sheet.cssRules) {
      if (!rule.style) continue;
      for (const prop of rule.style) {
        if (!prop.startsWith('--mm-')) continue;
        const path = prop.slice(5).split('-');
        let node = vars;
        while (path.length > 1) node = node[path.shift()] ??= {};
        node[path[0]] = computed.getPropertyValue(prop).trim();
      }
    }
  }

  mermaid.initialize({ startOnLoad: false, theme: 'base', themeVariables: vars });

  // mermaid は svg を width="100%" にするので、画面が狭いぶんだけ図が縮んで字が読めなくなる。
  // 下限を敷いて .scroller のスクロールへ落とす。自然幅より広げはしない（小さい図を引き伸ばさない）。
  // 塗りの上に載る字は、その塗りの明るさで決める。
  // 並び順では当てられない — 円グラフは 1% に満たない区画を描かずに色番号だけ進めるので、
  // 何番目の区画がどの色になるかは描いてみるまで分からない
  const lum = c => {
    const [r, g, b] = c.match(/[\d.]+/g).slice(0, 3).map(Number);
    const f = v => (v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
  };

  const probe = document.createElement('span');
  const asRgb = v => {
    probe.style.color = '';
    probe.style.color = v;
    return getComputedStyle(probe).color;
  };

  // 背景と文字色のうち、その塗りとの差が大きいほうを返す
  const inkOn = fill => {
    const on = lum(fill) + 0.05;
    const gap = c => { const l = lum(c) + 0.05; return l > on ? l / on : on / l; };
    const ink = asRgb(computed.getPropertyValue('--ink').trim());
    const surface = asRgb(computed.getPropertyValue('--surface').trim());
    return gap(ink) >= gap(surface) ? ink : surface;
  };

  mermaid.run({ querySelector: '.mermaid' }).then(() => {
    document.querySelectorAll('.mermaid svg').forEach(svg => {
      const natural = svg.viewBox.baseVal && svg.viewBox.baseVal.width;
      if (natural) svg.style.minWidth = Math.min(natural, 736) + 'px';
    });

    document.head.appendChild(probe);
    document.querySelectorAll('.mermaid svg').forEach(svg => {
      const slices = svg.querySelectorAll('.pieCircle');
      svg.querySelectorAll('text.slice').forEach((t, i) => {
        if (slices[i]) t.style.setProperty('fill', inkOn(getComputedStyle(slices[i]).fill), 'important');
      });
    });
    probe.remove();
  });
})();
