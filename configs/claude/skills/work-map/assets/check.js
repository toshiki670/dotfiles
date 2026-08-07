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
  // 字を切るのは foreignObject なので、そこへ収まっているかを見る。要素の矩形は当てにならない
  // — 中の div も span も p も foreignObject にクランプされ、その箱と同じ値を返す。
  // テキストノードを Range で選べば、実際に字が占めた幅が出る。
  let worst = { px: 0, text: '' };

  const inkOf = node => {
    // 字を持つ最も内側の要素まで降りる
    let el = node, next;
    while ((next = [...el.children].find(c => c.textContent.trim() === el.textContent.trim()))) el = next;
    const texts = [...el.childNodes].filter(n => n.nodeType === Node.TEXT_NODE && n.length);
    if (!texts.length) return null;
    const range = document.createRange();
    range.setStart(texts[0], 0);
    range.setEnd(texts[texts.length - 1], texts[texts.length - 1].length);
    return range.getBoundingClientRect();
  };

  const note = (ink, box, label) => {
    if (!ink || !box || box.width < 1 || box.height < 1) return;
    const px = Math.max(box.left - ink.left, ink.right - box.right, box.top - ink.top, ink.bottom - box.bottom);
    if (px > worst.px) worst = { px: Math.round(px * 10) / 10, text: label.trim() };
  };

  document.querySelectorAll('.mermaid foreignObject').forEach(fo => {
    note(inkOf(fo), fo.getBoundingClientRect(), fo.textContent);
  });

  // htmlLabels を切った場合は <text> が字そのものなので、ノードの図形と比べる
  document.querySelectorAll('.mermaid g.node').forEach(g => {
    if (g.querySelector('foreignObject')) return;
    const text = g.querySelector('text');
    const shape = g.querySelector(':scope > rect, :scope > polygon, :scope > path, :scope > circle, :scope > ellipse');
    if (text && shape) note(text.getBoundingClientRect(), shape.getBoundingClientRect(), g.textContent);
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
