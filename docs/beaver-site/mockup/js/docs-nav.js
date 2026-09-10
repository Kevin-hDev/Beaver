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

  // La racine est en anglais, la copie française vit sous fr/ : la langue
  // se lit dans le chemin, et chaque entrée du sommaire porte ses deux
  // libellés (docs-nav-data.js reste l'autorité unique des deux langues).
  var isFrench = /\/fr\//.test(location.pathname);
  function labelOf(entry) {
    return isFrench ? entry.label : (entry.label_en || entry.label);
  }
  function groupOf(section) {
    return isFrench ? section.group : (section.group_en || section.group);
  }

  // Liens vers le reste du site dans l'en-tête : sans eux, une page de doc
  // n'offrait aucun retour visible vers l'accueil (relevé par Kevin le
  // 10 sept. 2026). Injectés ici plutôt que copiés dans chaque page.
  var top = document.querySelector('.docs-top');
  var lang = top && top.querySelector('.lang');
  if (top && lang) {
    var root = article ? '../' : '';
    var siteLinks = document.createElement('nav');
    siteLinks.className = 'dt-links';
    siteLinks.setAttribute('aria-label',
      /\/fr\//.test(location.pathname) ? 'Navigation du site' : 'Site navigation');
    var french = /\/fr\//.test(location.pathname);
    [
      [french ? 'Le harnais' : 'The harness', root + 'index.html#harnais', false],
      [french ? 'Ce qu’il embarque' : 'What it ships with', root + 'barrage.html', false],
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

  /* Accordéon : les 11 groupes sont repliés par défaut, sauf ceux que le
     lecteur a ouverts (mémorisés dans localStorage) et celui de la page
     courante. Le défilement du sommaire est lui aussi restauré à chaque
     changement de page, pour que le lecteur retrouve la colonne exactement
     là où il l'a laissée. */
  var OPEN_KEY = 'beaver-docs-nav-open';
  var SCROLL_KEY = 'beaver-docs-nav-scroll';

  // Navigation privée ou stockage bloqué : le sommaire fonctionne sans
  // mémoire, on cesse simplement d'essayer d'écrire.
  var storageOff = false;

  function readOpenGroups() {
    try {
      var raw = JSON.parse(localStorage.getItem(OPEN_KEY));
      return Array.isArray(raw) ? raw : [];
    } catch (e) {
      storageOff = true;
      return [];
    }
  }
  function writeOpenGroups(names) {
    if (storageOff) return;
    try {
      localStorage.setItem(OPEN_KEY, JSON.stringify(names));
    } catch (e) {
      storageOff = true;
    }
  }

  var openGroups = readOpenGroups();
  var currentGroup = null;
  BEAVER_DOCS_NAV.forEach(function (section) {
    section.pages.forEach(function (page) {
      if (page.slug === current) currentGroup = section.group;
    });
  });
  if (currentGroup && openGroups.indexOf(currentGroup) === -1) {
    openGroups.push(currentGroup);
  }

  function setExpanded(toggle, pages, expanded, animate) {
    toggle.setAttribute('aria-expanded', expanded ? 'true' : 'false');
    if (!animate) {
      pages.style.maxHeight = expanded ? 'none' : '0px';
      return;
    }
    if (expanded) {
      pages.style.maxHeight = pages.scrollHeight + 'px';
      pages.addEventListener('transitionend', function onEnd(e) {
        if (e.propertyName !== 'max-height') return;
        pages.removeEventListener('transitionend', onEnd);
        // Hauteur libérée après l'animation : sinon elle reste figée et
        // coupe le contenu si la liste change de taille ensuite.
        if (toggle.getAttribute('aria-expanded') === 'true') {
          pages.style.maxHeight = 'none';
        }
      });
    } else {
      if (pages.style.maxHeight === 'none' || pages.style.maxHeight === '') {
        pages.style.maxHeight = pages.scrollHeight + 'px';
        void pages.offsetHeight; // fige la hauteur de départ avant d'animer vers 0
      }
      pages.style.maxHeight = '0px';
    }
  }

  BEAVER_DOCS_NAV.forEach(function (section) {
    var group = document.createElement('div');
    group.className = 'group';

    var title = document.createElement('h2');
    title.className = 'group-title';
    var toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'group-toggle';
    toggle.textContent = groupOf(section);
    var pagesId = 'nav-' + section.group.toLowerCase()
      .normalize('NFD').replace(/[^a-z0-9]+/g, '-');
    toggle.setAttribute('aria-controls', pagesId);
    title.appendChild(toggle);
    group.appendChild(title);

    var pages = document.createElement('div');
    pages.className = 'group-pages';
    pages.id = pagesId;
    section.pages.forEach(function (page) {
      var link = document.createElement('a');
      link.href = prefix + page.slug + '.html';
      link.textContent = labelOf(page);
      if (page.slug === current) link.setAttribute('aria-current', 'page');
      pages.appendChild(link);
    });
    group.appendChild(pages);

    setExpanded(toggle, pages, openGroups.indexOf(section.group) !== -1, false);

    toggle.addEventListener('click', function () {
      var expand = toggle.getAttribute('aria-expanded') !== 'true';
      setExpanded(toggle, pages, expand, true);
      var names = readOpenGroups();
      var at = names.indexOf(section.group);
      if (expand && at === -1) names.push(section.group);
      if (!expand && at !== -1) names.splice(at, 1);
      writeOpenGroups(names);
    });

    host.appendChild(group);
  });
  writeOpenGroups(openGroups);

  // Restaure la position de défilement du sommaire, puis la suit : après un
  // clic sur une page, la colonne se raffiche exactement au même endroit.
  function readScroll() {
    if (storageOff) return null;
    try {
      return sessionStorage.getItem(SCROLL_KEY);
    } catch (e) {
      storageOff = true;
      return null;
    }
  }
  function writeScroll(value) {
    if (storageOff) return;
    try {
      sessionStorage.setItem(SCROLL_KEY, value);
    } catch (e) {
      storageOff = true;
    }
  }
  var savedScroll = readScroll();
  if (savedScroll !== null) host.scrollTop = parseInt(savedScroll, 10) || 0;
  var scrollPending = false;
  host.addEventListener('scroll', function () {
    if (scrollPending) return;
    scrollPending = true;
    requestAnimationFrame(function () {
      scrollPending = false;
      writeScroll(String(host.scrollTop));
    });
  }, { passive: true });

  // Les transitions ne s'activent qu'après le premier rendu : l'état initial
  // s'affiche d'un bloc, sans replier les groupes sous les yeux du lecteur.
  requestAnimationFrame(function () {
    requestAnimationFrame(function () { host.classList.add('nav-anim'); });
  });

  var toc = document.getElementById('docs-toc');
  if (!toc) return;

  var headings = document.querySelectorAll('.doc h2[id]');
  if (headings.length) {
    var tocTitle = document.createElement('h2');
    tocTitle.textContent = isFrench ? 'Sur cette page' : 'On this page';
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
