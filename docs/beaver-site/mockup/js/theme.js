/* Beaver — thème clair/sombre et écran de chargement */

(function () {
  'use strict';

  var root = document.documentElement;
  var query = new URLSearchParams(location.search);
  var forced = query.get('theme');
  var saved = localStorage.getItem('beaver-theme');

  if (forced) root.dataset.theme = forced;
  else if (saved) root.dataset.theme = saved;
  else if (matchMedia('(prefers-color-scheme: light)').matches) root.dataset.theme = 'light';

  var toggle = document.getElementById('theme-toggle');
  if (toggle) {
    toggle.addEventListener('click', function () {
      var next = root.dataset.theme === 'light' ? '' : 'light';
      root.dataset.theme = next;
      localStorage.setItem('beaver-theme', next);
    });
  }

  // Les animations qui doivent être vues attendent ce signal : lancées sous
  // l'écran de chargement, leur première étape se joue dans le vide et la
  // personne attend le cycle suivant.
  function ready() {
    if (root.dataset.ready === 'true') return;
    root.dataset.ready = 'true';
    dispatchEvent(new CustomEvent('beaver:ready'));
  }

  var loader = document.getElementById('loader');
  if (!loader) { ready(); return; }

  function dismiss() {
    loader.classList.add('done');
    ready();
    setTimeout(function () { loader.remove(); }, 900);
  }

  var calm = matchMedia('(prefers-reduced-motion: reduce)').matches;
  var alreadySeen = sessionStorage.getItem('beaver-loader') === 'seen';

  // Rejoué à chaque page, l'écran de chargement devient un péage.
  // Une fois par visite suffit à poser l'ambiance.
  if (calm || alreadySeen || query.get('skip') === 'loader') {
    loader.remove();
    ready();
    return;
  }

  sessionStorage.setItem('beaver-loader', 'seen');

  var counter = loader.querySelector('.count');
  var started = performance.now();
  var CEILING_MS = 900;

  // Le compteur suit le temps réel plutôt qu'une progression inventée :
  // ce qu'il affiche correspond à ce que la personne attend vraiment.
  function step(now) {
    var progress = Math.min(1, (now - started) / CEILING_MS);
    counter.textContent = Math.round(progress * 100);
    if (progress < 1) requestAnimationFrame(step);
    else setTimeout(dismiss, 160);
  }

  requestAnimationFrame(step);

  // Filet de sécurité : le contenu ne reste jamais masqué, quoi qu'il arrive.
  setTimeout(dismiss, CEILING_MS + 700);
})();
