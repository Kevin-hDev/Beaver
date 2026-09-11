/* Dessin du barrage : chargement du SVG, recadrage sur la boîte englobante
   réelle du tracé, échantillonnage des zones encrées, teinte à l'accent.

   Le SVG n'occupe que la moitié haute de son viewBox : tout passe par la
   boîte englobante mesurée sur les pixels, jamais par le viewBox.
   Le chemin est résolu depuis l'URL de ce script : les pages vivent à deux
   profondeurs (racine et fr/), un chemin relatif à la page serait faux.
   La lecture des pixels exige un service http (canevas « tainted » en file://). */

(function () {
  'use strict';

  var SAMPLE_W = 1200;      // résolution d'analyse du dessin
  var TARGET_POINTS = 2000; // nombre de cibles tirées dans l'encre
  var INK_MIN = 96;         // alpha au-delà duquel un pixel compte comme encré

  var SRC = new URL('../assets/dam-ink.svg', document.currentScript.src).href;

  var img = new Image();
  var tintCv = document.createElement('canvas');
  var bbox = null;          // boîte englobante, en pixels de l'analyse
  var scale = 1;            // analyse -> pixels intrinsèques de l'image

  var api = {
    ready: false,
    points: [],  // { u, v } normalisés dans la boîte englobante
    ratio: 1,    // largeur / hauteur de la boîte englobante
    canvas: tintCv,
    load: load,
    tint: tint
  };
  window.DamInk = api;

  // done(ok) — un échec n'est pas une erreur : l'appelant se rabat sur
  // l'animation sans barrage.
  function load(done) {
    img.onload = function () {
      api.ready = analyse();
      done(api.ready);
    };
    img.onerror = function () { done(false); };
    img.src = SRC;
  }

  function analyse() {
    var w = SAMPLE_W;
    var h = Math.round(SAMPLE_W * img.naturalHeight / img.naturalWidth);
    scale = img.naturalWidth / w;

    var cv = document.createElement('canvas');
    cv.width = w; cv.height = h;
    var c = cv.getContext('2d', { willReadFrequently: true });
    c.drawImage(img, 0, 0, w, h);
    var data;
    try { data = c.getImageData(0, 0, w, h).data; } catch (e) { return false; }

    var x0 = w, y0 = h, x1 = -1, y1 = -1, ink = 0, x, y;
    for (y = 0; y < h; y++) {
      for (x = 0; x < w; x++) {
        if (data[(y * w + x) * 4 + 3] < INK_MIN) continue;
        ink++;
        if (x < x0) x0 = x;
        if (x > x1) x1 = x;
        if (y < y0) y0 = y;
        if (y > y1) y1 = y;
      }
    }
    if (x1 < 0) return false;

    bbox = { x: x0, y: y0, w: x1 - x0 + 1, h: y1 - y0 + 1 };
    api.ratio = bbox.w / bbox.h;

    // Grille + jitter plutôt que tirage libre : le pur aléatoire fait des
    // paquets et laisse des trous dans les traits fins.
    var step = Math.max(2, Math.sqrt(ink / TARGET_POINTS));
    var pts = [];
    for (y = bbox.y; y < bbox.y + bbox.h; y += step) {
      for (x = bbox.x; x < bbox.x + bbox.w; x += step) {
        var px = Math.min(w - 1, Math.round(x + Math.random() * step));
        var py = Math.min(h - 1, Math.round(y + Math.random() * step));
        if (data[(py * w + px) * 4 + 3] < INK_MIN) continue;
        pts.push({ u: (px - bbox.x) / bbox.w, v: (py - bbox.y) / bbox.h });
      }
    }
    // Mélange puis coupe : garder les premiers reviendrait à garder le haut.
    for (var i = pts.length - 1; i > 0; i--) {
      var j = Math.floor(Math.random() * (i + 1));
      var t = pts[i]; pts[i] = pts[j]; pts[j] = t;
    }
    api.points = pts.slice(0, TARGET_POINTS);
    return api.points.length > 0;
  }

  // Teinte : le dessin est mono-couleur, on le repeint en bloc avec
  // 'source-in' plutôt que de réécrire le fill du SVG et de le recharger.
  function tint(w, h, color) {
    if (!bbox) return;
    w = Math.max(1, Math.round(w));
    h = Math.max(1, Math.round(h));
    tintCv.width = w; tintCv.height = h;
    var c = tintCv.getContext('2d');
    c.clearRect(0, 0, w, h);
    c.drawImage(img, bbox.x * scale, bbox.y * scale, bbox.w * scale, bbox.h * scale, 0, 0, w, h);
    c.globalCompositeOperation = 'source-in';
    c.fillStyle = color;
    c.fillRect(0, 0, w, h);
    c.globalCompositeOperation = 'source-over';
  }
})();
