# L'analyse avancée

**Emplacement site** — Forecast › L'analyse avancée
**Répond à** — « Beaver affiche une décomposition, des anomalies, une dérive et une importance des variables : d'où sortent ces chiffres, sur quoi portent-ils, et jusqu'où puis-je m'y fier ? »
**Sources** — `src-tauri/src/services/forecast/advanced/` (`mod.rs`, `decomposition.rs`, `anomalies.rs`, `drift.rs`, `variables.rs`, `variable_samples.rs`, `variable_scoring.rs`, `sanitize.rs`, `stats.rs`, `types.rs`) ; `provenance.rs` ; `types.rs` ; `evaluation/baselines.rs` (période saisonnière) ; `src-tauri/src/services/agent_local/tool_dispatcher_forecast_output.rs`, `tool_catalog.rs` ; `src/components/forecast/sections/` (`forecast-analysis.tsx`, `forecast-analysis-utils.ts`, `forecast-advanced-analysis-utils.ts`, `forecast-analysis-types.ts`, `forecast-view-data.ts`) ; `src/components/forecast/use-forecast-layer-sources.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026. Aucune prévision n'a été lancée dans l'application : les seuils, les méthodes et les libellés viennent de la lecture du code, pas d'un écran observé.

> **Ce fichier ne parle pas des backtests.** Tout ce qui concerne les mesures d'erreur, les baselines et le classement des modèles est dans `08-forecast/evaluation-et-comparaison.md`. Les deux ne se ressemblent pas : le backtest juge un modèle, l'analyse avancée décrit vos données.

---

## Plan de page proposé

1. Ce que c'est, et quand ça se calcule
2. Le point le plus important : ça parle de votre historique, pas de votre prévision
3. Les sept blocs de la section Analyse
4. La décomposition
5. Les anomalies résiduelles
6. La dérive
7. L'importance des variables
8. Ce que l'analyse avancée ne dit pas
9. Où on la retrouve ailleurs

---

## Contenu

### 1. Ce que c'est, et quand ça se calcule

L'analyse avancée est un ensemble de **quatre calculs** que Beaver fait sur vos données : une décomposition, une détection d'anomalies, une mesure de dérive et une importance des variables (`advanced/mod.rs:17-29`).

**Elle n'a pas de bouton.** Elle est produite automatiquement à la fin de chaque prévision, juste après l'enregistrement de la provenance (`provenance.rs:63`). L'utilisateur n'a rien à lancer, et ne peut pas l'éviter.

Elle est **enregistrée dans le fichier d'analyse** (`types.rs:75`), donc conservée avec la prévision et disponible dans les exports.

**Elle ne coûte pas d'appel au modèle.** Les quatre calculs sont faits en Rust, sur votre machine, à partir des données déjà présentes en mémoire. Une prévision distante ne renvoie aucune de ces informations : le champ arrive vide du client et n'est rempli qu'ensuite, localement (`client_nixtla.rs:74`, `client_local_response.rs:68`, puis `provenance.rs:63`).

### 2. Le point le plus important : ça parle de votre historique, pas de votre prévision

La décomposition et la dérive sont calculées sur `input_data.history` — **les données que vous avez fournies**, pas les valeurs prédites (`decomposition.rs:10`, `drift.rs:9`). Les anomalies en découlent, puisqu'elles portent sur les résidus de la décomposition (`anomalies.rs:13-17`). L'importance des variables est calculée sur les lignes du fichier source (`variable_samples.rs:15`).

C'est contre-intuitif et c'est à dire clairement sur le site : **ces quatre blocs décrivent le passé.** Une anomalie signalée est un point bizarre dans vos données historiques, pas une prévision surprenante. Une dérive détectée dit que la fin de votre historique ne ressemble plus à son début — ce qui est un avertissement sur la fiabilité de la prévision, mais pas une mesure de la prévision elle-même.

À l'inverse, les blocs **Tendance**, **Incertitude** et **Points marquants** de la même section sont calculés côté interface, à partir des valeurs prédites (`forecast-analysis-utils.ts:24-87`). Deux origines, un seul écran.

### 3. Les sept blocs de la section Analyse

La section affiche **sept blocs repliables** (`forecast-analysis.tsx:93-156`), dans cet ordre :

| Bloc | Sous-titre affiché | Origine | Ouvert au départ |
|---|---|---|---|
| **Tendance** | « Lecture générale de la prévision. » | prédictions, calcul dans l'interface | **oui** |
| **Décomposition** | « Tendance, saisonnalité et résidus séparés. » | analyse avancée | non |
| **Incertitude** | « Amplitude de la plage de confiance. » | quantiles, calcul dans l'interface | **oui** |
| **Points marquants** | « Valeurs et mouvements importants. » | prédictions, calcul dans l'interface | **oui** |
| **Anomalies** | « Écarts inhabituels mesurés sur les résidus. » | analyse avancée | non |
| **Dérive** | « Évolution entre l'historique de référence et la période récente. » | analyse avancée | non |
| **Variables** | « Impact vérifié par permutation chronologique. » | analyse avancée | non |

Titres et sous-titres : `src/i18n/fr.json`, `forecast.analysis`. États d'ouverture : `forecast-analysis.tsx:33-41`.

**Les quatre blocs de l'analyse avancée sont donc tous repliés à l'ouverture**, et les trois ouverts sont ceux calculés dans l'interface. À signaler dans « Points à confirmer ».

**Un sélecteur de série** apparaît en haut dès qu'il y a plus d'une série (`forecast-analysis.tsx:80-91`). Décomposition, anomalies et dérive sont calculées **série par série** ; l'importance des variables, elle, est calculée sur toutes les séries confondues — son champ interne vaut `all_series` (`variables.rs:47`).

Pour mémoire, les trois blocs calculés dans l'interface :
- **Tendance** — direction (Hausse / Baisse / Stable, avec un seuil de ±2 %), variation totale en valeur et en pourcentage, valeur de début, valeur de fin (`forecast-analysis-utils.ts:24-40`).
- **Incertitude** — plage moyenne, plage maximale, période où elle est maximale ; la plage maximale est signalée en avertissement si elle dépasse 1,5 fois la moyenne (`forecast-analysis-utils.ts:42-58`).
- **Points marquants** — quatre événements : point le plus haut, point le plus bas, plus forte hausse, plus forte baisse (`forecast-analysis-utils.ts:60-81`).

### 4. La décomposition

Séparer une série en trois couches : une tendance, un motif qui se répète, et ce qui reste.

**Deux méthodes, choisies automatiquement** (`decomposition.rs:39-40`, `:60-64`) :

| Méthode affichée | Choisie quand | Ce qu'elle fait |
|---|---|---|
| **Additive classique** | la fréquence implique une période > 1 **et** l'historique couvre **au moins deux périodes complètes** | moyenne mobile sur une période, puis effet saisonnier moyen par phase, recentré |
| **Moyenne mobile** | tous les autres cas | moyenne mobile sur 5 points, aucune saisonnalité extraite |

Les noms affichés viennent de `src/i18n/fr.json`, `forecast.analysis.methods`.

**La période est déduite de la fréquence**, exactement comme pour les baselines du backtest (`decomposition.rs:38` appelle `evaluation::seasonal_period`) : jour 7, heure 24, mois 12, trimestre 4, semaine 52, seconde et minute 60, jour ouvré 5, année 1. Aucun réglage manuel.

**En dessous de 5 points**, la décomposition n'est pas faite du tout et le bloc affiche « Historique insuffisant pour la décomposition. » (`decomposition.rs:27-36` ; `src/i18n/fr.json`, `forecast.analysis.noDecomposition`).

**Trois valeurs sont affichées** (`forecast-advanced-analysis-utils.ts:51-74`) : la méthode, la **période saisonnière** (ou « Non détectée » quand elle vaut 1), et la **force saisonnière** en pourcentage.

**La force saisonnière** vaut `1 − variance(résidus) ÷ variance(résidus + saison)`, bornée entre 0 et 1 (`decomposition.rs:100-112`). Elle se lit comme « la part de ce qui n'est pas la tendance qui s'explique par le motif saisonnier ». Elle n'est calculée que par la méthode additive ; avec la moyenne mobile, le bloc affiche « Indisponible ».

**Les points décomposés eux-mêmes** — observé, tendance, saisonnier, résidu, date par date — sont enregistrés dans le fichier d'analyse (`advanced/types.rs:33-40`) mais **ne sont affichés nulle part** : l'interface ne montre que les trois valeurs de synthèse. Ils sortent en revanche dans les exports.

### 5. Les anomalies résiduelles

Beaver ne cherche pas les valeurs « hautes » ou « basses » : il cherche les points qui **s'écartent de ce que la décomposition attendait**, c'est-à-dire les gros résidus (`anomalies.rs:7-65`).

**La méthode est robuste**, ce qui a une conséquence pratique : quelques valeurs extrêmes ne faussent pas la détection des autres. Le centre est la **médiane** des résidus et la dispersion leur **écart absolu médian**, converti en équivalent d'écart-type par la division par 0,6745 (`anomalies.rs:33-37`, `stats.rs:35-42`). Le nom interne de la méthode est `seasonal_robust_residual`.

**Le calcul se fait par phase saisonnière** quand la phase compte au moins 4 points et que ses résidus ne sont pas tous identiques ; sinon il retombe sur l'ensemble de la série (`anomalies.rs:28-32`). Concrètement, avec une fréquence journalière, les lundis sont comparés aux lundis.

**Le seuil est de 3,5** (`anomalies.rs:5`, `:42-44`). En dessous, rien n'est signalé. Un point à 3,5 s'écarte de trois fois et demie la dispersion habituelle de sa phase.

**Deux niveaux de gravité** : `high` au-delà de 6, `medium` entre 3,5 et 6 (`anomalies.rs:57`).

**Cent au maximum sont conservées**, les plus fortes d'abord (`anomalies.rs:4`, `:62-63`). L'interface n'en affiche que **huit** par série (`forecast-advanced-analysis-utils.ts:22`), chacune sous la forme « Anomalie résiduelle », la valeur observée, puis « {date} · score {score} » à une décimale (`src/i18n/fr.json`, `forecast.analysis.residualAnomaly` et `residualAnomalyMeta`).

Quand il n'y en a aucune : « Aucune anomalie résiduelle détectée. »

**Les anomalies apparaissent aussi sur le graphe principal**, comme une couche activable nommée **« Anomalies résiduelles »** (`use-forecast-layer-sources.ts:94-98` ; `src/i18n/fr.json`, `forecast.view.filters.residualAnomalies`).

### 6. La dérive

La dérive répond à une question : **la fin de votre historique ressemble-t-elle encore à son début ?** Si non, un modèle entraîné sur l'ensemble part d'un passé qui n'est plus d'actualité.

**Il faut au moins 24 points**, sinon le bloc affiche « Historique insuffisant pour mesurer la dérive. » (`drift.rs:22-23` ; `src/i18n/fr.json`, `forecast.analysis.noDriftData`).

**Deux fenêtres de même taille** sont comparées : les premiers points contre les derniers. Leur taille est le **tiers de l'historique**, ramenée entre 8 et 256 points (`drift.rs:25-30`).

**Quatre écarts sont mesurés** (`drift.rs:31-45`) :

| Écart | Ce qu'il compare |
|---|---|
| **Décalage de moyenne** | la différence des moyennes, rapportée à la dispersion commune |
| **Changement de dispersion** | le logarithme du rapport des variances, en valeur absolue |
| **Changement de pente** | la différence des pentes, ramenée à l'échelle de la fenêtre |
| **Écart de distribution** | la distance de Kolmogorov-Smirnov entre les deux échantillons, de 0 à 1 (`stats.rs:65-91`) |

**Le score de dérive est le plus grand des quatre**, l'écart de distribution comptant double (`drift.rs:46-50`). Il n'y a donc pas de moyenne : un seul indicateur au rouge suffit à déclencher l'alerte.

**Deux seuils** (`drift.rs:62-70`) : dérive **détectée** à partir de 1,0, gravité `high` à partir de 2,0.

**Trois valeurs sont affichées** (`forecast-advanced-analysis-utils.ts:76-100`) : l'**État** — « Dérive détectée » en avertissement ou « Stable » —, le **Score de dérive** à deux décimales, et l'**Écart de distribution** en pourcentage. Le décalage de moyenne, le rapport de variance et le changement de pente sont calculés et enregistrés, mais pas affichés.

### 7. L'importance des variables

Ce bloc n'apparaît avec du contenu que si votre prévision utilise des **variables de contexte** (`variables.rs:6-8`). Sans elles, l'état interne est « sans objet » et l'écran affiche « Importance indisponible ou données insuffisantes. »

**Ce que Beaver mesure exactement.** Pour chaque variable, il construit des paires : d'un côté la **variation de la cible** d'un pas au suivant, de l'autre la **valeur de la variable** à ce pas (`variable_samples.rs:41-50`). Il ajuste une droite entre les deux sur les **70 premiers pour cent** des paires, chronologiquement, et garde les 30 % restants pour vérifier (`variable_samples.rs:54-56`).

**L'importance est ensuite mesurée par permutation.** Beaver compare l'erreur de cette droite sur la partie de vérification à l'erreur obtenue quand les valeurs de la variable sont **décalées circulairement** — trois décalages différents, dont on prend la moyenne (`variable_scoring.rs:42-56`). Si mélanger la variable dégrade nettement la prédiction, c'est qu'elle portait de l'information ; si ça ne change rien, le score est nul (`variable_scoring.rs:57`, plancher à zéro).

Le décalage circulaire, plutôt qu'un mélange aléatoire, préserve l'ordre du temps — d'où le nom interne de la méthode : `chronological_permutation_on_naive_residual`.

**Minimum requis par variable** : 12 paires d'entraînement et 6 de vérification (`variables.rs:14-16`). Une variable qui n'y arrive pas est écartée sans être signalée. Si aucune ne passe, le bloc affiche le message d'indisponibilité.

**Les scores sont normalisés** pour que leur somme fasse 100 %, puis triés du plus fort au plus faible (`variables.rs:33-41`). L'interface affiche au plus **huit** variables, chacune sous la forme « {n} % · {direction} » avec la direction en positive, négative ou neutre selon le signe de la pente (`forecast-advanced-analysis-utils.ts:35-49`, `variable_scoring.rs:87-95`).

**Une réserve importante, à ne pas gommer sur le site.** Cette importance est calculée par un **modèle linéaire interne à Beaver**, pas par le modèle de prévision que vous avez choisi. Elle dit « cette variable est liée aux variations de la cible dans vos données », pas « le modèle s'est servi de cette variable ». Le sous-titre affiché — « Impact vérifié par permutation chronologique » — est exact sur la méthode et muet sur ce point ; le site doit le dire.

**La fiabilité est calculée mais pas affichée.** Le code range le résultat en trois niveaux selon le nombre de points de vérification — élevée à partir de 30, modérée à partir de 12, faible en dessous (`variables.rs:53-61`) — et le résumé transmis à l'agent porte une consigne explicite : l'importance « doit être rapportée avec sa fiabilité » (`tool_dispatcher_forecast_output.rs:159`). L'écran, lui, ne montre rien de tel.

### 8. Ce que l'analyse avancée ne dit pas

**Elle ne juge pas le modèle.** Aucun de ces quatre calculs ne compare la prévision à la réalité : pour ça, il faut un backtest (`08-forecast/evaluation-et-comparaison.md`).

**Elle ne détecte pas d'anomalie dans le futur.** Les anomalies portent sur l'historique fourni.

**Elle est effacée en silence quand un calcul dérape.** Une dernière passe vérifie que toutes les valeurs sont finies ; sinon la décomposition repasse en « données insuffisantes » et ses points sont vidés, les anomalies incriminées sont retirées, la dérive est remise à zéro (`sanitize/apply`, `advanced/sanitize.rs:3-73`). L'utilisateur voit alors un bloc « insuffisant » sans savoir qu'un calcul a échoué plutôt que manqué de données.

**Elle n'est pas rejouée** quand vous modifiez quelque chose sans relancer de prévision : elle est produite une fois, à la fin du calcul (`provenance.rs:63`), avec un horodatage (`advanced/mod.rs:21`).

### 9. Où on la retrouve ailleurs

**Dans les exports.** Les points de décomposition, les anomalies, la dérive et l'importance des variables sortent dans le tableur et dans le rapport PDF (`export/xlsx_advanced.rs`, `export/report_advanced.rs`, `export/advanced_rows.rs`). Voir `08-forecast/exports.md`.

**Dans la section Rapport** de l'espace Forecast, qui affiche la même section Analyse à côté du menu d'export. Voir `08-forecast/scenarios-notes-rapports.md`.

**Auprès de l'agent.** Quand l'agent de Beaver consulte une analyse, il reçoit la décomposition en résumé, les anomalies les plus fortes, l'importance des variables et la dérive, accompagnées d'une phrase d'interprétation qui lui rappelle la nature exacte de chaque chiffre (`tool_dispatcher_forecast_output.rs:139-161`). Le nombre d'anomalies transmises est plafonné, et un indicateur signale la troncature (`:152-156`, `:163-169`). Les outils Forecast de l'agent sont **désactivés par défaut** et s'activent dans ses réglages (`tool_catalog.rs:73-79`).

---

## Encadrés

> **ℹ Ces quatre analyses décrivent vos données, pas votre prévision.**
> Décomposition, anomalies et dérive portent toutes sur l'historique que vous avez fourni. Une anomalie signalée est un point bizarre dans le passé, pas une prévision surprenante.

> **ℹ Aucun bouton : l'analyse est faite à chaque prévision.**
> Elle est calculée sur votre machine, en local, même quand la prévision vient d'un modèle distant, et elle est enregistrée avec l'analyse.

> **⚠ L'importance des variables ne dit pas ce que le modèle a fait.**
> Elle est calculée par un petit modèle linéaire interne à Beaver, indépendamment du modèle de prévision choisi. Elle mesure un lien dans vos données, pas une décision du modèle.

> **ℹ Une anomalie est jugée par rapport à sa saison.**
> Avec une fréquence journalière, un lundi est comparé aux autres lundis, pas à l'ensemble des jours — quand l'historique en contient assez.

> **⚠ Une dérive détectée ne rend pas la prévision fausse.**
> Elle dit que la fin de votre historique ne ressemble plus à son début. C'est une raison de regarder le backtest de plus près, pas un verdict.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Historique insuffisant pour la décomposition. » | moins de 5 points dans la série | allonger l'historique |
| « Historique insuffisant pour mesurer la dérive. » | moins de 24 points | allonger l'historique |
| Période saisonnière « Non détectée » | fréquence non reconnue, ou historique de moins de deux périodes complètes | vérifier la fréquence déclarée ; avec une fréquence journalière il faut au moins 14 points |
| Force saisonnière « Indisponible » | la méthode retenue est la moyenne mobile, qui n'extrait pas de saisonnalité | même cause que la ligne précédente |
| « Importance indisponible ou données insuffisantes. » | aucune variable de contexte, ou moins de 12 + 6 paires exploitables par variable | ajouter des lignes, ou vérifier que la colonne est bien numérique |
| Une variable de contexte n'apparaît pas dans la liste | elle n'a pas atteint le minimum de points, et son absence n'est pas signalée | vérifier les cases vides et les valeurs non numériques de cette colonne |
| « Aucune anomalie résiduelle détectée. » | aucun résidu n'atteint le seuil de 3,5 | c'est un bon résultat, pas une panne |
| Le bloc Anomalies affiche 8 lignes alors qu'il y en a plus | l'affichage est plafonné à 8 par série | les autres — jusqu'à 100 — sont dans les exports |
| Les blocs de l'analyse avancée semblent vides à l'ouverture | ils sont repliés par défaut | les déplier |

---

## Renvois

- `08-forecast/evaluation-et-comparaison.md` — les mesures qui jugent le modèle, et non les données
- `08-forecast/donnees-et-audit.md` — les contrôles de qualité faits à l'import, distincts des anomalies résiduelles
- `08-forecast/scenarios-notes-rapports.md` — la section Rapport, qui reprend cette même section Analyse
- `08-forecast/exports.md` — les points de décomposition et les 100 anomalies, qui n'existent que là
- `04-agent/fonctionnement.md` — ce que l'agent reçoit d'une analyse Forecast

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **La fiabilité de l'importance des variables n'est affichée nulle part.** Elle est calculée en trois niveaux (`variables.rs:53-61`), transmise à l'agent avec la consigne écrite de toujours la mentionner (`tool_dispatcher_forecast_output.rs:159`), et absente de l'écran (`forecast-advanced-analysis-utils.ts:35-49`). Un utilisateur voit donc « Prix · 62 % · négative » sans savoir si ce chiffre repose sur 6 points ou sur 300. **C'est le point le plus important de ce fichier** : à corriger côté produit, ou à expliquer très clairement sur le site.
2. **Le sous-titre « Impact vérifié par permutation chronologique » peut se lire de travers.** Le mot « vérifié » suggère que le modèle de prévision a été mis à l'épreuve, alors que la mesure porte sur un modèle linéaire interne. À reformuler côté produit, et en attendant à expliquer sur le site.
3. **Les quatre blocs de l'analyse avancée sont repliés par défaut**, alors que les trois calculés dans l'interface sont ouverts (`forecast-analysis.tsx:33-41`). Rien dans le code n'explique ce choix. À confirmer : est-ce voulu ? Une dérive détectée mérite peut-être d'être visible sans clic.
4. **Les anomalies affichées sur le graphe portent la source `llm`** (`forecast-view-data.ts:104`) alors qu'elles sont calculées localement en Rust et n'ont rien à voir avec un modèle de langage. C'est probablement un choix de couleur qui détourne un champ de sens. À vérifier et à corriger dans le code ; à ne pas répercuter sur le site.
5. **Un calcul qui échoue est présenté comme un manque de données.** La passe de nettoyage remet le statut à « données insuffisantes » quand une valeur non finie apparaît (`sanitize.rs:12-17`, `:64-73`), ce qui produit exactement le même message à l'écran qu'un historique trop court. Les deux situations demandent pourtant des actions différentes. À signaler comme demande d'évolution.
6. **Trois des quatre indicateurs de dérive ne sont jamais affichés** : décalage de moyenne, rapport de variance, changement de pente (`drift.rs:58-61` contre `forecast-advanced-analysis-utils.ts:83-99`). Or le score affiché est le plus grand des quatre : l'utilisateur voit un score de 2,4 sans savoir lequel des quatre phénomènes l'a produit. À proposer comme évolution.
7. **Les points de décomposition ne sont affichés nulle part** (`advanced/types.rs:33-40`), alors que c'est la seule information qui permettrait de *voir* la décomposition plutôt que d'en lire trois chiffres. À confirmer : est-ce un graphe prévu et non fait, ou un choix ?
8. **Le mot « série » du sélecteur suppose des identifiants lisibles.** Le sélecteur affiche l'identifiant brut de la série (`forecast-analysis.tsx:85`). À vérifier avec un fichier réel dont les séries portent des codes.

**Non vérifié — hors de portée d'une lecture du code**

9. **Aucune prévision n'a été lancée.** La pertinence pratique du seuil d'anomalie à 3,5, du seuil de dérive à 1,0 et de la découpe 70/30 n'a pas été éprouvée sur de vraies données.
10. **Le comportement avec des séries de fréquences irrégulières** — dates manquantes, pas de temps variable — n'a pas été examiné ici. La période saisonnière est déduite d'une étiquette de fréquence, pas des dates réelles.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

11. Les sept blocs dépliés en même temps : longueur de la page, tenue du défilement.
12. Le bloc Anomalies avec ses 8 lignes, et la lisibilité de la mention « {date} · score {score} ».
13. Les deux tons du bloc Dérive — avertissement quand elle est détectée, favorable sinon (`forecast-advanced-analysis-utils.ts:87`) — dans les deux thèmes.
14. La barre d'importance des variables quand une seule variable pèse 100 % et les autres 0 %.
15. Le sélecteur de série avec plus de dix séries.
