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

  // Liens vers le reste du site dans l'en-tête : sans eux, une page de doc
  // n'offrait aucun retour visible vers l'accueil (relevé par Kevin le
  // 10 sept. 2026). Injectés ici plutôt que copiés dans chaque page.
  var top = document.querySelector('.docs-top');
  var lang = top && top.querySelector('.lang');
  if (top && lang) {
    var root = article ? '../' : '';
    var siteLinks = document.createElement('nav');
    siteLinks.className = 'dt-links';
    siteLinks.setAttribute('aria-label', 'Navigation du site');
    [
      ['Le harnais', root + 'index.html#harnais', false],
      ['Ce qu’il embarque', root + 'barrage.html', false],
      ['Docs', root + 'docs.html', true]
    ].forEach(function (item) {
      var link = document.createElement('a');
      link.href = item[1];
      link.textContent = item[0];
      if (item[2]) link.setAttribute('aria-current', 'page');
      siteLinks.appendChild(link);
    });
    top.insertBefore(siteLinks, lang);
  }

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
