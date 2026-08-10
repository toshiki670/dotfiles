// mermaid の図を描く。断片は本文側に <pre class="mermaid"> で書く。
//
// 色は head.html の :root にある --mm-* を読んで渡す。値の出所はページのトークン1箇所で、
// 型を増やすときも写像を CSS へ足すだけで済む。残りの - は入れ子に開くので、
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
  mermaid.run({ querySelector: '.mermaid' }).then(() => {
    document.querySelectorAll('.mermaid svg').forEach(svg => {
      const natural = svg.viewBox.baseVal && svg.viewBox.baseVal.width;
      if (natural) svg.style.minWidth = Math.min(natural, 736) + 'px';
    });
  });
})();
