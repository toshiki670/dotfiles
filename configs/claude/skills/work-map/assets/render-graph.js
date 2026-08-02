// 有向グラフを描く。座標は dagre、意匠はページ側の CSS 変数。
//
// 呼び出し側が先に GRAPH を定義しておく:
//   const GRAPH = {
//     mount: 'dag',                                  // 描画先の element id
//     nodes: [{ id, label, kind }],                  // kind: 'settled' | '' | 'open'
//     edges: [[fromId, toId]],
//   };
//
// kind はページの状態語彙に対応する。settled = 確定した前提、'' = 導かれる帰結、open = 未確定。

(function () {
  const NW = 178, NH = 42;

  function build(nodes, edges) {
    const g = new dagre.graphlib.Graph();
    g.setGraph({ rankdir: 'LR', ranksep: 88, nodesep: 14, edgesep: 10, marginx: 22, marginy: 22 });
    g.setDefaultEdgeLabel(() => ({}));
    nodes.forEach(n => g.setNode(n.id, { width: NW, height: NH }));
    edges.forEach(([a, b]) => g.setEdge(a, b));
    dagre.layout(g);
    return g;
  }

  function arrow(points) {
    const last = points[points.length - 1], prev = points[points.length - 2];
    const dx = last.x - prev.x, dy = last.y - prev.y;
    const len = Math.hypot(dx, dy) || 1;
    const ux = dx / len, uy = dy / len;
    return `${last.x},${last.y} `
      + `${last.x - ux * 9 - uy * 4},${last.y - uy * 9 + ux * 4} `
      + `${last.x - ux * 9 + uy * 4},${last.y - uy * 9 - ux * 4}`;
  }

  function esc(s) {
    return String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  }

  const g = build(GRAPH.nodes, GRAPH.edges);
  const { width, height } = g.graph();
  const byId = Object.fromEntries(GRAPH.nodes.map(n => [n.id, n]));

  const edges = g.edges().map(e => {
    const pts = g.edge(e).points;
    const d = pts.map((p, i) => `${i ? 'L' : 'M'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(' ');
    return `<path class="edge" d="${d}" /><polygon class="arrow-head" points="${arrow(pts)}" />`;
  }).join('');

  const nodes = g.nodes().map(id => {
    const n = byId[id], p = g.node(id);
    return `<g><rect class="node-box ${n.kind || ''}" x="${(p.x - NW / 2).toFixed(1)}"`
      + ` y="${(p.y - NH / 2).toFixed(1)}" width="${NW}" height="${NH}" />`
      + `<text class="node-label" x="${p.x.toFixed(1)}" y="${(p.y + 4.5).toFixed(1)}"`
      + ` text-anchor="middle">${esc(n.label)}</text></g>`;
  }).join('');

  document.getElementById(GRAPH.mount || 'dag').innerHTML =
    `<svg viewBox="0 0 ${Math.ceil(width)} ${Math.ceil(height)}" role="img"`
    + ` aria-label="${esc(GRAPH.label || '依存関係')}。ノード ${GRAPH.nodes.length} 件、エッジ ${GRAPH.edges.length} 件">`
    + edges + nodes + '</svg>';
})();
