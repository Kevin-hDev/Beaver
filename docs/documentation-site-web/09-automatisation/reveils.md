# Les réveils programmés

**Emplacement site** — Automatisation › Les réveils
**Répond à** — « Comment faire travailler Beaver à heure fixe, et que se passe-t-il si mon ordinateur est éteint à ce moment-là ? »
**Sources** — `src-tauri/src/models/config.rs` ; `src-tauri/src/commands/heartbeat.rs`, `commands/heartbeat_validation.rs` ; `src-tauri/src/services/scheduler/` (`mod.rs`, `runtime.rs`, `runtime_decisions.rs`, `next_fire.rs`, `due.rs`, `fire.rs`, `fire_once.rs`, `in_flight.rs`, `agentic.rs`, `state.rs`, `work_supervision.rs`) ; `src-tauri/src/runtime_state.rs`, `src-tauri/src/app_events.rs`, `src-tauri/src/app_exit/cleanup.rs` ; `src-tauri/src/services/file_watcher.rs` ; `src-tauri/src/services/agent_local/session_store_create.rs`, `services/agent_local/session_index.rs` ; `src-tauri/src/services/provider_usage/types.rs` ; `src/hooks/use-wakeups.ts` ; `src/components/heartbeat/` (`heartbeat-tab.tsx`, `wakeup-list.tsx`, `wakeup-row.tsx`, `wakeup-details.tsx`, `new-wakeup-dialog.tsx`, `schedule-picker.tsx`, `wakeup-form-fields.tsx`, `badges.tsx`) ; `src/lib/wakeup-format.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026. Rien n'a été observé à l'écran : la passe d'interface reste à faire, sa liste de contrôle est en fin de fichier.

> **Cette page décrit le mécanisme et le formulaire.** La lecture des résultats passés est dans `09-automatisation/historique-des-reveils.md`.

---

## Plan de page proposé

1. Ce qu'est un réveil
2. Les trois rythmes possibles
3. Créer un réveil : chaque champ
4. Ce qui se passe à l'heure dite
5. La conversation créée par un réveil
6. Le réveil ponctuel : une seule fois, vraiment
7. Mettre en pause : un réveil, ou tous
8. Si Beaver n'était pas là à l'heure prévue
9. Les limites

---

## Contenu

### 1. Ce qu'est un réveil

Un réveil est une consigne écrite d'avance, associée à une heure. À l'heure dite, Beaver ouvre une conversation, y pose la consigne, et laisse l'agent travailler jusqu'à ce qu'il ait répondu.

Ce n'est pas une simple notification : c'est **un tour d'agent complet**, avec ses outils. Le code le dit explicitement en commentaire — le planificateur ne possède pas de second moteur, tout réveil emprunte celui de l'agent (`services/scheduler/fire.rs:81-82`).

**Le planificateur est interne à Beaver.** Aucune tâche n'est déposée dans le planificateur du système d'exploitation : c'est une boucle qui tourne dans l'application, démarrée avec elle (`src-tauri/src/runtime_state.rs:59-63` ; la boucle elle-même est `services/scheduler/runtime.rs:20-90`). Conséquence directe, et à écrire en tête de page : **si Beaver ne tourne pas, rien ne se déclenche.** L'application le dit d'ailleurs dans le formulaire de création (`src/i18n/fr.json:767`) :

> « Les réveils ne se déclenchent que si votre ordinateur est allumé et Beaver lancé. »

L'écran s'appelle **« Réveils »** (`src/i18n/fr.json:715` et `:760`), et son sous-titre est **« Créer des réveils automatisés pour planifier différentes tâches »** (`:716`). L'onglet, lui, porte encore le nom **« Heartbeat »** (`:3`) — voir « Anomalies relevées ».

### 2. Les trois rythmes possibles

Le formulaire propose trois onglets (`src/components/heartbeat/schedule-picker.tsx:38-48`), libellés **« Ponctuel »**, **« Journalier »**, **« Hebdo »** (`src/i18n/fr.json:789-791`).

| Rythme | Ce qu'on saisit | Comment c'est rangé | Source |
|---|---|---|---|
| **Ponctuel** | Une date et une heure | `2026-09-15T08:00` | `services/scheduler/next_fire.rs:46` |
| **Journalier** | Une heure | `08:00` | `next_fire.rs:50-52` |
| **Hebdo** | Un jour et une heure | numéro de jour + `08:00` | `next_fire.rs:67-69` |

**Les jours de la semaine sont numérotés de 0 à 6, en commençant par lundi** (`next_fire.rs:125-135` ; libellés « Lundi » à « Dimanche », `src/i18n/fr.json:794-800`, lundi = 0). Un numéro supérieur à 6 est refusé (`commands/heartbeat_validation.rs:90-92`).

**Tout est en heure locale de la machine** (`next_fire.rs:47`, `:57`, `:63` — le type `Local` de la bibliothèque de dates). Un réveil quotidien à 8 h reste à 8 h après un changement d'heure ou un déplacement dans un autre fuseau : c'est l'heure affichée par l'ordinateur qui fait foi, pas un décalage figé.

**La valeur par défaut du formulaire est « Journalier à 08:00 »** (`src/components/heartbeat/new-wakeup-dialog.tsx:19-21`). Changer d'onglet réinitialise l'heure : « Ponctuel » se pose sur l'instant présent, « Journalier » et « Hebdo » sur 08:00, et « Hebdo » sur lundi (`schedule-picker.tsx:15-31`).

### 3. Créer un réveil : chaque champ

Le bouton s'appelle **« Nouveau réveil »** (`src/i18n/fr.json:718`), et le formulaire **« Nouveau réveil »** ou **« Modifier le réveil »** selon le cas (`:773-774`).

| Champ | Libellé affiché | Obligatoire | Limite | Source de la limite |
|---|---|---|---|---|
| Nom | **« Nom »** | Oui | **120 caractères** | `commands/heartbeat_validation.rs:5`, `:28` |
| Description | **« Description »** | Non | **300 caractères** côté moteur, **200** côté saisie | `heartbeat_validation.rs:9` ; `new-wakeup-dialog.tsx:156` |
| Fournisseur | **« Fournisseur »** | Oui | **64 caractères** | `heartbeat_validation.rs:7` |
| Modèle | **« Modèle »** | Oui | **160 caractères** | `heartbeat_validation.rs:6` |
| Projet | **« Projet »** | Non | — | `new-wakeup-dialog.tsx:169-179` |
| Consigne | **« Prompt »** | Oui | **12 000 caractères** | `heartbeat_validation.rs:8` |
| Planification | **« Planification »** | Oui | — | `schedule-picker.tsx:35` |

(Libellés vérifiés à `src/i18n/fr.json:775-784`.)

**Un champ vide ou fait uniquement d'espaces est refusé** pour le nom, le modèle, le fournisseur et la consigne (`heartbeat_validation.rs:60-69`). La description, elle, peut rester vide (`:71-76`).

**Seuls les modèles capables d'utiliser des outils sont proposés.** La liste déroulante des modèles est filtrée sur cette capacité (`new-wakeup-dialog.tsx:52-54`) ; quand aucun ne convient, le champ est désactivé et affiche **« Aucun modèle tool-capable »** (`src/components/heartbeat/wakeup-form-fields.tsx:64-67` ; libellé `src/i18n/fr.json:781`). C'est logique : un réveil sans outils ne pourrait que produire du texte, jamais lire un fichier ni exécuter une commande.

**Certains fournisseurs sont refusés côté moteur**, même si leur nom est saisi à la main. La règle est unique : le fournisseur doit être disponible pour un usage d'automatisation (`heartbeat_validation.rs:48-58`). Les tests du projet fixent trois cas concrets : le compte web OpenAI/Codex est **accepté** (`heartbeat_validation.rs:139-153`), les comptes web **Grok (`xai-oauth`) et Kimi (`moonshot-oauth`) sont refusés** parce que réservés aux conversations manuelles (`:156-165`), et `groq`, retiré du produit, l'est aussi (`:168-173`). Le message technique correspondant est « Provider réservé aux conversations manuelles » (`services/scheduler/fire.rs:78`).

**Le champ « Projet ».** Un réveil peut être rattaché à un projet enregistré, ou rester dans l'espace de travail par défaut, proposé sous le libellé **« Espace de travail Beaver »** (`src/i18n/fr.json:779` ; `new-wakeup-dialog.tsx:174-177`). Le projet est vérifié à l'enregistrement : un identifiant de projet inconnu fait échouer la création (`commands/heartbeat.rs:93-98`). Le dossier du projet devient le dossier de travail de la conversation créée par le réveil.

**Un réveil ponctuel dont la date est déjà passée est refusé à la création**, tant qu'il est actif (`heartbeat_validation.rs:80-85`).

### 4. Ce qui se passe à l'heure dite

La boucle du planificateur ne se réveille pas toutes les secondes : elle calcule le prochain instant utile et dort jusque-là, **avec un plafond d'une heure** (`services/scheduler/runtime.rs:18`, `:61-71`). Elle est aussi réveillée immédiatement dès qu'un réveil est créé, modifié, activé ou supprimé (`commands/heartbeat.rs:65`, `:89`, `:106`, `:129`, `:148` → `services/scheduler/mod.rs:51-54`).

Le prochain instant est toujours **strictement futur** (`services/scheduler/next_fire.rs:8`, `:39`, `:59`, `:81`). Un réveil quotidien à 8 h consulté à 8 h précises pointe donc sur le lendemain.

**Les réveils dus à la même minute partent ensemble** — le déclenchement sélectionne tous ceux dont le prochain instant tombe sur la cible (`services/scheduler/due.rs:25-36`, test `:87`).

**Une même occurrence ne peut pas partir deux fois.** Chaque déclenchement réserve une place dans un registre, indexée par réveil et par instant prévu ; un doublon est écarté sans bruit (`services/scheduler/in_flight.rs:43-64`, `runtime_decisions.rs:17-19`).

**Le déclenchement peut être refusé pour deux raisons, et ce refus est journalisé** (`services/scheduler/runtime_decisions.rs:36-64`) :

- **Beaver est en train de fermer** — le réveil ne démarre pas, et la boucle s'arrête là ;
- **Trop d'opérations en cours** — la capacité est de **64 réveils simultanés** (`services/scheduler/work_supervision.rs:10`, `in_flight.rs:53-55`), et la boucle continue avec les suivants.

### 5. La conversation créée par un réveil

Chaque déclenchement **crée une conversation neuve** — il ne réutilise jamais celle du déclenchement précédent (`services/scheduler/fire.rs:105-123`).

**Son nom est construit automatiquement** (`fire.rs:106-113`) :

| Fournisseur | Nom de la conversation |
|---|---|
| Ollama | `Heartbeat • <nom du réveil> • <modèle>` |
| Tout autre | `Heartbeat • <nom du réveil> • <fournisseur> • <modèle>` |

**Elle est marquée comme issue d'un réveil** par un drapeau enregistré dans le fichier de la conversation (`fire.rs:114-121` → `services/agent_local/session_store_create.rs:88`). Ce drapeau sert au comptage de la consommation : tout ce qu'un réveil dépense est rangé sous l'origine **« Automatisation »** dans l'écran de consommation (`services/provider_usage/types.rs:181-183` ; libellé `src/i18n/fr.json:32`).

**Le réveil travaille en accès complet.** Le mode de permission est fixé en dur à « accès complet » (`services/scheduler/agentic.rs:108`), avec le commentaire d'explication en `fire.rs:81-82`. **C'est le point le plus important de la page pour l'utilisateur** : un réveil ne demande jamais d'approbation, puisqu'il n'y a personne devant l'écran pour répondre. La consigne qu'on lui confie est exécutée telle quelle, outils compris.

**Le mode Plan est désactivé** pour un réveil (`agentic.rs:112`).

**Deux fins possibles**, et elles ne se ressemblent pas :

- **L'agent a produit du texte** — le réveil est enregistré comme réussi, avec l'identifiant de la conversation et une estimation des jetons produits, puis l'interface est prévenue par l'événement `wakeup-completed` (`fire.rs:29-39`).
- **L'agent n'a produit aucun texte** — c'est traité comme un échec, avec le message technique « L'automatisation n'a produit aucun résultat. » (`fire.rs:90-92`), et l'événement `wakeup-failed` (`fire.rs:45-58`).

**Une conversation restée vide est supprimée.** Si le tour échoue avant d'avoir écrit le moindre message, la conversation créée quelques instants plus tôt est effacée (`fire.rs:96-103`). L'utilisateur ne se retrouve donc pas avec une liste de coquilles vides après une panne de fournisseur.

### 6. Le réveil ponctuel : une seule fois, vraiment

Un réveil ponctuel se désactive lui-même **avant** de commencer son travail, pas après (`services/scheduler/fire_once.rs:34-41`, `:56-78`). L'ordre est délibéré : la décision durable — « cette occurrence est consommée » — est écrite sur le disque avant l'action, si bien qu'une coupure de courant en plein milieu ne peut pas produire un second déclenchement au redémarrage.

Conséquence visible : **après son unique exécution, un réveil ponctuel reste dans la liste, marqué « Inactif »** (`src/i18n/fr.json:725`). Il n'est pas supprimé. Il faut le supprimer à la main si on n'en veut plus.

**Un réveil ponctuel interrompu par la fermeture de l'application** est journalisé avec le statut **« Annulé »** (`fire.rs:41-44`, `src/i18n/fr.json:742`), et il reste désactivé — il ne repartira pas au prochain démarrage.

### 7. Mettre en pause : un réveil, ou tous

**Deux interrupteurs existent, et ils ne font pas la même chose.**

**L'interrupteur individuel**, dans le détail d'un réveil (`src/components/heartbeat/wakeup-details.tsx:96-101`), active ou désactive ce réveil seul. Son infobulle est **« Activer/Désactiver »** (`src/i18n/fr.json:714`).

**L'interrupteur général**, en haut du panneau latéral (`src/components/heartbeat/heartbeat-tab.tsx:72-78`), met tout en veille. Son infobulle bascule entre **« Mettre en veille »** et **« Reprendre »** (`src/i18n/fr.json:761-762`).

La mise en veille générale a un comportement précis, qu'il faut décrire pour éviter une mauvaise surprise (`commands/heartbeat.rs:134-150`) :

1. Chaque réveil **actif** est désactivé et **marqué comme désactivé par la veille générale**.
2. À la reprise, **seuls les réveils portant cette marque sont réactivés** — ceux que vous aviez désactivés vous-même restent désactivés.

Pendant la veille, **l'interrupteur individuel est bloqué** (`wakeup-details.tsx:99`) et son infobulle devient **« Réveils en veille — désactive le master switch »** (`src/i18n/fr.json:722`). Le moteur refuse d'ailleurs la même opération de son côté (`commands/heartbeat.rs:117-119`).

**Un réveil créé pendant la veille naît désactivé** et marqué comme tel : il repartira à la reprise (`commands/heartbeat.rs:57-59`). De même, modifier un réveil pendant la veille en le passant actif le refait basculer en veille (`commands/heartbeat.rs:76-79`).

**Pendant la veille, plus aucun instant n'est calculé** : le planificateur ne cherche même pas de prochain déclenchement (`services/scheduler/runtime.rs:97-99`), et le champ **« Prochain déclenchement »** devient vide pour tous les réveils (`commands/heartbeat.rs:171-176`).

### 8. Si Beaver n'était pas là à l'heure prévue

C'est la question que se posera tout lecteur de la page, et la réponse est complète dans le code.

**Beaver retient la date de son dernier passage** dans un fichier dédié, `~/.local/share/cl-go-dash/heartbeat-runtime.json`, écrit de façon atomique (`services/scheduler/state.rs:10-12`, `:22-38`).

**Au démarrage suivant**, il compare cette date à l'heure présente et cherche, pour chaque réveil actif, la dernière occurrence tombée dans l'intervalle (`services/scheduler/runtime.rs:107-144` → `due.rs:38-51`). Chacune est journalisée avec le statut **« Raté »** (`services/scheduler/log.rs:66-81` ; libellé `src/i18n/fr.json:741`), accompagnée du message **« Le réveil a été manqué pendant l'indisponibilité de Beaver. »** (`src/i18n/fr.json:754`).

**Une occurrence ratée n'est jamais rattrapée.** Elle est constatée et enregistrée, pas exécutée. À écrire tel quel sur le site : un rapport quotidien de 8 h manqué ne s'exécutera pas à 10 h au lancement de l'application ; il sera signalé comme raté, et le prochain sera celui du lendemain.

**Une seule exception, pour les réveils ponctuels** : quand une occurrence ponctuelle est constatée ratée, Beaver la désactive aussi, dans cet ordre — d'abord le journal, ensuite la désactivation (`services/scheduler/runtime.rs:119-134` ; `runtime_decisions.rs:66-78`, dont le commentaire dit que le journal fait autorité).

**Une tolérance de cinq minutes** sépare « en retard » de « raté » (`services/scheduler/due.rs:6`, `:53-55`, test `:95`). Un réveil déclenché avec quatre minutes de retard — machine réveillée de veille, disque occupé — s'exécute normalement. Au-delà de cinq minutes, il est classé raté.

**Le cas particulier de la fenêtre fermée.** Le comportement diffère selon le système, et c'est vérifiable (`src-tauri/src/app_events.rs:13-19`, `:28-38`) :

| Système | Fermer la fenêtre principale | Les réveils continuent-ils ? |
|---|---|---|
| **macOS** | La fenêtre est **masquée**, l'application reste vivante | **Oui** |
| **Windows, Linux** | L'application **quitte** | **Non** |

Deux réglages avancés servent précisément ce besoin : **« Lancer au démarrage »** (« Ouvrir Beaver automatiquement au démarrage du système ») et **« Démarrage masqué »** (« Démarrer en arrière-plan sans ouvrir la fenêtre ») — `src/i18n/fr.json:1049-1052`. La page devrait renvoyer vers eux ici, c'est le moment où le lecteur en a besoin.

**À la fermeture volontaire**, les réveils en cours sont prévenus et attendus, dans le budget de temps commun à toute la fermeture (`src-tauri/src/app_exit/cleanup.rs:147` ; `services/scheduler/mod.rs:60-71`). Un réveil interrompu là est journalisé « Annulé », pas « Échoué ».

### 9. Les limites

| Limite | Valeur | Source |
|---|---|---|
| Nombre de réveils enregistrés | **64** | `commands/heartbeat_validation.rs:4`, `:11-16` |
| Réveils exécutés en même temps | **64** | `services/scheduler/work_supervision.rs:10` |
| Longueur de la consigne | **12 000 caractères** | `heartbeat_validation.rs:8` |
| Longueur du nom | **120 caractères** | `heartbeat_validation.rs:5` |
| Longueur de la description | **300 caractères** (moteur) | `heartbeat_validation.rs:9` |
| Tolérance de retard avant « Raté » | **5 minutes** | `services/scheduler/due.rs:6` |
| Sommeil maximal de la boucle | **60 minutes** | `services/scheduler/runtime.rs:18` |

---

## Tableaux

### Où vit chaque chose

| Élément | Emplacement | Autorité |
|---|---|---|
| Définition des réveils et veille générale | `~/.local/share/cl-go-dash/config.json`, champs `scheduled_wakeups[]` et `heartbeat.global_paused` | `src-tauri/src/models/config.rs:7-9`, `:129-130` |
| Date du dernier passage du planificateur | `~/.local/share/cl-go-dash/heartbeat-runtime.json` | `services/scheduler/state.rs:10-12` |
| Journal des exécutions | `~/.local/share/cl-go-dash/logs/wakeups.jsonl` | `services/scheduler/log.rs:15-19` |
| Conversations créées par les réveils | `~/.local/share/cl-go-dash/agent-sessions/*.json` | `services/agent_local/session_store_create.rs:88` |

### Ce que chaque statut veut dire

| Statut affiché | Ce qui s'est passé | Source |
|---|---|---|
| **« Réussi »** | L'agent a produit du texte | `services/scheduler/fire.rs:29-30` ; `src/i18n/fr.json:739` |
| **« Échoué »** | Erreur du fournisseur, ou aucun texte produit | `fire.rs:45-50` ; `:740` |
| **« Raté »** | Beaver ne tournait pas à l'heure prévue | `services/scheduler/log.rs:70-81` ; `:741` |
| **« Annulé »** | Fermeture de l'application pendant l'exécution | `fire.rs:41-44` ; `:742` |
| **« Jamais exécuté »** | Aucune exécution enregistrée pour ce réveil | `src/lib/wakeup-format.ts:56-57` ; `:743` |

### Ce que l'interface rafraîchit toute seule

| Déclencheur | Effet | Source |
|---|---|---|
| Modification de `config.json` sur le disque | Rechargement complet de la liste | `services/file_watcher.rs:45-50` → `src/hooks/use-wakeups.ts:48` |
| Écriture dans `logs/wakeups.jsonl` | Rechargement complet | `file_watcher.rs:51-53` → `use-wakeups.ts:49` |
| Fin d'un réveil, réussie ou non | Rechargement complet | `services/scheduler/fire.rs:33`, `:51` → `use-wakeups.ts:55-56` |

Le regroupement des changements de fichiers est de **200 millisecondes** (`services/file_watcher.rs:15`).

---

## Encadrés

> **⚠ À placer en tête de page — Beaver doit tourner.**
> Le planificateur vit dans l'application, pas dans celui du système. Ordinateur éteint, en veille profonde, ou Beaver fermé : rien ne se déclenche, et l'occurrence sera simplement notée « Raté » au démarrage suivant. Sur macOS, fermer la fenêtre ne ferme pas l'application ; sur Windows et Linux, si.

> **⚠ À placer dans la section Conversation créée — Un réveil travaille en accès complet.**
> Il ne peut pas demander d'approbation : personne n'est devant l'écran. Tout ce que la consigne demande — lire, écrire, exécuter une commande — se fait sans confirmation. Écrivez la consigne d'un réveil avec cette idée en tête.

> **ℹ À placer dans la section Occurrences ratées — Ce qui est raté n'est pas rattrapé.**
> Beaver constate l'occurrence manquée et l'inscrit au journal ; il ne l'exécute pas en différé. Un rapport quotidien de 8 h manqué ne sera pas produit à 10 h : il sera signalé raté, et le suivant sera celui du lendemain.

> **ℹ À placer dans la section Pause — La reprise ne réveille que ce que la veille avait endormi.**
> Un réveil que vous aviez désactivé vous-même reste désactivé après une mise en veille générale suivie d'une reprise. Seuls ceux que la veille avait éteints sont rallumés.

> **ℹ À placer dans la section Ponctuel — Un réveil ponctuel ne se supprime pas tout seul.**
> Après son unique exécution, il reste dans la liste, marqué « Inactif ». C'est utile pour retrouver ce qu'il a produit ; pensez à le supprimer quand il ne sert plus.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Mon réveil ne s'est pas déclenché cette nuit » | L'ordinateur était éteint ou en veille, ou Beaver fermé | L'exécution figure au journal en « Raté ». Activer « Lancer au démarrage », et sur Windows/Linux ne pas fermer la fenêtre |
| « Le modèle que je veux n'apparaît pas dans la liste » | Il ne sait pas utiliser d'outils | Choisir un modèle capable d'outils ; le champ affiche « Aucun modèle tool-capable » quand aucun ne convient |
| « Mon compte Grok ou Kimi n'est pas proposé » | Ces connexions par compte web sont réservées aux conversations manuelles | Utiliser une clé API du même fournisseur, ou un autre fournisseur |
| « Je ne peux pas activer un réveil » | La veille générale est active | Basculer l'interrupteur du haut du panneau ; l'infobulle le dit |
| « Tous mes réveils se sont désactivés d'un coup » | La veille générale a été activée | Reprendre : ceux qu'elle avait éteints se rallument |
| « Mon réveil ponctuel est passé et il est toujours là » | Comportement attendu : il est désactivé, pas supprimé | Le supprimer à la main |
| « Le réveil s'est exécuté mais est marqué échoué » | L'agent n'a produit aucun texte | Vérifier la consigne : elle doit demander une réponse écrite |
| « La conversation du réveil a disparu » | Le tour a échoué avant d'écrire quoi que ce soit : la conversation vide est supprimée | Consulter le journal des exécutions pour la cause |
| « Ma création de réveil échoue sans dire pourquoi » | Le formulaire affiche un message unique quel que soit le motif | Vérifier les longueurs des champs et la date d'un réveil ponctuel |

---

## Renvois

- `09-automatisation/historique-des-reveils.md` — lire les résultats passés et les codes d'erreur
- `04-agent/permissions.md` — pourquoi l'accès complet, et ce qu'il autorise
- `04-agent/fonctionnement.md` — ce qu'est un tour d'agent
- `10-reglages/avance.md` — « Lancer au démarrage » et « Démarrage masqué »
- `06-modeles/providers-comptes-web.md` — pourquoi certains comptes web sont réservés aux conversations manuelles
- `12-reference/emplacement-des-donnees.md` — `config.json`, `heartbeat-runtime.json`, `logs/wakeups.jsonl`

---

## Anomalies relevées

Constatées dans le code, non corrigées.

1. **La description est bornée à 200 caractères à la saisie et 300 côté moteur.** Le champ du formulaire impose `maxLength={200}` (`src/components/heartbeat/new-wakeup-dialog.tsx:156`), alors que la validation accepte jusqu'à 300 (`commands/heartbeat_validation.rs:9`). Deux autorités sur la même limite : personne ne peut atteindre les 300, et le jour où la borne du formulaire bouge, les deux divergeront sans que rien ne le signale.

2. **Les messages d'erreur du moteur ne remontent pas jusqu'à l'utilisateur.** Le moteur produit des motifs précis, écrits en français dans le code : « Maximum 64 réveils » (`heartbeat_validation.rs:13`), « Provider non supporté » (`:56`), « Champ prompt trop long » (`:66`), « Date ponctuelle déjà passée » (`:83`), « Heure invalide » (`:100`), « Réveil introuvable » (`commands/heartbeat.rs:85`, `:124`), « Réveils en veille » (`:118`). Le formulaire les remplace tous par un message unique, `t("errors.operationFailed")` (`new-wakeup-dialog.tsx:104`). L'utilisateur ne sait donc jamais **quel** champ pose problème. Ces chaînes ne sont par ailleurs pas des clés de traduction : elles sont en dur, en français, dans le code Rust.

3. **Le drapeau « issu d'un réveil » traverse la frontière et n'est jamais lu.** Il est calculé (`services/agent_local/session_index.rs:180`), déclaré dans le type transmis à l'interface (`src/types/agent-session.generated.ts:50`), et **aucun composant ne le consulte** — recherche faite sur tout `src/`. Dans la liste des conversations, une conversation créée par un réveil ne se distingue donc que par son nom, qui commence par `Heartbeat •`.

4. **Le panneau latéral n'affiche que les réveils actifs** (`src/components/heartbeat/heartbeat-tab.tsx:50-51`). Un réveil désactivé disparaît de ce panneau tout en restant dans la liste principale. Un réveil ponctuel déjà exécuté disparaît donc du panneau latéral juste après son exécution — au moment précis où l'utilisateur va le chercher.

5. **Deux vocabulaires cohabitent pour la même chose.** L'onglet s'appelle **« Heartbeat »** (`src/i18n/fr.json:3`), l'écran **« Réveils »** (`:715`), les conversations créées portent le préfixe `Heartbeat •` (`services/scheduler/fire.rs:107`, `:111`), et le fichier d'état s'appelle `heartbeat-runtime.json`. Le mot anglais n'est traduit nulle part.

6. **Le tutoiement et le vouvoiement se croisent dans le même écran.** « Clique sur Nouveau réveil pour commencer. » (`src/i18n/fr.json:717`) et « désactive le master switch » (`:722`) tutoient ; « Les réveils ne se déclenchent que si **votre** ordinateur est allumé » (`:767`) vouvoie. « master switch » est en outre un terme anglais dans une interface française, et ne désigne aucun élément nommé ainsi à l'écran.

7. **La liste principale identifie un réveil par son modèle, pas par son nom.** La ligne affiche le modèle en titre, puis la description — et ne retombe sur le nom que si la description est vide (`src/components/heartbeat/wakeup-row.tsx:15-23`). Deux réveils sur le même modèle, tous deux sans description, sont visuellement identiques.

8. **La liste des fournisseurs retombe sur une valeur en dur.** Quand aucun fournisseur n'est disponible, le sélecteur affiche l'unique option `Ollama`, écrite en dur dans le composant (`src/components/heartbeat/wakeup-form-fields.tsx:52`).

---

## Points à confirmer

**Écarts à arbitrer avant publication**

0. **L'accès complet des conversations de réveil — tranché.** Décision du 10 septembre 2026 : l'accès complet des conversations de réveil est un choix assumé, documenté clairement sur la page.

1. **Le vocabulaire « Heartbeat » / « Réveils ».** Le site doit-il employer un seul mot, et lequel ? Si c'est « Réveils », le nom de l'onglet et le préfixe des conversations restent en désaccord avec la documentation. Décision produit, à prendre avec l'équipe. **Tranché le 10 septembre 2026 (Kevin) : « Heartbeat » reste en anglais dans toutes les langues — terme universel des planificateurs, dont la traduction ne porterait plus le même sens. L'interface est donc correcte telle quelle ; la page du site continue de citer les deux libellés tels que l'application les affiche.**

2. **Faut-il documenter que les conversations de réveil s'appellent `Heartbeat • …` ?** C'est aujourd'hui le seul moyen de les repérer dans la liste des conversations (anomalie 3). Tant que le drapeau n'est pas exploité à l'écran, la documentation devrait probablement le dire.

3. **La borne de description (anomalie 1) est-elle 200 ou 300 ?** La documentation doit annoncer une seule valeur. En l'état, la seule vraie est 200.

**Non vérifié — hors de portée d'une lecture du code**

4. **Le comportement en veille prolongée et à la sortie de veille.** Le code compare des dates ; il ne fait aucune hypothèse sur le fait qu'une machine en veille reprend la boucle immédiatement ou avec du retard. Un réveil prévu pendant une veille de trois heures sera-t-il vu comme « raté » ou déclenché avec retard à la sortie de veille ? Cela dépend du moment où le fil d'exécution reprend, ce qu'une lecture du code ne peut pas trancher. **À provoquer sur une machine réelle avant publication.**

5. **Le changement d'heure (été/hiver).** Le code utilise l'heure locale et écarte les instants ambigus ou inexistants (`next_fire.rs:47`, `:63`, `:79` — la méthode `single()` renvoie « rien » quand une heure locale correspond à zéro ou deux instants). Un réveil quotidien réglé à 2 h 30 pourrait donc, la nuit du passage à l'heure d'été, ne pas avoir de prochain instant calculable. Le comportement exact n'a pas été provoqué. **À vérifier, et à documenter s'il est confirmé.**

6. **La formulation dans les six autres langues.** Tous les libellés cités ont été relus en français seulement.

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

7. **La disposition générale de l'écran** : le panneau latéral et son interrupteur, la liste principale, le détail d'un réveil.
8. **L'interrupteur individuel désactivé pendant la veille** : son apparence, et si l'infobulle « Réveils en veille — désactive le master switch » est réellement lisible dans les deux thèmes.
9. **La confirmation de suppression** : le bouton devient « Confirmer la suppression » pendant **3 secondes** puis revient à son état initial (`src/components/heartbeat/wakeup-details.tsx:38-45`). Ce délai court n'a pas été observé, et il s'écarte de la règle du projet qui demande une annulation plutôt qu'une confirmation.
10. **Le sélecteur de date « Ponctuel »** utilise un champ `datetime-local` du navigateur (`src/components/heartbeat/schedule-picker.tsx:67`), et le champ d'heure un `time` (`:75`) — des composants natifs, alors que la règle d'interface du projet demande des composants maison. À regarder dans les deux thèmes et sur les trois systèmes.
11. **Le formulaire quand aucun modèle capable d'outils n'est disponible** : le bouton d'enregistrement est désactivé (`new-wakeup-dialog.tsx:110`), mais rien ne dit à l'utilisateur pourquoi hors du champ de modèle.
