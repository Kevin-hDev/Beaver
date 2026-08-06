/* Beaver — ligne d'installation qui se tape seule, et copie en un clic */

(function () {
  'use strict';

  var COMMAND = 'curl -fsSL https://beaver.dev/install.sh | bash';

  var target = document.getElementById('typed');
  var copy = document.getElementById('copy-install');
  var calm = matchMedia('(prefers-reduced-motion: reduce)').matches;

  if (copy) {
    copy.addEventListener('click', function () {
      if (navigator.clipboard) navigator.clipboard.writeText(COMMAND);
      copy.textContent = 'Copié';
      setTimeout(function () { copy.textContent = 'Copier'; }, 1500);
    });
  }

  if (!target) return;

  if (calm) {
    target.textContent = COMMAND;
    return;
  }

  var typed = 0;
  var started = false;

  function type() {
    target.textContent = COMMAND.slice(0, ++typed);
    if (typed < COMMAND.length) setTimeout(type, 32);
  }

  // La frappe démarre quand le bloc entre à l'écran, pas au chargement :
  // sinon elle s'est terminée avant que la personne y arrive.
  var observer = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (!entry.isIntersecting || started) return;
      started = true;
      setTimeout(type, 260);
    });
  }, { threshold: 0.4 });

  observer.observe(target.closest('.term') || target);
})();
