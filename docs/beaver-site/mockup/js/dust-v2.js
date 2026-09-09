/* Beaver — poussière de bois v2.1 : la rivière apporte les branches,
   le barrage les attrape.

   - une rivière en bas de l'écran (ondulations + branches au fil du courant)
   - les particules du barrage VIENNENT de la rivière, puis sont capturées
   - couche croisée de branches diagonales, dessinée SOUS et SUR les bûches
     pour un vrai effet entrelacé
   - tons de bois variés (déclinaisons ombrées de l'accent)
   - silhouette irrégulière : rangs bombés, brindilles qui dépassent
   - assemblage chorégraphié : rangs du bas d'abord, approche en spirale
   - poussière ambiante permanente (le hero ne reste jamais vide)

   Debug : ?phase=<0..2> force la phase (0 dispersé, 1 barrage, 2 dispersé). */

(function () {
  'use strict';

  var canvas = document.getElementById('dust');
  if (!canvas || matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  var ctx = canvas.getContext('2d');
  var W = 0, H = 0, T = 0;

  var query = new URLSearchParams(location.search);
  var forced = query.has('phase') ? parseFloat(query.get('phase')) : null;

  var ROWS = 9, SEG = 16, GAP = 13, THICK = 8;
  var CROSS = 36, TWIGS = 14, RIVER = 42, DUST = 36;

  var rows = [], parts = [], cross = [], twigs = [], river = [], dust = [];

  function box() { return { w: Math.min(400, W * 0.27), cx: W * 0.72, baseY: H * 0.73 }; }
  function band() { return { top: 0.77, bot: 0.99 }; }

  function rnd(a, b) { return a + Math.random() * (b - a); }
  function scatter() { return { x: Math.random(), y: Math.random() }; }
  function inRiver() { var b = band(); return { x: Math.random(), y: rnd(b.top, b.bot) }; }

  function buildRows() {
    rows = [];
    for (var r = 0; r < ROWS; r++) {
      var t = r / (ROWS - 1);
      rows.push({
        span: 0.66 + (1 - t) * 0.36 + Math.random() * 0.07,
        drift: (Math.random() - 0.5) * 0.10,
        angle: (Math.random() - 0.5) * 0.16,
        lift: (Math.random() - 0.5) * 3,
        tone: rnd(0.72, 1.0) // chaque rang a son bois
      });
    }
  }

  function build() {
    buildRows();
    parts = []; cross = []; twigs = []; river = []; dust = [];
    var r, i, a, b;
    for (r = 0; r < ROWS; r++) for (i = 0; i < SEG; i++) {
      a = inRiver(); b = scatter();
      parts.push({
        row: r, slot: i,
        ax: a.x, ay: a.y, bx: b.x, by: b.y,
        r: rnd(0.6, 1.9),
        cur: rnd(0.00035, 0.0009),
        vx: rnd(-0.00015, 0.00015), vy: rnd(0.00005, 0.00025),
        o: rnd(0.10, 0.34),
        spin: rnd(0.35, 0.85) * (Math.random() < 0.5 ? -1 : 1)
      });
    }
    for (i = 0; i < CROSS; i++) {
      cross.push({
        u: Math.random(), v: Math.random(),
        ang: (Math.random() < 0.5 ? -1 : 1) * rnd(0.45, 1.0),
        len: rnd(0.10, 0.26),
        a: inRiver(), b: scatter(),
        o: rnd(0.10, 0.28),
        under: i % 5 < 2 // 40% passent sous les bûches, 60% dessus
      });
    }
    for (i = 0; i < TWIGS; i++) {
      twigs.push({
        row: Math.floor(Math.random() * ROWS),
        end: Math.random() < 0.5 ? -1 : 1,
        ang: rnd(0.5, 1.5), len: rnd(10, 26),
        o: rnd(0.2, 0.5)
      });
    }
    for (i = 0; i < RIVER; i++) {
      river.push({
        x: Math.random(), y: Math.random(),
        sp: rnd(0.0005, 0.0016), len: rnd(14, 44),
        o: rnd(0.14, 0.42)
      });
    }
    for (i = 0; i < DUST; i++) {
      dust.push({
        x: Math.random(), y: Math.random(),
        vx: rnd(-0.00012, 0.00012), vy: rnd(0.00004, 0.00018),
        r: rnd(0.6, 1.6), o: rnd(0.09, 0.24)
      });
    }
  }

  function resize() {
    var ratio = Math.min(devicePixelRatio || 1, 2);
    W = innerWidth; H = innerHeight;
    canvas.width = W * ratio; canvas.height = H * ratio;
    canvas.style.width = W + 'px'; canvas.style.height = H + 'px';
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  }

  function heroHeight() {
    var hero = document.querySelector('.hero');
    return (hero && hero.offsetHeight) || innerHeight;
  }

  var driftOnly = canvas.dataset.mode === 'drift';

  function phase() {
    if (forced !== null && !isNaN(forced)) return forced;
    if (driftOnly) return 0;
    var h = heroHeight();
    if (scrollY < h * 0.55) return scrollY / (h * 0.55);
    if (scrollY < h * 1.05) return 1;
    return 1 + Math.min(1, (scrollY - h * 1.05) / (h * 0.75));
  }

  function smooth(t) { return t * t * (3 - 2 * t); }
  function clamp01(t) { return Math.max(0, Math.min(1, t)); }

  // Tons de bois : l'accent décliné en ombres. Cache parsé, rafraîchi
  // avec le thème.
  var toneCache = {};
  function accentHex() {
    return getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() || '#E8862E';
  }
  var accent = accentHex();
  setInterval(function () {
    var next = accentHex();
    if (next !== accent) { accent = next; toneCache = {}; }
  }, 1200);

  function shade(f) {
    var key = accent + f;
    if (toneCache[key]) return toneCache[key];
    var m = /^#?([0-9a-f]{6})$/i.exec(accent);
    if (!m) return accent;
    var n = parseInt(m[1], 16);
    var r = Math.min(255, Math.round(((n >> 16) & 255) * f));
    var g = Math.min(255, Math.round(((n >> 8) & 255) * f));
    var b = Math.min(255, Math.round((n & 255) * f));
    var out = 'rgb(' + r + ',' + g + ',' + b + ')';
    toneCache[key] = out;
    return out;
  }

  function bar(x, y, w, h, angle) {
    ctx.save();
    ctx.translate(x, y);
    if (angle) ctx.rotate(angle);
    ctx.beginPath();
    if (ctx.roundRect) ctx.roundRect(-w / 2, -h / 2, w, h, Math.min(h / 2, w / 2));
    else ctx.rect(-w / 2, -h / 2, w, h);
    ctx.fill();
    ctx.restore();
  }

  function frame() {
    requestAnimationFrame(frame);
    T += 16;

    var ph = phase();
    var gathered = smooth(1 - Math.abs(ph - 1));
    var B = box(), RB = band();

    canvas.style.opacity = 0.36 + gathered * 0.28;
    ctx.clearRect(0, 0, W, H);

    var i, r, p, k, x, y;

    // ---- poussière ambiante : toujours là, partout -----------------------
    ctx.fillStyle = shade(1);
    for (i = 0; i < dust.length; i++) {
      var d = dust[i];
      d.x += d.vx; d.y += d.vy;
      if (d.y > 1.02) { d.y = -0.02; d.x = Math.random(); }
      if (d.x > 1.02) d.x = -0.02; else if (d.x < -0.02) d.x = 1.02;
      ctx.globalAlpha = d.o;
      bar(d.x * W, d.y * H, d.r * 2, d.r * 2, 0);
    }

    // ---- la rivière : ondulations + branches au courant ------------------
    var waterK = 1 - gathered * 0.8;

    ctx.strokeStyle = shade(0.85);
    ctx.lineWidth = 1.4;
    for (i = 0; i < 4; i++) {
      ctx.globalAlpha = 0.10 + 0.04 * Math.sin(T * 0.0004 + i);
      var wy = H * (RB.top + 0.045 * i + 0.04 * (RB.bot - RB.top));
      ctx.beginPath();
      for (x = 0; x <= W; x += 28) {
        var yy = wy + Math.sin(x * 0.012 + T * 0.0009 + i * 1.7) * 3.2;
        if (x === 0) ctx.moveTo(x, yy); else ctx.lineTo(x, yy);
      }
      ctx.stroke();
    }

    ctx.fillStyle = shade(0.9);
    for (i = 0; i < river.length; i++) {
      var f = river[i];
      f.x += f.sp * (0.6 + f.y * 0.8);
      if (f.x > 1.06) { f.x = -0.06; f.y = Math.random(); }
      ctx.globalAlpha = f.o * waterK;
      bar(f.x * W, H * (RB.top + f.y * (RB.bot - RB.top)),
          f.len, 2.2, Math.sin(T * 0.001 + i) * 0.05);
    }

    // ---- lit « boue » derrière les bûches --------------------------------
    if (gathered > 0.5) {
      ctx.globalAlpha = (gathered - 0.5) * 0.16;
      ctx.fillStyle = shade(0.55);
      var wBase = B.w * rows[0].span * 0.52, wTop = B.w * rows[ROWS - 1].span * 0.52;
      var yTop = B.baseY - (ROWS - 1) * GAP;
      ctx.beginPath();
      ctx.moveTo(B.cx - wBase, B.baseY + 6);
      ctx.lineTo(B.cx + wBase, B.baseY + 6);
      ctx.lineTo(B.cx + wTop, yTop - 6);
      ctx.lineTo(B.cx - wTop, yTop - 6);
      ctx.closePath();
      ctx.fill();
    }

    // ---- croisé SOUS les bûches ------------------------------------------
    var kc = smooth(clamp01(gathered * 1.2 - 0.15));
    ctx.fillStyle = shade(0.55);
    for (i = 0; i < cross.length; i++) {
      var c = cross[i];
      if (!c.under) continue;
      var from = ph <= 1 ? c.a : c.b;
      x = from.x * W + (B.cx + (c.u - 0.5) * B.w * 0.92 - from.x * W) * kc;
      y = from.y * H + (B.baseY - c.v * (ROWS - 1) * GAP * 0.94 - from.y * H) * kc;
      ctx.globalAlpha = c.o * (0.25 + 0.75 * kc);
      bar(x, y, c.len * B.w * (0.3 + 0.7 * kc), 4, c.ang * (0.35 + 0.65 * kc));
    }

    // ---- bûches principales : capturées depuis la rivière ----------------
    for (i = 0; i < parts.length; i++) {
      p = parts[i]; r = rows[p.row];

      p.ax += p.cur; if (p.ax > 1.04) { p.ax = -0.04; p.ay = rnd(RB.top, RB.bot); }
      p.bx += p.vx; p.by += p.vy;
      if (p.by > 1.02) { p.by = -0.02; p.bx = Math.random(); }

      k = smooth(clamp01(gathered * 1.3 - p.row * 0.045));

      var loose = ph <= 1 ? { x: p.ax, y: p.ay } : { x: p.bx, y: p.by };

      var span = B.w * r.span;
      var step = span / (SEG - 1);
      var offset = (p.slot - (SEG - 1) / 2) * step;
      var tx = B.cx + r.drift * B.w + offset * Math.cos(r.angle);
      var ty = B.baseY - p.row * GAP + r.lift + offset * Math.sin(r.angle)
             + Math.pow(offset / (B.w * 0.5), 2) * 7;

      var s = 1 - k;
      var sw = Math.sin(T * 0.002 + p.slot * 1.3 + p.row) * s * 16 * p.spin;

      x = loose.x * W + (tx - loose.x * W) * k;
      y = loose.y * H + (ty - loose.y * H) * k + sw;

      var w = p.r * 2 + k * (step * 1.9 - p.r * 2);
      var h = p.r * 2 + k * (THICK - p.r * 2);

      ctx.globalAlpha = p.o + Math.pow(k, 2.2) * (1 - p.o);
      ctx.fillStyle = shade(r.tone);
      bar(x, y, w, h, r.angle * k + s * p.spin * 2.2);
    }

    // ---- croisé SUR les bûches -------------------------------------------
    ctx.fillStyle = shade(0.8);
    for (i = 0; i < cross.length; i++) {
      var c2 = cross[i];
      if (c2.under) continue;
      var from2 = ph <= 1 ? c2.a : c2.b;
      x = from2.x * W + (B.cx + (c2.u - 0.5) * B.w * 0.92 - from2.x * W) * kc;
      y = from2.y * H + (B.baseY - c2.v * (ROWS - 1) * GAP * 0.94 - from2.y * H) * kc;
      ctx.globalAlpha = c2.o * (0.25 + 0.75 * kc);
      bar(x, y, c2.len * B.w * (0.3 + 0.7 * kc), 4, c2.ang * (0.35 + 0.65 * kc));
    }

    // ---- brindilles qui dépassent -----------------------------------------
    ctx.fillStyle = shade(0.95);
    for (i = 0; i < twigs.length; i++) {
      var tw = twigs[i]; r = rows[tw.row];
      var kt = clamp01((smooth(clamp01(gathered * 1.3 - tw.row * 0.045)) - 0.82) / 0.18);
      if (kt <= 0) continue;
      var half = B.w * r.span / 2;
      var ex = B.cx + r.drift * B.w + tw.end * half * Math.cos(r.angle);
      var ey = B.baseY - tw.row * GAP + r.lift + tw.end * half * Math.sin(r.angle)
             + Math.pow(half / (B.w * 0.5), 2) * 7;
      ctx.globalAlpha = tw.o * kt;
      bar(ex + tw.end * tw.len * 0.4 * kt, ey - tw.len * 0.3 * kt,
          tw.len * kt, 2.4, r.angle + tw.end * tw.ang);
    }

    ctx.globalAlpha = 1;
  }

  build();
  resize();
  addEventListener('resize', function () { resize(); });
  requestAnimationFrame(frame);
})();
