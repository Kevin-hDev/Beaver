# Dépannage — Forecast

**Emplacement site** — Référence › Dépannage › Forecast
**Répond à** — « Mon modèle de prévision ne s'installe pas, mes données sont refusées, ou le calcul échoue : qu'est-ce que je fais ? »
**Sources** — `src-tauri/src/commands/forecast.rs`, `commands/forecast_models.rs` ; `src-tauri/src/services/forecast/` (`limits.rs`, `validation.rs`, `input_data.rs`, `file_input.rs`, `hardware_profile.rs`, `client_nixtla.rs`, `client_nixtla_retry.rs`, `client_http.rs`, `sidecar.rs`, `sidecar_spawn.rs`, `sidecar_process_env.rs`, `sidecar_runtime.rs`, `sidecar_runtime_install.rs`, `storage.rs`, `storage_index.rs`, `data_quality/audit.rs`, `data_quality/profile.rs`, `data_quality/types.rs`, `model_manager/` : `mod.rs`, `install.rs`, `install_plan.rs`, `download.rs`, `download_io.rs`, `model_receipt.rs`, `smoke.rs`, `runtime_install.rs`, `uninstall.rs`) ; `src-tauri/src/services/model_downloads.rs`, `model_downloads_types.rs` ; `src-tauri/src/services/agent_local/tool_dispatcher_forecast_execute.rs`, `tool_dispatcher_forecast_error.rs` ; `src-tauri/src/storage_migration_files.rs` ; `src/components/forecast/forecast-panel.tsx`, `forecast-errors.ts`, `use-forecast-result.ts`, `forecast-model-readiness.ts`, `model-browser/model-install-btn.tsx`, `model-browser/model-specs.tsx`, `workbench/forecast-workbench-data.tsx` ; `src/components/settings/forecast-settings-models.tsx` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026. Aucune panne n'a été provoquée sur une machine : les enchaînements décrits viennent de la lecture du code, et les points d'affichage non observés sont listés en fin de fichier.

---

## Avertissement au rédacteur

Trois règles pour cette page, chacune tirée d'un constat du code.

**1. Le moteur de Forecast a des messages précis. L'utilisateur ne les voit presque jamais.**
Le code Rust renvoie une trentaine de phrases distinctes — « Modèle non installé », « Ressources insuffisantes pour ce modèle », « Clé API Nixtla non configurée ». Le panneau Forecast les remplace toutes par une seule (`src/components/forecast/forecast-errors.ts:3-10`). **Ne jamais écrire sur le site que l'utilisateur verra une de ces phrases**, sauf à la place où elle apparaît vraiment : dans un résultat d'outil, quand c'est l'agent qui a lancé la prévision.

**2. Les briefs de la section 08 Forecast font autorité sur le détail.** Cette page-ci est organisée par symptôme et renvoie ; elle ne réexplique ni le catalogue de modèles, ni les vingt-deux contrôles de qualité des données, ni la sélection automatique.

**3. Plusieurs échecs n'ont aucune issue prévue par le code.** Ils sont rassemblés dans la dernière section, et il faut le dire tel quel.

**Le vocabulaire du moteur est en français, écrit en dur dans le code Rust**, sans passer par les traductions. Un utilisateur qui a mis l'application en anglais, en japonais ou en allemand reçoit ces phrases-là en français dès qu'il passe par l'agent. Voir « Anomalies relevées », point 8.

---

## Plan de page proposé

1. Avant de chercher : les deux endroits où le message est différent
2. Le modèle de prévision ne s'installe pas
3. L'installation semble bloquée
4. Le modèle est installé, mais Beaver affiche « Préparation requise »
5. « Ressources insuffisantes » : la mémoire
6. Les données sont refusées
7. « Le calcul a échoué » : la liste des causes réelles
8. TimeGPT-2 : clé, quota, panne
9. Le moteur local ne démarre pas
10. « Le stockage Forecast est plein »
11. Ce qui n'a pas de solution

---

## Contenu

### 1. Avant de chercher : les deux endroits où le message est différent

Le même échec produit deux textes très différents selon la façon dont la prévision a été lancée. C'est le premier tri à faire, et il évite la moitié des recherches inutiles.

**Depuis le panneau Forecast** (import d'un fichier, puis bouton **Lancer le forecast**), il n'existe que trois messages possibles :

| Ce qui s'affiche | Quand | Source |
|---|---|---|
| « Import impossible. Vérifie le format du fichier. » | Le fichier n'a pas pu être lu | `forecast-panel.tsx:95` ; texte `fr.json:2480` |
| « Le calcul a échoué. Vérifie les colonnes, le modèle et la clé API. » | **Toute** panne pendant le calcul | `forecast-panel.tsx:125` → `forecast-errors.ts:9` ; texte `fr.json:2481` |
| « Le stockage Forecast est plein. Supprime d'anciennes analyses puis réessaie. » | Le seul cas particulier reconnu | `forecast-errors.ts:7-8` ; texte `fr.json:2482` |

La deuxième phrase couvre une trentaine de causes distinctes. Elle nomme trois pistes — les colonnes, le modèle, la clé API — mais **elle ne dit pas laquelle est en cause**, et il arrive que la cause ne soit aucune des trois.

**Depuis une conversation avec l'agent** (« fais-moi une prévision sur ce fichier »), le message exact du moteur est renvoyé tel quel dans le résultat de l'outil, dans un champ `error`, accompagné de l'étape suivante suggérée (`tool_dispatcher_forecast_error.rs:67-76`).

**Le conseil pratique à écrire sur le site, et il est utile :** quand le panneau affiche « Le calcul a échoué » sans qu'on comprenne pourquoi, **redemander la même prévision à l'agent** fait apparaître la vraie phrase.

Les écrans de l'espace Forecast — Données, Scénarios, Comparaisons, Notes, Évaluation — suivent la même règle : ils affichent un message traduit et générique, jamais celui du moteur (`use-forecast-result.ts:25` pour la section Données, `sections/forecast-scenarios.tsx:54` pour les scénarios ; chaque section a sa propre clé `loadFailed`, `fr.json:2087-2098`, `:2120`, `:2145-2146`, `:2307-2310`, `:2328`, `:2361-2364`, `:2367`).

### 2. Le modèle de prévision ne s'installe pas

**Ce qui s'affiche**, à côté du bouton d'installation, est unique : **« Le téléchargement a échoué »** (`model-install-btn.tsx:116-118` ; texte `fr.json:1389`). Le moteur ne transmet qu'un seul code d'échec, `model-download-failed`, quelle que soit la cause réelle (`services/model_downloads.rs:210`).

Un second message existe, et il ne veut pas dire la même chose : **« Impossible d'ajouter ce modèle à la file d'attente. »** (`model-install-btn.tsx:119-123` ; texte `fr.json:604`). Celui-là signifie que l'installation n'a **pas** démarré — la file est pleine, seize téléchargements en attente au maximum (`model_downloads_types.rs:3`).

**Les causes réelles, dans l'ordre où elles surviennent :**

| Cause | Message interne | Source |
|---|---|---|
| Les fichiers du moteur livrés avec l'application sont introuvables ou incomplets | « Ressources Forecast incomplètes » | `storage_migration_files.rs:31`, appelé avant toute installation `model_downloads.rs:180-184` |
| **Python 3.12 n'est pas installé sur la machine** | « Runtime Python compatible indisponible » | `sidecar_runtime_install.rs:130-138` |
| Le téléchargement des poids n'aboutit pas | « Téléchargement du modèle impossible » | `download.rs:121-123` |
| Le téléchargement dépasse un délai | « Téléchargement du modèle expiré », « Téléchargement expiré » | `download.rs:120` (30 s pour la réponse) ; `download_io.rs:88` (60 s entre deux morceaux) |
| Le fichier reçu ne correspond pas à son empreinte | « Téléchargement du modèle invalide » | `download_io.rs:113-115` |
| Le disque est plein | « Écriture du téléchargement impossible » | `download_io.rs:101-103` |
| L'installation des bibliothèques Python échoue | « Installation du moteur Forecast impossible » | `sidecar_runtime_install.rs:67-71`, `:115-119` |
| Le test final du modèle échoue ou dépasse quinze minutes | « Validation du modèle Forecast impossible », « Validation du modèle Forecast expirée » | `smoke.rs:76-84` ; délai `smoke.rs:7` |

**Le cas Python mérite un paragraphe à lui seul sur le site.** Beaver cherche successivement `python3.12`, `python3` puis `python` dans le chemin du système, et **n'accepte que la version 3.12 exactement** : ni 3.11, ni 3.13 (`sidecar_runtime_install.rs:131-138`, `:154-156`, avec un test qui verrouille ce comportement, `:170-175`). C'est la seule dépendance externe des modèles de prévision locaux, et l'application ne l'installe pas elle-même.

**Le cas du réseau d'entreprise, à mentionner.** Les poids viennent uniquement de `huggingface.co` et des domaines liés (`download.rs:76`, `:85-95`), et les bibliothèques Python uniquement de `https://pypi.org/simple`, avec vérification obligatoire des empreintes et refus de tout paquet à compiler (`sidecar_runtime_install.rs:54-65`). Un miroir interne, un proxy qui réécrit les réponses ou un blocage de ces deux domaines fait échouer l'installation sans qu'aucun réglage ne permette de la détourner.

**Il n'y a pas de reprise.** Une installation interrompue ou en échec repart de zéro : le dossier de travail temporaire est supprimé à la première erreur (`install.rs:58-61`, `:98-102`), et chaque fichier en cours d'écriture porte l'extension `.part`, effacée en cas d'échec (`download_io.rs:30`, `:43-46`). Rien de ce qui a déjà été téléchargé n'est conservé.

### 3. L'installation semble bloquée

C'est le symptôme le plus courant, et **ce n'est pas une panne**. La barre de progression ne mesure pas une seule opération mais quatre, et la troisième est longue et silencieuse.

| Ce que montre la barre | Ce qui se passe réellement | Source |
|---|---|---|
| 0 à 70 % | Téléchargement des poids du modèle | `install.rs:14`, `:54-57` |
| 72 % | Création de l'environnement Python | `runtime_install.rs:35` |
| 76 % | Préparation de l'installateur de bibliothèques | `runtime_install.rs:36` |
| **80 %** | **Installation des bibliothèques Python — l'étape la plus longue** | `runtime_install.rs:37` |
| 98 % | Vérification du moteur | `runtime_install.rs:38` |
| 99 % | Mise en place définitive du modèle | `install.rs:162-167` |

La phase affichée à côté du pourcentage porte les libellés **Téléchargement**, **Préparation du moteur**, **Installation**, **Terminé** (`fr.json:596-602`).

**Le blocage apparent à 80 % est l'installation des bibliothèques**, qui peut durer plusieurs minutes sans que le pourcentage bouge : ce n'est pas une progression continue, ce sont quatre paliers. À la toute fin, un test réel du modèle est exécuté, avec un délai maximal de **quinze minutes** (`smoke.rs:7`) — un échec à ce moment-là annule l'installation entière.

**Annuler est possible** à tout instant, par le bouton **Annuler** ou par la touche `Échap` quand l'écran des modèles est au premier plan (`model-install-btn.tsx:77-84`). L'annulation supprime ce qui a été téléchargé.

### 4. Le modèle est installé, mais Beaver affiche « Préparation requise »

Beaver distingue quatre états internes pour un modèle local (`model_manager/mod.rs:19-25`, `:75-92`) :

| État interne | Ce qu'il signifie | Ce que l'utilisateur lit |
|---|---|---|
| `not_installed` | Les poids ne sont pas là | **« Non installé »** (`fr.json:2686`) |
| `invalid` | Les fichiers sont là mais n'ont pas la taille attendue | **« Préparation requise »** (`fr.json:2670`) |
| `update_required` | Le reçu, le moteur ou le test de validité n'est plus à jour | **« Mise à jour requise »** (`fr.json:2671`) |
| `ready` | Les cinq conditions sont réunies | **« Installé »** (`fr.json:2669`) |

Le passage à `ready` exige cinq choses simultanément (`model_manager/mod.rs:63-92`) : le dossier du modèle et son marqueur `.complete` (`:105-108`), des fichiers à la taille attendue, un reçu `.artifacts-v1.json` conforme au catalogue de la version installée, un moteur Python prêt pour la famille du modèle, et une preuve de test de fumée à jour.

**Conséquence à écrire clairement :** après une mise à jour de Beaver qui change la liste des bibliothèques Python, un modèle déjà installé bascule en **Mise à jour requise**, et le bouton affiche **Préparer** au lieu d'Installer (`model-install-btn.tsx:114`). **Les poids ne sont pas retéléchargés** : Beaver reconnaît qu'ils sont valides et ne refait que le moteur et la validation (`install_plan.rs:37-44` — état `RuntimeOnly` ou `Validate` plutôt que `Full`).

**Un écart d'affichage résolu ici** : l'état `invalid` — fichiers présents mais de mauvaise taille, typiquement après une copie interrompue — n'a pas de libellé propre. Il retombe sur **« Préparation requise »** (`model-specs.tsx:174-179` avec `forecast-model-readiness.ts:11-20`). Le bouton **Préparer** relance alors bien un téléchargement complet, puisque le plan calculé est `Full` (`install_plan.rs:42`). L'affichage est trompeur, l'action est la bonne.

### 5. « Ressources insuffisantes » : la mémoire

Beaver mesure la mémoire disponible avant de lancer un modèle local et refuse plutôt que de laisser la machine s'effondrer.

**Deux refus distincts** (`hardware_profile.rs:103-114`) :

- **« Ressources insuffisantes pour ce modèle »** — la mémoire a été mesurée, et elle ne couvre pas le besoin du modèle **majoré de 20 %** (`hardware_profile.rs:5`, `:120-123`).
- **« Ressources indisponibles pour ce modèle »** — la mémoire n'a **pas** pu être mesurée et le modèle demande plus de **2 048 Mo** (`limits.rs:38`). En dessous de ce seuil, Beaver laisse passer.

Le détail du calcul — mémoire vidéo contre mémoire vive, les cinq verdicts — est dans `08-forecast/selection-du-modele.md`, qui fait autorité.

**Le point de dépannage propre à cette page, et il est important : ce contrôle n'a lieu qu'au lancement d'une prévision, jamais à l'installation.** La chaîne d'installation n'appelle nulle part la vérification des ressources (`model_manager/install.rs` en entier). **On peut donc installer plusieurs gigaoctets de modèle, puis découvrir au premier calcul que la machine ne peut pas le faire tourner.** Écrire ce point sur le site évite un téléchargement inutile ; renvoyer à `06-modeles/materiel-et-vram.md` pour choisir en amont.

Et rappeler que la mémoire mesurée est la mémoire **disponible au moment du calcul** (`hardware_profile.rs:37-46`) : fermer un navigateur peut suffire à faire passer un modèle qui était refusé cinq minutes plus tôt.

### 6. Les données sont refusées

Deux moments distincts, deux messages différents.

**À l'import du fichier** — « Import impossible. Vérifie le format du fichier. » (`fr.json:2480`). Les causes vérifiées :

| Cause | Limite | Source |
|---|---|---|
| Extension non reconnue | `csv`, `tsv`, `xlsx`, `xls`, `ods`, `xlsm` seulement | `file_input.rs:40-53` |
| Fichier trop lourd | **50 Mo** | `limits.rs:2` ; contrôle `file_input.rs:30-32` |
| Fichier introuvable, ou chemin qui ne désigne pas un fichier | — | `file_input.rs:24-29` |

**À l'audit, juste avant le calcul** — le contrôle de qualité examine chaque ligne et produit une liste d'anomalies. Une seule anomalie de gravité **Erreur** suffit à bloquer le calcul (`data_quality/types.rs:57-62`, appelé depuis `input_data.rs:42-45`). Les limites de volume :

| Limite | Valeur | Source |
|---|---|---|
| Lignes lues dans un fichier | **5 000** | `limits.rs:3` |
| Données transmises en une fois | **5 Mo** | `limits.rs:1` ; contrôle `data_quality/audit.rs:14-16` |
| Colonnes par ligne | **256** | `limits.rs:4` |
| Caractères dans une cellule | **32 768** | `limits.rs:5` |
| Séries distinctes | **256** | `limits.rs:9` |
| Séries × horizon | **100 000** | `limits.rs:11`, `:64-75` |
| Variables de contexte | **64** | `limits.rs:7` |

**La liste complète des vingt-deux anomalies et de leurs messages est dans `08-forecast/donnees-et-audit.md`**, qui fait autorité et donne le remède colonne par colonne. Cette page ne la duplique pas.

**Ce qu'il faut ajouter ici, parce que ça ne se voit qu'à l'écran :** la section **Données** de l'espace Forecast affiche le nombre d'anomalies et leur gravité — « Erreur bloquante · 3 », « Avertissement · 12 » — **sans jamais nommer l'anomalie** (`workbench/forecast-workbench-data.tsx:85-95` ; libellés `fr.json:2136-2137`). Les codes internes n'ont aucune traduction. L'utilisateur sait donc qu'il y a trois problèmes bloquants, et rien de plus.

**Le détour qui marche**, et qui vaut d'être écrit : demander l'audit à l'agent. Le message précis — « Dates dupliquées », « Colonne cible non numérique », « Fréquence incohérente » (`data_quality/types.rs:92-108`) — remonte alors dans le résultat de l'outil.

### 7. « Le calcul a échoué » : la liste des causes réelles

Le parcours d'un calcul lancé depuis le panneau, dans l'ordre exact du code (`commands/forecast.rs:28-90`). Chaque ligne produit la même phrase à l'écran.

| Étape | Ce qui peut échouer | Message interne |
|---|---|---|
| Lecture du fichier | Fichier illisible ou format refusé | « Impossible de lire les données source » (`:33-35`) |
| Contrôle de la demande | Horizon nul ou au-delà de la limite du modèle | « Horizon invalide » (`validation.rs:61-63`) |
| | Fréquence hors liste | « Fréquence invalide » (`validation.rs:131-133`) ; liste `validation.rs:3-5` |
| | Fréquence non gérée par ce modèle | « Fréquence non supportée par ce moteur » (`validation.rs:64-66`) |
| | Niveau de confiance non géré | « Niveau de confiance non supporté par ce moteur » (`validation.rs:89-97`) |
| | Colonne série alors que le modèle ne sait pas | « Multi-séries non supporté par ce moteur » (`validation.rs:54-56`) |
| | Colonnes de contexte alors que le modèle ne sait pas | « Variables de contexte non supportées par ce moteur » (`validation.rs:57-59`) |
| | Colonnes en double, ou cible = date | « Colonnes invalides », « Covariables invalides » (`validation.rs:102-104`, `:117-123`) |
| Audit des données | Une anomalie bloquante | voir section 6 |
| Contexte futur | Lignes futures connues + colonnes de contexte, modèle incapable | « Variables futures non supportées par ce moteur » (`commands/forecast.rs:115-120`) |
| Choix du moteur | Modèle absent du catalogue | « Modèle inconnu » (`:59`) |
| | Modèle sans moteur exécutable | « Moteur indisponible » (`:60`, `:62-64`) |
| Modèle distant | Aucune clé enregistrée | « Clé API Nixtla non configurée » (`:67-68`) |
| Modèle local | Modèle pas totalement prêt | « Modèle non installé » (`:73-74`) |
| | Mémoire insuffisante | voir section 5 (`:76`) |
| | Le moteur ne démarre pas | « Impossible de démarrer le service de prédiction » (`:78-80`) |
| Calcul | Le moteur répond mal, ou pas | « Erreur du service de prédiction » (`:71`, `:89`) |
| Enregistrement | Plus de place | voir section 10 (`:106`) |

**Le tri le plus efficace à proposer au lecteur**, dans cet ordre :

1. **Le modèle choisi est-il marqué « Installé » ?** Sinon, c'est là que ça s'arrête (sections 2 à 4).
2. **La section Données signale-t-elle une erreur bloquante ?** Si oui, c'est là (section 6).
3. **Le modèle est-il distant ?** Alors vérifier la clé (section 8).
4. **Sinon, relancer la même demande via l'agent** pour lire la phrase exacte (section 1).

### 8. TimeGPT-2 : clé, quota, panne

**Sans clé enregistrée**, l'écran des modèles affiche **« Clé API non configurée »** à la place de la mention « Cloud (clé requise) » (`settings/forecast-settings-models.tsx:152` ; `model-browser/model-specs.tsx:169-172` ; textes `fr.json:2690-2691`), et le calcul refuse avec « Clé API Nixtla non configurée ».

**Avec une clé**, l'appel part vers `https://api.nixtla.io/v2/forecast` (`client_nixtla.rs:11`, `:84-86`) avec :

- un délai de **60 secondes** par tentative (`client_nixtla_retry.rs:8`) ;
- **trois tentatives au maximum**, avec des pauses de **2 puis 4 secondes** (`client_nixtla_retry.rs:9`, `:17-32`) ;
- des reprises réservées à quatre situations : **429** (trop de requêtes), **502**, **503** et **504** (`client_nixtla_retry.rs:50-58`). Une clé refusée ou un quota épuisé ne sont **pas** retentés : l'échec est immédiat.

**Toutes les issues produisent la même phrase : « Erreur du service de prédiction »** (`client_nixtla.rs:30` ; `client_nixtla_retry.rs:26`, `:28`, `:32` ; réponse illisible `client_http.rs:9`, `:19-26`). Clé invalide, quota dépassé, service en panne, réponse malformée : rien ne les distingue.

**Le seul endroit qui porte l'information est le journal technique de Beaver**, qui écrit le code HTTP renvoyé, sans le corps de la réponse (`client_nixtla.rs:29`, ligne `[nixtla] erreur {status}`). À écrire sur le site avec le renvoi vers la page qui explique où lire ce journal.

**Ce que l'utilisateur peut faire**, faute de mieux : vérifier la validité de la clé depuis l'écran des clés API — un bouton de test existe et interroge le fournisseur (voir `11-securite/vault-et-cles-api.md`) — puis vérifier l'état du service chez Nixtla. Si le test de clé passe et que le calcul échoue quand même, c'est le quota ou le service.

Le détail de ce qui sort de la machine, des variantes du modèle et de leurs réglages est dans `08-forecast/modele-cloud-timegpt.md`.

### 9. Le moteur local ne démarre pas

Entre le clic sur **Lancer le forecast** et le calcul, Beaver démarre un petit service Python local. Toutes ses pannes deviennent **« Impossible de démarrer le service de prédiction »** côté commande, puis « Le calcul a échoué » à l'écran.

Les causes, avec leur message interne (`sidecar.rs:125-192`) :

| Cause | Message interne | Source |
|---|---|---|
| Le service précédent refuse de s'arrêter | « Sidecar Forecast indisponible » | `sidecar.rs:137-139` |
| Le fichier `server.py` est absent | « Sidecar Python non installé » | `sidecar.rs:143-144` |
| Le moteur Python de cette famille n'est pas prêt | « Moteur Forecast non préparé » | `sidecar_spawn.rs:49-52` |
| Le processus ne se lance pas | « Impossible de lancer le sidecar Forecast » | `sidecar_spawn.rs:94` |
| Le service ne répond pas à temps | « Sidecar Forecast: timeout au démarrage » | `sidecar_spawn.rs:119` |

**Le délai de démarrage est de trente secondes** : soixante tentatives espacées d'une demi-seconde (`sidecar_spawn.rs:105-106`). Sur une machine lente ou pour un gros modèle, un premier lancement peut donc échouer alors que rien n'est cassé — **réessayer est une action utile ici**, contrairement à la plupart des autres échecs de cette page.

Deux précisions à donner, parce qu'elles rassurent :

- **Le service n'écoute que la machine locale**, sur `127.0.0.1` et un port libre choisi au lancement (`sidecar_spawn.rs:112`), protégé par un jeton tiré au hasard à chaque démarrage (`sidecar.rs:148`).
- **Il est forcé hors ligne** : les variables `HF_HUB_OFFLINE`, `TRANSFORMERS_OFFLINE`, `HF_HUB_DISABLE_TELEMETRY` et `DO_NOT_TRACK` sont posées à chaque lancement, et un test verrouille ce comportement (`sidecar_process_env.rs:28-36`, test `:45-56`). Un modèle installé calcule sans réseau, et ne peut pas aller chercher un fichier manquant en ligne — ce qui explique qu'une installation incomplète échoue au lieu de se réparer toute seule.

### 10. « Le stockage Forecast est plein »

C'est le seul message précis que le panneau sait afficher : **« Le stockage Forecast est plein. Supprime d'anciennes analyses puis réessaie. »** (`fr.json:2482`).

Le mécanisme réel est plus subtil que la phrase (`storage_index.rs:173-191`) :

- la limite est de **500 analyses** au total (`limits.rs:19`) ;
- une fois cette limite atteinte, Beaver **supprime automatiquement la plus ancienne analyse du même espace de travail** et enregistre la nouvelle ;
- **le message n'apparaît que si l'espace de travail courant ne contient aucune analyse à supprimer** — c'est-à-dire quand les 500 analyses appartiennent toutes à d'autres projets ou conversations.

**Conséquence à écrire, parce qu'elle est contre-intuitive :** le conseil du message — supprimer d'anciennes analyses — est juste, mais il faut les supprimer **dans les autres espaces de travail**, pas dans celui où le message apparaît, puisque celui-ci est vide de toute analyse.

Une seconde limite existe, séparée : une analyse individuelle ne peut pas dépasser **64 Mo** une fois enregistrée, sans quoi elle est refusée avec « Analyse Forecast trop volumineuse » (`storage.rs:54-56` ; `limits.rs:15`). Elle se rencontre sur des prévisions à très nombreuses séries.

### 11. Ce qui n'a pas de solution

À écrire sans l'adoucir. Dans ces cas, le code ne prévoit aucune issue.

- **Distinguer une clé TimeGPT refusée d'un quota dépassé.** Le message est le même et rien dans l'interface ne les sépare. Seul le journal technique porte le code HTTP.
- **Savoir quelle anomalie de données bloque, depuis l'écran.** Le compteur donne le nombre, jamais le nom. Le détour par l'agent est le seul chemin.
- **Savoir pourquoi une installation a échoué.** Un seul code d'échec pour une quinzaine de causes ; ni le message interne ni le journal ne sont exposés dans l'écran des modèles.
- **Reprendre une installation interrompue.** Tout est supprimé et retéléchargé.
- **Faire tourner un modèle local sans Python 3.12.** Aucun repli, aucun réglage, aucun moteur intégré de secours.
- **Utiliser un miroir de paquets interne.** Les adresses de `huggingface.co` et de `pypi.org` sont écrites dans le code et ne sont pas paramétrables.
- **Installer un modèle sur une machine sans Internet.** Les poids et les bibliothèques sont téléchargés à l'installation ; seule l'exécution, ensuite, fonctionne hors ligne.

---

## Tableaux

### Le message affiché et sa cause réelle

À reprendre tel quel sur le site : c'est la table d'entrée de la page.

| Ce que vous lisez | Où | Ce que ça veut vraiment dire |
|---|---|---|
| « Import impossible. Vérifie le format du fichier. » | Panneau Forecast | Extension refusée, fichier au-delà de 50 Mo, ou fichier illisible |
| « Le calcul a échoué. Vérifie les colonnes, le modèle et la clé API. » | Panneau Forecast | **Trente causes possibles** — voir la section 7 |
| « Le stockage Forecast est plein. Supprime d'anciennes analyses puis réessaie. » | Panneau Forecast | 500 analyses, dont aucune dans l'espace de travail courant |
| « Le téléchargement a échoué » | Écran des modèles | **Quinze causes possibles** — voir la section 2 |
| « Impossible d'ajouter ce modèle à la file d'attente. » | Écran des modèles | L'installation n'a pas démarré : 16 téléchargements déjà en attente |
| « Non installé » / « Préparation requise » / « Mise à jour requise » / « Installé » | Écran des modèles | Les quatre états — voir la section 4 |
| « Clé API non configurée » | Écran des modèles, modèle distant | Aucune clé Nixtla enregistrée |
| « Erreur bloquante · N » | Espace Forecast, section Données | N anomalies bloquantes, **sans leur nom** |
| « Impossible de charger les données de l'analyse. » | Espace Forecast, section Données | L'analyse n'a pas pu être relue sur le disque |

### Les délais qui comptent

| Opération | Délai | Source |
|---|---|---|
| Connexion pour télécharger un modèle | 15 s | `download.rs:63` |
| Première réponse du serveur de modèles | 30 s | `download.rs:16` |
| Entre deux morceaux de téléchargement | 60 s | `download_io.rs:13` |
| Test du modèle après installation | **15 minutes** | `smoke.rs:7` |
| Démarrage du moteur local | **30 s** (60 × 500 ms) | `sidecar_spawn.rs:105-106` |
| Une tentative vers TimeGPT | 60 s | `client_nixtla_retry.rs:8` |
| Reprises vers TimeGPT | 2 tentatives de plus, à 2 s puis 4 s | `client_nixtla_retry.rs:9` |

### Où vivent les fichiers de Forecast

Tous sous `~/.local/share/cl-go-dash/`, sur les trois systèmes.

| Dossier | Contenu | Source |
|---|---|---|
| `forecast-models/<modèle>/` | Poids d'un modèle installé, plus son marqueur `.complete` et son reçu `.artifacts-v1.json` | `model_manager/mod.rs:38-40`, `:105-108` ; `model_receipt.rs:11` |
| `forecast-models/.<modèle>.staging/` | Installation en cours ; supprimé en cas d'échec | `install.rs:51`, `:58-61` |
| `forecast-sidecar/` | Le service Python et les moteurs par famille | `model_manager/mod.rs:42-44` |
| `forecast-sidecar/.hf-cache/` | Cache local du chargeur de modèles, jamais alimenté en ligne | `sidecar_process_env.rs:16`, `:30-32` |
| `forecast-analyses/` | Les analyses enregistrées et leur index | `storage_index.rs` ; limite 500 |

---

## Encadrés

> **ℹ À placer en tête de page — Le message affiché ne nomme presque jamais la cause.**
> Le panneau Forecast n'a que trois messages pour une trentaine de pannes différentes. Quand « Le calcul a échoué » ne suffit pas, redemandez la même prévision à l'agent dans une conversation : il reçoit la phrase exacte du moteur et vous la montre.

> **⚠ À placer dans la section Installation — Les modèles de prévision locaux demandent Python 3.12.**
> Exactement 3.12 : ni 3.11, ni 3.13. Beaver ne l'installe pas et n'a aucun repli. Sans lui, l'installation échoue avec le message générique « Le téléchargement a échoué », qui ne dit pas que Python est en cause.

> **⚠ À placer dans la section Installation — Une installation interrompue repart de zéro.**
> Il n'y a pas de reprise : le dossier de travail est supprimé à la première erreur et tout est retéléchargé. Sur un modèle de plusieurs gigaoctets et une connexion instable, mieux vaut lancer l'installation à un moment où la machine ne sera pas coupée.

> **ℹ À placer dans la section Installation — La barre bloquée à 80 % n'est pas une panne.**
> C'est l'installation des bibliothèques Python. Elle peut durer plusieurs minutes sans que le pourcentage bouge, puis repart à 98 % et 99 %.

> **⚠ À placer dans la section Mémoire — Rien ne vérifie la mémoire avant de télécharger.**
> Le contrôle des ressources n'a lieu qu'au premier calcul. Vous pouvez donc installer plusieurs gigaoctets, puis lire « Le calcul a échoué » parce que la machine ne peut pas faire tourner ce modèle. Vérifiez la mémoire demandée sur la fiche du modèle **avant** d'installer.

> **ℹ À placer dans la section Moteur local — Un modèle installé calcule sans Internet.**
> Le service de prévision est démarré en mode hors ligne, ne parle qu'à votre machine et ne peut rien télécharger pendant un calcul. Seule l'installation a besoin du réseau.

---

## Pièges et erreurs fréquentes

Ce tableau ne reprend pas ceux de `08-forecast/` — il porte sur ce qui trompe **entre** les écrans.

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le calcul a échoué » sans qu'aucune des trois pistes citées ne s'applique | Le message est unique pour une trentaine de causes | Relancer la même demande via l'agent pour lire la vraie phrase |
| « Le téléchargement a échoué » alors que la connexion fonctionne | Le message couvre aussi Python absent, pip en échec, disque plein, test final raté | Vérifier d'abord que `python3.12` répond dans un terminal |
| L'installation reste à 80 % pendant plusieurs minutes | C'est l'installation des bibliothèques, pas le téléchargement | Attendre ; la barre repart à 98 % |
| Le modèle affiche **Préparation requise** alors qu'il vient d'être installé | Fichiers de mauvaise taille, ou moteur non préparé | Cliquer **Préparer** ; les poids valides ne sont pas retéléchargés |
| Un modèle installé refuse de calculer | La mémoire n'est vérifiée qu'au lancement | Fermer des applications, ou choisir un modèle plus léger |
| « Le stockage Forecast est plein » alors que ce projet ne contient aucune analyse | Les 500 analyses appartiennent à d'autres espaces de travail | Supprimer des analyses **dans les autres projets ou conversations** |
| La section Données dit « Erreur bloquante · 2 » sans en dire plus | Les codes d'anomalie ne sont pas traduits | Demander l'audit à l'agent ; ou consulter la liste des causes dans la page Données |
| Le calcul échoue une fois puis passe au second essai | Le moteur local a dépassé les 30 s de démarrage au premier lancement | Comportement attendu sur une machine lente |
| Beaver affiche des messages en français dans un résultat d'outil alors que l'application est en anglais | Les messages du moteur ne passent pas par les traductions | Aucun contournement ; voir « Anomalies relevées » |
| L'installation échoue sur un poste d'entreprise | `huggingface.co` ou `pypi.org` bloqués, ou proxy qui réécrit les réponses | Aucun réglage ne permet de changer ces adresses |

---

## Renvois

- `08-forecast/modeles-locaux.md` — le catalogue, les états d'un modèle, l'installation en détail
- `08-forecast/donnees-et-audit.md` — la liste complète des contrôles de données et leurs remèdes
- `08-forecast/modele-cloud-timegpt.md` — TimeGPT-2, ses variantes, ce qui sort de la machine
- `08-forecast/selection-du-modele.md` — les modes Manuel et Auto, les cinq verdicts de ressources
- `06-modeles/materiel-et-vram.md` — choisir un modèle que la machine peut faire tourner
- `11-securite/vault-et-cles-api.md` — enregistrer et tester une clé API
- `13-depannage/installation.md` — l'installation de l'application elle-même
- `13-depannage/faq.md` — les questions générales

---

## Points à confirmer

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

1. **Le message d'échec à côté du bouton d'installation** : sa position, sa persistance, et s'il disparaît quand on relance une installation (`model-install-btn.tsx:116-123`).
2. **La barre de progression aux paliers 70 → 72 → 76 → 80** : l'utilisateur voit-il le libellé de phase changer, ou seulement le pourcentage ?
3. **L'état `invalid` affiché « Préparation requise »** : à confirmer à l'écran, en abîmant volontairement un fichier de poids.
4. **La section Données quand une erreur bloquante existe** : le bandeau passe en `callout-warning` (`forecast-workbench-data.tsx:74-76`), à vérifier dans les deux thèmes.
5. **L'écran des modèles distants sans clé** : vérifier que « Clé API non configurée » est bien lisible et mène l'utilisateur vers l'écran des clés.

**Non vérifié — hors de portée d'une lecture du code**

6. **Le comportement quand Python 3.12 est présent mais amputé** — par exemple sans le module `venv`, courant sur certaines distributions Linux où il est livré séparément. Le code n'a qu'un message générique (`sidecar_runtime_install.rs:39-44`) ; ni testé, ni observé. Repris de `08-forecast/modeles-locaux.md`, toujours ouvert.
7. **Le comportement disque plein pendant une installation** n'a pas été provoqué. Le message interne est « Écriture du téléchargement impossible », mais l'utilisateur ne le voit pas : reste à savoir si l'échec est propre et si le dossier temporaire est bien nettoyé quand le disque ne permet même plus la suppression.
8. **Le délai de 30 secondes au démarrage du moteur** est-il suffisant pour les plus gros modèles du catalogue sur une machine modeste ? Aucune mesure. Si c'est trop court, le symptôme « échoue une fois, passe au second essai » deviendrait « échoue toujours ».
9. **Le comportement de TimeGPT quand la clé est valide mais le quota épuisé** n'a pas été provoqué faute de compte. Le code HTTP renvoyé par Nixtla dans ce cas n'est pas connu ; s'il s'agit d'un **429**, Beaver le retente trois fois avant d'abandonner, ce qui allonge l'attente sans aucune chance de succès.
10. **La traduction dans les six autres langues** des trois messages de `forecast.errors` (`fr.json:2479-2483`) n'a pas été relue.

**Écarts relevés — à arbitrer avant publication**

11. **Le message « Le calcul a échoué. Vérifie les colonnes, le modèle et la clé API. » nomme trois pistes dont aucune n'est en cause dans plusieurs des trente scénarios** — moteur qui ne démarre pas, mémoire insuffisante, stockage, moteur non préparé. Faut-il décrire le comportement réel sur le site, ou attendre une correction ? Question pour l'équipe.
12. **Le tutoiement des messages produit** (« Vérifie », « Supprime ») alors que ces fichiers vouvoient. Décision de ton à prendre une fois pour toute la documentation ; les messages doivent être **cités tels quels** pour que l'utilisateur reconnaisse sa phrase.

---

## Anomalies relevées

Constatées dans le code, **non corrigées** — à transmettre à l'équipe, pas à résoudre dans la documentation.

1. **Le panneau Forecast masque tous les messages du moteur.** Une trentaine de phrases distinctes, écrites et maintenues dans le code Rust, sont remplacées par une seule à l'affichage (`forecast-errors.ts:3-10` ; `forecast-panel.tsx:125`). L'information existe, elle est produite, et elle est jetée juste avant l'écran.

2. **Le chemin de l'agent et celui du panneau ne font pas le même contrôle.** Le panneau exige un modèle totalement prêt — `is_ready`, cinq conditions (`commands/forecast.rs:73`). L'agent se contente de la présence des poids — `is_installed` (`tool_dispatcher_forecast_execute.rs:70-72`). Conséquence : via l'agent, un modèle dont le moteur n'est pas préparé passe le contrôle et échoue plus loin, sur « Impossible de démarrer le service de prédiction » au lieu de « Modèle non installé ».

3. **Un seul code d'échec pour toute la chaîne d'installation.** `model-download-failed` couvre quinze causes distinctes (`model_downloads.rs:210`), y compris l'absence de Python, qui est la seule sur laquelle l'utilisateur peut agir.

4. **La liste des anomalies de données affiche leur gravité et leur nombre, jamais leur nom** (`forecast-workbench-data.tsx:85-95`). Les vingt-deux codes internes n'ont aucune clé de traduction. Déjà relevé dans `08-forecast/donnees-et-audit.md` ; confirmé ici.

5. **Aucun contrôle d'espace disque avant une installation de plusieurs gigaoctets.** Recherche infructueuse dans tout `services/forecast/`. Un disque plein se manifeste par « Écriture du téléchargement impossible », qui ne mentionne pas le disque, et l'utilisateur ne voit même pas cette phrase.

6. **Les ressources ne sont vérifiées qu'au lancement d'un calcul, jamais à l'installation** (`commands/forecast.rs:76` ; rien d'équivalent dans `model_manager/install.rs`). On peut installer un modèle que la machine ne pourra jamais faire tourner.

7. **TimeGPT : une seule phrase pour clé refusée, quota dépassé, service en panne et réponse illisible** (`client_nixtla.rs:30` ; `client_nixtla_retry.rs:26-32` ; `client_http.rs:9`). Le code HTTP est connu du programme et écrit dans le journal (`client_nixtla.rs:29`), mais n'est jamais transmis à l'interface. Déjà relevé dans `08-forecast/modele-cloud-timegpt.md`.

8. **Les messages du moteur sont écrits en dur en français dans le code Rust**, sans passer par le système de traduction — par exemple `install.rs:43`, `smoke.rs:79`, `sidecar_spawn.rs:119`, `validation.rs:19`. Ils ne sont pas visibles depuis le panneau, mais ils le sont dans un résultat d'outil : un utilisateur en anglais, en chinois ou en japonais reçoit alors une phrase en français.

9. **Le message « Le stockage Forecast est plein » conseille une action qui ne s'applique pas là où il s'affiche.** Il apparaît précisément quand l'espace de travail courant est vide d'analyses (`storage_index.rs:181-187`), alors qu'il invite à en supprimer.

10. **Le message d'erreur de test de modèle mentionne un « sidecar »**, terme technique jamais expliqué — « Sidecar Python non installé », « Sidecar Forecast: timeout au démarrage » (`sidecar.rs:144` ; `sidecar_spawn.rs:119`). Sans conséquence tant qu'ils restent masqués par le panneau ; visibles via l'agent.
