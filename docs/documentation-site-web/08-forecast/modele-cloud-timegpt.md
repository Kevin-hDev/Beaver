# Le modèle distant TimeGPT-2 (Nixtla)

**Emplacement site** — Forecast › Le modèle distant TimeGPT-2
**Répond à** — « Comment utiliser TimeGPT, qu'est-ce qui sort de ma machine quand je m'en sers, et qu'est-ce qu'il apporte par rapport aux modèles locaux ? »
**Sources** — `src-tauri/src/services/forecast/catalog_specs/nixtla.rs`, `catalog_specs/providers.rs` ; `client_nixtla.rs`, `client_nixtla_retry.rs`, `client_nixtla_options.rs`, `nixtla_multiseries.rs`, `nixtla_exogenous.rs`, `client_http.rs` ; `registry_specs.rs`, `registry_caps.rs`, `registry_params.rs` ; `model_config/schema.rs` ; `interval_capability.rs`, `intervals.rs` ; `selection_policy.rs` ; `limits.rs` ; `src-tauri/src/commands/forecast.rs` ; `src-tauri/src/services/secure_http.rs` ; `src/components/forecast/model-browser/model-specs.tsx` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026. Aucun appel réel à l'API Nixtla n'a été effectué : tout ce qui concerne les réponses du service vient de la lecture du code d'analyse et des tests.

> **Cette page ne donne aucun prix ni aucune limite d'usage.** Ces informations appartiennent à Nixtla et changent sans que Beaver en sache rien. Le site renvoie vers `https://dashboard.nixtla.io`, l'adresse inscrite dans le catalogue (`catalog_specs/providers.rs:25`).

---

## Plan de page proposé

1. Ce qu'est TimeGPT-2 dans Beaver
2. Les quatre variantes du catalogue
3. Ce que TimeGPT-2 sait faire que les modèles locaux ne savent pas
4. Configurer la clé API
5. **Ce qui sort de votre machine, exactement**
6. Comment l'appel est fait, et ce qu'il devient s'il échoue
7. Les réglages propres à TimeGPT-2
8. Les limites
9. TimeGPT-2 et la sélection automatique

---

## Contenu

### 1. Ce qu'est TimeGPT-2 dans Beaver

TimeGPT-2 est le **seul modèle de prévision distant** du catalogue de Beaver (`registry_specs.rs:49-76` : les quatre seules entrées de type `CloudApi`). Rien n'est téléchargé, rien n'est installé : vos données sont envoyées au service de Nixtla, qui renvoie la prévision.

Dans l'interface, ces modèles portent la mention **« Cloud (clé requise) »**, et l'absence de clé se traduit par **« Clé API non configurée »** (`src/i18n/fr.json`, `forecast.models.cloud` et `forecast.models.noKeyConfigured`). Le fournisseur s'appelle **Nixtla (TimeGPT-2)** et sa description affichée est « Famille TimeGPT en cloud, accès par API. » (`catalog_specs/providers.rs:23` ; `src/i18n/fr.json`, `forecast.providers.nixtla`).

### 2. Les quatre variantes du catalogue

Le catalogue en contient **quatre**, toutes distantes (`catalog_specs/nixtla.rs:3-89`) :

| Modèle affiché | Identifiant | Horizon max | Fréquences | Variables de contexte | Multivarié conjoint |
|---|---|---|---|---|---|
| **TimeGPT-2 Mini** | `timegpt-2-mini` | **5 000** | T à Y | **oui** | — |
| **TimeGPT-2 Standard** | `timegpt-2-standard` | **5 000** | T à Y | **oui** | — |
| **TimeGPT-2 Pro** | `timegpt-2-pro` | **5 000** | T à Y | **oui** | — |
| **TimeGPT-2.1** | `timegpt-2.1` | **5 000** | T à Y | **oui** | **oui** |

Les quatre partagent la famille `timegpt-2` et affichent **TimeGPT-2** comme nom de famille (`src/i18n/fr.json`, `forecast.models.families.timegpt-2`). Aucune ne publie de nombre de paramètres, de taille de fichier ni de besoin en mémoire : ces champs valent `—` ou zéro, parce que rien ne tourne chez vous (`catalog_specs/nixtla.rs:8-11`).

**Ce qui distingue les quatre variantes n'est pas décrit dans le code de Beaver.** Mini, Standard et Pro ont exactement les mêmes capacités déclarées ; seule TimeGPT-2.1 ajoute le multivarié conjoint (`registry_specs.rs:49-76`). Ne pas inventer une hiérarchie de qualité ou de vitesse sur le site : renvoyer à la documentation de Nixtla.

**Un point à signaler honnêtement.** Beaver traduit l'identifiant `timegpt-2-standard` en `timegpt-2` au moment d'appeler l'API (`client_nixtla.rs:92-97`). Le commentaire qui accompagne cette traduction, daté du **30 juillet 2026**, note que `timegpt-2` a disparu de la documentation publique de Nixtla et que l'équipe n'a pas pu trancher, faute de clé, entre « l'ancien nom reste accepté » et « il est devenu `timegpt-2-lab` ». Voir « Points à confirmer ».

### 3. Ce que TimeGPT-2 sait faire que les modèles locaux ne savent pas

Trois différences, toutes vérifiées :

**L'horizon.** TimeGPT-2 accepte **5 000 pas** — c'est aussi la borne absolue de Beaver (`limits.rs:10`). Les modèles locaux plafonnent entre **1 000 et 4 096** selon la famille.

**Les variables de contexte, avec le futur.** Les quatre variantes lisent des colonnes explicatives sur le passé **et** sur la période à prévoir (`registry_caps.rs:63-73`). Parmi les modèles locaux, seuls Chronos-2 et TimesFM 2.5 en sont capables.

**Les variables de contexte non numériques.** TimeGPT-2 accepte des **catégories textuelles** — un nom de promotion, un type de jour — que Beaver repère et signale au service comme telles (`nixtla_exogenous.rs:42-73`). Une colonne doit être entièrement numérique ou entièrement textuelle, passé et futur confondus ; un mélange est refusé avec « Covariables invalides » (`nixtla_exogenous.rs:57-70`). Les valeurs `true` et `false` sont converties en 1 et 0 ; une catégorie fait au plus **256 caractères** (`nixtla_exogenous.rs:3`, `:13-19`).

**Le multivarié conjoint, seulement pour TimeGPT-2.1, et seulement sous condition.** Beaver n'active le mode conjoint que si le fichier contient **plus d'une série** et que **toutes les séries portent exactement les mêmes dates**, historique et futur compris (`nixtla_multiseries.rs:65-67`, `:173-193`). Sinon, la requête part en mode séries indépendantes, sans que rien ne le signale à l'écran. À dire sur le site : des séries mal alignées perdent silencieusement l'avantage de la version 2.1.

### 4. Configurer la clé API

La clé se saisit dans l'écran des clés API, sous le fournisseur **Nixtla (TimeGPT-2)**. Elle est rangée dans le coffre chiffré comme toutes les autres, sous l'identifiant `nixtla` — voir `11-securite/vault-et-cles-api.md` pour le mécanisme complet, qui n'est pas à redire ici.

Trois faits propres à Forecast :

- **Tant que la clé n'est pas enregistrée, les quatre variantes sont visibles mais non sélectionnables.** Leur état est `provider_required` (`model_listing.rs:152-166` ; `src/components/forecast/forecast-model-readiness.ts:15-17`).
- **La clé est lue au moment de chaque prévision**, jamais stockée dans l'analyse (`commands/forecast.rs:67-68`). Une prévision lancée sans clé échoue avec « Clé API Nixtla non configurée ».
- **La clé n'est jamais écrite dans les traces.** En cas d'échec, Beaver n'enregistre que le code d'état HTTP (`client_nixtla.rs:27-32`).

L'adresse d'inscription proposée par le catalogue est **`https://dashboard.nixtla.io`** (`catalog_specs/providers.rs:25`).

### 5. Ce qui sort de votre machine, exactement

C'est le cœur de la page, et il se décrit champ par champ. La requête est construite dans `nixtla_multiseries.rs:12-70`.

**Ce qui part :**

| Champ envoyé | Contenu | Source |
|---|---|---|
| `series.y` | **Toutes les valeurs de la colonne cible**, série après série, à la suite | `nixtla_multiseries.rs:26-30` |
| `series.sizes` | Le nombre de points de chaque série | `:25` |
| `series.X` | Les **valeurs passées** des colonnes de contexte choisies | `:36-48` |
| `series.X_future` | Leurs **valeurs futures** | `:38-48` |
| `series.categorical_exog` | La position des colonnes de contexte textuelles | `:50-52` |
| `freq` | La fréquence déclarée (`D`, `M`, `W`…) | `:59` |
| `h` | L'horizon demandé | `:60` |
| `model` | L'identifiant de la variante | `:61` |
| `level` | Le niveau de confiance, en pourcentage entier | `:62` |
| `clean_ex_first` | Un réglage de nettoyage des variables de contexte, à **vrai** par défaut | `:63` |
| Réglages d'affinage | `finetune_steps`, `finetune_loss`, `finetune_depth`, `feature_contributions`, s'ils ont été modifiés | `client_nixtla_options.rs:27-42` |

**Ce qui ne part pas, et qui mérite d'être dit :**

- **Aucune date.** La requête ne contient que des valeurs et des tailles de séries ; les dates des prévisions sont **recalculées sur votre machine** à partir de la fréquence et de la fin de l'historique (`client_nixtla.rs:48` → `nixtla_multiseries.rs:84` → `input_future::expected_dates_by_series`).
- **Aucun nom de colonne.** Les colonnes sont transmises comme des listes de nombres, dans l'ordre. Le nom de votre colonne cible et ceux de vos variables restent chez vous.
- **Aucun identifiant de série.** Les séries sont transmises collées les unes aux autres, séparées par le tableau des tailles.
- **Aucune ligne du fichier autre que ces colonnes.** Les colonnes que vous n'avez pas choisies comme cible, date ou contexte ne sont pas lues.
- **Aucune note, aucun scénario, aucun nom d'analyse.** Ces éléments vivent uniquement sur votre disque.

**À écrire tel quel sur le site** : ce qui part chez Nixtla, ce sont **les valeurs de vos séries**. C'est déjà beaucoup — une série de chiffre d'affaires reste une série de chiffre d'affaires — mais le contexte qui la rend identifiable (noms, dates, autres colonnes) ne quitte pas la machine.

### 6. Comment l'appel est fait, et ce qu'il devient s'il échoue

**L'adresse** : `POST https://api.nixtla.io/v2/forecast` (`client_nixtla.rs:11`, `:84-86`). Elle est écrite en dur, elle n'est pas configurable.

**L'authentification** : un en-tête `Bearer`, posé au dernier moment à partir d'une copie protégée de la clé, effacée de la mémoire après usage (`client_nixtla_retry.rs:35-48`).

**La protection contre les redirections** : le client refuse de suivre une redirection vers un autre serveur, et un test dédié vérifie que **ni la clé ni les données** n'atteignent la destination d'une redirection (`client_nixtla_retry.rs:70-94`, test `redirect_never_receives_forecast_secret_or_payload`).

**Les délais et les reprises** (`client_nixtla_retry.rs:8-33`) :

| Élément | Valeur |
|---|---|
| Délai maximal d'une requête | **60 secondes** |
| Délai maximal du client | **120 secondes** |
| Nouvelles tentatives | **2**, après **2 s** puis **4 s** |
| Codes qui déclenchent une reprise | **429**, **502**, **503**, **504** |
| Taille maximale de la réponse lue | **32 Mo** |

Une erreur d'authentification ou de requête (400, 401, 403…) **n'est pas retentée** : elle échoue immédiatement. Le message affiché est générique — « Erreur du service de prédiction » — dans tous les cas (`client_nixtla.rs:31`).

**La réponse est vérifiée avant d'être acceptée** : le nombre de valeurs renvoyées doit correspondre exactement au nombre de séries multiplié par l'horizon, et chaque borne d'intervalle doit être présente et finie (`nixtla_multiseries.rs:85-92`, `:100-117`). Une réponse incomplète est rejetée plutôt qu'affichée partiellement.

### 7. Les réglages propres à TimeGPT-2

TimeGPT-2 est la seule famille dont les réglages portent sur l'**affinage** du modèle plutôt que sur la longueur de contexte (`registry_params.rs:10-17` ; bornes dans `model_config/schema.rs:47-58`) :

| Réglage | Valeur par défaut | Bornes |
|---|---|---|
| `horizon_max_override` | — | relève la borne d'horizon |
| `clean_ex_first` | **vrai** | vrai ou faux |
| `finetune_steps` | **0** | de 0 à **1 000** |
| `finetune_loss` | **`default`** | `default`, `mae`, `mse`, `rmse` |
| `finetune_depth` | **1** | de **1 à 5** |
| `feature_contributions` | **faux** | vrai ou faux |

La borne de `finetune_depth` est protégée par un test dédié qui rejette 0 et 6 (`model_config/schema.rs:129-146`).

**Ce que TimeGPT-2 n'a pas**, contrairement aux modèles locaux : pas de `context_length`, pas de `quantiles` supplémentaires, pas de `non_negative_output`. Ces réglages ne sont pas proposés parce que le service ne les accepte pas.

**L'intervalle de confiance** est **continu** : tout niveau entier de **50 % à 99 %** est accepté (`interval_capability.rs:12-30`, `:43-46`). C'est un avantage sur TimesFM, Toto, FlowState, TabPFN-TS, TiRex et Kairos, qui n'offrent que 60 % ou 80 %.

### 8. Les limites

Ces bornes s'appliquent à toute prévision, distante ou locale (`limits.rs:1-38`, `:64-75`) :

| Limite | Valeur |
|---|---|
| Horizon | **5 000** pas |
| Séries dans un fichier | **256** |
| Prédictions au total (séries × horizon) | **100 000** |
| Colonnes de contexte | **64** |
| Lignes du fichier source | **5 000** |
| Colonnes du fichier source | **256** |
| Données collées directement | **5 Mo** |
| Fichier tableur importé | **50 Mo** |
| Réponse du service | **32 Mo** |

Le dépassement du produit séries × horizon est refusé avant l'appel, avec « Volume de prédictions trop important » (`limits.rs:64-75`).

### 9. TimeGPT-2 et la sélection automatique

Beaver propose deux modes de choix du modèle : **manuel** et **automatique** (`selection_policy.rs:11-24`). Un troisième réglage les accompagne : **autoriser ou non le cloud dans le mode automatique**.

Trois faits :

- **Le mode automatique est refusé au cloud par défaut** : `allow_cloud_in_auto` vaut **faux** tant que l'utilisateur ne l'a pas activé (`selection_policy.rs:30-38`). Le mode par défaut lui-même est **manuel**.
- **Le même réglage borne l'évaluation.** Pendant un backtest, un modèle distant qui n'est pas celui de l'analyse est refusé avec le code `cloud_not_allowed` si le cloud n'est pas autorisé (`evaluation/model_runner.rs:94-102`).
- **Choisir TimeGPT-2 à la main reste toujours possible**, quel que soit ce réglage : il ne concerne que la sélection automatique.

Le détail du parcours de sélection est dans `08-forecast/selection-du-modele.md`.

---

## Encadrés

> **ℹ Ce qui part chez Nixtla, ce sont vos valeurs — pas leurs étiquettes.**
> La requête contient les nombres de vos séries et de vos variables de contexte. Elle ne contient ni dates, ni noms de colonnes, ni identifiants de séries, ni nom d'analyse : les dates de la prévision sont recalculées sur votre machine.

> **⚠ Sans clé API, les quatre variantes restent visibles mais ne peuvent pas être choisies.**
> C'est volontaire : elles apparaissent dans le catalogue avec la mention « Clé API non configurée » plutôt que de disparaître sans explication.

> **ℹ TimeGPT-2 accepte des variables de contexte textuelles.**
> Un nom de campagne, un type de jour : Beaver les repère et les signale comme catégories. Une colonne doit rester d'un seul type — que des nombres ou que du texte — sur toute sa longueur, passé et futur compris.

> **⚠ Le multivarié conjoint de TimeGPT-2.1 exige des séries parfaitement alignées.**
> Plus d'une série, et exactement les mêmes dates pour toutes. Si l'alignement manque, la requête part quand même, en mode séries indépendantes, sans avertissement à l'écran.

> **ℹ Une erreur d'authentification n'est pas retentée.**
> Seules les surcharges et les pannes passagères du service — 429, 502, 503, 504 — déclenchent deux nouvelles tentatives, à deux puis quatre secondes.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Clé API Nixtla non configurée » | Aucune clé enregistrée pour le fournisseur `nixtla` | La saisir dans l'écran des clés API |
| Les modèles TimeGPT-2 sont grisés | Même cause | Idem |
| « Erreur du service de prédiction » | Message unique pour toutes les pannes distantes : clé refusée, quota atteint, service indisponible, réponse illisible | Vérifier la clé et l'état du service chez Nixtla |
| « Covariables invalides » | Une colonne de contexte mélange nombres et texte, ou contient une case vide | Compléter la colonne, ou lui donner un type unique |
| « Données de contexte futur invalides » | Il manque des lignes futures : il en faut exactement une par pas d'horizon et par série | Compléter le fichier |
| « Volume de prédictions trop important » | Séries × horizon dépasse **100 000** | Réduire l'horizon ou le nombre de séries |
| « Réponse de prédiction incomplète » | Le service a renvoyé moins de valeurs qu'attendu | Réessayer ; signaler si cela persiste |
| La version 2.1 ne semble rien changer | Les séries ne sont pas alignées sur les mêmes dates | Aligner les dates de toutes les séries |

---

## Renvois

- `08-forecast/modeles-locaux.md` — les vingt modèles qui tournent sans réseau
- `08-forecast/selection-du-modele.md` — le mode automatique et l'autorisation du cloud
- `08-forecast/evaluation-et-comparaison.md` — comparer TimeGPT-2 aux modèles locaux sur vos propres données
- `11-securite/vault-et-cles-api.md` — où vit la clé et qui peut la relire
- `01-decouverte/local-vs-cloud.md` — le choix général entre traitement local et distant

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **L'identifiant envoyé pour TimeGPT-2 Standard est incertain.** Beaver traduit `timegpt-2-standard` en `timegpt-2` (`client_nixtla.rs:92-97`), et le commentaire du code, daté du **30 juillet 2026**, indique que ce nom a disparu de la documentation de Nixtla, qui ne liste plus que `timegpt-2-pro`, `timegpt-2-lab`, `timegpt-2-mini` et `timegpt-2.1`. Deux questions pour l'équipe : la variante « Standard » fonctionne-t-elle encore ? Et faut-il la retirer du catalogue si elle ne fonctionne plus ? **Tant que ce point n'est pas tranché, ne pas promettre sur le site que les quatre variantes marchent.** C'est le point le plus important de ce fichier.
2. **Le catalogue n'expose aucune variante `timegpt-2-lab`**, alors que la documentation de Nixtla la citait à cette date. À décider : l'ajouter, ou ne rien dire.
3. **Le message d'erreur distant est unique.** « Erreur du service de prédiction » couvre la clé refusée, le quota dépassé, le service en panne et la réponse illisible (`client_nixtla.rs:31` ; `client_nixtla_retry.rs:26-32`). L'utilisateur ne peut pas distinguer « ma clé est mauvaise » de « le service est en panne ». À signaler comme demande d'évolution, et à documenter tel quel en attendant.
4. **Le repli silencieux du mode multivarié** de TimeGPT-2.1 (`nixtla_multiseries.rs:65-67`) n'est signalé nulle part dans l'interface d'après la lecture du code. À vérifier à l'écran, et à traiter côté produit si c'est confirmé : une capacité annoncée qui se désactive sans le dire est exactement le cas que la règle d'interface du projet interdit.

**Non vérifié — hors de portée d'une lecture du code**

5. **Le comportement réel de l'API Nixtla** — aucun appel n'a été effectué. La forme de la réponse est déduite du code d'analyse (`nixtla_multiseries.rs:71-120`) et des tests (`client_nixtla_response_tests.rs`), pas d'une réponse observée.
6. **Les différences entre Mini, Standard, Pro et 2.1** en qualité, en vitesse et en usage. Beaver n'en sait rien ; le site doit renvoyer à Nixtla plutôt que de trancher.
7. **Les conditions d'utilisation de Nixtla** — notamment ce que le service fait des données reçues. Point à vérifier avec Kevin avant de publier une page qui décrit ce qui sort de la machine : dire ce que Beaver envoie est vérifié, dire ce que Nixtla en fait ne l'est pas.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

8. L'apparence d'un modèle distant non configuré dans le catalogue : mention exacte, bouton présent ou absent.
9. L'écran des réglages d'affinage, et ce qui se passe quand `finetune_steps` est laissé à zéro.
10. L'endroit exact où se règle l'autorisation du cloud en mode automatique.
