# Contre-review — Corrections Forecast Charts V2

Date : 23 juillet 2026

Branche : `feat/forecast-charts-v2`

Plage examinée : `1931ed53..90ff2d37`

Commits :

- `e85e10b4` — séries, libellé sMAPE et lint ;
- `5bfcc2e6` — molette, trackpad et limitation des reconstructions ECharts ;
- `d8dd899a` — saisonnalité et fuseaux horaires ;
- `90ff2d37` — accessibilité des cartes et tableau différé.

## Résumé exécutif

Les huit findings de la review initiale sont corrigés sur leurs scénarios principaux. Les tests ajoutés sont pertinents et toutes les validations annoncées sont confirmées.

La contre-review identifie néanmoins une régression moyenne introduite par l'optimisation ECharts : les couleurs des graphes peuvent rester sur l'ancien thème après un changement de thème. Deux écarts faibles restent également présents autour des fréquences trimestrielle/annuelle et d'une taille CSS codée directement.

| Niveau | Nombre |
| --- | ---: |
| Critique | 0 |
| Élevé | 0 |
| Moyen | 1 |
| Faible | 2 |

Risque global : **moyen**.

Verdict : **correction du rafraîchissement des thèmes recommandée avant fusion**. Les deux findings faibles peuvent être traités dans le même passage ou suivis séparément.

## Changements examinés

| Mesure | Valeur |
| --- | ---: |
| Commits | 4 |
| Fichiers modifiés | 27 |
| Lignes ajoutées | 975 |
| Lignes supprimées | 167 |
| Fichiers de test ajoutés | 4 |

Le périmètre reste limité au frontend Forecast, aux traductions et aux tests associés. Aucun changement backend, permission, stockage ou secret n'est présent.

## Vérification des huit findings initiaux

### 1. Séries synchronisées — CORRIGÉ

La sélection est remontée dans le workbench puis transmise au graphe principal, au fan chart et à la saisonnalité :

- `src/components/forecast/workbench/forecast-workbench-forecast.tsx:58-73`
- `src/components/forecast/workbench/forecast-workbench-forecast.tsx:87-121`
- `src/components/forecast/sections/forecast-view.tsx:31-52`
- `src/components/forecast/charts/forecast-fan-chart.tsx:43-50`
- `src/components/forecast/charts/forecast-seasonality-chart.tsx:61-67`

Les titres des deux cartes complémentaires indiquent la série lorsqu'il en existe plusieurs. Un test d'intégration couvre le changement A vers B.

### 2. Faux libellé « Fiabilité » — CORRIGÉ

La carte est maintenant présentée comme une erreur de backtest sMAPE dans les sept langues. Le contenu du graphe correspond donc au nom affiché.

Exemples :

- `src/i18n/fr.json:1249`
- `src/i18n/en.json:1201`

Les noms internes `ForecastReliabilityChart` et `buildReliabilityBars` restent historiques et légèrement trompeurs, mais cela ne produit plus d'erreur visible pour l'utilisateur.

### 3. Lint React — CORRIGÉ

Les refs ne sont plus modifiées pendant le rendu :

- `src/components/forecast/charts/forecast-seasonality-chart.tsx:79-95`
- `src/components/forecast/evaluation/forecast-reliability-chart.tsx:58-72`

Les données dérivées utilisent `useMemo`, les fonctions ECharts utilisent `useCallback`, et les refs sont synchronisées dans des effets. L'assertion TypeScript inutile du test a également été retirée.

Résultat confirmé : lint sans erreur.

### 4. Scroll bloqué aux limites du zoom — CORRIGÉ

Le gestionnaire calcule désormais `wouldChange` avant d'appeler `preventDefault()` :

- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:96-115`

À pleine étendue lors d'un dézoom, ou au zoom minimal lors d'un zoom avant, l'événement reste disponible pour le panneau parent. Le comportement est couvert par un test DOM.

### 5. Zoom trackpad trop agressif — CORRIGÉ

Les deltas sont normalisés, accumulés par seuil puis appliqués au maximum une fois par frame :

- `src/components/forecast/charts/forecast-chart-zoom-utils.ts:27-40`
- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:51-94`
- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:115-129`

Les entrées du graphe sont aussi mémorisées pour éviter un `setOption()` complet à chaque mise à jour du zoom :

- `src/components/forecast/charts/use-forecast-chart-option-input.ts:10-67`
- `src/components/forecast/charts/forecast-chart.tsx:152-164`

Le double chemin `dispatchAction` puis synchronisation reste présent, mais l'écho est dédupliqué par la fenêtre courante. Les tests confirment les limites, les petits deltas et `deltaMode`.

### 6. Saison­nalité infra-mensuelle et fuseaux — CORRIGÉ

Toutes les observations infra-mensuelles participent maintenant à une moyenne mensuelle :

- `src/components/forecast/charts/forecast-seasonality-data.ts:73-105`

Les dates `YYYY-MM-DD` sont lues directement depuis leur chaîne, sans conversion UTC vers l'heure locale :

- `src/components/forecast/charts/forecast-seasonality-data.ts:27-42`

Le libellé indique honnêtement « premier mois = 100 ». Les tests passent également avec le processus lancé sous le fuseau `Pacific/Honolulu`.

### 7. Tabulation dans les cartes repliées — CORRIGÉ

Le corps fermé reçoit `inert` et `aria-hidden` :

- `src/components/forecast/charts/forecast-chart-card.tsx:81-90`

Les éléments restent montés pour l'animation, mais quittent l'ordre de tabulation et l'arbre d'accessibilité.

### 8. Tableau de prédictions toujours rendu — CORRIGÉ

Les lignes ne sont montées que lorsque le tableau est ouvert :

- `src/components/forecast/sections/forecast-view.tsx:168-192`

Le tableau possède également une hauteur maximale et son propre scroll :

- `src/components/forecast/forecast-view-table.css:1-9`

Un test vérifie le montage et le démontage des lignes.

## Nouveau finding

### MOYEN — Les graphes ne rafraîchissent plus leur palette lors d'un changement de thème

Fichier principal :

- `src/components/forecast/charts/forecast-chart.tsx:152-164`

Commit d'origine : `5bfcc2e6`

Couverture de test : absente.

Le commit mémorise volontairement les entrées utilisées pour construire les options ECharts. L'effet ne se relance maintenant que lorsque `optionInput` change.

Les couleurs ne font cependant pas partie de `optionInput`. Elles sont lues ponctuellement depuis les variables CSS avec `getComputedStyle()` :

- `src/components/forecast/charts/forecast-chart-palette.ts:3-26`

Un changement de thème modifie uniquement `data-theme` et `data-palette` sur le document :

- `src/hooks/use-theme.ts:34-60`

Comme ECharts dessine ces couleurs dans un canvas, les anciennes couleurs restent inscrites jusqu'à ce qu'une donnée du graphe change, que le graphe soit remonté, ou qu'un resize provoque une nouvelle application des options.

Le même principe concerne également les graphes de saisonnalité et de backtest, dont les effets ne dépendent pas du thème :

- `src/components/forecast/charts/forecast-seasonality-chart.tsx:79-95`
- `src/components/forecast/evaluation/forecast-reliability-chart.tsx:58-72`

Impact :

- mélange visible entre le nouveau thème de l'application et l'ancienne palette du canvas ;
- comportement particulièrement visible lors d'un changement automatique du thème système ;
- les quatre consommateurs de `ForecastChart` sont concernés.

Correction recommandée :

- exposer une révision de thème ou écouter les changements de `data-theme` et `data-palette` ;
- réappliquer les options uniquement lors d'un vrai changement de thème ;
- ne pas revenir à une reconstruction sur chaque rendu React ;
- ajouter un test vérifiant qu'une palette différente déclenche un nouveau `setOption()`.

## Findings faibles

### FAIBLE — La saisonnalité reste structurée comme une série mensuelle pour Q et Y

Références :

- `src/components/forecast/charts/forecast-seasonality-data.ts:18-25`
- `src/components/forecast/charts/forecast-seasonality-data.ts:92-110`
- `src/components/forecast/workbench/forecast-workbench-forecast.tsx:113-123`

`COMPLETE_YEAR_POINTS` reste fixé à 12 :

- une année trimestrielle de quatre points n'est jamais considérée complète ;
- une fréquence annuelle ne peut pas atteindre les trois mois requis par année ;
- la carte peut être affichée après 24 observations annuelles tout en restant vide.

Les fréquences journalière, horaire, hebdomadaire et mensuelle sont correctement gérées pour le finding initial. Les cas Q/Y n'ont pas de test.

Recommandation :

- utiliser 12 périodes attendues pour M et 4 pour Q ;
- masquer la carte pour Y, qui ne contient pas de saisonnalité intra-annuelle ;
- tester explicitement Q et Y.

### FAIBLE — La hauteur du tableau est codée directement dans le composant

Référence :

- `src/components/forecast/forecast-view-table.css:5`

`max-height: 340px` corrige bien le débordement, mais contourne la règle du projet qui centralise les tailles dans `src/styles/tokens.css`.

Recommandation : ajouter un token de layout Forecast puis l'utiliser dans le CSS du tableau.

## Observation non bloquante sur la molette

L'accumulateur des petits deltas reste actif pendant toute la durée de vie du hook :

- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:51-55`
- `src/components/forecast/charts/use-forecast-wheel-zoom.ts:115-129`

Il n'existe pas de délai séparant deux gestes. Deux petits mouvements éloignés dans le temps mais allant dans la même direction peuvent donc se cumuler. L'impact est faible ; un reset après une courte période d'inactivité rendrait la notion de « geste » plus exacte.

## Validation exécutée

| Commande | Résultat |
| --- | --- |
| `npx tsc --noEmit` | Réussi |
| `npm run lint` | Réussi |
| `npm test` | 311 fichiers, 1 445 tests réussis |
| Tests CEF | 2 réussis |
| `npm run build` | Réussi |
| 5 suites ciblées sous `TZ=Pacific/Honolulu` | 24 tests réussis |
| `git diff 1931ed53..HEAD --check` | Réussi |

Le build conserve les avertissements généraux déjà connus sur la taille de certains bundles et l'externalisation navigateur du module `crypto`. Aucun de ces avertissements ne vient des quatre corrections.

## Couverture et rayon d'impact

| Zone | Rayon d'impact | Couverture |
| --- | --- | --- |
| Série active | 3 graphes | Test d'intégration |
| Pipeline molette | 4 usages de `ForecastChart` | Tests DOM et fonctions pures |
| Saison­nalité | 1 carte, toutes les fréquences | D/M et fuseaux testés ; Q/Y absents |
| Carte repliable | 4 cartes | Test des attributs d'accessibilité |
| Tableau différé | Graphe principal | Test de montage/démontage |
| Changement de thème | Tous les canvas Forecast | Aucun test |

## Contexte des flux sensibles

### Synchronisation de série

Entrées : analyse, liste de séries, sélection utilisateur et données chargées par chaque graphe.

Invariants vérifiés :

- une série invalide retombe sur la première série ;
- la même série est transmise aux trois graphes ;
- les titres complémentaires reflètent la série active ;
- le changement de série reconstruit les données filtrées.

### Pipeline molette

Entrées : delta, mode du delta, position du pointeur, fenêtre courante et limites.

Invariants vérifiés :

- la fenêtre reste entre 0 et 100 ;
- le zoom ne descend pas sous l'étendue minimale ;
- un événement sans effet à une limite n'est pas capturé ;
- les petits deltas n'entraînent plus une mise à jour par événement ;
- l'écho ECharts ne produit pas une seconde mise à jour logique du parent.

### Palette ECharts

Entrées : variables CSS, attributs du thème et données du graphe.

Invariant manquant : une modification du thème doit réappliquer les couleurs au canvas même si les données Forecast n'ont pas changé.

## Recommandations

### Avant fusion

- corriger le rafraîchissement des palettes ECharts lors d'un changement de thème ;
- ajouter un test de changement de thème sans changement de données.

### Faible priorité

- adapter les règles de complétude aux fréquences Q/Y ;
- remplacer `340px` par un token de layout ;
- envisager un reset temporel de l'accumulateur trackpad ;
- renommer ultérieurement les symboles internes « reliability » vers « backtest error ».

## Méthode

Stratégie : review ciblée et différentielle des quatre commits de correction.

Les 27 fichiers modifiés ont été triés. Les onze fichiers de production ont été relus avec leurs appels directs, leurs tests, les fréquences backend acceptées et le cycle de thème. Les quatre commits ont été comparés individuellement à `1931ed53`. Les validations frontend complètes et des tests de fuseau ciblés ont ensuite été exécutés.

Confiance : **élevée** sur les huit corrections et sur le finding de thème ; **moyenne** sur le ressenti exact du trackpad, qui dépend du matériel et du système.
