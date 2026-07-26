# Review différentielle — Forecast Charts V2

Date : 23 juillet 2026

Branche : `feat/forecast-charts-v2`

Base : `main` (`2614ddf9430b410cd0f9f156446679970896eaf5`)

HEAD : `1931ed53677fc3857966ddd8c26a2d1c6c401898`

## Résumé

La branche apporte une nette amélioration visuelle et fonctionnelle des graphiques Forecast :

- graphe principal enrichi ;
- cartes repliables ;
- graphe d'incertitude ;
- saisonnalité interactive ;
- graphe de comparaison ;
- navigation par glissement, molette et raccourcis de zoom ;
- prise en charge des six palettes de thème et des sept langues.

La structure générale est cohérente et les tests existants passent. La branche ne devrait toutefois pas être fusionnée en l'état : deux défauts peuvent présenter des résultats trompeurs, le lint échoue, et plusieurs interactions restent fragiles.

| Niveau | Nombre |
| --- | ---: |
| Critique | 0 |
| Élevé | 3 |
| Moyen | 4 |
| Faible | 1 |

Verdict : **corrections requises avant fusion**.

## Périmètre

Commits examinés :

1. `c1150784` — redesign du graphe principal ;
2. `6f7f3054` — cartes, filtres et graphes complémentaires ;
3. `b35d574e` — tokens des thèmes ;
4. `f6d5ad6f` — défilement et clipping des cartes ;
5. `4a731661` — tableau repliable, fan chart et saisonnalité ;
6. `b9a074cd` — déplacement naturel et barres de zoom ;
7. `a3477188` — dimensions des barres et niveau de zoom ;
8. `71fe3f3c` — hover continu et cas sans effet ;
9. `1931ed53` — pipeline de zoom à la molette.

Différence totale : 62 fichiers, environ 2 707 ajouts et 130 suppressions. Le changement reste limité au frontend Forecast, aux thèmes, aux traductions et aux tests associés.

## Problèmes confirmés

### 1. ÉLEVÉ — Les graphes complémentaires ne suivent pas la série choisie

Le sélecteur de série vit uniquement dans `ForecastView` :

- `src/components/forecast/sections/forecast-view.tsx:30`
- `src/components/forecast/sections/forecast-view.tsx:38-41`
- `src/components/forecast/sections/forecast-view.tsx:68-81`

Le workbench, le fan chart et la saisonnalité utilisent toujours la première série :

- `src/components/forecast/workbench/forecast-workbench-forecast.tsx:57-59`
- `src/components/forecast/charts/forecast-fan-chart.tsx:37-40`
- `src/components/forecast/charts/forecast-seasonality-chart.tsx:58-63`

Conséquence : avec plusieurs séries, l'utilisateur peut sélectionner la série B dans le graphe principal tout en voyant silencieusement les résultats de la série A dans les autres cartes. Les graphiques ne précisent pas cette différence.

Correction recommandée :

- remonter `selectedSeries` dans `ForecastWorkbenchForecast` ;
- transmettre la même série aux trois graphes ;
- afficher son nom dans les cartes si plusieurs séries existent ;
- ajouter un test d'intégration couvrant un changement de série.

### 2. ÉLEVÉ — Le graphe « Fiabilité » affiche en réalité la sMAPE

Le code construit les barres à partir de `metrics.smape` :

- `src/components/forecast/evaluation/forecast-reliability-data.ts:11-30`
- `src/components/forecast/evaluation/forecast-reliability-option.ts:40-43`

L'interface le présente pourtant comme un graphe de fiabilité dans les sept langues, par exemple :

- `src/i18n/fr.json:1249`
- `src/i18n/en.json:1201`

La sMAPE mesure une erreur de prévision ponctuelle. La fiabilité probabiliste correspond plutôt à la calibration ou à la couverture des intervalles. Ces données existent déjà dans `ModelBacktestResult.calibration` :

- `src/components/forecast/evaluation/forecast-evaluation-types.ts:11-23`

Conséquence : l'utilisateur peut croire qu'il évalue la fiabilité des intervalles alors qu'il compare seulement une mesure d'erreur moyenne.

Correction recommandée :

- soit renommer la carte en « sMAPE par modèle » ou « Erreur du backtest » ;
- soit construire un vrai graphe de calibration à partir des couvertures théorique et mesurée.

### 3. ÉLEVÉ — La branche ne passe pas le lint

`npm run lint` retourne trois erreurs :

- affectation de `applyOptionRef.current` pendant le rendu dans
  `src/components/forecast/charts/forecast-seasonality-chart.tsx:72-82` ;
- même défaut dans
  `src/components/forecast/evaluation/forecast-reliability-chart.tsx:50-63` ;
- assertion TypeScript inutile dans
  `src/components/forecast/charts/__tests__/forecast-wheel-zoom-event.test.ts:92`.

Les deux premières erreurs violent le modèle de rendu React : une ref est modifiée pendant le rendu pour contourner les dépendances des effets. Cela peut laisser les effets avec un état incohérent lors de rendus interrompus ou concurrents.

Correction recommandée :

- produire les options avec `useMemo` ;
- appliquer les options dans un effet possédant des dépendances explicites ;
- garder les refs uniquement pour les instances ECharts et les callbacks exécutés hors rendu ;
- supprimer l'assertion inutile du test.

### 4. MOYEN — La molette bloque le défilement même quand aucun zoom n'est possible

Le gestionnaire appelle `preventDefault()` avant de savoir si l'action produira un zoom :

- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:48-53`
- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:66-74`

Le défilement est donc bloqué lorsque :

- `deltaY` vaut zéro ;
- le graphe n'est pas encore initialisé ;
- le zoom est déjà à sa limite et la fenêtre calculée ne change pas.

Le comportement touche les quatre usages de `ForecastChart`, y compris le graphe du panneau latéral. Une personne peut ainsi rester bloquée au-dessus du graphe alors qu'elle cherche simplement à faire défiler le panneau ou l'espace Forecast.

Correction recommandée :

- calculer la prochaine fenêtre avant `preventDefault()` ;
- bloquer le défilement uniquement si la fenêtre va réellement changer ;
- ajouter un test DOM vérifiant qu'un événement sans effet reste disponible pour le conteneur parent.

### 5. MOYEN — Le zoom à la molette est trop agressif avec un trackpad

Chaque événement non nul est transformé en un cran complet, sans tenir compte de l'amplitude de `deltaY` ni de `deltaMode` :

- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:66-70`

Un trackpad macOS émet de nombreux petits événements. Le zoom peut donc atteindre très vite sa limite. Chaque événement déclenche aussi :

1. un `dispatchAction` ECharts ;
2. un événement `datazoom` ;
3. un appel direct supplémentaire à `syncZoomState` ;
4. un rendu du parent ;
5. un `setOption(..., true)` complet car l'effet dépend de l'objet `props`.

Références :

- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:72-74`
- `src/components/forecast/workbench/forecast-workbench-forecast.tsx:33-40`
- `src/components/forecast/charts/forecast-chart.tsx:156-167`

Conséquence : zoom brutal, risque de saccades et de flickering sur les longues séries.

Correction recommandée :

- normaliser ou accumuler les deltas ;
- limiter les mises à jour à une fois par frame ;
- ne conserver qu'un chemin de synchronisation ;
- stabiliser les callbacks et éviter de reconstruire toutes les options pour un simple changement de fenêtre.

### 6. MOYEN — La saisonnalité est incorrecte pour les données infra-mensuelles

La carte est affichée dès que l'historique contient plus de 24 points, quelle que soit la fréquence :

- `src/components/forecast/workbench/forecast-workbench-forecast.tsx:96-104`

Le calcul range ensuite chaque point par mois et conserve uniquement la première valeur rencontrée pour chaque mois :

- `src/components/forecast/charts/forecast-seasonality-data.ts:51-74`

Pour des données quotidiennes, horaires ou hebdomadaires, le graphe ne représente donc ni une moyenne mensuelle, ni une vraie saisonnalité adaptée à la fréquence. Il affiche arbitrairement le premier point de chaque mois.

Deux problèmes supplémentaires sont présents :

- le texte indique « janvier = 100 », alors que le code prend le premier mois disponible ;
- `new Date("YYYY-MM-DD")` est lu en UTC puis converti avec `getFullYear()` et `getMonth()` locaux. En fuseau américain, `2026-01-01` devient localement le 31 décembre 2025.

Références :

- `src/components/forecast/charts/forecast-seasonality-data.ts:47-49`
- `src/components/forecast/charts/forecast-seasonality-data.ts:57-60`
- `src/i18n/fr.json:1256`
- `src/i18n/en.json:1208`

Correction recommandée :

- rendre le calcul dépendant de la fréquence ;
- agréger les valeurs lorsqu'il existe plusieurs observations dans une période ;
- utiliser un parseur de date sans conversion de fuseau ou les accesseurs UTC ;
- aligner le libellé avec la vraie période de référence.

### 7. MOYEN — Les contrôles d'une carte repliée restent accessibles au clavier

Une carte fermée garde tous ses enfants montés :

- `src/components/forecast/charts/forecast-chart-card.tsx:63-83`

Le CSS masque seulement le contenu avec une ligne de grille nulle et de l'opacité :

- `src/components/forecast/charts/forecast-chart-card.css:92-107`

Les boutons, sliders et contrôles de période restent donc atteignables avec la touche Tab alors qu'ils sont invisibles.

Correction recommandée :

- rendre le corps `inert` et `aria-hidden` lorsqu'il est fermé ;
- ou démonter le contenu après la transition ;
- ajouter un test clavier sur une carte repliée.

### 8. FAIBLE — Le tableau replié rend toujours toutes les prédictions

Les lignes sont créées même quand le tableau est fermé :

- `src/components/forecast/sections/forecast-view.tsx:136-162`

Le conteneur possède `overflow: auto`, mais aucune hauteur maximale :

- `src/components/forecast/forecast-view-table.css:1-8`
- `src/components/forecast/forecast-view-table.css:60-75`

Une prévision longue génère donc inutilement tous les éléments DOM lorsque le tableau est replié. Lorsqu'il est ouvert, la carte peut aussi devenir très haute.

Correction recommandée :

- ne rendre les lignes que lorsque le tableau est ouvert ;
- fixer une hauteur maximale ;
- envisager une liste virtualisée pour les horizons les plus longs.

## Points positifs

- Les nouveaux tokens sont enregistrés dans toutes les palettes examinées.
- Les nouveaux textes existent dans les sept langues.
- Les fichiers ajoutés restent correctement répartis par responsabilité.
- Le calcul pur des fenêtres de zoom est séparé de l'intégration React/ECharts et possède des tests dédiés.
- Le clipping des cartes et le scroll du workbench ont été pris en compte.
- Aucun changement backend, permission, secret ou accès disque n'est introduit par cette branche.
- `git diff main...HEAD --check` ne détecte aucune erreur d'espacement.

## Vérifications exécutées

| Vérification | Résultat |
| --- | --- |
| `npm test` | 307 fichiers, 1 431 tests réussis |
| Tests CEF lancés par `npm test` | 2 réussis |
| `npm run build` | Réussi |
| `npm run lint` | Échec : 3 erreurs |
| `git diff main...HEAD --check` | Réussi |
| Contrôle de fuseau `America/New_York` | Confirme le décalage de `2026-01-01` vers le 31 décembre 2025 en heure locale |

Les tests existants valident surtout les fonctions pures et la configuration ECharts. Ils ne couvrent pas :

- la synchronisation entre le sélecteur de série et les graphes complémentaires ;
- le comportement réel du hook de molette ;
- les petits deltas d'un trackpad ;
- la saisonnalité quotidienne, horaire ou hebdomadaire ;
- les fuseaux négatifs ;
- la navigation clavier dans les cartes fermées.

## Reconstruction des flux sensibles

### Sélection de série

Entrées :

- analyse active ;
- liste facultative de séries ;
- série choisie ;
- historique, prédictions, quantiles et scénarios.

Invariant attendu : tous les visuels affichés ensemble doivent représenter la même série.

Invariant actuel : seul le graphe principal respecte la sélection ; les compagnons reprennent l'index zéro.

### Zoom à la molette

Entrées :

- delta et position du pointeur ;
- fenêtre courante ;
- rectangle du graphe ;
- niveau minimal de zoom ;
- état d'initialisation ECharts.

Effets :

- interception du scroll natif ;
- modification de la fenêtre ECharts ;
- synchronisation de l'état React ;
- reconstruction éventuelle des options.

Invariants attendus :

- aucune interception si la fenêtre ne change pas ;
- fenêtre toujours bornée entre 0 et 100 ;
- une seule mise à jour logique par geste ;
- comportement stable entre souris et trackpad.

Les bornes sont correctement protégées par les utilitaires. Les deux autres invariants ne le sont pas.

### Saison­nalité

Entrées :

- dates et valeurs historiques ;
- fréquence ;
- fuseau local ;
- langue ;
- série active.

Invariants attendus :

- toutes les observations pertinentes contribuent au calcul ;
- aucune date ne change de période à cause du fuseau ;
- la période de référence affichée correspond à la période réellement utilisée ;
- la série correspond au reste de l'écran.

Ces quatre invariants peuvent actuellement être violés.

## Ordre de correction recommandé

1. Synchroniser la série active dans tout le workbench.
2. Corriger ou renommer le graphe de fiabilité.
3. Résoudre les trois erreurs de lint.
4. Revoir le calcul de saisonnalité et les dates.
5. Corriger l'interception de la molette puis normaliser les trackpads.
6. Protéger l'accessibilité des cartes fermées.
7. Optimiser le tableau de prédictions.
8. Ajouter les tests d'intégration correspondant à chaque correction.

## Méthode

La review compare intégralement `main...feat/forecast-charts-v2`, reconstitue les flux de données et d'interaction des graphes, inspecte les consommateurs des composants partagés, puis confronte les invariants attendus aux tests et aux commandes de validation du projet.
