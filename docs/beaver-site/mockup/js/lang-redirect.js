/* Beaver — langue du site.
   La racine est en anglais. À l'arrivée, si le navigateur est en français
   (ou si le lecteur a déjà choisi FR), on redirige vers la copie fr/.
   Un clic sur FR/EN mémorise le choix, qui prime ensuite sur la détection.
   Les pages fr/ ne redirigent jamais : on n'arrache pas au lecteur une
   adresse qu'il a ouverte exprès. */

(function () {
  'use strict';

  // Stockage indisponible (navigation privée stricte) : détection sans
  // mémoire, et on ne tente plus d'écrire.
  var storageOff = false;

  function readChoice() {
    try {
      return localStorage.getItem('beaver-lang');
    } catch (e) {
      storageOff = true;
      return null;
    }
  }

  document.addEventListener('click', function (e) {
    var target = e.target && e.target.closest && e.target.closest('.lang-opt');
    if (!target || storageOff) return;
    try {
      localStorage.setItem('beaver-lang', target.getAttribute('hreflang'));
    } catch (err) {
      storageOff = true;
    }
  });

  var path = location.pathname;
  if (/\/fr\//.test(path)) return;

  var choice = readChoice();
  var wantsFrench = choice
    ? choice === 'fr'
    : /^fr(-|$)/i.test(navigator.language || '');
  if (!wantsFrench) return;

  var next;
  if (/\/docs\/[^/]+\.html$/.test(path)) {
    next = path.replace(/\/docs\/([^/]+\.html)$/, '/fr/docs/$1');
  } else if (/\/[^/]*\.html$/.test(path)) {
    next = path.replace(/\/([^/]*\.html)$/, '/fr/$1');
  } else {
    next = path.replace(/\/?$/, '/fr/index.html');
  }
  location.replace(next + location.search + location.hash);
})();
