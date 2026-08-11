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

  // (3) ラベルどうしが重なっていないか。
  // 図形を持たず <text> だけで組む型（venn など）は上の2つに掛からない。
  // 領域が狭いところへ長いラベルを置くと、隣の字の上に乗ったまま素通りする。
  //
  // 字は回転して置かれることがある（日付軸の labelRotation、縦書きの軸題）。
  // 軸並行の矩形で測ると、斜めの字は実際より太って見えて隣と重なる。
  // 四隅を画面座標へ移し、平行四辺形どうしの交わりを面積で見る。
  const quad = t => {
    const b = t.getBBox(), m = t.getScreenCTM();
    const p = [[b.x, b.y], [b.x + b.width, b.y], [b.x + b.width, b.y + b.height], [b.x, b.y + b.height]]
      .map(([x, y]) => [m.a * x + m.c * y + m.e, m.b * x + m.d * y + m.f]);
    // 下の切り取りは辺の向きに依存する。回り方を揃えておく
    const turn = p.reduce((s, [x1, y1], i) => {
      const [x2, y2] = p[(i + 1) % 4];
      return s + (x2 - x1) * (y2 + y1);
    }, 0);
    return turn > 0 ? p.reverse() : p;
  };

  const cut = (p1, p2, p3, p4) => {
    const d = (p1[0] - p2[0]) * (p3[1] - p4[1]) - (p1[1] - p2[1]) * (p3[0] - p4[0]);
    const u = p1[0] * p2[1] - p1[1] * p2[0], v = p3[0] * p4[1] - p3[1] * p4[0];
    return [(u * (p3[0] - p4[0]) - (p1[0] - p2[0]) * v) / d, (u * (p3[1] - p4[1]) - (p1[1] - p2[1]) * v) / d];
  };

  // 一方を他方の4辺で順に切り落とす（Sutherland–Hodgman）。凸どうしなので交わりも凸
  const overlap = (sub, cl) => {
    let poly = sub;
    for (let i = 0; i < 4 && poly.length; i++) {
      const e0 = cl[i], e1 = cl[(i + 1) % 4];
      const side = ([x, y]) => (e1[0] - e0[0]) * (y - e0[1]) - (e1[1] - e0[1]) * (x - e0[0]);
      const input = poly;
      poly = [];
      for (let j = 0; j < input.length; j++) {
        const cur = input[j], prev = input[(j + input.length - 1) % input.length];
        if (side(cur) >= 0) {
          if (side(prev) < 0) poly.push(cut(prev, cur, e0, e1));
          poly.push(cur);
        } else if (side(prev) >= 0) {
          poly.push(cut(prev, cur, e0, e1));
        }
      }
    }
    if (poly.length < 3) return 0;
    return Math.abs(poly.reduce((s, [x1, y1], i) => {
      const [x2, y2] = poly[(i + 1) % poly.length];
      return s + x1 * y2 - x2 * y1;
    }, 0)) / 2;
  };

  let clash = null;
  document.querySelectorAll('.mermaid svg').forEach(svg => {
    const boxes = [...svg.querySelectorAll('text')]
      .filter(t => t.textContent.trim())
      .map(t => ({ s: t.textContent.trim(), r: t.getBoundingClientRect(), q: quad(t) }));
    for (let i = 0; i < boxes.length; i++) {
      for (let j = i + 1; j < boxes.length; j++) {
        // 軸並行の矩形が離れていれば平行四辺形も交わらない。重いほうを先に落とす
        const a = boxes[i].r, b = boxes[j].r;
        if (Math.min(a.right, b.right) <= Math.max(a.left, b.left)) continue;
        if (Math.min(a.bottom, b.bottom) <= Math.max(a.top, b.top)) continue;
        const area = overlap(boxes[i].q, boxes[j].q);
        if (area > 1 && (!clash || area > clash.area)) {
          clash = { area: Math.round(area), a: boxes[i].s, b: boxes[j].s };
        }
      }
    }
  });
  if (clash) report.fail.push(`ラベルが ${clash.area}px² 重なる: 「${clash.a}」と「${clash.b}」`);

  // (4) 外部参照 0 か。実際に取りに行った先を見るので、断片の grep より強い
  const remote = performance.getEntriesByType('resource')
    .map(r => r.name)
    .filter(n => !n.startsWith('file:') && !n.startsWith('data:') && !n.startsWith('blob:'));
  if (remote.length) report.fail.push(`外部へ取りに行った: ${remote.slice(0, 3).join(', ')}`);

  // (5) 文書幅が viewport を超えないか
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

  // (6) 系列が環の色数に収まっているか。
  // 環の値は head.html の --series-N が持つ。書き方（16進・rgb()・色名）に依らず比べたいので、
  // 字の色として一度ブラウザに解かせてから突き合わせる
  const solve = document.createElement('span');
  document.head.appendChild(solve);
  const asColor = v => {
    solve.style.color = '';
    solve.style.color = v;
    return getComputedStyle(solve).color;
  };

  const declared = new Set();
  for (const sheet of document.styleSheets) {
    for (const rule of sheet.cssRules) {
      if (!rule.style) continue;
      for (const prop of rule.style) {
        if (/^--series-\d+$/.test(prop)) declared.add(prop);
      }
    }
  }
  const ring = new Set([...declared]
    .map(p => asColor(getComputedStyle(document.documentElement).getPropertyValue(p).trim())));
  report.ring = ring.size;

  const paint = el => {
    const s = getComputedStyle(el);
    return s.fill === 'none' ? s.stroke : s.fill;
  };

  figures.forEach((el, i) => {
    const svg = el.querySelector('svg');
    if (!svg) return;
    // 1系列ぶんの印を1つずつ拾う。棒は1系列が複数の矩形になるので群の先頭だけを見る
    const colors = [
      ...[...svg.querySelectorAll('path.pieCircle')].map(paint),
      ...[...svg.querySelectorAll('g[class*="-plot-"]')].map(g => g.querySelector('path, rect')).filter(Boolean).map(paint),
      ...[...svg.querySelectorAll('path[class^="radarCurve-"]')].map(paint),
    ];
    if (!colors.length) return;
    const off = colors.filter(c => !ring.has(c));
    if (off.length) {
      report.fail.push(`図 ${i + 1}: 環に無い色の系列がある（${off[0]}）。系列は環の ${ring.size} 色まで`);
    } else if (new Set(colors).size !== colors.length) {
      report.fail.push(`図 ${i + 1}: 系列 ${colors.length} 個が環の ${ring.size} 色を使い回している`);
    }
  });
  solve.remove();

  document.title = 'WMCHECK ' + JSON.stringify(report);
})();
