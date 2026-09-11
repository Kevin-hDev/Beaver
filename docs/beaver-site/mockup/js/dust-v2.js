/* Beaver — poussière de bois v3 : la rivière apporte la matière,
   le dessin du barrage l'attrape.

   Les particules venues de la rivière convergent vers des points tirés dans
   les zones encrées du dessin (dam-ink.js), les rangs du bas d'abord ; au-delà
   de ~85 % de convergence le dessin teinté monte en fondu pendant qu'elles
   s'effacent, et l'inverse à la dispersion.

   data-mode="drift" sur le canevas : pas de barrage, juste la dérive — le
   dessin n'est alors même pas téléchargé. Même repli si le SVG manque.

   Debug : ?phase=<0..2> force la phase (0 dispersé, 1 barrage, 2 dispersé). */

(function () {
  'use strict';

  var canvas = document.getElementById('dust');
  if (!canvas) return;

  var ctx = canvas.getContext('2d');
  var W = 0, H = 0, T = 0;

  var query = new URLSearchParams(location.search);
  var forced = query.has('phase') ? parseFloat(query.get('phase')) : null;
  var calm = matchMedia('(prefers-reduced-motion: reduce)').matches;
  var driftOnly = canvas.dataset.mode === 'drift';

  var RIVER = 42, DUST = 36, DRIFT_PARTS = 150;
  var hasInk = false;
  var parts = [], river = [], dust = [];

  function box() {
    var w = Math.min(430, W * 0.30);
    var ratio = (window.DamInk && window.DamInk.ratio) || 3;
    return { w: w, h: w / ratio, cx: W * 0.72, baseY: H * 0.73 };
  }
  function band() { return { top: 0.77, bot: 0.99 }; }

  function rnd(a, b) { return a + Math.random() * (b - a); }
  function smooth(t) { return t * t * (3 - 2 * t); }
  function clamp01(t) { return Math.max(0, Math.min(1, t)); }

  function build() {
    // Sans dessin, des cibles factices : elles ne servent jamais puisque la
    // convergence reste à zéro, mais la rivière garde ses particules.
    var pts = hasInk ? window.DamInk.points : [], i;
    if (!hasInk) {
      for (i = 0; i < DRIFT_PARTS; i++) pts.push({ u: Math.random(), v: Math.random() });
    }
    var b = band();
    parts = [];
    for (i = 0; i < pts.length; i++) {
      parts.push({
        u: pts[i].u, v: pts[i].v,
        ax: Math.random(), ay: rnd(b.top, b.bot),   // départ rivière
        bx: Math.random(), by: Math.random(),       // position dispersée
        r: rnd(0.55, 1.5),
        cur: rnd(0.00035, 0.0009),
        vx: rnd(-0.00015, 0.00015), vy: rnd(0.00005, 0.00025),
        o: rnd(0.10, 0.34),
        spin: rnd(0.35, 0.85) * (Math.random() < 0.5 ? -1 : 1),
        // les points du bas se posent d'abord
        delay: (1 - pts[i].v) * 0.30 + Math.random() * 0.05
      });
    }
    river = []; dust = [];
    for (i = 0; i < RIVER; i++) {
      river.push({ x: Math.random(), y: Math.random(), sp: rnd(0.0005, 0.0016),
                   len: rnd(14, 44), o: rnd(0.14, 0.42) });
    }
    for (i = 0; i < DUST; i++) {
      dust.push({ x: Math.random(), y: Math.random(),
                  vx: rnd(-0.00012, 0.00012), vy: rnd(0.00004, 0.00018),
                  r: rnd(0.6, 1.6), o: rnd(0.09, 0.24) });
    }
  }

  function resize() {
    var ratio = Math.min(devicePixelRatio || 1, 2);
    W = innerWidth; H = innerHeight;
    canvas.width = W * ratio; canvas.height = H * ratio;
    canvas.style.width = W + 'px'; canvas.style.height = H + 'px';
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    retint();
  }

  function phase() {
    if (forced !== null && !isNaN(forced)) return forced;
    if (!hasInk) return 0;
    var hero = document.querySelector('.hero');
    var h = (hero && hero.offsetHeight) || innerHeight;
    if (scrollY < h * 0.55) return scrollY / (h * 0.55);
    if (scrollY < h * 1.05) return 1;
    return 1 + Math.min(1, (scrollY - h * 1.05) / (h * 0.75));
  }

  // Tons de bois : l'accent décliné en ombres. Cache parsé, rafraîchi
  // avec le thème.
  var toneCache = {};
  function accentHex() {
    return getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() || '#E8862E';
  }
  var accent = accentHex();
  setInterval(function () {
    var next = accentHex();
    if (next !== accent) { accent = next; toneCache = {}; retint(); repaintCalm(); }
  }, 1200);

  function shade(f) {
    var cle = accent + f;
    if (toneCache[cle]) return toneCache[cle];
    var m = /^#?([0-9a-f]{6})$/i.exec(accent);
    if (!m) return accent;
    var n = parseInt(m[1], 16);
    var out = 'rgb(' + Math.round(((n >> 16) & 255) * f) + ',' +
              Math.round(((n >> 8) & 255) * f) + ',' + Math.round((n & 255) * f) + ')';
    toneCache[cle] = out;
    return out;
  }

  // Le dessin teinté n'est régénéré qu'au redimensionnement et au changement
  // de thème — jamais dans la boucle d'animation.
  function retint() {
    if (!hasInk || !W) return;
    var b = box(), s = Math.min(devicePixelRatio || 1, 2) * 1.5;
    window.DamInk.tint(b.w * s, b.h * s, accent);
  }

  function drawInk(alpha) {
    var b = box();
    ctx.globalAlpha = alpha;
    ctx.drawImage(window.DamInk.canvas, b.cx - b.w / 2, b.baseY - b.h, b.w, b.h);
    ctx.globalAlpha = 1;
  }

  function frame() {
    requestAnimationFrame(frame);
    T += 16;

    var ph = phase();
    var gathered = hasInk ? smooth(1 - Math.abs(ph - 1)) : 0;
    var B = box(), RB = band();
    var i, x, k;

    canvas.style.opacity = 0.36 + gathered * 0.64;
    ctx.clearRect(0, 0, W, H);

    // ---- poussière ambiante ---------------------------------------------
    ctx.fillStyle = shade(1);
    for (i = 0; i < dust.length; i++) {
      var d = dust[i];
      d.x += d.vx; d.y += d.vy;
      if (d.y > 1.02) { d.y = -0.02; d.x = Math.random(); }
      if (d.x > 1.02) d.x = -0.02; else if (d.x < -0.02) d.x = 1.02;
      ctx.globalAlpha = d.o;
      ctx.fillRect(d.x * W - d.r, d.y * H - d.r, d.r * 2, d.r * 2);
    }

    // ---- la rivière ------------------------------------------------------
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
      ctx.fillRect(f.x * W - f.len / 2, H * (RB.top + f.y * (RB.bot - RB.top)) - 1.1, f.len, 2.2);
    }

    // ---- fondu croisé : le dessin prend le relais en fin de convergence ---
    var inkK = smooth(clamp01((gathered - 0.85) / 0.15));

    // ---- particules capturées par le dessin ------------------------------
    ctx.fillStyle = shade(0.95);
    for (i = 0; i < parts.length; i++) {
      var p = parts[i];

      p.ax += p.cur; if (p.ax > 1.04) { p.ax = -0.04; p.ay = rnd(RB.top, RB.bot); }
      p.bx += p.vx; p.by += p.vy;
      if (p.by > 1.02) { p.by = -0.02; p.bx = Math.random(); }

      // 2.4 : à mi-parcours la silhouette doit déjà se lire. Plus bas, la
      // moitié du trajet ne donne qu'un nuage.
      k = smooth(clamp01(gathered * 2.4 - p.delay));
      var lx = (ph <= 1 ? p.ax : p.bx) * W;
      var ly = (ph <= 1 ? p.ay : p.by) * H;

      var tx = B.cx - B.w / 2 + p.u * B.w;
      var ty = B.baseY - B.h + p.v * B.h;

      var s = 1 - k;
      x = lx + (tx - lx) * k;
      var y = ly + (ty - ly) * k + Math.sin(T * 0.002 + i * 0.7) * s * 14 * p.spin;

      var r = p.r * (0.8 + k * 0.3);
      ctx.globalAlpha = (p.o + Math.pow(k, 2.2) * (1 - p.o)) * (1 - inkK);
      ctx.fillRect(x - r, y - r, r * 2, r * 2);
    }

    if (inkK > 0) drawInk(inkK);
    ctx.globalAlpha = 1;
  }

  // Mouvement réduit : le dessin, net et immobile, sans particules.
  // À noter : css/motion.css masque #dust dans ce mode, donc rien n'est visible
  // aujourd'hui — ce dessin réapparaît le jour où cette règle CSS tombe.
  function repaintCalm() {
    if (!calm || !hasInk) return;
    ctx.clearRect(0, 0, W, H);
    canvas.style.opacity = 1;
    drawInk(1);
  }

  function start(ok) {
    hasInk = ok;
    resize();
    addEventListener('resize', function () { resize(); repaintCalm(); });
    if (calm) { repaintCalm(); return; }
    build();
    requestAnimationFrame(frame);
  }

  // Le dessin n'est téléchargé que là où le barrage peut se former.
  if (driftOnly || !window.DamInk) start(false);
  else window.DamInk.load(start);
})();
