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

  // (6) 隣り合う系列を見分けられるか。
  // 色数では見ない — 何色まで置けるかは、色の差が基準を満たすかで決まる。
  // 明るさの差だけでは色相を見ず、色相の差だけでは色覚によって潰れるので両方を見る。
  // 印が下地の上に直接置かれる型（線・レーダー）は、下地との差も同じ基準で見る。
  // 円グラフは区画どうしが接していて境目の線が残るため、下地との差は問われない。
  const DE_MIN = 18.5;
  const RATIO_MIN = 2.13;

  const solve = document.createElement('span');
  document.head.appendChild(solve);
  const rgbOf = v => {
    solve.style.color = '';
    solve.style.color = v;
    return getComputedStyle(solve).color.match(/[\d.]+/g).slice(0, 3).map(Number);
  };

  const lum = ([r, g, b]) => {
    const f = v => (v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
  };
  const contrast = (a, b) => {
    const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
    return (x + 0.05) / (y + 0.05);
  };

  const lab = ([r, g, b]) => {
    const f = v => (v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    const [R, G, B] = [f(r), f(g), f(b)];
    const x = (0.4124564 * R + 0.3575761 * G + 0.1804375 * B) / 0.95047;
    const y = 0.2126729 * R + 0.7151522 * G + 0.0721750 * B;
    const z = (0.0193339 * R + 0.1191920 * G + 0.9503041 * B) / 1.08883;
    const g_ = t => t > 216 / 24389 ? Math.cbrt(t) : (841 / 108) * t + 4 / 29;
    return [116 * g_(y) - 16, 500 * (g_(x) - g_(y)), 200 * (g_(y) - g_(z))];
  };

  // CIEDE2000。明るさだけの比較では色相の違いが 1.0 と出てしまうため
  const deltaE = (c1, c2) => {
    const [L1, a1, b1] = lab(c1), [L2, a2, b2] = lab(c2);
    const rad = d => d * Math.PI / 180;
    const C1 = Math.hypot(a1, b1), C2 = Math.hypot(a2, b2);
    const Cb = (C1 + C2) / 2;
    const G = 0.5 * (1 - Math.sqrt(Cb ** 7 / (Cb ** 7 + 25 ** 7)) || 0);
    const A1 = (1 + G) * a1, A2 = (1 + G) * a2;
    const P1 = Math.hypot(A1, b1), P2 = Math.hypot(A2, b2);
    const h1 = (A1 || b1) ? (Math.atan2(b1, A1) * 180 / Math.PI + 360) % 360 : 0;
    const h2 = (A2 || b2) ? (Math.atan2(b2, A2) * 180 / Math.PI + 360) % 360 : 0;
    const dL = L2 - L1, dC = P2 - P1;
    let dh = 0;
    if (P1 * P2) {
      dh = h2 - h1;
      if (dh > 180) dh -= 360; else if (dh < -180) dh += 360;
    }
    const dH = 2 * Math.sqrt(P1 * P2) * Math.sin(rad(dh) / 2);
    const Lb = (L1 + L2) / 2, Pb = (P1 + P2) / 2;
    let hb = h1 + h2;
    if (P1 * P2) {
      if (Math.abs(h1 - h2) <= 180) hb /= 2;
      else hb = (hb < 360) ? (hb + 360) / 2 : (hb - 360) / 2;
    }
    const T = 1 - 0.17 * Math.cos(rad(hb - 30)) + 0.24 * Math.cos(rad(2 * hb))
      + 0.32 * Math.cos(rad(3 * hb + 6)) - 0.20 * Math.cos(rad(4 * hb - 63));
    const Sl = 1 + (0.015 * (Lb - 50) ** 2) / Math.sqrt(20 + (Lb - 50) ** 2);
    const Sc = 1 + 0.045 * Pb;
    const Sh = 1 + 0.015 * Pb * T;
    const Rt = -Math.sin(rad(2 * 30 * Math.exp(-(((hb - 275) / 25) ** 2))))
      * (2 * Math.sqrt(Pb ** 7 / (Pb ** 7 + 25 ** 7)) || 0);
    return Math.sqrt((dL / Sl) ** 2 + (dC / Sc) ** 2 + (dH / Sh) ** 2 + Rt * (dC / Sc) * (dH / Sh));
  };

  const paint = el => {
    const s = getComputedStyle(el);
    return rgbOf(s.fill === 'none' ? s.stroke : s.fill);
  };

  figures.forEach((el, i) => {
    const svg = el.querySelector('svg');
    if (!svg) return;
    // 1系列ぶんの印を並び順に1つずつ拾う。棒は1系列が複数の矩形になるので群の先頭だけを見る。
    // 印は要素名で探さない — radar は graticule の書き方で曲線が path と polygon に入れ替わる
    const enclosed = [...svg.querySelectorAll('.pieCircle')].map(paint);
    const onGround = [
      ...[...svg.querySelectorAll('g[class*="-plot-"]')].map(g => g.querySelector('path, rect')).filter(Boolean).map(paint),
      ...[...svg.querySelectorAll('[class^="radarCurve-"]')].map(paint),
    ];
    const series = [...enclosed, ...onGround];
    if (series.length < 1) return;

    // 同じ色が2つの系列に回ってきたら、離れていても見分けられない
    const seen = new Map();
    for (let j = 0; j < series.length; j++) {
      const key = series[j].join(',');
      if (seen.has(key)) {
        report.fail.push(`図 ${i + 1}: ${seen.get(key) + 1}番目と${j + 1}番目の系列が同じ色で描かれている`);
        break;
      }
      seen.set(key, j);
    }

    for (let j = 0; j + 1 < series.length; j++) {
      const de = deltaE(series[j], series[j + 1]);
      const ra = contrast(series[j], series[j + 1]);
      if (de < DE_MIN || ra < RATIO_MIN) {
        report.fail.push(`図 ${i + 1}: ${j + 1}番目と${j + 2}番目の系列を見分けられない`
          + `（ΔE ${de.toFixed(1)} / 明るさの差 ${ra.toFixed(2)} 倍。基準は ${DE_MIN} と ${RATIO_MIN} 倍）`);
        break;
      }
    }

    if (onGround.length) {
      // 下地は型で違う。折れ線は自前の下地を敷き、無い型はページの面がそのまま下地になる
      const plate = svg.querySelector('rect.background');
      const ground = rgbOf(plate ? getComputedStyle(plate).fill
        : getComputedStyle(document.documentElement).getPropertyValue('--surface').trim());
      for (let j = 0; j < onGround.length; j++) {
        const ra = contrast(onGround[j], ground);
        if (ra < RATIO_MIN) {
          report.fail.push(`図 ${i + 1}: ${j + 1}番目の系列が下地に埋もれる`
            + `（明るさの差 ${ra.toFixed(2)} 倍。基準は ${RATIO_MIN} 倍）`);
          break;
        }
      }
    }
  });
  // (7) 区画の上に載る字が読めるか。
  // 描く側は字が乗っている塗りから色を選ぶ。円グラフは 1% に満たない区画を描かずに
  // 色番号だけ進めるので、何番目かで当てると外れる。選び損ねをここで捕まえる
  document.querySelectorAll('.mermaid svg').forEach(svg => {
    const slices = svg.querySelectorAll('.pieCircle');
    svg.querySelectorAll('text.slice').forEach((t, i) => {
      if (!slices[i]) return;
      const ra = contrast(rgbOf(getComputedStyle(t).fill), rgbOf(getComputedStyle(slices[i]).fill));
      if (ra < 4.5) {
        report.fail.push(`区画の上の「${t.textContent.trim()}」が読めない（${ra.toFixed(2)}:1。4.5:1 が要る）`);
      }
    });
  });

  solve.remove();

  document.title = 'WMCHECK ' + JSON.stringify(report);
})();
