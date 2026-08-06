/* Beaver — graphique de prévision : tracé lissé, zone de forecast, bande de
   confiance. Aligné sur le langage visuel du chart v2 de l'application. */

(function () {
  'use strict';

  var svg = document.getElementById('chart');
  if (!svg) return;

  var HIST = 40;
  var PRED = 12;
  var W = 1000;
  var TOP = 18;
  var BOT = 352;
  var RANGE = BOT - TOP;

  // Série déterministe : le graphe est identique à chaque chargement, donc
  // comparable d'une capture à l'autre.
  var hist = [];
  for (var i = 0; i < HIST; i++) {
    hist.push(50 + Math.sin(i * 0.55) * 9 + Math.sin(i * 0.21) * 6 + Math.sin(i * 1.7) * 2.2);
  }
  var pred = [];
  var last = hist[HIST - 1];
  for (var j = 0; j < PRED; j++) {
    last += 1.5 + Math.sin(j * 0.8) * 1.1;
    pred.push(last);
  }

  var xs = function (i) { return (i / (HIST + PRED - 1)) * W; };
  var ys = function (v) { return BOT - ((v - 20) / 80) * RANGE; };

  // Catmull-Rom converti en Bézier : des courbes lissées, pas des segments.
  function path(values, offset) {
    var pts = values.map(function (v, i) { return [xs(offset + i), ys(v)]; });
    var d = 'M' + pts[0][0].toFixed(1) + ' ' + pts[0][1].toFixed(1);
    for (var i = 0; i < pts.length - 1; i++) {
      var p0 = pts[Math.max(0, i - 1)];
      var p1 = pts[i];
      var p2 = pts[i + 1];
      var p3 = pts[Math.min(pts.length - 1, i + 2)];
      d += 'C' + (p1[0] + (p2[0] - p0[0]) / 6).toFixed(1) + ' ' + (p1[1] + (p2[1] - p0[1]) / 6).toFixed(1) +
           ' ' + (p2[0] - (p3[0] - p1[0]) / 6).toFixed(1) + ' ' + (p2[1] - (p3[1] - p1[1]) / 6).toFixed(1) +
           ' ' + p2[0].toFixed(1) + ' ' + p2[1].toFixed(1);
    }
    return d;
  }

  var grid = document.getElementById('grid');
  for (var g = 0; g <= 4; g++) {
    var y = TOP + (RANGE / 4) * g;
    grid.innerHTML += '<line x1="0" y1="' + y + '" x2="' + W + '" y2="' + y + '" stroke-width="1"/>';
  }

  var histEl = document.getElementById('hist');
  var predEl = document.getElementById('pred');
  var bandEl = document.getElementById('band');
  var splitEl = document.getElementById('split');
  var areaEl = document.getElementById('harea');
  var zoneEl = document.getElementById('fzone');

  histEl.setAttribute('d', path(hist, 0));
  predEl.setAttribute('d', path(pred, HIST - 1));
  splitEl.setAttribute('x1', xs(HIST - 1));
  splitEl.setAttribute('x2', xs(HIST - 1));
  areaEl.setAttribute('d', path(hist, 0) + 'L' + xs(HIST - 1) + ' ' + BOT + 'L0 ' + BOT + 'Z');
  zoneEl.setAttribute('x', xs(HIST - 1));
  zoneEl.setAttribute('width', W - xs(HIST - 1));

  // La bande s'élargit avec l'horizon : plus on prévoit loin, moins on sait.
  var up = 'M' + xs(HIST - 1) + ' ' + ys(pred[0]);
  var down = '';
  pred.forEach(function (v, i) { up += 'L' + xs(HIST - 1 + i) + ' ' + ys(v + 1.5 + i * 0.55); });
  for (var k = PRED - 1; k >= 0; k--) { down += 'L' + xs(HIST - 1 + k) + ' ' + ys(pred[k] - 1.5 - k * 0.55); }
  bandEl.setAttribute('points', (up + down).replace(/[ML]/g, ' ').trim());

  function reveal(el, seconds, delay) {
    var length = el.getTotalLength();
    el.style.strokeDasharray = length;
    el.style.strokeDashoffset = length;
    el.style.transition = 'stroke-dashoffset ' + seconds + 's cubic-bezier(.4,0,.2,1) ' + delay + 's';
    requestAnimationFrame(function () {
      requestAnimationFrame(function () { el.style.strokeDashoffset = 0; });
    });
  }

  // Sur fond clair, en dessous de ~.14 la bande de confiance disparaît —
  // et c'est justement elle qui dit que la prévision n'est pas une certitude.
  function settle() {
    splitEl.style.opacity = 0.8;
    bandEl.style.opacity = 0.16;
    areaEl.style.opacity = 1;
    zoneEl.style.opacity = 0.08;
  }

  if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
    settle();
    return;
  }

  var played = false;
  new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (!entry.isIntersecting || played) return;
      played = true;
      reveal(histEl, 1.5, 0.1);
      reveal(predEl, 0.9, 1.5);
      splitEl.style.transition = 'opacity .5s 1.4s';
      bandEl.style.transition = 'opacity 1.1s 2s';
      areaEl.style.transition = 'opacity 1.1s .8s';
      zoneEl.style.transition = 'opacity 1.1s 1.7s';
      settle();
    });
  }, { threshold: 0.3 }).observe(svg.closest('.chart-card') || svg);
})();
