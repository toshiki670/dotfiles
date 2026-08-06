// 描いてみないと分からない欠陥を build 時に見つける。生成物には入らない —
// build.sh が一時複製にだけ足して headless Chrome で走らせる。
//
// 結果は document.title へ JSON で書き出す。--dump-dom した DOM から拾えて、
// かつ検査自身がレイアウトを動かさない唯一の置き場。

(async () => {
  const figures = [...document.querySelectorAll('.mermaid')];

  // mermaid が描き終わるまで待つ。data-processed は run() が付ける
  const deadline = Date.now() + 15000;
  while (figures.some(el => !el.hasAttribute('data-processed')) && Date.now() < deadline) {
    await new Promise(r => setTimeout(r, 50));
  }

  const report = { viewport: window.innerWidth, figures: figures.length, fail: [] };

  // (1) mermaid が SVG を返したか
  figures.forEach((el, i) => {
    if (!el.querySelector('svg')) report.fail.push(`図 ${i + 1}: SVG が出ていない（構文エラーか未描画）`);
    if (el.querySelector('.error-icon, .error-text')) report.fail.push(`図 ${i + 1}: mermaid が構文エラーを描いた`);
  });

  // (2) ラベルが箱からはみ出していないか。
  // 比べるのは実際に字が占めた範囲とノードの図形。foreignObject を相手にすると、
  // 中の要素がその箱ちょうどに広がるので常に一致してしまい、検査が空振りする。
  let worst = { px: 0, text: '' };
  document.querySelectorAll('.mermaid g.node').forEach(g => {
    const shape = g.querySelector(':scope > rect, :scope > polygon, :scope > path, :scope > circle, :scope > ellipse');
    if (!shape) return;
    const box = shape.getBoundingClientRect();
    if (box.width < 1 || box.height < 1) return;

    const fo = g.querySelector('foreignObject');
    let ink;
    if (fo) {
      const range = document.createRange();
      range.selectNodeContents(fo.querySelector('div') || fo);
      ink = range.getBoundingClientRect();
    } else {
      const text = g.querySelector('text');
      if (!text) return;
      ink = text.getBoundingClientRect();
    }

    const px = Math.max(box.left - ink.left, ink.right - box.right, box.top - ink.top, ink.bottom - box.bottom);
    if (px > worst.px) worst = { px: Math.round(px * 10) / 10, text: g.textContent.trim() };
  });

  report.labelOverflow = worst.px;
  if (worst.px > 0.5) report.fail.push(`ラベルが ${worst.px}px はみ出す: 「${worst.text}」`);

  // (3) 外部参照 0 か。実際に取りに行った先を見るので、断片の grep より強い
  const remote = performance.getEntriesByType('resource')
    .map(r => r.name)
    .filter(n => !n.startsWith('file:') && !n.startsWith('data:') && !n.startsWith('blob:'));
  if (remote.length) report.fail.push(`外部へ取りに行った: ${remote.slice(0, 3).join(', ')}`);

  // (4) 文書幅が viewport を超えないか
  report.scrollWidth = document.documentElement.scrollWidth;
  if (report.scrollWidth > window.innerWidth) {
    const culprits = [...document.querySelectorAll('body *')]
      .filter(el => el.getBoundingClientRect().right > window.innerWidth + 0.5)
      .map(el => {
        const c = el.getAttribute('class');
        return el.tagName.toLowerCase() + (c ? '.' + c.trim().split(/\s+/).join('.') : '');
      })
      .slice(0, 3);
    report.fail.push(`文書幅 ${report.scrollWidth} が viewport ${window.innerWidth} を超える: ${culprits.join(', ') || '不明'}`);
  }

  document.title = 'WMCHECK ' + JSON.stringify(report);
})();
