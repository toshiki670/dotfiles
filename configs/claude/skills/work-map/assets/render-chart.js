// 時系列チャートをライブラリなしで描く。
//
// 呼び出し側が先に SERIES を定義しておく:
//   const SERIES = {
//     mount: 'chart',                        // 描画先の element id
//     label: 'エラー数',                      // aria-label に入る量の名前
//     points: [['2026-06-02', 3], …],        // [ISO 日付, 値]。欠損日は 0 で埋められる
//   };
//
// 必要な計算は (1) 値域→ピクセル域の写像 (2) きりの良い目盛り (3) ラベルの間引き の3つだけ。
// 目盛り生成が込み入る要求（対数軸・積み上げ）が出たら d3-scale の導入を検討する。

(function () {
  const W = 900, H = 260, PAD = { t: 18, r: 16, b: 34, l: 42 };
  const IW = W - PAD.l - PAD.r, IH = H - PAD.t - PAD.b;

  // 欠損日を 0 で埋める。時系列は「無かった日」も形の一部。
  function fill(rows) {
    const out = [];
    const d = new Date(rows[0][0] + 'T00:00:00Z');
    const end = new Date(rows[rows.length - 1][0] + 'T00:00:00Z');
    const map = Object.fromEntries(rows);
    while (d <= end) {
      const key = d.toISOString().slice(0, 10);
      out.push([key, map[key] || 0]);
      d.setUTCDate(d.getUTCDate() + 1);
    }
    return out;
  }

  // 目盛りを 1 / 2 / 5 の倍数に丸める。
  // 上限は max を必ず超えるところまで伸ばす（さもないと最大点が枠外に出る）。
  function ticks(max, want = 4) {
    const raw = max / want;
    const mag = Math.pow(10, Math.floor(Math.log10(raw)));
    const n = raw / mag;
    const step = (n <= 1 ? 1 : n <= 2 ? 2 : n <= 5 ? 5 : 10) * mag;
    const out = [];
    for (let v = 0; v < max + step; v += step) out.push(v);
    return out;
  }

  const data = fill(SERIES.points);
  const n = data.length;
  const yMax = Math.max(...data.map(d => d[1]));
  const ys = ticks(yMax);
  const top = ys[ys.length - 1];

  const sx = i => PAD.l + (n === 1 ? IW / 2 : (i * IW) / (n - 1));
  const sy = v => PAD.t + IH - (v / top) * IH;

  const pts = data.map((d, i) => ({ x: sx(i), y: sy(d[1]) }));
  const line = pts.map((p, i) => `${i ? 'L' : 'M'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(' ');
  const area = `${line} L ${pts[n - 1].x.toFixed(1)} ${sy(0).toFixed(1)}`
    + ` L ${pts[0].x.toFixed(1)} ${sy(0).toFixed(1)} Z`;

  const grid = ys.map(v =>
    `<line class="grid" x1="${PAD.l}" y1="${sy(v).toFixed(1)}" x2="${W - PAD.r}" y2="${sy(v).toFixed(1)}" />`
    + `<text class="tick" x="${PAD.l - 8}" y="${(sy(v) + 3.5).toFixed(1)}" text-anchor="end">${v}</text>`
  ).join('');

  // 週の頭だけラベルを出す
  const xlab = data.map((d, i) => {
    if (new Date(d[0] + 'T00:00:00Z').getUTCDay() !== 1) return '';
    return `<text class="tick" x="${sx(i).toFixed(1)}" y="${H - 12}" text-anchor="middle">${d[0].slice(5)}</text>`;
  }).join('');

  // 最大値だけ直接ラベルを付ける（全点に数字を置かない）
  const peak = data.reduce((a, d, i) => (d[1] > a.v ? { v: d[1], i } : a), { v: -1, i: 0 });
  const mark = `<circle class="peak" cx="${sx(peak.i).toFixed(1)}" cy="${sy(peak.v).toFixed(1)}" r="3.5" />`
    + `<text class="peak-label" x="${sx(peak.i).toFixed(1)}" y="${(sy(peak.v) - 10).toFixed(1)}"`
    + ` text-anchor="middle">${peak.v}</text>`;

  document.getElementById(SERIES.mount || 'chart').innerHTML =
    `<svg viewBox="0 0 ${W} ${H}" role="img"`
    + ` aria-label="${SERIES.label || '時系列'}。${n} 日分、最大 ${peak.v}（${data[peak.i][0]}）">`
    + grid + `<path class="area" d="${area}" /><path class="line" d="${line}" />` + mark + xlab + '</svg>';
})();
