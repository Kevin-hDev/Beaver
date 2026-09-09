/* Beaver — ligne d'installation qui se tape seule, choix du système, copie en un clic */

(function () {
  'use strict';

  var COMMANDS = {
    unix: {
      prompt: '$',
      text: 'curl -fsSL https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.sh | bash',
    },
    windows: {
      prompt: '>',
      text: 'irm https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.ps1 | iex',
    },
  };

  var target = document.getElementById('typed');
  var copy = document.getElementById('copy-install');
  var promptChar = document.getElementById('prompt-char');
  var tabs = document.querySelectorAll('.os-tab');
  var calm = matchMedia('(prefers-reduced-motion: reduce)').matches;

  var os = 'unix';
  var timer = null;
  var started = false;

  function current() { return COMMANDS[os]; }

  if (copy) {
    copy.addEventListener('click', function () {
      if (navigator.clipboard) navigator.clipboard.writeText(current().text);
      copy.textContent = 'Copié';
      setTimeout(function () { copy.textContent = 'Copier'; }, 1500);
    });
  }

  function type(i) {
    var text = current().text;
    target.textContent = text.slice(0, i + 1);
    if (i + 1 < text.length) timer = setTimeout(function () { type(i + 1); }, 32);
  }

  function show() {
    if (timer) clearTimeout(timer);
    if (calm) { target.textContent = current().text; return; }
    type(0);
  }

  tabs.forEach(function (tab) {
    tab.addEventListener('click', function () {
      if (tab.dataset.os === os) return;
      os = tab.dataset.os;
      tabs.forEach(function (t) {
        var active = t.dataset.os === os;
        t.classList.toggle('is-active', active);
        t.setAttribute('aria-selected', active ? 'true' : 'false');
      });
      if (promptChar) promptChar.textContent = current().prompt;
      if (target) { started = true; show(); }
    });
  });

  if (!target) return;

  if (calm) {
    target.textContent = current().text;
    return;
  }

  // La frappe démarre quand le bloc entre à l'écran, pas au chargement :
  // sinon elle s'est terminée avant que la personne y arrive.
  var observer = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (!entry.isIntersecting || started) return;
      started = true;
      timer = setTimeout(function () { type(0); }, 260);
    });
  }, { threshold: 0.4 });

  observer.observe(target.closest('.term') || target);
})();
