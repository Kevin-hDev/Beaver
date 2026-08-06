/* Beaver — le barrage se construit : une bûche posée par branche atteinte */

(function () {
  'use strict';

  var feats = Array.prototype.slice.call(document.querySelectorAll('.feat'));
  var logs = Array.prototype.slice.call(document.querySelectorAll('#stack .log'));
  if (!feats.length || !logs.length) return;

  var query = new URLSearchParams(location.search);

  // Le contenu doit rester lisible même si rien ne s'exécute : en mouvement
  // réduit, ou en aperçu forcé, tout est posé d'emblée.
  if (matchMedia('(prefers-reduced-motion: reduce)').matches || query.get('state') === 'all') {
    feats.forEach(function (feat) { feat.classList.add('on'); });
    logs.forEach(function (log) { log.classList.add('on'); });
    return;
  }

  var observer = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      var i = feats.indexOf(entry.target);
      if (i < 0 || !logs[i]) return;

      if (entry.isIntersecting) {
        entry.target.classList.add('on');
        logs[i].classList.add('on');
        logs.forEach(function (log, j) { log.classList.toggle('now', j === i); });
      } else if (entry.boundingClientRect.top > 0) {
        // Seulement en remontant : une bûche déjà posée ne se retire pas
        // quand on continue de descendre.
        entry.target.classList.remove('on');
        logs[i].classList.remove('on', 'now');
      }
    });
  }, { threshold: 0.45 });

  feats.forEach(function (feat) { observer.observe(feat); });
})();
