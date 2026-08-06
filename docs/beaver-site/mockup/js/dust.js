/* Beaver — la poussière de bois s'assemble en barrage, puis se disperse à
   nouveau, ailleurs. Trois temps pilotés par le défilement :
   dispersion → entassement → dispersion.

   Le barrage est volontairement irrégulier : longueurs inégales, bûches
   inclinées, base plus large que le sommet. Une pile de poutres alignées
   ne se lit pas comme un barrage de castor. */

(function () {
  'use strict';

  var canvas = document.getElementById('dust');
  if (!canvas || matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  var ctx = canvas.getContext('2d');
  var width = 0;
  var height = 0;
  var particles = [];
  var rows = [];

  var ROWS = 8;
  var PER_ROW = 18;
  var THICKNESS = 8;
  var ROW_GAP = 13;

  function stackBox() {
    return { w: Math.min(400, width * 0.28), cx: width * 0.72, cy: height * 0.62 };
  }

  function buildRows() {
    rows = [];
    for (var r = 0; r < ROWS; r++) {
      var fromTop = (ROWS - 1 - r) / (ROWS - 1);
      rows.push({
        // La base porte la charge : les rangs bas sont les plus longs.
        span: 0.62 + fromTop * 0.38 + Math.random() * 0.1,
        drift: (Math.random() - 0.5) * 0.14,
        angle: (Math.random() - 0.5) * 0.13,
        lift: (Math.random() - 0.5) * 4
      });
    }
  }

  function scatter() {
    return { x: Math.random(), y: Math.random() };
  }

  function build() {
    buildRows();
    particles = [];
    for (var r = 0; r < ROWS; r++) {
      for (var i = 0; i < PER_ROW; i++) {
        var a = scatter();
        var b = scatter();
        particles.push({
          row: r,
          slot: i,
          ax: a.x, ay: a.y,
          bx: b.x, by: b.y,
          r: Math.random() * 1.4 + 0.6,
          vx: (Math.random() - 0.5) * 0.00015,
          vy: Math.random() * 0.0002 + 0.00005,
          o: Math.random() * 0.26 + 0.1
        });
      }
    }
  }

  function resize() {
    var ratio = Math.min(devicePixelRatio || 1, 2);
    width = innerWidth;
    height = innerHeight;
    canvas.width = width * ratio;
    canvas.height = height * ratio;
    canvas.style.width = width + 'px';
    canvas.style.height = height + 'px';
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  }

  function heroHeight() {
    var hero = document.querySelector('.hero');
    return (hero && hero.offsetHeight) || innerHeight;
  }

  // Les pages sans hero gardent la poussière en suspension, mais pas la
  // construction du barrage : elles ont déjà leur propre pile de bûches.
  var driftOnly = canvas.dataset.mode === 'drift';

  // 0 dispersé au départ · 1 barrage · 2 dispersé de nouveau, ailleurs.
  function phase() {
    if (driftOnly) return 0;
    var h = heroHeight();
    if (scrollY < h * 0.55) return scrollY / (h * 0.55);
    if (scrollY < h * 1.05) return 1;
    return 1 + Math.min(1, (scrollY - h * 1.05) / (h * 0.75));
  }

  function smooth(t) { return t * t * (3 - 2 * t); }

  function accent() {
    return getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() || '#E8862E';
  }

  var color = accent();
  setInterval(function () { color = accent(); }, 1200);

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

    var ph = phase();
    var gathered = smooth(1 - Math.abs(ph - 1));
    var box = stackBox();

    // La transparence d'ensemble vit sur le canvas : des segments
    // semi-transparents qui se chevauchent cumulent leur opacité et la
    // bûche redevient un chapelet.
    canvas.style.opacity = 0.34 + gathered * 0.3;

    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = color;

    for (var i = 0; i < particles.length; i++) {
      var p = particles[i];
      var row = rows[p.row];

      p.ax += p.vx; p.ay += p.vy;
      p.bx += p.vx; p.by += p.vy;
      if (p.ay > 1.02) { p.ay = -0.02; p.ax = Math.random(); }
      if (p.by > 1.02) { p.by = -0.02; p.bx = Math.random(); }

      var loose = ph <= 1
        ? { x: p.ax, y: p.ay }
        : { x: p.bx, y: p.by };

      var span = box.w * row.span;
      var step = span / (PER_ROW - 1);
      var offset = (p.slot - (PER_ROW - 1) / 2) * step;
      var tx = box.cx + row.drift * box.w + offset * Math.cos(row.angle);
      var ty = box.cy - p.row * ROW_GAP + row.lift + offset * Math.sin(row.angle);

      var k = smooth(Math.min(1, gathered));
      var x = loose.x * width + (tx - loose.x * width) * k;
      var y = loose.y * height + (ty - loose.y * height) * k;

      var w = p.r * 2 + k * (step * 1.9 - p.r * 2);
      var h = p.r * 2 + k * (THICKNESS - p.r * 2);

      ctx.globalAlpha = p.o + Math.pow(k, 2.2) * (1 - p.o);
      bar(x, y, w, h, row.angle * k);
    }

    ctx.globalAlpha = 1;
  }

  build();
  resize();
  addEventListener('resize', function () { resize(); });
  requestAnimationFrame(frame);
})();
