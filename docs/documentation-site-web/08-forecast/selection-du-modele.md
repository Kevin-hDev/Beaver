# Choisir le modèle : Manuel ou Auto

**Emplacement site** — Forecast › Choisir le modèle
**Répond à** — « Dois-je choisir le modèle moi-même ou laisser Beaver le faire, et sur quoi le mode Auto se décide-t-il vraiment ? »
**Sources** — `src-tauri/src/services/forecast/selection_policy.rs`, `selected_model.rs`, `auto_selection.rs`, `auto_selection_candidate.rs`, `auto_selection_rank.rs`, `auto_selection_ui.rs`, `selection_tickets.rs`, `validation_selection.rs`, `hardware_profile.rs`, `limits.rs`, `catalog.rs`, `interval_capability.rs` ; `src-tauri/src/commands/forecast.rs`, `commands/forecast_models.rs` ; `src-tauri/src/services/agent_local/tool_dispatcher_forecast_models.rs`, `tool_definitions_forecast.rs` ; `src/components/forecast/widgets/forecast-model-selector.tsx`, `forecast-selection-mode-control.tsx`, `src/components/forecast/model-selection/use-forecast-selection-policy.ts`, `src/components/forecast/forecast-panel.tsx`, `use-forecast-config-models.ts`, `forecast-model-meta.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, critère par critère. Aucun écran n'a été observé ; les points d'affichage sont listés en fin de fichier.

> **Cette page décrit le mécanisme de choix, pas les modèles.** Le catalogue, leurs tailles et leur installation sont dans `08-forecast/modeles-locaux.md` et `08-forecast/modele-cloud-timegpt.md`.

---

## Plan de page proposé

1. Les deux modes, et où les changer
2. Ce que fait le mode Manuel
3. Ce que fait le mode Auto — et ce qu'il ne fait pas
4. Les éliminations : ce qui exclut un modèle
5. Le classement : ce qui départage ceux qui restent
6. La mesure des ressources de votre machine
7. Le verrou : le ticket de sélection
8. Les modèles distants dans le mode Auto
9. Ce qui est gardé avec le résultat

---

## Contenu

### 1. Les deux modes, et où les changer

Beaver connaît **exactement deux modes** de sélection, et pas plus (`services/forecast/selection_policy.rs:11-16`) :

| Libellé affiché | Identifiant dans le code |
|---|---|
| **Manuel** | `manual` |
| **Auto** | `auto` |

Le réglage vit dans la **liste déroulante du sélecteur de modèle**, en bas du panneau Forecast quand la vue principale est ouverte (`forecast-panel.tsx:180-190` ; le bloc de mode est inséré dans la liste elle-même, `widgets/forecast-model-selector.tsx:120-126`). Il porte le titre **« Sélection »**, avec deux boutons **« Manuel »** et **« Auto »** (`widgets/forecast-selection-mode-control.tsx:23-39` ; `fr.json`, `forecast.selection`).

En mode Auto, un interrupteur supplémentaire apparaît sous les deux boutons : **« Autoriser les modèles cloud »** (`forecast-selection-mode-control.tsx:41-51`).

**Le bouton qui ouvre la liste change de libellé selon le mode** : en Manuel il affiche le nom du modèle retenu, en Auto il affiche simplement **« Auto »** (`forecast-model-selector.tsx:166-170`).

**Le réglage est global à l'application**, pas propre à une conversation : il est écrit dans un seul fichier, `~/.local/share/cl-go-dash/forecast-selection-policy.json`, borné à **4 Ko** et écrit par fichier temporaire puis renommage (`selection_policy.rs:9`, `:77-79`, `:137-140`).

**Les valeurs de départ** (`selection_policy.rs:31-39`) : mode **Manuel**, aucun modèle retenu, modèles distants **non autorisés** en Auto.

Un fichier plus ancien, `forecast-selected-model.json`, est lu une seule fois pour récupérer le modèle qui y était enregistré, puis remplacé par le nouveau format (`selection_policy.rs:93-107`).

### 2. Ce que fait le mode Manuel

Un modèle est retenu, et c'est celui-là qui sert. Le contrôle est strict des deux côtés (`selected_model.rs:11-31`) :

- **Aucune donnée de sélection automatique n'est acceptée** : si une demande arrive avec un identifiant de sélection ou des raisons de choix, elle est refusée — « Métadonnées Auto interdites en mode Manuel ».
- **Aucun modèle n'est retenu ?** Le calcul s'arrête : « Aucun modèle Forecast sélectionné ».
- **Un modèle différent demandé ?** Refus : « Le modèle demandé ne correspond pas à la sélection manuelle ».

Ce dernier point mérite d'être dit sur le site : en mode Manuel, **l'agent ne peut pas changer de modèle de sa propre initiative**. L'outil `forecast_models` lui renvoie le modèle imposé et son état, avec la consigne de ne pas en sortir (`services/agent_local/tool_dispatcher_forecast_models.rs:49-63`).

Une commodité côté écran : quand le mode est Manuel et que le modèle retenu n'existe plus ou n'est plus utilisable, l'interface en choisit un autre dans la liste plutôt que de rester sur un choix mort (`forecast-model-selector.tsx:75-84`).

### 3. Ce que fait le mode Auto — et ce qu'il ne fait pas

C'est le point le plus important de la page, et le plus facile à mal comprendre.

**Le mode Auto ne choisit pas le modèle à votre place depuis l'écran.** Il construit une **liste courte de modèles acceptables**, classée, et vérifie que le modèle finalement lancé en fait partie. Le choix dans cette liste reste fait par quelqu'un : vous depuis l'écran, l'agent depuis la conversation.

**Depuis l'écran de configuration.** En mode Auto, aucun modèle n'est présélectionné : la liste s'ouvre vide et vous choisissez (`forecast-panel.tsx:158-159` ; `use-forecast-config-models.ts:22`). Au lancement, Beaver reconstruit la liste des candidats et **vérifie que votre choix y figure**, sinon le calcul est refusé avec « Le modèle ne fait pas partie des candidats Auto » (`commands/forecast.rs:39-58` → `auto_selection_ui.rs:26-33`). Le résultat est alors enregistré comme un **choix explicite de l'utilisateur**, avec la raison `user_requested` (`commands/forecast.rs:42-46`).

**Depuis la conversation.** L'agent appelle l'outil `forecast_models` en lui passant l'identifiant du profil de données. Il reçoit alors les candidats classés, la base du classement, un profil de la tâche, un profil du matériel, et un **identifiant de sélection** à repasser au moment de calculer (`tool_dispatcher_forecast_models.rs:101-165`). La consigne qui accompagne la réponse est explicite : choisir **un seul** candidat, et ne jamais qualifier de « meilleur modèle » un classement fondé uniquement sur les capacités et les ressources (`:136`).

**Cinq candidats au maximum** sont renvoyés, avec **huit raisons** au plus chacun (`limits.rs:25-26`, `auto_selection.rs:139-153`).

**Le mode Auto exige un profil de données à jour.** Un profil calculé par une version antérieure, sans niveau de confiance enregistré, fait échouer la sélection avec « Profil Forecast obsolète : relancer forecast_data_audit » (`tool_dispatcher_forecast_models.rs:78-83` ; `auto_selection_candidate.rs:34`).

### 4. Les éliminations : ce qui exclut un modèle

Avant tout classement, chaque modèle du catalogue passe une série de filtres. Un seul échec l'écarte. L'ordre est celui du code (`auto_selection_candidate.rs:17-47`).

| Filtre | Motif interne | Ce que ça veut dire |
|---|---|---|
| Le modèle est au catalogue | `unknown_model` | L'identifiant n'existe pas |
| Un moteur sait le faire tourner | `adapter_unavailable` | Aucun adaptateur pour ce modèle |
| Le modèle est utilisable | `not_runnable` | Modèle distant sans fournisseur configuré, par exemple |
| Le moteur est prêt | `runtime_not_ready` | Modèle local non installé, ou son environnement d'exécution pas encore en place |
| Les modèles distants sont autorisés | `cloud_not_allowed` | Le modèle est distant et l'interrupteur « Autoriser les modèles cloud » est fermé |
| Le profil de données porte un niveau de confiance | `profile_outdated` | Profil trop ancien : relancer l'audit |
| Le modèle sait produire **exactement** le niveau de confiance demandé | `confidence_unsupported` | Voir l'encadré ci-dessous |
| Le modèle sait faire la tâche | `task_incompatible` | Détail juste en dessous |
| La machine peut le porter | `resources_insufficient` | Voir la section 6 |
| La machine est mesurable, pour un gros modèle | `resources_unknown` | Mémoire non mesurée **et** modèle demandant plus de **2 048 Mo** (`limits.rs:38`) |

**Le niveau de confiance n'est jamais arrondi.** C'est une décision de conception écrite noir sur blanc dans les consignes données à l'agent : « Never round an explicit request; ask the user to change the level or selected model if they are incompatible » (`tool_dispatcher_forecast_models.rs:62`). Un modèle qui ne sait produire que des fourchettes à 60 % et 80 % est éliminé si vous demandez 85 % — il n'est pas utilisé « au plus proche ».

**« Sait faire la tâche » recouvre six conditions**, toutes vérifiées ensemble (`auto_selection_candidate.rs:72-86`) :

1. l'horizon demandé ne dépasse pas l'horizon maximal du modèle ;
2. le modèle accepte la fréquence choisie ;
3. s'il y a plus d'une série, le modèle sait traiter plusieurs séries ;
4. s'il y a des covariables, le modèle sait les exploiter ;
5. s'il y a des covariables **et** des lignes futures connues, le modèle sait exploiter le futur connu ;
6. le modèle sait produire une fourchette — cette condition est exigée **de tous**, sans exception.

### 5. Le classement : ce qui départage ceux qui restent

Deux classements différents, et la page doit dire lequel s'applique quand — c'est ce qui sépare une recommandation mesurée d'une recommandation seulement plausible.

**Cas 1 — au moins un candidat a été mesuré sur des fenêtres passées.** Le classement s'appelle alors `rolling_backtest` (`auto_selection.rs:130-134`). Les modèles mesurés passent devant tous les autres, et entre eux (`auto_selection_rank.rs:6-35`) :

1. d'abord ceux qui **battent la meilleure référence simple** ; ensuite ceux dont on ne sait pas ; en dernier ceux qui ne la battent pas ;
2. puis une comparaison sur plusieurs mesures à la fois — précision, qualité de la fourchette, durée, mémoire observée (`evaluation::ranking::compare_quality`).

**Cas 2 — aucun candidat n'a été mesuré.** Le classement s'appelle `capabilities_and_resources` et repose sur trois critères seulement (`auto_selection_rank.rs:14-17`) :

1. **l'aisance en mémoire**, dans cet ordre : Confortable, puis Distant, puis Contraint, puis Inconnu, puis Insuffisant (`auto_selection_rank.rs:45-53`) ;
2. à aisance égale, **le modèle qui demande le moins de mémoire** ;
3. à égalité complète, l'ordre alphabétique de l'identifiant.

**Le mot « recommandé » est réservé.** Le premier candidat n'est marqué comme recommandé que dans deux cas : soit aucun candidat n'a été mesuré, soit celui-là bat la meilleure référence simple (`auto_selection.rs:117-126`). **Un modèle mesuré qui ne bat pas la référence n'est jamais recommandé**, même s'il arrive en tête.

**Quand vous nommez explicitement un modèle**, il est remonté en première position et marqué comme demandé, quel que soit son rang (`auto_selection.rs:139-153`). S'il a été éliminé, il n'est pas remplacé en silence : son motif d'exclusion est renvoyé et l'agent a pour consigne de l'expliquer et de ne changer de modèle qu'après votre accord (`tool_dispatcher_forecast_models.rs:131-133`).

### 6. La mesure des ressources de votre machine

Le mode Auto mesure la machine avant de classer. La mesure est **propre à Forecast** et n'est exposée nulle part ailleurs (`tool_dispatcher_forecast_models.rs:153-159`, champ `"scope": "forecast_only"`).

Ce qui est relevé (`hardware_profile.rs:37-73`) : le type de mémoire graphique — dédiée, partagée avec le processeur, ou inconnue —, la mémoire graphique totale, la mémoire graphique disponible, et la mémoire vive disponible.

**Comment un modèle est jugé** (`hardware_profile.rs:116-130`) :

| Verdict | Condition |
|---|---|
| **Insuffisant** | La mémoire disponible est inférieure au besoin **majoré de 20 %** |
| **Confortable** | La mémoire disponible vaut **au moins deux fois** le besoin |
| **Contraint** | Entre les deux |
| **Inconnu** | La mémoire n'a pas pu être mesurée |
| **Distant** | Le modèle tourne chez un fournisseur : aucune mémoire locale n'est en jeu (`:79-81`) |

**La marge de 20 % est le point à retenir** : un modèle qui demande 4 Go ne passe pas sur 4 Go disponibles, il en faut 4,8. Et quand un modèle peut tourner sur processeur **ou** sur carte graphique, c'est **le meilleur des deux verdicts** qui est retenu (`hardware_profile.rs:82-96`, `:132-144`).

Ce même contrôle est réappliqué juste avant de démarrer un modèle local, en dehors de toute sélection automatique : le calcul est refusé avec « Ressources insuffisantes pour ce modèle » (`commands/forecast.rs:76` → `hardware_profile.rs:99-114`).

### 7. Le verrou : le ticket de sélection

Quand l'agent obtient une liste de candidats, il reçoit aussi un **ticket** — un identifiant à usage unique qui atteste que ce choix a bien été fait sur ces données-là (`selection_tickets.rs:52-88`).

| Propriété | Valeur | Source |
|---|---|---|
| Tickets gardés en même temps | **32**, le plus ancien évincé au-delà | `limits.rs:27`, `selection_tickets.rs:75-77` |
| Durée de vie | **30 minutes** | `selection_tickets.rs:13`, `:137-139` |
| Usage | **Unique** — consommé au premier emploi | `selection_tickets.rs:104` |
| Ce qui est vérifié à l'emploi | La conversation, le profil de données, **l'empreinte des données**, et le fait que le modèle figurait bien parmi les candidats | `selection_tickets.rs:105-115` |
| Comparaison de l'identifiant | En **temps constant** | `selection_tickets.rs:102` |
| Tickets gardés sur le disque | **Aucun** — ils vivent en mémoire et disparaissent à la fermeture | `selection_tickets.rs:14-15` |

**Ce que ça change pour l'utilisateur** : une sélection automatique établie puis laissée de côté plus d'une demi-heure, ou établie sur un fichier ensuite modifié, est refusée avec « Sélection Auto expirée ou invalide » (`selection_tickets.rs:141-143`). C'est voulu : cela empêche qu'un choix motivé par des données devienne, sans qu'on le voie, un choix appliqué à d'autres données.

**Les raisons de choix sont fermées.** Seules huit valeurs sont acceptées : `top_backtest`, `beats_baseline`, `precision_requested`, `speed_requested`, `local_required`, `cloud_allowed`, `user_requested`, `resource_fit` — sans doublon, et huit au maximum (`validation_selection.rs:3-12`, `limits.rs:28`). Une raison inventée fait refuser la demande.

### 8. Les modèles distants dans le mode Auto

L'interrupteur **« Autoriser les modèles cloud »** est **fermé par défaut** (`selection_policy.rs:36`). Tant qu'il l'est, tout modèle distant est écarté de la sélection avec le motif `cloud_not_allowed` (`auto_selection_candidate.rs:31-33`), et une demande qui viserait quand même un modèle distant est refusée : « Les modèles cloud ne sont pas autorisés en mode Auto » (`selected_model.rs:51-53`, `:72-74`).

**C'est une décision par défaut en faveur du local**, à présenter comme telle : rien ne part vers un service externe tant que vous ne l'avez pas ouvert explicitement.

Deux nuances à ne pas omettre :

- l'interrupteur ne concerne que le mode **Auto**. En mode Manuel, un modèle distant retenu à la main est utilisé sans cet interrupteur ;
- un modèle distant reçoit le verdict de ressources **Distant**, qui le place **au-dessus de « Contraint »** mais **en dessous de « Confortable »** dans le classement par ressources (`auto_selection_rank.rs:45-53`). Autrement dit : quand un modèle local tient largement sur la machine, il passe devant le modèle distant.

### 9. Ce qui est gardé avec le résultat

Chaque analyse enregistre **comment son modèle a été choisi** (`commands/forecast.rs:92-101`) : la source du choix — Manuel, Auto, ou choix explicite de l'utilisateur —, la base du classement, le verdict de ressources, les raisons retenues, et la mesure sur fenêtres passées quand elle existe (`selection_tickets.rs:35-50`).

**Pourquoi ça compte** : trois mois plus tard, une prévision enregistrée dit encore pourquoi ce modèle-là a été retenu. Sans cette trace, un choix se relit comme une préférence arbitraire.

---

## Tableaux

### Manuel ou Auto : ce qui change

| | **Manuel** | **Auto** |
|---|---|---|
| Qui restreint la liste | Personne : tout le catalogue utilisable | Beaver, par élimination puis classement |
| Qui choisit dans la liste | Vous | Vous depuis l'écran, l'agent depuis la conversation |
| L'agent peut-il changer de modèle | **Non** | Oui, parmi les candidats |
| Modèles distants | Utilisables si configurés | **Refusés** tant que l'interrupteur est fermé |
| Profil de données à jour exigé | Non | **Oui** |
| Le choix est-il vérifié au lancement | Il doit correspondre au modèle retenu | Il doit figurer parmi les candidats |
| Trace enregistrée avec le résultat | Source « Manuel » | Base du classement, ressources, raisons, mesures |

### Les cinq verdicts de ressources

| Verdict | Condition | Rang au classement |
|---|---|---|
| **Confortable** | Mémoire disponible ≥ 2 × le besoin | 4 (le meilleur) |
| **Distant** | Modèle chez un fournisseur externe | 3 |
| **Contraint** | Entre le besoin majoré de 20 % et le double | 2 |
| **Inconnu** | Mémoire non mesurable | 1 |
| **Insuffisant** | Mémoire disponible < besoin + 20 % | 0 — **éliminé** |

---

## Encadrés

> **ℹ À placer en tête de page — Ce que « Auto » veut dire ici.**
> Auto ne décide pas seul : il élimine les modèles qui ne peuvent pas faire le travail sur vos données et votre machine, classe ceux qui restent, et n'en garde que cinq. Le choix final dans cette liste courte reste le vôtre, ou celui de l'agent quand vous lui confiez le travail. En contrepartie, un modèle qui ne convient pas ne peut plus être lancé par erreur.

> **⚠ À placer dans « Les éliminations » — Le niveau de confiance n'est jamais arrondi.**
> Si vous demandez une fourchette à 85 %, un modèle qui ne sait produire que 60 % ou 80 % est écarté — il n'est pas utilisé « au plus proche ». C'est délibéré : une fourchette annoncée à 85 % qui en vaut 80 est une information fausse, et rien à l'écran ne la distinguerait d'une information juste.

> **ℹ À placer dans « Le classement » — Deux classements très différents.**
> Tant qu'aucun modèle n'a été mesuré sur vos données passées, le classement ne parle que de compatibilité et de mémoire disponible : il dit « ce modèle peut faire le travail », jamais « ce modèle prévoit mieux ». Pour obtenir un classement fondé sur la qualité, il faut lancer une évaluation — voir `08-forecast/evaluation-et-comparaison.md`.

> **ℹ À placer dans « Les modèles distants » — Le local est le choix par défaut.**
> En mode Auto, les modèles distants sont écartés tant que vous n'avez pas ouvert « Autoriser les modèles cloud ». Rien ne part vers un service externe sans ce geste.

> **⚠ À placer dans « Le ticket » — Une sélection automatique se périme.**
> Elle vaut trente minutes, et pour ces données-là. Modifier le fichier ou attendre trop longtemps invalide le choix, qui doit être refait. C'est ce qui empêche qu'un modèle retenu pour un jeu de données soit appliqué à un autre sans que personne le remarque.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Aucun modèle Forecast sélectionné » | Mode Manuel, aucun modèle retenu | Ouvrir le sélecteur et en choisir un |
| « Le modèle demandé ne correspond pas à la sélection manuelle » | L'agent a proposé un autre modèle que celui imposé | Passer en **Auto**, ou changer le modèle retenu |
| « Le modèle ne fait pas partie des candidats Auto » | Le modèle choisi a été éliminé par un des filtres | Lire le motif d'exclusion ; souvent un niveau de confiance, un horizon ou une fréquence non pris en charge |
| « Les modèles cloud ne sont pas autorisés en mode Auto » | L'interrupteur est fermé | Ouvrir **« Autoriser les modèles cloud »**, ou choisir un modèle local |
| « Profil Forecast obsolète : relancer forecast_data_audit » | Le profil de données date d'une version antérieure | Relancer le calcul depuis le fichier, ce qui refait l'audit |
| « Sélection Auto expirée ou invalide » | Plus de 30 minutes écoulées, données modifiées, ou ticket déjà utilisé | Refaire la sélection |
| « Ressources insuffisantes pour ce modèle » | La mémoire disponible ne couvre pas le besoin majoré de 20 % | Fermer des applications, ou prendre un modèle plus léger |
| « Ressources indisponibles pour ce modèle » | Mémoire non mesurable **et** modèle demandant plus de 2 048 Mo | Idem ; Beaver refuse plutôt que de parier |
| Mon modèle habituel n'apparaît plus en Auto | Un des filtres l'écarte pour **ces** données | Changer d'horizon, de fréquence ou de confiance, ou repasser en Manuel |
| Le premier candidat n'est pas marqué « recommandé » | Il a été mesuré et ne bat pas la référence simple | Comportement voulu : le mot est réservé |

---

## Renvois

- `08-forecast/vue-densemble.md` — où se trouve le sélecteur et à quel moment du parcours il intervient
- `08-forecast/donnees-et-audit.md` — le profil de données et son empreinte, dont dépend toute sélection automatique
- `08-forecast/evaluation-et-comparaison.md` — obtenir un classement fondé sur des mesures et non sur des capacités
- `08-forecast/modeles-locaux.md` — le catalogue local, les tailles et l'installation
- `08-forecast/modele-cloud-timegpt.md` — le modèle distant et sa clé API
- `06-modeles/materiel-et-vram.md` — la mesure de la mémoire de la machine, côté modèles de langage
- `11-securite/vault-et-cles-api.md` — où vit la clé du fournisseur distant

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **Le mot « Auto » promet plus que ce que le mode fait depuis l'écran.** Un lecteur attend un choix automatique ; le code construit une liste et vérifie le choix (`commands/forecast.rs:39-58`). Le résultat est même enregistré comme un **choix explicite de l'utilisateur**, avec la raison `user_requested` (`:42-46`). Question pour l'équipe : la page décrit-elle honnêtement ce comportement — recommandation retenue ici — ou faut-il d'abord revoir le libellé ? La formulation proposée : « Auto restreint et classe ; vous choisissez dans la liste courte. »
2. **Les motifs d'exclusion et les raisons de choix ne sont pas traduits.** `unknown_model`, `cloud_not_allowed`, `confidence_unsupported`, `resources_insufficient`, `top_backtest`, `resource_fit`… sont des identifiants techniques renvoyés tels quels (`auto_selection_candidate.rs`, `validation_selection.rs:3-12`). Aucune clé correspondante dans `fr.json`. Ils sont destinés à l'agent, qui les reformule ; mais si l'écran doit un jour les montrer, il faudra les libellés en sept langues.
3. **`CLAUDE.md` cite encore `forecast-selected-model.json`** comme fichier de la sélection. Le fichier qui fait autorité est **`forecast-selection-policy.json`** ; l'ancien n'est plus lu qu'une fois, pour migration (`selection_policy.rs:77-83`, `:93-107`). Correction à faire dans la documentation interne du dépôt.
4. **Le réglage de sélection est global, alors que l'analyse est propre à une conversation.** Changer le mode dans une conversation le change partout (`selection_policy.rs:77-79`, un seul fichier). C'est cohérent avec un réglage de préférence, mais surprenant à côté d'un panneau dont tout le reste est par conversation. À vérifier que c'est bien voulu avant de l'écrire comme une décision.

**Non vérifié — hors de portée d'une lecture du code**

5. **Le détail de la comparaison multi-mesures** du classement `rolling_backtest` : cette page s'arrête à « précision, qualité de la fourchette, durée, mémoire » (`auto_selection_rank.rs:23-34`). L'ordre exact et les pondérations vivent dans `evaluation/ranking.rs`, non lu ici. À traiter dans `08-forecast/evaluation-et-comparaison.md`, qui fait autorité sur ce point.
6. **Le comportement quand la mémoire graphique n'est pas mesurable** — cas d'un pilote absent ou d'une machine virtuelle. Le code retombe sur « Inconnu » et n'élimine qu'au-dessus de 2 048 Mo (`auto_selection_candidate.rs:45-47`), mais aucun essai n'a été fait sur une telle machine.
7. **Le comportement de l'agent quand le modèle demandé a été exclu.** Le code lui donne la consigne de l'expliquer sans remplacement silencieux (`tool_dispatcher_forecast_models.rs:132`), mais c'est une consigne, pas une contrainte : elle dépend du modèle de langage utilisé. À ne pas présenter comme une garantie.

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

8. Le bloc **« Sélection »** dans la liste déroulante du sélecteur de modèle : sa position, la lisibilité des deux boutons, et l'apparition de l'interrupteur « Autoriser les modèles cloud » quand on passe en Auto.
9. Ce que montre la liste des modèles **en mode Auto** : les candidats retenus sont-ils distingués des autres à l'écran, ou la liste reste-t-elle celle du catalogue complet ?
10. L'état du bouton pendant le chargement de la politique (`forecast-model-selector.tsx:151-152`, `aria-busy`), qui désactive le sélecteur le temps de lire le fichier.
11. Tous les messages de refus de cette page ne recevront pas de capture : les provoquer suppose de fabriquer une machine sans mémoire, un ticket expiré ou un catalogue incomplet. Ils sont documentés d'après le code, avec leur source.
