/* Beaver — démonstration du hero : l'application se remodule d'elle-même.
   C'est la thèse du site, jouée avant d'être écrite. */

(function () {
  'use strict';

  var demo = document.getElementById('demo');
  var label = document.getElementById('demo-label');
  var dots = document.getElementById('demo-dots');
  if (!demo || !label || !dots) return;

  // Les légendes suivent la langue de la page : le même script sert /index.html
  // et /en/index.html.
  var english = document.documentElement.lang === 'en';

  var STATES = english ? [
    { id: '1', text: 'Beaver, as installed' },
    { id: '2', text: 'you add <b>your panel</b>' },
    { id: '3', text: 'you replace <b>whatever you want</b>' }
  ] : [
    { id: '1', text: 'Beaver, à l\'installation' },
    { id: '2', text: 'tu ajoutes <b>ton panneau</b>' },
    { id: '3', text: 'tu remplaces <b>ce que tu veux</b>' }
  ];

  var index = 0;
  var timer = null;
  var CYCLE_MS = 3400;
  var calm = matchMedia('(prefers-reduced-motion: reduce)').matches;

  var buttons = STATES.map(function (_state, i) {
    var button = document.createElement('button');
    button.type = 'button';
    button.setAttribute('aria-label', english
      ? 'Step ' + (i + 1) + ' of ' + STATES.length
      : 'Étape ' + (i + 1) + ' sur ' + STATES.length);
    button.addEventListener('click', function () { show(i); hold(); });
    dots.appendChild(button);
    return button;
  });

  function show(next) {
    index = next % STATES.length;
    var state = STATES[index];
    demo.dataset.state = state.id;
    label.innerHTML = state.text;
    buttons.forEach(function (button, i) {
      button.setAttribute('aria-current', i === index ? 'true' : 'false');
    });
  }

  var firstRun = true;

  function play() {
    stop();
    // Le premier changement arrive tôt : la démonstration ne vaut que si
    // l'interface a déjà bougé quand le regard s'y pose.
    var lead = firstRun ? 900 : CYCLE_MS;
    firstRun = false;
    timer = setTimeout(function () {
      show(index + 1);
      timer = setInterval(function () { show(index + 1); }, CYCLE_MS);
    }, lead);
  }

  function stop() {
    if (!timer) return;
    clearTimeout(timer);
    clearInterval(timer);
    timer = null;
  }

  // Un clic manuel suspend la boucle un moment : sinon la démo reprend la
  // main sur la personne qui vient justement d'en prendre le contrôle.
  function hold() {
    stop();
    setTimeout(play, CYCLE_MS * 2);
  }

  show(0);
  if (calm) return;

  function start() {
    // Rien ne tourne tant que la démo n'est pas à l'écran.
    var seen = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) play();
        else stop();
      });
    }, { threshold: 0.35 });

    seen.observe(demo);
    demo.addEventListener('mouseenter', stop);
    demo.addEventListener('mouseleave', play);
  }

  // Le compte à rebours ne part qu'une fois l'écran de chargement retiré :
  // sinon la première étape se joue derrière lui et il faut attendre le
  // cycle suivant pour voir enfin quelque chose bouger.
  if (document.documentElement.dataset.ready === 'true') start();
  else addEventListener('beaver:ready', start, { once: true });
})();
