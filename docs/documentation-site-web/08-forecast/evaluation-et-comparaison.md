# Évaluer et comparer les modèles

**Emplacement site** — Forecast › Évaluer et comparer
**Répond à** — « Comment savoir si ma prévision vaut quelque chose, et comment comparer plusieurs modèles sur mes propres données ? »
**Sources** — `src-tauri/src/services/forecast/evaluation/` (`runner.rs`, `folds.rs`, `fold_sources.rs`, `baselines.rs`, `baseline_runner.rs`, `model_runner.rs`, `model_request.rs`, `model_observations.rs`, `metrics.rs`, `ranking.rs`, `calibration.rs`, `memory_sampler.rs`, `types.rs`) ; `advanced/ensemble.rs`, `advanced/ensemble_combine.rs` ; `limits.rs` ; `src-tauri/src/commands/forecast_evaluation.rs` ; `src-tauri/src/services/agent_local/` (`tool_definitions_forecast_evaluation.rs`, `tool_definitions_forecast.rs`, `tool_dispatcher_forecast_evaluation.rs`, `tool_dispatcher_forecast_analyze.rs`, `tool_catalog.rs`) ; `src/components/forecast/evaluation/` (`forecast-evaluation-view.tsx`, `forecast-evaluation-table.tsx`, `forecast-evaluation-utils.ts`, `forecast-evaluation-types.ts`, `use-forecast-evaluation.ts`, `forecast-reliability-data.ts`, `forecast-reliability-option.ts`) ; `src/components/forecast/workbench/forecast-workbench-section.tsx` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026. Aucun backtest n'a été lancé dans l'application : tout ce qui suit vient de la lecture du code et de ses tests unitaires.

> **Ce fichier ne décrit pas les modèles eux-mêmes.** La liste du catalogue, leurs capacités et leur installation sont dans `08-forecast/modeles-locaux.md` et `08-forecast/modele-cloud-timegpt.md`.

---

## Plan de page proposé

1. Ce que fait l'évaluation, en une phrase
2. Comment l'historique est découpé
3. Les quatre références de comparaison
4. Les sept mesures calculées, les quatre affichées
5. La couverture : la promesse de la plage de confiance
6. Le classement, et « Meilleur que la baseline »
7. Ce que le backtest change dans la prévision elle-même
8. Quand un modèle ne peut pas être évalué
9. La section Comparaison et l'ensemble pondéré
10. Les limites

---

## Contenu

### 1. Ce que fait l'évaluation, en une phrase

Beaver rejoue la prévision **dans le passé**, sur des périodes dont il connaît déjà le résultat, et compare ce que le modèle aurait prédit à ce qui s'est réellement produit. Le nom affiché est **« Validation temporelle glissante »** (`src/i18n/fr.json`, `forecast.workbench.evaluation.rollingBacktest`).

Le point essentiel : pendant chaque essai, la partie « à prédire » est **effacée des données** avant d'être envoyée au modèle. Le code remplace la colonne cible par une valeur vide sur toute la fenêtre de test (`folds.rs:108-119`). Le modèle ne peut donc pas tricher, même par accident. L'interface le dit en une ligne : « Le backtest compare le modèle actuel à quatre baselines sans utiliser le futur pendant l'entraînement. » (`src/i18n/fr.json`, `forecast.workbench.evaluation.emptyEvaluation`).

L'évaluation se lance depuis la section **Évaluation** de l'espace Forecast, avec le bouton **« Lancer le backtest »** (`forecast-evaluation-view.tsx:55-60`).

### 2. Comment l'historique est découpé

Beaver construit un « plan de backtest » avant de lancer quoi que ce soit (`folds.rs:36-69`).

**L'horizon d'évaluation** est le plus petit de trois nombres (`folds.rs:49-51`) :
- l'horizon de votre prévision,
- **256** pas, plafond absolu (`limits.rs:34`),
- **le tiers du nombre de points** de la série la plus courte.

**Le nombre de fenêtres** est de **3 par défaut**, borné entre 1 et 5 (`folds.rs:52-54`, `limits.rs:33`). L'interface demande toujours 3 (`use-forecast-evaluation.ts:63`). Si l'historique ne permet pas d'en caler trois, Beaver en fait moins : le nombre réel est `(points − 3) ÷ horizon` (`folds.rs:55-56`).

**Les fenêtres se décalent dans le temps.** Chaque fenêtre entraîne sur tout ce qui précède et teste sur les `horizon` points suivants (`folds.rs:93-94`) ; la dernière fenêtre finit sur les derniers points connus. La fenêtre d'entraînement grandit à chaque essai — c'est ce que veut dire « glissante ».

**Le minimum absolu est de 4 points** par série (`folds.rs:41-44`). En dessous, le backtest est refusé.

**Deux avertissements** peuvent s'afficher au-dessus du tableau (`folds.rs:131-139` ; `src/i18n/fr.json`, `forecast.workbench.evaluation.planWarnings`) :

| Code | Déclencheur | Texte affiché |
|---|---|---|
| `short_history` | moins de 3 fenêtres | « Moins de trois fenêtres sont disponibles : interprétez le classement avec prudence. » |
| `reduced_horizon` | horizon rogné par le plafond ou par la longueur de l'historique | « L'horizon d'évaluation a été réduit pour conserver des fenêtres temporelles valides. » |

En résumé, sous le tableau : **« {n} fenêtres · horizon {h} »** (`src/i18n/fr.json`, `forecast.workbench.evaluation.summary`).

### 3. Les quatre références de comparaison

Un chiffre d'erreur seul ne dit rien. Beaver évalue donc systématiquement **quatre méthodes simples** sur exactement les mêmes fenêtres, et les affiche à côté du modèle (`baselines.rs:10`, `runner.rs:23-28`) :

| Nom affiché | Identifiant | Ce qu'elle fait | Minimum de points |
|---|---|---|---|
| **Naive** | `naive` | répète la dernière valeur connue | 1 (`baselines.rs:76-79`) |
| **Naive saisonnier** | `seasonal_naive` | répète le dernier cycle complet | une période entière (`baselines.rs:81-89`) |
| **Drift** | `drift` | prolonge la droite qui joint le premier et le dernier point | 2 (`baselines.rs:91-101`) |
| **ETS** | `ets` | lissage exponentiel avec tendance (Holt), meilleur couple de coefficients parmi 0,2 / 0,5 / 0,8 | 3 (`baselines.rs:103-121`) |

Les noms affichés viennent de `src/i18n/fr.json`, `forecast.workbench.evaluation.baselines`.

**La période saisonnière est déduite de la fréquence**, sans réglage possible (`baselines.rs:60-74`) : seconde 60, minute 60, heure 24, jour 7, jour ouvré 5, semaine 52, mois 12, trimestre 4, année 1. Toute fréquence non reconnue tombe sur 1 — c'est-à-dire pas de saisonnalité.

**Si aucune des quatre baselines n'aboutit, le backtest entier échoue** (`runner.rs:29-31`, message interne « Aucune baseline exploitable pour ce backtest »). C'est un choix cohérent : sans référence, un classement n'a pas de sens.

**Les baselines obtiennent aussi une plage de confiance.** Elle est construite à partir de leurs propres erreurs passées : Beaver rejoue la méthode pas à pas sur l'entraînement, prend la valeur au niveau de confiance demandé dans la liste des écarts absolus, et l'ajoute de part et d'autre de la prédiction (`baseline_runner.rs:70-83`). Les quatre méthodes sont donc jugées sur la même grille que le modèle, intervalles compris.

### 4. Les sept mesures calculées, les quatre affichées

Le code calcule **sept** mesures par modèle et par backtest (`metrics.rs:13-52`, `types.rs:85-95`) :

| Mesure | Ce qu'elle mesure | Formule vérifiée |
|---|---|---|
| **MASE** | erreur rapportée à celle d'une prévision naïve saisonnière ; **1,00 = aussi bon que la méthode naïve**, en dessous c'est mieux | erreur absolue moyenne ÷ échelle (`metrics.rs:35-39`) |
| **sMAPE** | erreur en pourcentage, symétrique, de 0 à 200 | `200 × |erreur| ÷ (|réel| + |prévu|)` (`metrics.rs:156-163`) |
| **MAE** | erreur absolue moyenne, dans l'unité de vos données | `metrics.rs:22` |
| **RMSE** | racine de l'erreur quadratique moyenne ; pénalise les grosses erreurs | `metrics.rs:23-28` |
| **Biais** | erreur moyenne **signée** : positif = le modèle surestime | `metrics.rs:29-33` |
| **Stabilité** | écart-type des erreurs moyennes d'une fenêtre à l'autre ; **plus c'est bas, plus le modèle est régulier** | `metrics.rs:40-41`, `:165-190` |
| **Perte quantile** | qualité de la plage de confiance elle-même (perte pinball moyenne sur les bornes basse, médiane et haute) | `metrics.rs:54-79` |

**L'échelle du MASE** est l'erreur moyenne d'une prévision naïve d'un pas, décalée d'une période saisonnière quand l'historique le permet, sinon d'un seul pas (`metrics.rs:111-129`). Quand cette échelle est nulle — une série parfaitement plate —, elle est ramenée à 1 pour éviter une division par zéro (`metrics.rs:124-128`).

**Le tableau n'en montre que quatre** (`forecast-evaluation-table.tsx:27-33`) : Modèle, **MASE**, **sMAPE**, **MAE**, Couverture, Durée, Résultat. RMSE, biais, stabilité et perte quantile sont calculés, enregistrés dans l'analyse, utilisés pour départager le classement — mais **jamais affichés**. Les valeurs sont arrondies à deux décimales, la durée en millisecondes sous la seconde et en secondes au-delà (`forecast-evaluation-utils.ts:11-24`).

**Un graphe accompagne le tableau** : une barre par modèle évalué, en sMAPE, avec une ligne de moyenne en pointillés. Son titre est **« Erreur du backtest (sMAPE) »** et son axe vertical porte l'étiquette « sMAPE (%) » (`forecast-reliability-data.ts:16-31`, `forecast-reliability-option.ts:40-47` ; `src/i18n/fr.json`, `forecast.chartCard.reliability`). La barre du modèle courant, celles des baselines et celles des autres modèles portent trois couleurs distinctes (`forecast-reliability-option.ts:53-58`).

**Une mesure sort du lot : la mémoire.** Pendant l'évaluation d'un modèle **local**, Beaver échantillonne toutes les 100 millisecondes la mémoire de tout l'arbre de processus du moteur et retient le maximum (`memory_sampler.rs:7`, `:46-85` ; `model_runner.rs:117-122`). Elle n'est mesurée **que pour les modèles locaux** — un modèle distant ne consomme rien chez vous —, elle sert à départager deux modèles à qualité égale (`ranking.rs:49-53`), et elle n'est affichée nulle part.

### 5. La couverture : la promesse de la plage de confiance

À côté des mesures d'erreur, Beaver vérifie une deuxième chose : **la plage de confiance tient-elle sa promesse ?** Une plage à 80 % doit contenir la vraie valeur environ 80 fois sur 100.

Le calcul produit cinq chiffres (`metrics.rs:81-109`, `types.rs:97-104`) : la couverture théorique (le niveau demandé), la couverture mesurée (la part des points réellement tombés dans la plage), la largeur moyenne de la plage, la demi-largeur des résidus au niveau demandé, et le nombre de points ayant servi.

Le tableau affiche les deux premiers, séparés par une barre oblique : **« mesurée / théorique »**, en pourcentage à une décimale (`forecast-evaluation-table.tsx:85-89`, `forecast-evaluation-utils.ts:15-17`). Lire « 62,5% / 80,0% » veut dire : la plage promettait 80 %, elle n'a tenu que 62,5 % — elle est trop étroite.

### 6. Le classement, et « Meilleur que la baseline »

Les résultats sont triés selon **dix critères successifs**, chacun ne servant qu'à départager les égalités du précédent (`ranking.rs:38-55`) :

**MASE**, puis perte quantile, puis écart de couverture, puis sMAPE, puis MAE, puis RMSE, puis valeur absolue du biais, puis stabilité, puis mémoire maximale, puis durée.

Deux remarques sur cet ordre. Il place la **qualité de la plage de confiance en deuxième position**, avant toutes les autres mesures d'erreur : Beaver considère qu'un modèle qui se trompe autant mais annonce mieux son incertitude est meilleur. Et il place la **rapidité en tout dernier** : la vitesse ne fait jamais gagner un modèle moins juste.

Un résultat sans mesures — un modèle qui a échoué — est classé après tous les autres (`ranking.rs:18-20`). Le rang affiché est un simple numéro dans la première colonne (`forecast-evaluation-table.tsx:66`).

**La colonne « Résultat »** dit trois choses (`forecast-evaluation-table.tsx:96-104`) :
- **« Référence »** pour les quatre baselines ;
- **« Meilleur que la baseline »** ou **« Ne bat pas la baseline »** pour un modèle. La comparaison se fait contre **la meilleure des quatre baselines**, pas contre leur moyenne, et selon les mêmes dix critères (`runner.rs:107-130`) ;
- un motif d'échec quand l'évaluation n'a pas abouti (section 8).

### 7. Ce que le backtest change dans la prévision elle-même

C'est l'effet le moins visible et le plus utile : **un backtest recalibre la plage de confiance de votre prévision**.

Quand le modèle de l'analyse a été évalué avec au moins **3 points** de mesure, Beaver reprend la demi-largeur des résidus observés et **élargit** les quantiles bas et haut de la prévision jusqu'à au moins cette valeur (`calibration.rs:4-57`). Les bornes ne sont jamais rétrécies : le code prend le minimum pour la borne basse et le maximum pour la borne haute (`calibration.rs:32`, `:52`).

Autrement dit : si votre modèle s'est trompé de plus que ce que sa plage annonçait, la plage affichée s'élargit pour refléter les erreurs réellement constatées. Le plancher à zéro des grandeurs qui ne peuvent pas être négatives est réappliqué ensuite, et l'ordre des trois quantiles est revalidé (`calibration.rs:55-56`).

**Ce changement est enregistré** : il modifie l'analyse sauvegardée, pas seulement l'affichage (`runner.rs:49`).

### 8. Quand un modèle ne peut pas être évalué

Un modèle peut être refusé **avant** tout calcul, ou échouer en cours de route. Le code range chaque échec dans une **étape** et lui donne un caractère **réessayable ou non** (`types.rs:49-76`). L'interface, elle, regroupe tout cela en **six messages** (`forecast-evaluation-table.tsx:106-115` ; `src/i18n/fr.json`, `forecast.workbench.evaluation.warnings`) :

| Message affiché | Causes réelles |
|---|---|
| **Historique insuffisant** | pas assez de points pour la baseline concernée |
| **Service cloud non configuré** | pas de clé API, ou cloud interdit dans ce contexte |
| **Modèle non installé** | modèle local absent du disque |
| **Ressources insuffisantes** | la machine n'a pas la mémoire demandée par le modèle |
| **Évaluation incomplète** | le moteur n'a pas démarré, une fenêtre a échoué, ou les prédictions renvoyées sont incomplètes |
| **Résultat indisponible** | tout le reste |

Les vérifications faites avant de lancer un modèle (`model_runner.rs:74-116`) :
1. le modèle existe et sait prédire ;
2. il accepte le niveau de confiance de l'analyse ;
3. si vos données contiennent plusieurs séries, il doit savoir les traiter ;
4. si vous utilisez des variables de contexte, il doit accepter le contexte futur ;
5. modèle local : il doit être installé et la machine doit avoir les ressources ;
6. modèle distant : une clé API doit exister, et si le modèle évalué n'est pas celui de l'analyse, l'autorisation du cloud en mode automatique doit être active.

Un point qui compte : **un modèle distant peut être évalué**, ce qui envoie chez le fournisseur autant de requêtes qu'il y a de fenêtres — trois par défaut. Voir `08-forecast/modele-cloud-timegpt.md`.

### 9. La section Comparaison et l'ensemble pondéré

**Évaluation et Comparaison sont le même écran.** Le routeur de l'espace Forecast envoie les deux sections vers le même composant, avec un simple paramètre de mode (`forecast-workbench-section.tsx:47-55`). Deux différences seulement (`forecast-evaluation-view.tsx:55-72`, `:113-118`) :

| | Évaluation | Comparaison |
|---|---|---|
| Bouton | **Lancer le backtest** | **Créer l'ensemble**, et seulement si au moins deux modèles ont réussi |
| Texte quand c'est vide | « Le backtest compare le modèle actuel à quatre baselines… » | « Lancez un backtest dans l'espace Évaluation pour obtenir un classement fiable. » |

**L'ensemble pondéré** combine plusieurs modèles déjà évalués en une seule courbe (`advanced/ensemble.rs`) :

- il faut **au moins 2** modèles backtestés sans échec, et au plus **4** (`ensemble.rs:81-84`, `limits.rs:36`) ;
- **le poids de chaque modèle est l'inverse de son MASE**, normalisé pour que la somme fasse 1 (`ensemble.rs:85-96`) : un modèle deux fois plus juste pèse deux fois plus ;
- les modèles qui ne sont pas celui de l'analyse sont **relancés** sur les mêmes données pour produire leur prévision (`ensemble.rs:26-35`) ;
- les prévisions doivent être **parfaitement alignées** — mêmes dates, mêmes séries, même nombre de points — sinon la création est refusée (`ensemble.rs:99-121`) ;
- prédictions et bornes de confiance sont moyennées avec ces poids, et le résultat est refusé s'il produit un intervalle mal ordonné (`ensemble_combine.rs:11-32`).

**L'ensemble n'est pas validé en tant qu'ensemble.** Le code le dit lui-même : la méthode est `inverse_mase_weighted` et l'état de validation est `members_backtested_ensemble_not_backtested` (`ensemble_combine.rs:35-36`). L'interface le répète : « Ensemble créé à partir de {n} modèles backtestés. Sa combinaison reste à valider séparément. » (`src/i18n/fr.json`, `forecast.workbench.evaluation.ensembleReady`). **Le site doit reprendre cette réserve, pas la gommer** : on sait que chaque membre est bon, on ne sait pas que le mélange l'est.

Sur le graphe principal, la courbe de l'ensemble porte l'étiquette **« Ensemble pondéré »** (`src/i18n/fr.json`, `forecast.view.ensembleSeries`).

**Un backtest efface l'ensemble existant** (`runner.rs:48`). C'est volontaire — les poids viennent de l'évaluation qu'on est en train de remplacer — mais il faut le dire à l'utilisateur : relancer un backtest lui fait perdre son ensemble, et il devra le recréer.

**Deux chemins mènent au backtest, et ils n'offrent pas les mêmes possibilités.**

Depuis **l'interface**, le bouton envoie toujours une liste de modèles **vide** et **3 fenêtres** (`use-forecast-evaluation.ts:61-64`). Une liste vide signifie « le modèle de l'analyse » (`runner.rs:83-87`) : un backtest lancé à la souris évalue donc **un seul modèle** face aux quatre baselines. Et comme chaque backtest remplace l'évaluation précédente (`runner.rs:40-47`), on ne peut pas accumuler deux modèles en lançant deux fois. Le bouton « Créer l'ensemble », qui exige deux modèles réussis, **n'apparaît donc jamais par ce chemin**.

Depuis **l'agent**, en conversation, c'est différent. L'outil `forecast_backtest` accepte jusqu'à **5 identifiants de modèles** et un nombre de fenêtres de 1 à 5 (`tool_definitions_forecast_evaluation.rs:16-27` ; dispatch `tool_dispatcher_forecast_evaluation.rs:32-36`), et l'outil `forecast_analyze` avec l'action `ensemble` crée l'ensemble à partir de deux à quatre modèles évalués avec succès (`tool_definitions_forecast.rs:28` ; `tool_dispatcher_forecast_analyze.rs:180-192`). L'agent dispose aussi de `forecast_compare_models` pour relire le classement enregistré.

**Ces sept outils Forecast sont désactivés par défaut** et doivent être activés dans les réglages de l'agent (`tool_catalog.rs:73-79`, tous déclarés `optional_off`).

Ce que cela veut dire pour le site : **la comparaison de plusieurs modèles et l'ensemble pondéré passent aujourd'hui par la conversation, pas par les boutons de l'espace Forecast.** Voir « Points à confirmer ».

### 10. Les limites

| Limite | Valeur | Source |
|---|---|---|
| Backtests simultanés | **1** pour toute l'application | `limits.rs:37`, `runner.rs:8-16` |
| Modèles par backtest | 5 | `limits.rs:32` |
| Fenêtres | 5 (défaut 3) | `limits.rs:33` |
| Horizon d'évaluation | 256 pas | `limits.rs:34` |
| Lignes de résultats | 9 | `limits.rs:35` |
| Membres d'un ensemble | 2 à 4 | `limits.rs:36`, `ensemble.rs:82` |
| Points minimum par série | 4 | `folds.rs:41-44` |
| Points minimum pour recalibrer | 3 | `calibration.rs:14` |

Un deuxième backtest lancé pendant qu'un premier tourne est **refusé immédiatement**, pas mis en file d'attente (`runner.rs:14-16`).

---

## Encadrés

> **ℹ MASE 1,00, c'est le niveau « je répète la dernière valeur ».**
> C'est le repère le plus utile du tableau. En dessous de 1, le modèle fait mieux qu'une prévision naïve ; au-dessus, il fait moins bien — et un modèle qui fait moins bien qu'une répétition n'a aucun intérêt sur ces données.

> **ℹ Le backtest élargit la plage de confiance de votre prévision.**
> Si le modèle s'est trompé de plus que ce que sa plage annonçait, la plage affichée s'élargit après le backtest pour refléter ses erreurs réelles. Elle n'est jamais rétrécie.

> **⚠ Relancer un backtest supprime l'ensemble pondéré.**
> Les poids sont calculés à partir de l'évaluation en cours ; quand celle-ci est remplacée, l'ensemble l'est aussi. Il faut le recréer.

> **⚠ Un ensemble n'est pas un modèle validé.**
> Chacun de ses membres a été mesuré séparément. Leur mélange, lui, n'a été mesuré par rien. Beaver l'écrit dans le fichier d'analyse et l'affiche à l'écran.

> **ℹ Évaluer un modèle distant envoie vos données autant de fois qu'il y a de fenêtres.**
> Trois requêtes par défaut, une par fenêtre, en plus de la prévision d'origine.

> **ℹ Un seul backtest à la fois, pour toute l'application.**
> Ce n'est pas une file d'attente : le second est refusé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Le backtest refuse de démarrer, message d'historique trop court | moins de 4 points sur la série la plus courte, ou pas assez pour caler une seule fenêtre | allonger l'historique, ou réduire l'horizon de la prévision |
| L'horizon évalué est plus court que celui de la prévision | plafond à 256 pas, ou historique inférieur au triple de l'horizon | c'est normal ; l'avertissement « L'horizon d'évaluation a été réduit… » le signale |
| Moins de trois fenêtres | historique court | interpréter le classement avec prudence, comme le dit l'avertissement |
| « Ressources insuffisantes » sur un modèle installé | la mémoire vive libre ne couvre pas le besoin déclaré, majoré de la marge de sécurité | voir `08-forecast/modeles-locaux.md` |
| « Modèle non installé » alors qu'il apparaît dans la liste | modèle du catalogue jamais téléchargé, ou installation incomplète | l'installer depuis l'écran des modèles |
| « Service cloud non configuré » | pas de clé API Nixtla, ou cloud interdit en mode automatique | voir `08-forecast/modele-cloud-timegpt.md` |
| Le bouton « Créer l'ensemble » n'apparaît jamais | il exige **deux** modèles évalués dans la même évaluation, et l'interface n'en évalue qu'un | passer par l'agent (section 9), ou voir « Points à confirmer », point 1 |
| Un deuxième backtest est refusé | un autre tourne déjà | attendre la fin du premier |
| Toutes les lignes affichent « — » dans les colonnes de mesure | les observations contenaient une valeur non finie, ou des bornes de confiance mal ordonnées | vérifier la qualité des données dans la section Données |

---

## Renvois

- `08-forecast/modeles-locaux.md` — les modèles évaluables sans réseau, leur installation et leurs besoins matériels
- `08-forecast/modele-cloud-timegpt.md` — ce que devient chaque fenêtre de backtest quand le modèle est distant
- `08-forecast/selection-du-modele.md` — comment le mode automatique se sert des résumés de backtest
- `08-forecast/analyse-avancee.md` — les mesures calculées automatiquement à chaque prévision, sans backtest
- `08-forecast/donnees-et-audit.md` — les contrôles de qualité qui conditionnent un backtest exploitable
- `04-agent/fonctionnement.md` — les outils Forecast de l'agent, seul chemin actuel vers un backtest multi-modèles

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **L'ensemble pondéré est inatteignable depuis les boutons de l'interface.** Le bouton « Créer l'ensemble » n'apparaît qu'avec **au moins deux** modèles évalués avec succès (`forecast-evaluation-view.tsx:61`), or l'interface lance toujours le backtest avec une liste de modèles **vide** (`use-forecast-evaluation.ts:63`), ce qui n'évalue que le modèle de l'analyse (`runner.rs:83-87`), et chaque backtest **remplace** l'évaluation précédente (`runner.rs:40-47`). Le chemin qui fonctionne passe par l'agent : `forecast_backtest` avec plusieurs `model_ids`, puis `forecast_analyze` action `ensemble` (section 9). **Deux questions pour l'équipe :** l'interface doit-elle permettre de choisir les modèles à évaluer ? Et si la réponse est non, faut-il retirer un bouton qui ne peut jamais s'afficher ? **C'est le point le plus important de ce fichier** : le site ne doit pas décrire l'ensemble comme accessible depuis la section Comparaison, puisqu'il ne l'est pas.
2. **La limite de 5 modèles par backtest ne concerne que l'agent** (`limits.rs:32`, exposée dans le schéma de l'outil `tool_definitions_forecast_evaluation.rs:18`) : ne pas la présenter comme un réglage de l'interface. Et ces sept outils Forecast sont **désactivés par défaut** (`tool_catalog.rs:73-79`) : une page qui dirait « demandez à l'agent de comparer trois modèles » doit d'abord expliquer comment les activer. À coordonner avec la page de l'agent.
3. **Trois mesures sur sept ne sont visibles nulle part.** RMSE, biais et stabilité sont calculés, enregistrés, et utilisés pour classer — mais absents du tableau (`forecast-evaluation-table.tsx:27-33`) et même du type lu par l'interface pour la perte quantile, qui y figure sans être affichée (`forecast-evaluation-types.ts:1-9`). À décider : les afficher, ou expliquer sur le site qu'elles existent dans le fichier exporté. Le biais est la plus regrettable des trois : c'est la seule qui dit *dans quel sens* le modèle se trompe.
4. **Le détail par fenêtre est enregistré mais jamais lu.** Le backend produit, pour chaque modèle et chaque fenêtre, le nombre de points d'entraînement, de test et l'erreur moyenne (`types.rs:106-112`, `model_observations.rs:73-78`), mais le type de l'interface ne déclare pas ce champ (`forecast-evaluation-types.ts:19-29`). Un commentaire du code affirme d'ailleurs l'inverse — « le backtest ne stocke que des mesures agrégées, sans détail par fenêtre » (`forecast-reliability-data.ts:11-15`) — ce qui est faux côté backend. Commentaire à corriger dans le code ; à ne pas répercuter sur le site.
5. **La mémoire maximale mesurée n'est affichée nulle part**, alors qu'elle est échantillonnée pendant toute l'évaluation et sert à départager le classement (`memory_sampler.rs`, `ranking.rs:49-53`). C'est une information que l'utilisateur d'une machine modeste voudrait voir. À proposer comme évolution.
6. **Le nombre de fenêtres n'est pas réglable depuis l'interface.** Le backend accepte de 1 à 5 et l'agent peut le choisir, mais l'interface envoie toujours 3 (`use-forecast-evaluation.ts:63`). Le site ne doit donc pas le présenter comme un réglage de l'écran.
7. **Le mot « baseline » est laissé en anglais** dans les textes français (`src/i18n/fr.json`, `forecast.workbench.evaluation.baseline`, `beatsBaseline`, `missesBaseline`), tout comme les noms des quatre méthodes (Naive, Drift, ETS). Pour les noms de méthodes c'est l'usage du domaine ; pour le mot « baseline » lui-même, à arbitrer — « référence » est déjà utilisé ailleurs dans le même écran (`forecast.workbench.evaluation.reference`), ce qui fait deux mots pour la même idée.
8. **La colonne « Couverture » est affichée sans explication** dans l'interface. Sur le site, il faudra dire ce que veut dire « 62,5% / 80,0% » ; à l'écran, rien ne le dit.

**Non vérifié — hors de portée d'une lecture du code**

9. **Aucun backtest n'a été lancé.** Les durées réelles, la taille des écarts entre modèles et la lisibilité du graphe avec neuf barres n'ont pas été observées.
10. **La pertinence statistique du classement à trois fenêtres** n'est pas une question de code. Beaver avertit lui-même en dessous de trois fenêtres ; savoir si trois suffisent est un jugement de méthode, à porter par l'équipe.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

11. Le graphe « Erreur du backtest (sMAPE) » avec 5 barres puis avec 9 : rotation des étiquettes à 30° au-delà de 6 barres (`forecast-reliability-option.ts:37`), lisibilité de la ligne de moyenne.
12. Les trois couleurs de barres (modèle courant, baselines, autres modèles) dans les deux thèmes : elles doivent se distinguer autrement que par une nuance voisine.
13. L'état du tableau quand un modèle a échoué : la ligne s'affiche-t-elle avec des tirets et le motif dans la dernière colonne, comme le code le laisse attendre ?
14. La section Comparaison avant tout backtest : vérifier qu'elle affiche bien son texte propre et aucun bouton.
