# Préparer ses données et lire l'audit

**Emplacement site** — Forecast › Les données
**Répond à** — « À quoi doit ressembler mon fichier, qu'est-ce que Beaver vérifie avant de calculer, et que faire quand il refuse ? »
**Sources** — `src-tauri/src/services/forecast/file_input.rs`, `spreadsheet_mapping.rs`, `limits.rs`, `input_dates.rs`, `input_parse_utils.rs`, `input_series.rs` ; `services/forecast/data_quality/` (`audit.rs`, `audit_helpers.rs`, `profile.rs`, `sequences.rs`, `stats.rs`, `types.rs`) ; `services/forecast/data_profiles.rs`, `data_profiles_load.rs`, `data_profile_migration.rs`, `data_fingerprint.rs` ; `src-tauri/src/commands/forecast.rs` ; `src-tauri/src/services/agent_local/tool_definitions_forecast_audit.rs` ; `src/components/forecast/forecast-data.ts`, `forecast-limits.ts`, `forecast-config.tsx` ; `src/components/forecast/workbench/forecast-workbench-data.tsx`, `forecast-workbench-data-utils.ts` ; `src/components/forecast/sections/forecast-view-data.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, contrôle par contrôle. Aucun écran n'a été observé ; les points d'affichage sont listés en fin de fichier.

---

## Plan de page proposé

1. La forme attendue du fichier
2. Les quatre rôles de colonne
3. Les lignes futures connues
4. Les dates : ce qui est lu, ce qui ne l'est pas
5. La fréquence, et pourquoi elle compte autant
6. Ce que Beaver contrôle avant de calculer
7. Les contrôles qui bloquent
8. Les contrôles qui alertent sans bloquer
9. Lire le profil de données à l'écran
10. La durée de vie d'un profil

---

## Contenu

### 1. La forme attendue du fichier

Un tableau, une ligne par observation, une première ligne d'en-têtes. Six extensions sont acceptées par le sélecteur de fichier : `csv`, `tsv`, `xlsx`, `xls`, `ods`, `xlsm` (`src/components/forecast/forecast-empty.tsx:32`). Le moteur accepte les mêmes, plus rien d'autre : toute autre extension renvoie « Format non supporté » (`services/forecast/file_input.rs:40-54`).

Trois bornes s'appliquent dès la lecture, avant tout contrôle de contenu :

| Ce qui est borné | Valeur | Source |
|---|---|---|
| Taille du fichier ouvert | **50 Mo** | `limits.rs:2`, `file_input.rs:30-32` |
| Lignes lues | **5 000** | `limits.rs:3`, `file_input.rs:41-51` |
| Données une fois converties | **5 Mo** | `limits.rs:1`, `forecast-data.ts:64-66` |
| Colonnes | **256** | `limits.rs:4` |
| Caractères par nom de colonne | **80** | `limits.rs:6` |
| Caractères par cellule | **32 768** | `limits.rs:5` |

Deux commodités, appliquées automatiquement à la lecture (`src/components/forecast/forecast-data.ts:76-113`) :

- une colonne **sans en-tête** reçoit un nom de repli : `column_1`, `column_2`, et ainsi de suite ;
- un nombre écrit **avec une virgule décimale** est reconnu comme un nombre — `1234,5` devient `1234.5`. En revanche, un séparateur de milliers (`1 234,5`, `1.234,5`) fait échouer la conversion et la valeur reste du texte, ce qui provoquera plus loin un refus « Colonne cible non numérique ».

**Deux en-têtes identiques font échouer l'import** (`forecast-data.ts:109`). Renommer l'une des deux colonnes suffit.

### 2. Les quatre rôles de colonne

À la configuration, chaque colonne reçoit un rôle. Deux sont obligatoires, deux sont facultatifs.

| Rôle | Libellé affiché | Obligatoire | À quoi ça sert |
|---|---|---|---|
| Valeur à prévoir | **Cible** | Oui | C'est la colonne prolongée. Elle doit être numérique |
| Horodatage | **Colonne date** | Oui | Fixe l'ordre et le pas de temps |
| Séparateur de séries | **Colonne série** | Non | Permet de prévoir plusieurs séries d'un coup — un produit par magasin, par exemple |
| Variables explicatives | **Covariables** | Non | Colonnes supplémentaires que certains modèles savent exploiter |

Une colonne demandée mais absente du fichier bloque le calcul, avec un message qui nomme le rôle manquant : **« Colonne cible introuvable »**, **« Colonne date introuvable »** ou **« Covariable introuvable »** (`data_quality/types.rs:64-89`).

Sans colonne série, toutes les lignes forment une seule série nommée `series-1` en interne (`data_quality/audit.rs:88`). Avec une colonne série, la limite est de **256 séries** (`limits.rs:9`), et le produit *nombre de séries × horizon* ne doit pas dépasser **100 000 points prévus** (`limits.rs:11`, `:64-75`).

Une covariable dont les valeurs ne sont ni numériques, ni booléennes, ni vides déclenche une alerte `categorical_covariate` — un avertissement, pas un blocage (`audit.rs:73-84`). Au plus **64 covariables** (`limits.rs:7`).

### 3. Les lignes futures connues

C'est le mécanisme le moins évident du fichier d'entrée, et il mérite sa propre section sur le site.

**La règle** : une ligne qui porte une date mais **aucune valeur dans la colonne cible** est comprise comme une ligne future, pas comme une donnée manquante (`audit.rs:113-132`). Elle sert à deux choses : fixer les dates de sortie, et fournir à l'avance les valeurs des covariables sur la période à prévoir — une promotion prévue, une température annoncée, un jour férié connu.

**Trois contraintes**, toutes vérifiées :

1. Les lignes futures viennent **après** l'historique. Une ligne avec valeur qui apparaît après une ligne future déclenche `history_after_future`, bloquant (`audit.rs:115-122`).
2. Quand il y a des lignes futures, **chaque série doit en avoir exactement autant que l'horizon demandé**, sinon `invalid_future_rows`, bloquant (`data_quality/profile.rs:53-60`). L'écran de configuration l'annonce avant le lancement : **« Horizon incohérent — {{future}} lignes futures connues ont été détectées, mais l'horizon actuel est de {{horizon}}. »** (`fr.json`, `forecast.config.messages.futureRowsMismatchBody`).
3. La première ligne future doit suivre la dernière ligne d'historique **d'un seul pas de temps**, sinon `invalid_future_dates`, bloquant (`data_quality/sequences.rs:21-26`).

**Un modèle qui ne sait pas exploiter le futur connu refuse le calcul** plutôt que de l'ignorer : « Variables futures non supportées par ce moteur » (`commands/forecast.rs:110-122`). L'écran de configuration prévient avant : **« Contexte futur non disponible — Ce modèle lit l'historique principal, mais n'exploite pas les valeurs futures connues des covariables. »** (`fr.json`, `forecast.config.messages.futureContextUnsupportedBody`).

Quand aucune covariable n'est sélectionnée mais que des lignes futures existent, l'application le dit aussi : **« Dates futures seulement — Les lignes futures connues serviront surtout à fixer les dates de sortie tant qu'aucune covariable n'est sélectionnée. »**

### 4. Les dates : ce qui est lu, ce qui ne l'est pas

**Les huit formats reconnus**, dans l'ordre d'essai (`services/forecast/input_dates.rs:18-39`) :

| Format | Exemple |
|---|---|
| RFC 3339 | `2026-07-23T14:30:00Z`, `2026-07-23T14:30:00+02:00` |
| `%Y-%m-%d %H:%M:%S` | `2026-07-23 14:30:00` |
| `%Y-%m-%d %H:%M` | `2026-07-23 14:30` |
| `%Y-%m-%dT%H:%M:%S` | `2026-07-23T14:30:00` |
| `%Y-%m-%dT%H:%M` | `2026-07-23T14:30` |
| `%Y/%m/%d %H:%M:%S` | `2026/07/23 14:30:00` |
| `%Y/%m/%d %H:%M` | `2026/07/23 14:30` |
| Date seule | `2026-07-23` ou `2026/07/23` |

**Ce qui n'est pas lu**, et c'est le point à écrire en gras sur le site : **le format jour/mois/année n'est pas reconnu**. `23/07/2026` échoue et déclenche `invalid_date`, bloquant. Il en va de même pour les mois écrits en toutes lettres, les dates au format américain `07/23/2026`, et les numéros de série de tableur non convertis.

Une date lue sans heure est ramenée à minuit (`input_dates.rs:35-38`). Les dates des points prévus reprennent le format de la dernière date d'entrée : avec `T` si l'entrée en avait un, avec heure si l'entrée en avait, sinon en date seule (`input_dates.rs:119-127`).

### 5. La fréquence, et pourquoi elle compte autant

La fréquence n'est pas décorative : elle définit **ce qu'est un pas de temps**, et donc ce qu'est une période manquante, une date mal alignée, ou une ligne future correctement placée. Tous les contrôles de séquence en dépendent (`data_quality/sequences.rs:16-57`).

**Les huit fréquences proposées à l'écran** (`src/components/forecast/forecast-limits.ts:8` ; libellés dans `fr.json`, `forecast.frequency.*`) :

| Libellé affiché | Code | Pas de temps appliqué |
|---|---|---|
| **Minute** | `T` | 1 minute (`input_dates.rs:44`) |
| **Heure** | `H` | 1 heure (`:45`) |
| **Jour** | `D` | 1 jour (`:46`) |
| **Jour ouvré** | `B` | Le jour suivant en sautant samedi et dimanche (`:47`, `:82-92`) |
| **Semaine** | `W` | 7 jours (`:48`) |
| **Mois** | `M` | 1 mois calendaire (`:49`) |
| **Trimestre** | `Q` | 3 mois (`:50`) |
| **Année** | `Y` | 12 mois (`:51`) |

Le moteur reconnaît aussi des formes qui ne sont **pas proposées dans la liste déroulante** : `S` pour la seconde, `MIN` comme synonyme de la minute, `A` comme synonyme de l'année, et des multiples écrits `15MIN`, `2H`, `3D`, `2W` (`input_dates.rs:43`, `:51`, `:94-117`). Elles sont accessibles par l'agent, qui passe la fréquence en texte libre. **Les multiples ne fonctionnent pas pour les mois, trimestres et années** : `2M` n'est pas reconnu (`input_dates.rs:109-115`).

**Le piège à écrire sur le site** : « Jour ouvré » ne saute que les samedis et dimanches. Les jours fériés ne sont pas connus de Beaver et créent des périodes manquantes.

### 6. Ce que Beaver contrôle avant de calculer

L'audit tourne systématiquement, avant tout calcul, et son résultat s'appelle le **profil de données** (`commands/forecast.rs:37` → `data_quality/audit.rs:29-36`). L'agent peut le lancer seul avec l'outil `forecast_data_audit`.

Le profil retient (`data_quality/types.rs:27-54`) : les colonnes retenues pour chaque rôle, la fréquence, l'horizon, le niveau de confiance, le nombre de lignes source, le nombre de points d'historique — au total et par série —, le nombre de lignes futures, le nombre de séries et leurs identifiants, la première et la dernière date, le nombre de périodes manquantes, le nombre de valeurs atypiques, et la liste des anomalies relevées avec leur gravité et jusqu'à **dix exemples** chacune (`limits.rs:21`).

Chaque anomalie porte l'une de **deux gravités** (`types.rs:11-16`) :

- **Erreur** — le calcul est refusé ;
- **Avertissement** — le calcul a lieu, l'alerte est enregistrée avec le résultat.

Au plus **50 anomalies distinctes** sont retenues par profil (`limits.rs:20`). Une anomalie qui se répète est comptée, pas dupliquée (`audit_helpers.rs:15-22`).

**Le refus nomme la première erreur rencontrée, pas toutes** (`types.rs:57-62`). Un fichier qui cumule deux défauts demandera donc deux passages.

### 7. Les contrôles qui bloquent

Voici les refus, regroupés par ce qu'ils détectent. Le message donné est celui produit par le moteur (`data_quality/types.rs:92-108`).

| Ce qui est détecté | Message du moteur | Source |
|---|---|---|
| Une colonne demandée n'existe pas | **Colonne cible introuvable** / **Colonne date introuvable** / **Covariable introuvable** | `types.rs:64-89` ; détection `audit_helpers.rs:91-109` |
| Une date illisible | **Dates invalides** | `audit.rs:99-108` |
| Deux lignes à la même date dans une même série | **Dates dupliquées** | `sequences.rs:36-42` |
| Des dates dans le désordre | **Dates non chronologiques** | `sequences.rs:43-48` |
| Un écart de date qui ne tombe pas sur un multiple du pas | **Fréquence incohérente** | `sequences.rs:54` |
| Une valeur cible non numérique | **Colonne cible non numérique** | `audit.rs:133-138` |
| Une ligne d'historique après une ligne future | **Lignes futures invalides** | `audit.rs:115-122` |
| Une date future mal alignée | **Dates futures invalides** | `sequences.rs:21-26`, `:51`, `:53` |
| Un nombre de lignes futures différent de l'horizon | **Lignes futures invalides** | `profile.rs:53-60` |
| Moins de deux points d'historique dans une série | **Historique insuffisant** | `profile.rs:43-49` |
| Aucune série exploitable | **Historique insuffisant** | `profile.rs:121-128` |
| Plus de 256 séries | **Trop de séries** | `profile.rs:129-136` |
| Séries × horizon au-delà de 100 000 points | **Volume de prédictions trop important** | `profile.rs:137-144` |
| Une ligne qui n'est pas un objet, plus de 256 colonnes, une cellule de plus de 32 768 caractères, un identifiant de série illisible | **Données Forecast invalides** | `audit.rs:46-98` ; message de repli `types.rs:106` |

**Deux points de lecture importants.** D'abord, **« Historique insuffisant » veut dire moins de deux points**, pas « pas assez de recul » : c'est un plancher technique, très bas. Ensuite, les quatre derniers défauts du tableau tombent tous dans le même message générique **« Données Forecast invalides »**, qui ne dit pas lequel s'est produit.

### 8. Les contrôles qui alertent sans bloquer

Six alertes n'empêchent pas le calcul mais accompagnent le résultat.

| Ce qui est détecté | Code interne | Comment c'est mesuré |
|---|---|---|
| Des périodes absentes dans l'historique | `missing_periods` | Chaque écart de plus d'un pas entre deux dates consécutives compte les pas manquants (`sequences.rs:52`) |
| Un historique court par rapport à l'horizon | `short_history` | Moins de **deux fois l'horizon** de points d'historique (`profile.rs:50-52`) |
| Des valeurs atypiques | `possible_outliers` | Méthode de l'écart interquartile, avec un facteur **3** — au moins **8 valeurs** requises, sinon aucune détection (`stats.rs:1-19`) |
| Un changement de régime | `possible_regime_shift` | La moyenne de la seconde moitié s'écarte de la première de plus de **2,5 écarts-types** — au moins **12 valeurs** requises (`stats.rs:21-30`) |
| Des covariables sans valeur sur les lignes futures | `missing_future_covariates` | Compte les cases vides ou absentes sur les lignes futures (`audit_helpers.rs:120-133`) |
| Une covariable textuelle | `categorical_covariate` | Valeur ni numérique, ni booléenne, ni vide (`audit.rs:73-84`) |

**Comment lire le nombre de valeurs atypiques.** Le facteur 3 est volontairement prudent : la règle courante utilise 1,5, ce qui signale beaucoup plus de points. Un compte non nul ici désigne donc des valeurs franchement à part, pas une simple dispersion. **Beaver ne les retire pas et ne les corrige pas** — il les compte et vous laisse décider.

**Une série de moins de 8 valeurs n'aura jamais d'alerte d'atypie**, et une série de moins de 12 valeurs jamais d'alerte de changement de régime. L'absence d'alerte n'est pas une garantie.

### 9. Lire le profil de données à l'écran

Le profil s'affiche dans la section **Données** de la fenêtre **Espace Forecast** (`workbench/forecast-workbench-data.tsx:55-97`).

**Six compteurs** en haut (libellés dans `fr.json`, `forecast.workbench.data`) :

| Libellé affiché | Ce que c'est |
|---|---|
| **Lignes source** | Le nombre de lignes du fichier |
| **Points historiques** | Les lignes qui portent une valeur cible |
| **Lignes futures** | Les lignes datées sans valeur cible |
| **Séries** | Le nombre de séries distinctes |
| **Périodes manquantes** | Les pas de temps absents |
| **Valeurs atypiques** | Le compte de la méthode de l'écart interquartile |

**Quatre rappels de correspondance** ensuite : **Cible**, **Date**, **Fréquence**, **Variables externes** — cette dernière affichant **« Aucune »** quand aucune covariable n'est retenue.

**Un encadré de synthèse** enfin, avec trois états possibles :

| État | Libellé affiché | Quand |
|---|---|---|
| Vert | **Données validées** | Aucune erreur bloquante |
| Orange | **Attention requise** | Au moins une erreur bloquante enregistrée |
| Neutre | **Contrôle indisponible pour cette ancienne analyse** | L'analyse est antérieure au profil de données |

Les anomalies sont listées sous cet encadré, une ligne chacune, sous la forme **« Erreur bloquante · 3 »** ou **« Avertissement · 12 »** (`forecast-workbench-data.tsx:85-95`).

**Un écart à signaler, et c'est le plus gênant de cette page** : cette liste **n'affiche pas de quelle anomalie il s'agit**. Le code interne — `missing_periods`, `possible_outliers`, `short_history` — sert de clé de tri mais n'est ni affiché ni traduit. L'utilisateur voit donc *combien* d'alertes, jamais *lesquelles*. Voir « Points à confirmer ».

Le tableau des données sous ces blocs est **limité à 200 lignes et 32 colonnes**, avec la mention **« L'aperçu est limité aux 200 premières lignes. »** (`workbench/forecast-workbench-data-utils.ts:1-2`, `:27-31` ; `fr.json`, `forecast.workbench.data.previewLimited`).

Dans la vue principale du panneau, les alertes de qualité apparaissent aussi comme une **couche du graphique**, filtrable sous le nom **« Alertes qualité »**, positionnée à la date de fin de l'historique (`sections/forecast-view-data.ts:107-115` ; `fr.json`, `forecast.view.filters.dataQualityIssues`).

### 10. La durée de vie d'un profil

Un profil validé est enregistré et **réutilisable** : il évite de renvoyer les données à chaque prévision. C'est ce que fait l'agent, qui reçoit un identifiant de profil et le repasse à l'outil de calcul (`services/agent_local/tool_definitions_forecast_audit.rs:9`).

| Règle | Valeur | Source |
|---|---|---|
| Un profil invalide n'est **jamais** enregistré | — | `data_profiles.rs:29-30` |
| Profils gardés par espace de travail | **20**, les plus anciens supprimés au-delà | `limits.rs:18`, `data_profiles.rs:48-53` |
| Rangement | Un dossier par projet ou par conversation : `forecast-data-profiles-project-<id>/`, `forecast-data-profiles-session-<id>/` | `data_profiles.rs:126-133` |
| Écriture | Fichier temporaire puis renommage | `data_profiles.rs:54-56` |

Chaque profil porte une **empreinte de ses données** (`data_profiles.rs:90` → `data_fingerprint.rs`). Cette empreinte est ce qui permet de refuser une sélection de modèle établie sur d'autres données : le ticket de sélection est rejeté si l'empreinte ne correspond pas (`selection_tickets.rs:105-110`). Voir `08-forecast/selection-du-modele.md`.

---

## Encadrés

> **ℹ À placer en tête de page — Le fichier en quatre règles.**
> Une ligne par observation, une colonne de dates au format année-mois-jour, une colonne numérique à prévoir, et la même distance de temps entre deux lignes consécutives. Le reste — colonne de série, covariables, lignes futures — est facultatif.

> **⚠ À placer dans « Les dates » — Le format jour/mois/année n'est pas reconnu.**
> `23/07/2026` est refusé. Les formats acceptés commencent tous par l'année : `2026-07-23`, `2026/07/23`, avec ou sans heure. C'est la cause de refus la plus courante et la plus facile à corriger : reformatez la colonne de dates dans votre tableur avant l'import.

> **⚠ À placer dans « Les lignes futures » — Une cellule cible vide ne veut pas dire « valeur manquante ».**
> Beaver comprend une ligne datée sans valeur cible comme une **ligne future**. Un trou au milieu de votre historique sera donc lu comme le début du futur, et toute donnée qui suit déclenchera un refus. Supprimez les lignes incomplètes de l'historique, ou laissez-les de côté.

> **ℹ À placer dans « Les contrôles qui alertent » — Les valeurs atypiques sont comptées, pas corrigées.**
> Beaver ne retire jamais un point de vos données. Il vous signale les valeurs franchement à part — le seuil utilisé est prudent — et vous laisse juger si elles sont réelles ou fautives.

> **⚠ À placer dans « Les contrôles qui alertent » — L'absence d'alerte n'est pas une garantie.**
> La détection des valeurs atypiques demande au moins huit points, celle d'un changement de régime au moins douze. En dessous, aucune alerte ne peut apparaître, quelles que soient les données.

> **ℹ À placer dans « La fréquence » — « Jour ouvré » ignore les jours fériés.**
> Seuls les samedis et dimanches sont sautés. Un jour férié sera compté comme une période manquante.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Dates invalides » | Format jour/mois/année, mois en toutes lettres, ou numéro de série de tableur | Reformater la colonne en `AAAA-MM-JJ` |
| « Colonne cible non numérique » | Séparateur de milliers, symbole monétaire, ou texte dans la colonne | Nettoyer la colonne ; la virgule décimale seule est acceptée |
| « Fréquence incohérente » | La fréquence choisie ne correspond pas à l'espacement réel des lignes | Vérifier le pas réel du fichier et changer le réglage **Fréquence** |
| « Dates dupliquées » | Deux lignes à la même date, souvent après une fusion de fichiers | Agréger ou supprimer le doublon |
| « Dates non chronologiques » | Fichier trié à l'envers ou par une autre colonne | Trier par date croissante |
| « Lignes futures invalides » | Un trou dans l'historique lu comme un début de futur, ou un nombre de lignes futures ≠ horizon | Supprimer les lignes cibles vides du milieu, ou ajuster l'horizon |
| « Dates futures invalides » | La première ligne future ne suit pas la dernière ligne d'historique d'un seul pas | Combler l'écart ou supprimer les lignes futures |
| « Historique insuffisant » | Moins de deux points dans une série | Fournir plus de lignes, ou retirer la colonne série qui découpe trop finement |
| « Trop de séries » | Plus de **256** valeurs distinctes dans la colonne série | Filtrer le fichier, ou choisir une autre colonne série |
| « Volume de prédictions trop important » | Séries × horizon au-delà de **100 000** | Réduire l'horizon ou le nombre de séries |
| « Données Forecast invalides » | Message de repli : ligne mal formée, trop de colonnes, cellule trop longue, identifiant de série illisible | Vérifier la structure du fichier ; le message ne précise pas lequel des quatre |
| « Import impossible. Vérifie le format du fichier. » | Extension non acceptée, deux en-têtes identiques, ou dépassement d'une borne de lecture | Renommer la colonne en double, réduire le fichier |
| « Périodes manquantes : 43 » sans que rien ne soit cassé | Alerte, pas erreur : des pas de temps sont absents de l'historique | Combler les trous améliore la prévision, mais le calcul a bien eu lieu |
| Beaucoup de périodes manquantes sur une série quotidienne | La fréquence **Jour** est appliquée à des données de jours ouvrés | Choisir **Jour ouvré** |

---

## Renvois

- `08-forecast/vue-densemble.md` — le parcours complet et l'écran de configuration
- `08-forecast/selection-du-modele.md` — l'empreinte des données et les tickets de sélection
- `08-forecast/analyse-avancee.md` — les anomalies détectées *après* le calcul, sur les écarts du modèle, à ne pas confondre avec les valeurs atypiques de cette page
- `08-forecast/exports.md` — retrouver les données d'entrée dans le fichier exporté
- `08-forecast/evaluation-et-comparaison.md` — pourquoi un historique court limite l'évaluation
- `12-reference/emplacement-des-donnees.md` — où vivent les profils de données

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **Les anomalies sont affichées sans leur nature.** La liste de la section Données montre « Erreur bloquante · N » et « Avertissement · N », jamais de quelle anomalie il s'agit (`workbench/forecast-workbench-data.tsx:85-95`). Les vingt-deux codes internes (`missing_periods`, `possible_outliers`, `duplicate_date`…) n'ont **aucune traduction** dans `fr.json`. Le seul qui y figure, `short_history`, appartient à un autre écran — les avertissements de plan de l'évaluation (`fr.json`, `forecast.workbench.evaluation.planWarnings.short_history`) — et n'est pas utilisé ici. Concrètement, un utilisateur qui voit « Avertissement · 12 » n'a aucun moyen de savoir ce qui a été détecté. C'est le principal manque de cette page. Deux questions pour l'équipe : le site décrit-il le comportement réel, ou attend-on l'ajout des libellés ? Si les libellés sont ajoutés, c'est vingt-deux textes en sept langues.
2. **Quatre défauts bloquants partagent le message de repli « Données Forecast invalides »** — ligne mal formée, trop de colonnes, cellule trop longue, identifiant de série illisible (`types.rs:106`, codes émis en `audit.rs:46-98`). Le code sait lequel s'est produit et ne le dit pas. À signaler comme demande d'évolution.
3. **Deux mots pour la même chose.** L'écran de configuration dit **« Covariables »**, la section Données dit **« Variables externes »** (`fr.json`, `forecast.config.covariates` et `forecast.workbench.data.covariates`). À unifier dans l'application ; en attendant, la page doit donner les deux termes.
4. **La liste déroulante des fréquences est plus courte que ce que le moteur accepte.** L'écran propose huit fréquences (`forecast-limits.ts:8`), le moteur en comprend davantage — la seconde, les multiples comme `15MIN` ou `2H` (`input_dates.rs:94-117`). Ces formes ne sont accessibles que par l'agent. Décision produit : les documenter comme une possibilité de l'agent, ou les taire jusqu'à ce que l'écran les propose ? Recommandation : les mentionner dans la page sur les outils de l'agent, pas ici.
5. **« Historique insuffisant » couvre deux situations très différentes** — une série de moins de deux points, et aucune série exploitable du tout (`profile.rs:43-49` et `:121-128`). Même message, causes opposées.

**Non vérifié — hors de portée d'une lecture du code**

6. **Le comportement réel sur un fichier Excel à plusieurs feuilles.** La lecture appelle le lecteur de tableur sans préciser de feuille (`file_input.rs:45-52`, deux arguments à `None`). Laquelle est prise ? À vérifier par un essai avant de rédiger la section 1.
7. **Le sort des cellules de date stockées comme nombres par Excel.** Le lecteur de tableur peut les rendre sous forme numérique ; l'analyse de date, elle, n'accepte que du texte (`input_dates.rs:18`). À vérifier par un essai — c'est un cas très fréquent en pratique.
8. **Le comportement d'un fichier CSV avec point-virgule comme séparateur**, courant en France. Non vérifié dans le lecteur.

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

9. La section **Données** de la fenêtre Espace Forecast : disposition des six compteurs, couleur de l'encadré de synthèse dans les deux thèmes, lisibilité de la liste d'anomalies.
10. Les messages d'avertissement de l'écran de configuration (`fr.json`, `forecast.config.messages.*`) : lesquels apparaissent réellement, à quel moment, et sous quelle forme.
11. La couche **« Alertes qualité »** dans le graphique de la vue principale : à quoi elle ressemble, et si sa position à la date de fin d'historique est compréhensible pour un lecteur.
12. Les messages d'erreur de l'audit ne recevront pas de capture : les provoquer suppose de fabriquer un fichier fautif par cas. Ils sont documentés d'après le code, avec leur source, comme le prévoit la convention de ce dossier.
