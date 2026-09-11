# Le site Beaver — ce qui reste à faire

Écrit le 11 septembre 2026. Ce fichier est l'autorité unique sur le travail
restant autour du site ; `MAINTENANCE.md` (même dossier) explique comment le
site fonctionne, celui-ci dit ce qui n'est pas fini. Cocher ici, dater, et
noter la décision quand un point se tranche.

## 1 — Avant de faire circuler l'adresse

Le site est en ligne (https://kevin-hdev.github.io/Beaver/) mais l'adresse
n'est annoncée nulle part — c'est voulu : une cartouche unique par communauté,
rien ne se publie tant que la vitrine n'est pas irréprochable.

- [ ] **Passe visuelle finale** sur les deux langues, page par page, deux
  thèmes, haut et bas de page.
- [x] **Graphe de `barrage.html`** — fait le 11 septembre 2026 (5c8f7aab,
  vérifié en ligne) : axes gradués sur les valeurs réelles, pas relatifs avec
  « now »/« maintenant » sur la ligne pointillée, badge « série de
  démonstration » et note « valeur sans unité » (la série est fictive, aucune
  unité inventée — règle « toute représentation porte ses repères »).
- [ ] **Image de partage social** : Kevin la génère (1200 × 630, castor +
  wordmark + « Build your dam. », thème sombre — spécification donnée le
  11 septembre 2026). Puis : la placer dans `mockup/assets/`, et ajouter les
  balises `og:image` / `og:title` / `og:description` / `twitter:card` dans le
  `<head>` des pages (aucune n'existe aujourd'hui — sans elles l'image ne sert
  à rien). Titre et description par page, dans la langue de la page.
  `og:title`/`og:description` peuvent s'écrire avant l'image.
- [x] **Pilule d'en-tête de `barrage.html`** — tranché par Kevin le
  11 septembre 2026 : on aligne. GitHub + Download/Télécharger ajoutés
  (même commit 5c8f7aab, vérifié en ligne).

## 2 — Release 1.2.3 (prévue le 11 septembre 2026)

- [ ] Suivre le process Release de `CLAUDE.md` : bump des 3 fichiers,
  `app-release-notes.json` en 7 langues, `prepare-notes`, tag, release GitHub.
- [ ] Les notes visibles dans l'app doivent mentionner le **statut Linux**
  (1.2.3 = dernière version avec nouveautés Linux — la section CHANGELOG et le
  site le disent déjà, décision du 10 septembre 2026, autorité :
  `CROSS-PLATFORM.md`).

## 3 — Documentation du site : les sections de confort

Le « minimum requis » est atteint (95 pages × 2 langues, dépannage compris).
Restent des sections non bloquantes pour le lancement :

- [ ] `12-reference` et `14-projet` (briefs à écrire, puis pages).
- [ ] `11-securite` approfondie (les 4 briefs bloquants existent déjà).
- [ ] **Mode Plan** : toujours gelé côté site (la page n'existe pas tant que
  le chantier n'est pas stabilisé — règle « visible et fonctionnel, ou
  invisible »). La compression, elle, a été dégelée et publiée le 10 septembre.

## 4 — Débrief des anomalies (session dédiée avec Kevin)

Le registre est `docs/documentation-site-web/differents-points-a-traiter.md`
(~110 anomalies produit consignées pendant la rédaction). En tête de liste :

- Les deux correctifs frontend d'une ligne du diagnostic i18n (peupler
  `ERROR_CODE_KEYS`, repli legacy dans `tool-detail-row.tsx`).
- La plus sérieuse : l'application n'affiche nulle part sa licence AGPL v3
  (obligation de la licence).
- Les messages précis écrasés par des génériques (saisie de clé, panneau
  Forecast, réveils).
- Les deux du 10 septembre : libellé « Quantization » non traduit en français
  (fr.json:679) et formule VRAM française en « 0.5 GB » (fr.json:1086) — si
  corrigés, les citations du site suivent (page Matériel & VRAM + brief).

## 5 — La stratégie de mise en avant (après le site)

L'autorité est `docs/mise-en-avant/strategie.md` (hors dépôt). Résumé du
restant :

- [ ] **Phase 0, restes** : image animée + 3-4 captures dans le README ;
  démo vidéo de 30-60 secondes.
- [ ] **Phase 1** : emplacements passifs (liste des interfaces communautaires
  d'Ollama, listes « awesome », AlternativeTo).
- [ ] **Phase 2** : lancement Reddit (un subreddit à la fois, r/LocalLLaMA
  d'abord, l'angle histoire).
- [ ] **Phase 3** : Show HN puis Product Hunt.
- [ ] **Phase 4** : régularité (post par version notable, réponses rapides aux
  issues, suivre étoiles/téléchargements/visites).

## 6 — Entretien du dépôt (relevé en passant, hors site)

- [ ] `CROSS-PLATFORM.md` : sections anciennes périmées — il cite encore
  `services/ollama_lifecycle.rs` (supprimé le 15 août 2026, remplacé par
  `ollama_manager/`) et d'anciens logs sidecar. La section Linux en tête, elle,
  est à jour (11 septembre 2026).
- [ ] GitHub signale 4 alertes de dépendances sur `main` (2 fortes,
  2 modérées) — à vérifier ; précédent connu : le graphe de dépendances de
  GitHub a déjà produit des alertes à tort en août (code bon, graphe figé).
