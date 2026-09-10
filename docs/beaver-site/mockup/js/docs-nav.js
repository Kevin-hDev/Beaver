/* Beaver — construit le sommaire latéral et le « Sur cette page » de chaque
   page de documentation depuis docs-nav-data.js, l'autorité unique. */

(function () {
  'use strict';

  var host = document.getElementById('docs-nav');
  if (!host || typeof BEAVER_DOCS_NAV === 'undefined') return;

  // Les articles vivent dans docs/, l'accueil (docs.html) à la racine :
  // le préfixe des liens dépend d'où la page courante est servie.
  var article = location.pathname.match(/\/docs\/([a-z0-9-]+)\.html$/);
  var current = article ? article[1] : null;
  var prefix = article ? '' : 'docs/';

  BEAVER_DOCS_NAV.forEach(function (section) {
    var group = document.createElement('div');
    group.className = 'group';

    var title = document.createElement('h2');
    title.textContent = section.group;
    group.appendChild(title);

    section.pages.forEach(function (page) {
      var link = document.createElement('a');
      link.href = prefix + page.slug + '.html';
      link.textContent = page.label;
      if (page.slug === current) link.setAttribute('aria-current', 'page');
      group.appendChild(link);
    });

    host.appendChild(group);
  });

  var toc = document.getElementById('docs-toc');
  if (!toc) return;

  var headings = document.querySelectorAll('.doc h2[id]');
  if (headings.length) {
    var tocTitle = document.createElement('h2');
    tocTitle.textContent = 'Sur cette page';
    toc.appendChild(tocTitle);

    headings.forEach(function (heading) {
      var link = document.createElement('a');
      link.href = '#' + heading.id;
      link.textContent = heading.textContent;
      toc.appendChild(link);
    });
  }

  var mascot = document.createElement('div');
  mascot.className = 'mascot';
  mascot.innerHTML = '<div class="beaver-book" aria-hidden="true"></div>' +
    '<span>beaver — reading</span>';
  toc.appendChild(mascot);
})();
