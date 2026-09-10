# L'historique des réveils

**Emplacement site** — Automatisation › L'historique des réveils
**Répond à** — « Mon réveil s'est-il exécuté cette nuit, et si non, pourquoi ? »
**Sources** — `src-tauri/src/services/scheduler/log.rs`, `log_store.rs`, `log_metadata.rs` ; `src-tauri/src/models/config.rs` (`WakeupRun`, `WakeupRunStatus`, `WakeupRunErrorCode`, `WakeupStatusSummary`) ; `src-tauri/src/commands/heartbeat.rs` ; `src-tauri/src/services/scheduler/fire.rs`, `runtime.rs`, `runtime_decisions.rs` ; `src-tauri/src/services/file_watcher.rs` ; `src-tauri/src/services/private_store/atomic_write.rs` ; `src/components/heartbeat/wakeup-history.tsx`, `wakeup-details.tsx` ; `src/lib/wakeup-run-error.ts`, `src/lib/wakeup-format.ts` ; `src/hooks/use-wakeups.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026. Aucun affichage observé à l'écran.

> **Cette page décrit la lecture des résultats.** La création et le déclenchement des réveils sont dans `09-automatisation/reveils.md`.

---

## Plan de page proposé

1. Où se lit l'historique
2. Ce qu'une exécution enregistre
3. Les quatre statuts
4. Les sept causes d'échec
5. Combien de temps l'historique est conservé
6. Ce qui est écrit, et ce qui ne l'est pas
7. Retrouver la conversation produite

---

## Contenu

### 1. Où se lit l'historique

**L'historique ne s'affiche que dans le détail d'un réveil.** Il n'existe pas d'écran général listant toutes les exécutions de tous les réveils : la section est rendue en bas de la fiche d'un réveil (`src/components/heartbeat/wakeup-details.tsx:148`), filtrée sur ce réveil seul (`src/components/heartbeat/heartbeat-tab.tsx:112`).

Le titre de la section est **« Historique »** (`src/i18n/fr.json:747`), et sa version vide dit **« Aucune exécution »** (`:748`).

**Huit exécutions au maximum sont affichées**, les plus récentes (`src/components/heartbeat/wakeup-history.tsx:13`). Le reste du journal existe sur le disque mais n'est pas montré.

Deux informations résumées apparaissent aussi plus haut dans la même fiche (`wakeup-details.tsx:131-136`) : **« Dernier statut »** et **« Dernière exécution »** (`src/i18n/fr.json:733-734`).

**L'affichage se rafraîchit tout seul** quand le journal change sur le disque, sans intervention de l'utilisateur : le fichier est surveillé (`services/file_watcher.rs:51-53`) et l'interface recharge tout (`src/hooks/use-wakeups.ts:49`). Un réveil qui se termine pendant que la fiche est ouverte fait apparaître sa ligne immédiatement (`services/scheduler/fire.rs:33`, `:51` → `use-wakeups.ts:55-56`).

### 2. Ce qu'une exécution enregistre

Le journal est un fichier de lignes, **`~/.local/share/cl-go-dash/logs/wakeups.jsonl`** — une ligne par exécution, chacune un objet indépendant (`services/scheduler/log.rs:15-19`).

Chaque ligne porte, au plus, ces champs (`src-tauri/src/models/config.rs:181-194`) :

| Champ | Contenu | Toujours présent |
|---|---|---|
| `wakeup_id` | L'identifiant du réveil | Oui |
| `scheduled_for` | L'instant **prévu**, en heure locale | Oui |
| `fired_at` | L'instant **réel** de l'inscription, en temps universel | Oui |
| `status` | Réussi, échoué, raté ou annulé | Oui |
| `error_code` | Le motif, parmi sept valeurs | Seulement en cas d'échec ou de ratage |
| `session_id` | La conversation produite | Seulement en cas de réussite |
| `tokens` | Une estimation des jetons produits | Seulement en cas de réussite |

**Les deux dates ne mesurent pas la même chose**, et la distinction compte : `scheduled_for` est l'heure à laquelle le réveil devait partir, `fired_at` l'heure à laquelle son résultat a été inscrit — donc **après** le travail de l'agent, pas avant (`services/scheduler/log.rs:29-30`). L'écart entre les deux est la durée d'exécution, plus le retard éventuel.

**Ce que l'interface affiche est `fired_at`**, dans les deux endroits : la ligne d'historique (`src/components/heartbeat/wakeup-history.tsx:32`) et le champ « Dernière exécution » (`wakeup-details.tsx:135`). L'heure prévue n'est jamais montrée. Voir « Anomalies relevées ».

**Les identifiants sont nettoyés avant d'être écrits** : seuls lettres, chiffres, `-` et `_` sont conservés, et la longueur est coupée à **128 caractères** (`services/scheduler/log.rs:146-152`, borne `log_store.rs:14`).

**Une ligne identique n'est pas écrite deux fois.** Beaver garde en mémoire ce qu'il vient d'écrire et écarte le doublon (`services/scheduler/log_store.rs:97-99`).

### 3. Les quatre statuts

| Statut | Libellé affiché | Ce qui s'est passé | Source |
|---|---|---|---|
| `ok` | **« Réussi »** | L'agent a produit du texte | `services/scheduler/fire.rs:29-30` ; `src/i18n/fr.json:739` |
| `error` | **« Échoué »** | Le tour a échoué, ou n'a produit aucun texte | `fire.rs:45-50` ; `:740` |
| `missed` | **« Raté »** | Beaver ne tournait pas à l'heure prévue | `services/scheduler/log.rs:66-81` ; `:741` |
| `cancelled` | **« Annulé »** | Fermeture de l'application pendant l'exécution | `fire.rs:41-44` ; `:742` |

Quand aucune exécution n'a jamais eu lieu, le champ « Dernier statut » affiche **« Jamais exécuté »** (`src/lib/wakeup-format.ts:56-57` ; `src/i18n/fr.json:743`). Quand une date manque ou est illisible, l'interface affiche **« Aucun »** (`wakeup-format.ts:46-49` ; `:744`).

### 4. Les sept causes d'échec

Un statut « Échoué » ou « Raté » porte un **code de motif**, traduit par une phrase à l'écran (`src/lib/wakeup-run-error.ts:27-35` ; `src/components/heartbeat/wakeup-history.tsx:26`, `:33`).

| Code | Phrase affichée | Quand | Source |
|---|---|---|---|
| `failed` | **« Le réveil a échoué. »** | Motif par défaut, aucun autre ne correspond | `services/scheduler/log.rs:131` ; `src/i18n/fr.json:750` |
| `rate_limited` | **« La limite de requêtes du fournisseur a été atteinte. »** | L'erreur contient « rate limit » | `log.rs:123-124` ; `:751` |
| `authentication_failed` | **« L'authentification auprès du fournisseur a échoué. »** | L'erreur contient « clé api », « unauthorized » ou « auth » | `log.rs:125-127` ; `:752` |
| `ollama_unavailable` | **« Ollama était indisponible. »** | L'erreur contient « ollama » | `log.rs:128-129` ; `:753` |
| `missed_unavailable` | **« Le réveil a été manqué pendant l'indisponibilité de Beaver. »** | Occurrence constatée ratée au démarrage | `log.rs:76` ; `:754` |
| `scheduler_stopping` | **« Le réveil n'a pas démarré car Beaver était en cours de fermeture. »** | Refus d'admission pendant la fermeture | `log.rs:137-139` ; `:755` |
| `capacity_reached` | **« Le réveil n'a pas démarré car trop d'opérations étaient déjà en cours. »** | Plus de **64** réveils en cours | `log.rs:140-142` ; `services/scheduler/work_supervision.rs:10` ; `:756` |

**Les trois premiers motifs sont devinés à partir du texte de l'erreur**, pas remontés par le fournisseur (`services/scheduler/log.rs:121-133`). C'est une classification par mots-clés : elle range correctement les cas courants, mais une erreur formulée autrement retombe sur « Le réveil a échoué. ». À dire sur le site sans dramatiser, mais à dire : le motif est une indication, pas un diagnostic certain.

**Un code inconnu retombe sur « Le réveil a échoué. »** — l'interface vérifie que le code fait partie des sept avant de le traduire (`src/lib/wakeup-run-error.ts:25`, `:30-34`). Un journal écrit par une version future ne casse donc pas l'affichage.

### 5. Combien de temps l'historique est conservé

**Le journal est borné à 500 lignes** (`services/scheduler/log_store.rs:12`).

Le mécanisme n'est pas une suppression ligne à ligne. Quand l'ajout d'une ligne ferait dépasser le budget — **500 lignes × 2 048 octets, soit un peu plus d'un mégaoctet** (`log_store.rs:15-16`) —, Beaver **réécrit le fichier entier en ne gardant que les 249 lignes les plus récentes**, puis y ajoute la nouvelle (`log_store.rs:13`, `:107-119`, `:164-178`). Le fichier retombe donc à 250 lignes, et remonte jusqu'à ce que le budget soit à nouveau atteint.

Deux conséquences à écrire sur le site :

- **Il n'y a pas de durée de conservation en jours.** Un utilisateur qui a un réveil quotidien garde des mois d'historique ; celui qui en a vingt en garde quelques semaines. La borne est un nombre de lignes, partagé par tous les réveils.
- **La rotation perd la moitié de l'historique d'un coup**, elle ne rogne pas une ligne à la fois.

**Une ligne trop longue est refusée plutôt que tronquée** : au-delà de **2 048 octets**, l'écriture échoue (`log_store.rs:104-106`). Compte tenu des champs enregistrés, ce cas ne devrait pas se produire ; il existe comme garde-fou.

**La réécriture est atomique** : fichier temporaire puis renommage (`log_store.rs:116` → `services/private_store/atomic_write.rs`). Une coupure de courant pendant une rotation laisse soit l'ancien journal entier, soit le nouveau, jamais un mélange.

**En lecture, seul le dernier mégaoctet du fichier est lu** (`log_store.rs:180-203`), et la première ligne partielle est écartée pour ne pas produire un enregistrement tronqué (`:196-201`). Un fichier remplacé à la main par un fichier plus gros ne fera donc pas grossir la mémoire de Beaver.

**Les exécutions sont renvoyées de la plus récente à la plus ancienne**, triées sur `fired_at` (`log_store.rs:151-162`).

### 6. Ce qui est écrit, et ce qui ne l'est pas

**Ce qui n'est jamais dans le journal**, et c'est important pour la page :

- **Le texte de la consigne** — il vit dans `config.json`, pas ici ;
- **La réponse de l'agent** — elle vit dans la conversation ;
- **Le message d'erreur brut du fournisseur** — il est réduit à l'un des sept codes avant écriture (`services/scheduler/log.rs:104-119`, `:121-133`). Le journal du réveil ne contient donc **aucun extrait de réponse d'un service externe**.

**Quand le journal lui-même est indisponible**, l'erreur technique est toujours la même chaîne, `wakeup-log-unavailable` (`log_store.rs:209-211`), et le planificateur écrit un avertissement dans ses traces sans interrompre le réveil (`services/scheduler/fire.rs:62-66`, `runtime_decisions.rs:80-87`).

**Une décision non journalisée n'est pas considérée comme prise.** Si l'écriture échoue, Beaver n'avance pas sa date de dernier passage (`services/scheduler/runtime.rs:139-143`) : l'occurrence sera réexaminée au tour suivant. Le journal est l'autorité sur ce qui a été décidé, et un commentaire du code le dit explicitement (`runtime_decisions.rs:75`).

### 7. Retrouver la conversation produite

Une exécution réussie enregistre l'identifiant de la conversation créée (`services/scheduler/log.rs:34`) et une estimation des jetons produits (`:35`, calcul en `services/scheduler/agentic.rs:177-187`).

**Ces deux informations ne sont affichées nulle part.** L'historique montre le statut, l'heure et le motif d'erreur, rien d'autre (`src/components/heartbeat/wakeup-history.tsx:28-34`). Pour retrouver ce qu'un réveil a produit, il faut passer par la liste des conversations et chercher celle qui porte le préfixe `Heartbeat •` suivi du nom du réveil (`services/scheduler/fire.rs:107`, `:111`). Voir « Anomalies relevées ».

---

## Tableaux

### Le fichier, en un coup d'œil

| Propriété | Valeur | Source |
|---|---|---|
| Emplacement | `~/.local/share/cl-go-dash/logs/wakeups.jsonl` | `services/scheduler/log.rs:15-19` |
| Format | Une ligne par exécution, indépendante | `log_store.rs:100-103` |
| Lignes conservées | **500**, ramenées à **250** à la rotation | `log_store.rs:12-13` |
| Taille maximale d'une ligne | **2 048 octets** | `log_store.rs:15` |
| Budget total | environ **1 Mo** | `log_store.rs:16` |
| Écriture | Ajout en fin de fichier, forcé sur le disque ; réécriture atomique à la rotation | `log_store.rs:120-130` ; `:116` |
| Lecture | Le dernier mégaoctet, ligne partielle écartée | `log_store.rs:180-203` |
| Tri renvoyé | Du plus récent au plus ancien | `log_store.rs:160-161` |

### Ce que l'interface montre et ne montre pas

| Enregistré dans le journal | Montré à l'écran |
|---|---|
| Statut | **Oui** |
| Heure réelle (`fired_at`) | **Oui** |
| Motif d'échec | **Oui**, en une phrase |
| Heure prévue (`scheduled_for`) | Non |
| Identifiant de la conversation produite | Non |
| Jetons produits | Non |

---

## Encadrés

> **ℹ À placer en tête de page — L'historique est par réveil, et court.**
> Il s'affiche dans la fiche d'un réveil, pas sur un écran commun, et montre les **huit dernières** exécutions. Le fichier sur le disque en garde jusqu'à 500, toutes exécutions confondues.

> **ℹ À placer dans la section Conservation — Il n'y a pas de durée en jours.**
> C'est un nombre de lignes partagé par tous vos réveils : plus vous en avez, plus vite les anciennes disparaissent. Quand la limite est atteinte, la moitié la plus ancienne est effacée d'un coup.

> **ℹ À placer dans la section Motifs — Le motif est une indication.**
> Trois des sept motifs sont devinés à partir du texte de l'erreur du fournisseur. Une panne formulée autrement retombe sur « Le réveil a échoué. » — ce qui ne veut pas dire que la cause est inconnue, seulement que Beaver n'a pas su la nommer.

> **ℹ À placer dans la section Ce qui est écrit — Le journal ne contient rien de sensible.**
> Ni votre consigne, ni la réponse de l'agent, ni le message brut du fournisseur : seulement un identifiant, deux dates, un statut et un code. C'est un fichier qu'on peut joindre à un signalement de problème sans le relire.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Mon historique s'arrête il y a deux semaines » | La rotation à 500 lignes a effacé la moitié la plus ancienne | Comportement attendu ; copier le fichier si l'on veut conserver plus loin |
| « Je ne vois que huit lignes » | L'interface n'en montre que huit | Le fichier `logs/wakeups.jsonl` contient le reste |
| « Il n'y a pas d'écran listant tous mes réveils passés » | L'historique est par réveil | Ouvrir la fiche de chaque réveil |
| « Le réveil est marqué “Raté” alors que l'ordinateur tournait » | Beaver était fermé, ou le déclenchement avait plus de cinq minutes de retard | Vérifier « Lancer au démarrage » ; voir `reveils.md` |
| « “Le réveil a échoué.” ne me dit rien » | Le motif n'a pas été reconnu par mots-clés | Ouvrir la conversation correspondante, qui porte l'erreur détaillée |
| « L'heure affichée ne correspond pas à l'heure programmée » | L'heure montrée est celle de la **fin** d'exécution, pas celle prévue | Comportement attendu ; l'écart est la durée du travail |
| « Deux réveils ont échoué en même temps avec “trop d'opérations en cours” » | Plus de 64 réveils simultanés | Espacer les horaires |

---

## Renvois

- `09-automatisation/reveils.md` — créer un réveil, la pause générale, ce qui se passe si Beaver est fermé
- `12-reference/emplacement-des-donnees.md` — l'inventaire de `~/.local/share/cl-go-dash/`
- `13-depannage/` — que faire d'un échec répété

---

## Anomalies relevées

Constatées dans le code, non corrigées.

1. **L'heure prévue est enregistrée et jamais montrée.** Le champ `scheduled_for` figure dans chaque ligne du journal (`src-tauri/src/models/config.rs:183`) et n'apparaît dans aucun composant. L'écran affiche uniquement `fired_at` (`src/components/heartbeat/wakeup-history.tsx:32`, `wakeup-details.tsx:135`). Pour un réveil déclenché avec du retard, ou pour une occurrence ratée — dont le `fired_at` est l'instant du **démarrage suivant** de Beaver, pas l'heure manquée —, l'heure affichée ne dit pas ce que le lecteur croit lire. C'est exactement le cas « le chiffre est juste et la phrase est fausse ».

2. **L'identifiant de la conversation produite n'est exploité nulle part.** Il est écrit (`services/scheduler/log.rs:34`), transmis à l'interface dans le type `WakeupRun`, et aucun composant ne l'utilise — recherche faite sur tout `src/`. Il n'existe donc aucun lien cliquable entre une exécution réussie et la conversation qu'elle a produite, alors que l'information est disponible des deux côtés.

3. **Les jetons produits sont enregistrés et jamais affichés.** Même constat (`services/scheduler/log.rs:35`).

4. **Un champ hérité subsiste dans le journal.** Le champ `error`, texte libre des versions antérieures, est encore lu à l'ouverture des anciennes lignes mais n'est plus jamais écrit (`src-tauri/src/models/config.rs:188-189` — il porte la mention `skip_serializing`). Côté interface, un repli l'utilise encore : si une ligne ancienne porte ce champ sans code, elle est traduite par « Le réveil a échoué. » (`src/lib/wakeup-run-error.ts:34`). Ce n'est pas un défaut — c'est une lecture tolérante volontaire — mais c'est une dette à surveiller.

5. **Le fichier n'a pas de numéro de version.** Contrairement aux autres fichiers persistés de Beaver, `wakeups.jsonl` n'écrit aucun champ de version dans ses lignes (`src-tauri/src/models/config.rs:181-194`). La tolérance en lecture le compense aujourd'hui — une ligne illisible est simplement ignorée (`services/scheduler/log_store.rs:156`) —, mais rien ne permettrait de distinguer deux formats si le besoin apparaissait.

---

## Points à confirmer

**Écarts à arbitrer avant publication**

1. **Faut-il documenter que l'heure affichée est celle de fin d'exécution ?** (anomalie 1) Le site peut le dire — c'est exact et utile — ou attendre que l'interface affiche les deux dates. Décision à prendre avec l'équipe ; en l'état, ne pas le dire laisserait le lecteur mal interpréter chaque ligne « Raté ».

2. **Faut-il documenter comment retrouver la conversation d'un réveil ?** (anomalie 2) Tant qu'aucun lien n'existe, la seule méthode est de chercher le préfixe `Heartbeat •` dans la liste des conversations. C'est une information de contournement : à écrire ou non selon que l'équipe prévoit d'ajouter le lien.

**Non vérifié — hors de portée d'une lecture du code**

3. **Le comportement observé lors d'une rotation.** La perte des 250 lignes les plus anciennes est prouvée par le code, mais n'a pas été provoquée sur un vrai fichier. **À vérifier sur un journal réel avant de publier la valeur.**

4. **Le nombre de lignes réellement atteint en usage normal.** La borne est de 500, mais aucune mesure n'a été faite sur une installation utilisée. Le site ne devrait pas promettre une durée de conservation.

5. **La formulation dans les six autres langues.** Les sept phrases de motif ont été relues en français seulement (`src/i18n/fr.json:750-756`).

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

6. **La section Historique elle-même** : sa position en bas de la fiche, l'apparence des quatre statuts, et le fait que la classe de style porte le nom du statut (`src/components/heartbeat/wakeup-history.tsx:29`) — donc quatre couleurs à vérifier dans les deux thèmes, en s'assurant qu'« Échoué » et « Raté » ne se ressemblent pas.

7. **L'état vide** : « Aucune exécution » dans sa carte (`wakeup-history.tsx:19-21`).

8. **Les états manquants.** La section a un état vide et un état avec contenu ; elle n'a **ni état de chargement ni état d'erreur** (`wakeup-history.tsx:18-39`). Quand le journal est illisible, le chargement échoue en amont et l'interface remet toutes les listes à vide (`src/hooks/use-wakeups.ts:33-37`) : l'utilisateur voit « Aucune exécution » sans pouvoir distinguer un journal vide d'un journal cassé. À vérifier à l'écran, et probablement à signaler comme demande d'évolution.

9. **La longueur d'une phrase de motif** dans la ligne d'historique : les sept phrases font jusqu'à 74 caractères, sur une ligne qui porte déjà un statut et une date (`wakeup-history.tsx:28-34`). Le repli à l'étroit n'a pas été observé.
