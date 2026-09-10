# Le site Beaver — ce qu'il faut savoir pour la suite

Écrit le 11 septembre 2026. Ce fichier est le point d'entrée pour quiconque
reprend le site : ce qu'il est, comment il est publié, ce qui fait autorité,
ce qui casse en silence, et ce qui reste à faire.

## Ce que c'est

Un site entièrement statique : des fichiers HTML, CSS et JavaScript servis
tels quels, sans étape de construction, sans base de données, sans serveur à
administrer. Tout vit dans `docs/beaver-site/mockup/`.

- **Vitrine** : `index.html` (le harnais), `barrage.html` (ce qu'il embarque),
  `docs.html` (accueil de la documentation).
- **Documentation** : ~95 pages sous `docs/`.
- **Deux langues** : la racine est l'anglais (langue par défaut), `fr/` en est
  la copie française — mêmes noms de fichiers des deux côtés. Ce choix de slugs
  identiques dans les deux langues est un compromis assumé (décision du
  10 septembre 2026) : il rend la bascule FR/EN triviale sur chaque page.

## Hébergement et publication

- **Hébergeur** : GitHub Pages, gratuit, rien à payer ni à renouveler.
  Adresse : https://kevin-hdev.github.io/Beaver/
- **Publication** : le workflow `.github/workflows/site.yml` se déclenche à
  chaque push sur `main` qui touche `docs/beaver-site/mockup/**`, et publie le
  dossier tel quel. Compter moins d'une minute de workflow, puis quelques
  minutes de propagation.
- **Si le déploiement ne part pas** : c'est déjà arrivé (10 septembre 2026,
  l'événement de push s'était perdu après un incident de synchronisation).
  Le workflow a un déclencheur manuel : `gh workflow run site.yml`, ou l'onglet
  Actions de GitHub → Site → Run workflow.
- **Vérifier après chaque publication** : ouvrir une page modifiée en ligne et
  y chercher le changement. Un `curl` suffit ; ne jamais déclarer publié sans
  avoir vu la page servie.

## Les autorités — un endroit fait foi pour chaque chose

| Quoi | Où | Note |
|---|---|---|
| Navigation de la doc (groupes, libellés, ordre, deux langues) | `mockup/js/docs-nav-data.js` | Autorité unique bilingue ; toute page nouvelle s'y déclare |
| Redirection de langue | `mockup/js/lang-redirect.js` | Navigateur en français → `fr/` ; le choix manuel FR/EN est mémorisé et prime ; les pages `fr/` ne redirigent jamais |
| Direction artistique (couleurs, typos, relief, mascotte) | `direction-artistique.md` (ce dossier) | |
| Contenu : ce que dit chaque page | Les briefs de `docs/documentation-site-web/` | Un brief par page ; le registre `differents-points-a-traiter.md` y consigne anomalies et décisions |
| La vérité technique | Le code de l'application | Le code fait foi ; les briefs datent, `docs/` encore plus |
| Statut du support Linux | `CROSS-PLATFORM.md` (racine du dépôt) | 1.2.3 = dernière version avec nouveautés Linux ; le site le dit sur la page Installation Linux |

## Les règles d'écriture qui ne se devinent pas

- **Des guillemets = une citation d'écran vérifiée** dans `src/i18n/fr.json` /
  `en.json`. Une paraphrase s'écrit sans guillemets. Seule la ligne de code qui
  REND le texte à l'écran fait foi — une clé peut exister et n'être jamais
  affichée (les messages « fantômes » du coffre en sont le précédent).
- **Toute modification se fait dans les deux langues**, même correction, même
  endroit. Une page modifiée d'un seul côté diverge en silence.
- **Encarts** : structure `<div class="callout"><span class="t">Titre</span>
  texte…</div>` — le titre dans le `span.t`, pas de `<b>`.
- Le français **tutoie** le lecteur.
- Pas de jargon : le lecteur cible ne programme pas.

## Ce qui doit suivre l'application quand elle change

Le site cite l'écran réel : chaque évolution de l'app peut rendre une page
fausse. À repasser en revue quand l'app change :

- **Les textes cités entre guillemets** (messages, libellés de réglages, noms
  de thèmes — fr.json/en.json).
- **Les chiffres** : table des fournisseurs cloud (11 aujourd'hui), modèles
  Forecast, limites (tailles de fichiers, bornes), raccourcis.
- **Le CHANGELOG et les versions** citées.
- La page Matériel & VRAM cite la formule VRAM telle que l'app l'affiche —
  si `vramFormula` est corrigée un jour (elle dit « 0.5 GB » en français),
  la citation doit suivre (page + brief, voir le registre).

## Les pièges connus (chacun a déjà mordu)

1. **`.gitignore`** : `docs/` est privé, le site n'est visible de git que par
   des réinclusions explicites (`!/docs/beaver-site/`…). Le 10 septembre 2026,
   les 98 pages `fr/` étaient invisibles de git parce que la réinclusion visait
   encore l'ancien dossier `en/`. Après tout déplacement de dossier, vérifier
   avec `git status` que les fichiers apparaissent bien.
2. **Tester la redirection de langue** : `Emulation.setLocaleOverride` du
   protocole Chrome ne change pas `navigator.language` — il faut relancer le
   navigateur de test avec `--lang=fr-FR` puis `--lang=en-US`.
3. **Captures d'écran** : toujours en densité 2, sinon le verdict de lisibilité
   est faux. Toujours les deux thèmes (le bouton ◐) et les deux langues.
4. **Les workers du dépôt** : des branches/worktrees d'agents externes peuvent
   exister — vérifier qu'on est sur `main` avant de committer.

## Ce qui reste à faire (état au 11 septembre 2026)

- **Passe visuelle finale** sur les deux langues avant de faire circuler
  l'adresse (plan validé de longue date).
- **Image de partage social** : à créer, puis à déclarer — aucune balise
  `og:image` / `twitter:card` n'existe encore dans les `<head>` des pages.
  Sans ces balises, l'image ne servira à rien.
- **Pilule d'en-tête de `barrage.html`** : les liens GitHub/Télécharger ont été
  ajoutés seulement sur `index.html` — alignement à trancher.
- **Sections de confort** non écrites côté briefs (12-reference, 14-projet,
  11-securite approfondie) ; pages Mode Plan et Compression gelées.
- Débrief des anomalies relevées pendant la rédaction : registre
  `docs/documentation-site-web/differents-points-a-traiter.md`.
