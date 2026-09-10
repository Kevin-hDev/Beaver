# Dépannage — Le moteur local Ollama

**Emplacement site** — Référence › Dépannage › Ollama
**Répond à** — « Le moteur local est installé, mais quelque chose ne va pas : il est indisponible, un modèle refuse de se télécharger, la carte graphique n'est pas utilisée, ou les réglages n'ont aucun effet. »
**Sources** — `src-tauri/src/services/ollama_manager/` (`polling.rs`, `port.rs`, `constants.rs`, `types.rs`, `error.rs`, `manager_process.rs`, `manager_runtime.rs`, `manager_stop.rs`, `manager_startup.rs`, `spawn_settings.rs`, `compute_mode.rs`, `spawn_profile.rs`, `spawn_profile_paths.rs`, `spawn_gate_unix.rs`) ; `src-tauri/src/services/agent_local/ollama_registry.rs`, `ollama_stream_request.rs`, `ollama_retry_indicator.rs`, `agent_loop_support.rs` ; `src-tauri/src/services/model_downloads.rs`, `model_downloads_store.rs`, `model_downloads_types.rs` ; `src-tauri/src/services/gpu_detect.rs`, `services/gpu_vram.rs`, `services/gpu_vram/macos.rs` ; `src-tauri/src/commands/ollama_setup.rs` ; `src-tauri/src/services/llm/route_profile/policy_types.rs` ; `src/components/ollama/model-install-button.tsx`, `src/components/settings/ollama-settings-section.tsx`, `src/components/settings/ollama-restart-outcome.ts`, `src/lib/ollama-runtime-error.ts`, `src/hooks/use-available-models.ts`, `src/hooks/use-ollama-runtime-status.ts`, `src/lib/agent-error-codes.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026 sur la version **1.2.2**. Aucun de ces états n'a été provoqué à l'écran : la liste de contrôle est en fin de fichier.

---

## Avertissement au rédacteur

Cette page traite d'un moteur **déjà installé**. Tout ce qui concerne la première installation du moteur — téléchargement de l'archive, empreinte, reprise d'une installation interrompue — est dans *Dépannage › Installation*, sections 8 à 10, et **ne doit pas être répété ici**. Cette page-ci renvoie à celle-là.

Deux règles de rédaction, héritées de *Dépannage › Installation* :

1. **Chaque entrée commence par ce que l'utilisateur voit**, mot pour mot quand c'est un message de l'application.
2. **Aucune promesse de résolution qui n'existe pas.** Plusieurs situations décrites ici n'ont pas de remède depuis l'application : le dire, et donner l'issue réelle.

**Les messages de l'application tutoient** (« Vérifie la connexion puis réessaie ») alors que ces fichiers vouvoient. Citer les messages tels quels ; le ton du reste de la page est une décision du site.

---

## Plan de page proposé

1. Les trois états du moteur, et comment les lire
2. « Ollama indisponible » — le moteur ne répond plus
3. Le port 11434 et le moteur externe réutilisé
4. Ce qui ne fonctionne pas avec un moteur externe
5. Le téléchargement d'un modèle échoue
6. Le téléchargement d'un modèle a été interrompu
7. La carte graphique n'est pas utilisée
8. Le modèle est trop lourd pour la mémoire
9. Redémarrer le moteur
10. « Connexion à Ollama perdue » pendant une conversation

---

## Contenu

### 1. Les trois états du moteur, et comment les lire

Le moteur n'a que **trois états possibles**, et toute la page en découle (`services/ollama_manager/types.rs:23-27`) :

| État interne | Libellé affiché | Ce que ça veut dire |
|---|---|---|
| `Owned` | **« Ollama géré par Beaver »** | Beaver a lancé le moteur lui-même et le contrôle entièrement |
| `External` | **« Ollama externe »** | Un moteur tournait déjà sur la machine ; Beaver s'en sert sans le contrôler |
| `Unavailable` | **« Ollama indisponible »** | Aucun moteur joignable |

Ces trois libellés sont vérifiés dans `src/i18n/fr.json:687`, avec **« Chargement de l'état d'Ollama… »** pendant la mesure et un bouton **« Réessayer »**.

**La distinction entre « géré par Beaver » et « externe » n'est pas cosmétique.** Elle décide de ce qui marche et de ce qui ne marche pas : c'est la section 4, et c'est probablement l'information la plus utile de toute la page.

**Comment Beaver mesure cet état.** Une boucle de fond interroge le moteur **toutes les 2 secondes** sur son adresse `/api/version`, avec un délai de réponse de **2 secondes** (`polling.rs:9-10`, `:96-100`). Si l'appel échoue, l'état bascule immédiatement à « indisponible » (`polling.rs:101-105`). L'interface n'est prévenue **que lorsque l'état change**, pas à chaque tour de boucle (`polling.rs:139-143`).

### 2. « Ollama indisponible » — le moteur ne répond plus

**Symptôme** — l'onglet Réglages › Ollama affiche **« Ollama indisponible »**, la liste des modèles locaux se vide, et l'agent refuse toute requête locale.

**Les causes vérifiées, par ordre de fréquence probable :**

**a) Le processus du moteur s'est arrêté.** Quelle qu'en soit la raison — plantage, arrêt par le système, manque de mémoire — la boucle de mesure le constate en moins de deux secondes et bascule l'état.

**b) Le moteur ne répond pas en moins de 2 secondes.** Le délai de la sonde de santé est fixe (`polling.rs:10`). Une machine très chargée, ou un modèle en train de se charger en mémoire, peut dépasser ce délai : le moteur est alors annoncé indisponible **alors qu'il tourne encore**. L'état revient tout seul au tour suivant.

**c) Le moteur n'a jamais démarré.** Le lancement a un budget de **10 secondes** (`constants.rs:15`). Au-delà, ou si la version qui répond n'est pas celle installée, Beaver **arrête le processus qu'il vient de lancer** et renvoie l'erreur **« Ollama n'a pas pu démarrer. »** (`manager_process.rs:102-126` ; message `fr.json:705`).

**d) Une réparation est en attente.** Si l'installation du moteur est dans un état qui exige une reprise, le démarrage est refusé avant même d'essayer (`manager_process.rs:19-34`), avec l'un des messages de réparation du tableau de la section Tableaux.

**Résolution, dans cet ordre :**

1. **Attendre dix secondes et regarder de nouveau** — la mesure est automatique et se corrige seule dans le cas (b).
2. **Cliquer sur « Réessayer »** dans Réglages › Ollama (`fr.json:687`).
3. **Redémarrer le moteur** — voir la section 9.
4. **Redémarrer Beaver.** Une reprise de l'installation s'exécute au démarrage, avant toute tentative de lancement (voir *Dépannage › Installation*, section 9).
5. **Si le message est l'un des messages de réparation**, suivre le tableau de la section Tableaux : ils n'ont pas la même issue.

⚠️ **Il n'y a aucun journal du moteur à consulter.** Sa sortie est envoyée vers `/dev/null` sous macOS et Linux (`spawn_gate_unix.rs:63-66`), et n'est pas redirigée sous Windows. Ne publier aucun conseil du type « consultez le journal du moteur » : ce fichier n'existe pas. Les seules traces exploitables sont celles de Beaver lui-même, qui portent un code d'erreur, pas un message du moteur.

### 3. Le port 11434 et le moteur externe réutilisé

C'est le mécanisme le plus mal compris de tout le produit, et il mérite une section entière.

**Ce que Beaver fait, dans l'ordre, à chaque démarrage du moteur** (`manager_process.rs:47-59`) :

1. Il tente une connexion sur **`127.0.0.1:11434`**, le port habituel d'Ollama, avec un délai de **250 millisecondes** (`constants.rs:22`, `:24` ; `port.rs:66-84`).
2. **Si quelque chose répond sur ce port**, Beaver le déclare « Ollama externe » et **ne lance rien**.
3. **Si la connexion est refusée**, Beaver demande au système un **port libre au hasard** et y lance son propre moteur (`port.rs:46-64` — la réservation se fait sur `127.0.0.1:0`, ce qui laisse le système choisir).

**Trois conséquences à écrire explicitement sur le site :**

- **Le moteur lancé par Beaver n'écoute jamais sur 11434.** Il prend un port libre tiré au hasard à chaque lancement. Quelqu'un qui cherche Beaver sur 11434 ne le trouvera pas.
- **Un port 11434 occupé ne bloque donc jamais Beaver.** C'est l'inverse du réflexe habituel : ce n'est pas un conflit, c'est une détection.
- **Le moteur n'écoute que sur l'adresse locale.** L'adresse construite est toujours `http://127.0.0.1:<port>`, et toute autre forme est refusée — autre machine, identifiant, mot de passe, paramètre d'URL (`types.rs:131-163`). Le moteur lancé par Beaver n'est **pas** joignable depuis le réseau.

**Symptôme associé** — « Réglages affiche *Ollama externe* alors que je n'ai jamais installé Ollama ». Cause probable : un autre logiciel occupe le port 11434, ou une installation d'Ollama faite avant Beaver tourne encore, par exemple lancée automatiquement au démarrage de la session.

**Résolution** — arrêter l'autre moteur, puis redémarrer le moteur depuis Réglages › Ollama (section 9). Beaver ne trouvera plus rien sur 11434 et lancera le sien.

⚠️ **Beaver ne vérifie pas que ce qui répond sur 11434 est bien Ollama.** Le contrôle est une simple ouverture de connexion (`port.rs:71-77`). Voir « Anomalies relevées ».

### 4. Ce qui ne fonctionne pas avec un moteur externe

**C'est la section la plus utile de la page.** Un moteur externe n'est pas un moteur au rabais sur un ou deux détails : trois fonctions entières deviennent indisponibles, sans que rien à l'écran ne l'annonce à l'avance.

| Fonction | Moteur géré par Beaver | Moteur externe |
|---|---|---|
| Discuter avec un modèle local | Oui | **Oui** |
| Lister les modèles installés | Oui | **Oui** |
| **Télécharger un modèle** | Oui | **Non** (`ollama_registry.rs:84-88`) |
| **Supprimer un modèle** | Oui | **Non** (`ollama_registry.rs:184-189`) |
| **Créer un modèle personnalisé** | Oui | **Non** (`manager_process.rs:155-157`) |
| Choisir processeur ou carte graphique | Oui | **Sans effet** (`spawn_settings.rs:22-31`) |
| Charger plusieurs modèles à la fois | Oui | **Sans effet** (`spawn_settings.rs:26-29`) |
| Durée avant libération de la mémoire | Oui | **Oui** (`agent_local/ollama_wire.rs:48`) |
| Arrêter ou redémarrer le moteur | Oui | **Sans effet** (`manager_stop.rs:21-23`) |

**Pourquoi cette coupure existe.** Les réglages d'accélération et de multi-modèle sont transmis au moteur **au moment de son lancement**, sous forme de variables d'environnement (`compute_mode.rs:18-31`, `spawn_settings.rs:22-31`). Un moteur que Beaver n'a pas lancé n'a jamais reçu ces valeurs, et rien ne permet de les lui appliquer après coup. À l'inverse, la durée avant libération de la mémoire voyage dans chaque requête (`ollama_wire.rs:48`) : celle-là s'applique dans les deux cas.

**Résolution** — pour retrouver toutes les fonctions, arrêter le moteur externe puis redémarrer celui de Beaver (section 9). Il n'y a pas d'autre voie.

### 5. Le téléchargement d'un modèle échoue

**Symptôme** — sous le bouton d'installation du modèle, une ligne rouge : **« Le téléchargement a échoué »** (`model-install-button.tsx:115-119` ; texte `fr.json:1389`).

**Cause — et c'est le point à écrire honnêtement : ce message ne dit jamais laquelle.** Toutes les défaillances possibles du téléchargement aboutissent au même code unique `model-download-failed` (`services/model_downloads.rs:160-169`). Les causes que le code distingue en interne, sans jamais les transmettre à l'écran :

- **le moteur n'est pas géré par Beaver** — c'est le cas décrit en section 4, et probablement la première cause à vérifier (`ollama_registry.rs:84-88`) ;
- **le moteur est injoignable** au moment de la demande (`ollama_registry.rs:90-95`) ;
- **le registre de modèles refuse** le nom demandé, ou le modèle n'existe pas ;
- **le téléchargement dépasse une heure** — le délai global est de **3600 secondes** (`ollama_registry.rs:79-82`) ;
- **l'espace disque manque**, ce qui remonte comme une erreur du moteur ;
- **le flux de progression dépasse 500 000 lignes**, borne de sécurité (`ollama_registry.rs:103`, `:115-117`).

**Résolution, dans cet ordre :**

1. **Vérifier l'état du moteur dans Réglages › Ollama.** S'il affiche « Ollama externe », le téléchargement ne peut pas fonctionner : c'est la section 4.
2. **Vérifier l'espace disque libre.** Les modèles vivent dans `~/.ollama/models` (`spawn_profile_paths.rs:33`), **en dehors** du dossier de données de Beaver, et un modèle courant pèse plusieurs gigaoctets.
3. **Relancer le téléchargement.** Ollama réutilise les morceaux déjà obtenus : un second essai ne repart pas nécessairement de zéro.
4. **Vérifier le nom du modèle** dans le catalogue, en repassant par la recherche plutôt qu'en tapant un nom à la main.
5. **Sur un réseau d'entreprise**, vérifier que `ollama.com` est joignable : c'est l'adresse du registre de modèles (`ollama_registry.rs:9`).

**Une seule file, un seul téléchargement à la fois.** Beaver n'exécute **qu'un téléchargement de modèle en même temps** (`model_downloads_store.rs:15`) et n'en accepte pas plus de **16 en attente** (`model_downloads_types.rs:3`). Au-delà, le message est **« Impossible d'ajouter ce modèle à la file d'attente. »** (`fr.json:604`), affiché en notification passagère (`model-install-button.tsx:50-52`). Les modèles en attente affichent **« En attente »**, les autres l'étape en cours : **« Démarrage »**, **« Téléchargement »**, **« Installation »**, **« Préparation du moteur »**, **« Terminé »** (`fr.json:591`, `:596-602`).

### 6. Le téléchargement d'un modèle a été interrompu

**Symptôme** — l'utilisateur a cliqué sur **« Annuler »** (ou appuyé sur Échap, qui fait la même chose — `model-install-button.tsx:59-66`), ou bien Beaver a été fermé pendant un téléchargement.

**Ce que Beaver fait alors, et qu'il faut annoncer avant que ça surprenne** (`services/model_downloads.rs:146-158`) :

1. Il supprime les morceaux partiellement téléchargés du modèle, dans `~/.ollama/models/blobs` (`ollama_registry.rs:152-180`) — au plus **64 morceaux** sont suivis pendant un téléchargement (`ollama_registry.rs:121-125`) ;
2. Il demande au moteur de **supprimer le modèle** (`ollama_registry.rs:182-206`).

**Conséquence à écrire sans l'adoucir : un téléchargement annulé repart de zéro.** Ce n'est pas une reprise différée, c'est un nettoyage.

**L'exception à connaître** : ce nettoyage ne s'applique **pas** à une mise à jour de modèle (`model_downloads.rs:147`). Annuler la mise à jour d'un modèle déjà installé laisse la version précédente en place — c'est voulu, et c'est la bonne décision.

**La file d'attente ne survit pas à la fermeture.** Elle vit uniquement en mémoire (`model_downloads_store.rs:20`, `:25-29`) : aucun fichier ne l'enregistre. Après un redémarrage de Beaver, **rien ne reprend tout seul** et rien ne signale qu'un téléchargement était en cours. Il faut le relancer à la main.

**Résolution** — relancer le téléchargement depuis l'écran des modèles. Il n'y a pas d'autre voie, et il n'y a rien à nettoyer soi-même.

### 7. La carte graphique n'est pas utilisée

**Symptôme** — les modèles locaux répondent très lentement, et l'indicateur de matériel affiche **« CPU · RAM »** au lieu de « VRAM » ou « RAM ».

**Cet indicateur est fiable, et c'est le bon endroit où regarder.** Beaver le calcule à partir de ce que le moteur déclare réellement charger : si aucun modèle chargé n'occupe de mémoire graphique, l'étiquette devient « CPU · RAM » (`polling.rs:41-56`). Il s'affiche si le réglage **« Afficher l'état GPU »** est activé (`ollama-settings-section.tsx:118-127`).

**Trois causes distinctes, à ne pas confondre :**

**a) Le réglage est sur processeur seul.** Le mode « processeur » impose au moteur de n'utiliser aucun accélérateur (`compute_mode.rs:18-31`). Le réglage est dans Réglages › Ollama, sous **« Moteur d'inférence pour les modèles locaux »** (`fr.json:1070`), avec les deux choix processeur et carte graphique. ⚠️ **Ce réglage n'est pas affiché sur Mac** (`ollama-settings-section.tsx:90`) : sur un Mac à puce Apple, le moteur décide seul.

**b) Le constructeur de la carte n'a pas été reconnu.** La détection diffère par système (`gpu_detect.rs`) : sous Linux, lecture des identifiants constructeur dans `/sys/class/drm/*/device/vendor` — `0x1002` pour AMD, `0x10de` pour NVIDIA, `0x8086` pour Intel (`:49-68`) ; sous Windows, interrogation de la liste des cartes vidéo par PowerShell, avec reconnaissance sur les mots `nvidia`, `geforce`, `quadro`, `amd`, `radeon`, `intel` (`:70-97`) ; sous macOS, lecture de la marque du processeur (`:31-47`). Un pilote absent, ou une carte dont le nom ne contient aucun de ces mots, donne un constructeur inconnu.

**c) Le modèle ne tient pas dans la mémoire graphique** et le moteur en bascule une partie sur la mémoire ordinaire. C'est la section 8.

**Résolution :**

1. **Vérifier le réglage** dans Réglages › Ollama, hors Mac.
2. **Sous Linux avec une carte AMD ou NVIDIA** : vérifier que la carte est vue par le système avec `ls /sys/class/drm/`. Si rien n'y correspond, le pilote graphique n'est pas chargé — c'est un problème du système, pas de Beaver.
3. **Sous Windows** : le pilote de la carte doit être installé ; c'est lui qui fournit le nom que Beaver lit.
4. **Prendre un modèle plus petit ou plus compressé** — voir *Modèles › Matériel et VRAM*.

⚠️ **La détection du constructeur ne sert qu'à choisir l'archive à télécharger sous Linux** (voir *Dépannage › Installation*, section 12). Elle ne commande pas le choix processeur/carte graphique du moteur en fonctionnement : ce choix vient du réglage et de l'autodétection propre au moteur.

### 8. Le modèle est trop lourd pour la mémoire

**Symptôme** — au lancement d'une installation, une notification passagère : **« Ce modèle peut nécessiter plus de mémoire que votre machine n'en possède. Le téléchargement va continuer. »** (`fr.json:618`, affichée 4 secondes par `model-install-button.tsx:42`).

**Ce que fait réellement ce contrôle** (`commands/ollama_setup.rs:203-210`) : il compare la taille annoncée du fichier à la mémoire graphique mesurée, et prévient si elle ne tient pas. **Il ne bloque rien** — le message le dit lui-même.

**Trois limites de ce contrôle, toutes vérifiées, toutes à écrire :**

1. **Quand la mémoire n'est pas mesurable, le contrôle répond « ça tient ».** La valeur `0` est traitée comme un succès (`ollama_setup.rs:205-207`). Conséquence : sur une machine où la mesure échoue, **l'avertissement ne s'affiche jamais**, quelle que soit la taille du modèle.
2. **Il ne compte que le fichier du modèle**, pas le contexte de la conversation, qui s'y ajoute et grandit avec elle.
3. **C'est une notification passagère**, pas une confirmation. Quelqu'un qui regarde ailleurs pendant quatre secondes ne la verra pas, et rien ne la rappelle ensuite.

**La mesure de la mémoire décide aussi de la taille de contexte** accordée aux modèles locaux, en trois paliers (`services/gpu_vram.rs:44-48`) : **32 768 jetons** au-dessus de 24 000 Mo, **24 576 jetons** au-dessus de 12 000 Mo, **8 192 jetons** en dessous — et **8 192 également quand la mesure échoue**. Un contexte bloqué à 8 192 jetons sur une machine bien dotée est donc le symptôme d'une mesure ratée, pas d'un choix.

**Résolution** — voir *Modèles › Matériel et VRAM*, qui porte la table de correspondance complète. En deux lignes ici : prendre une compression plus forte du même modèle, ou descendre d'une taille.

### 9. Redémarrer le moteur

**Où** — Réglages › Ollama. Le bouton apparaît en permanence à côté du réglage d'accélération hors Mac, et une ligne **« Redémarrage nécessaire — Les changements seront appliqués après le redémarrage d'Ollama »** s'ajoute dès qu'un réglage concerné a été modifié (`ollama-settings-section.tsx:129-143` ; textes `fr.json:1076-1077`).

**Quels réglages exigent ce redémarrage** (`ollama-settings-section.tsx:72-75`, appelé en `:86`, `:100`, `:114`) : la durée avant libération de la mémoire, le choix processeur ou carte graphique, et le chargement de plusieurs modèles à la fois. Le réglage « Afficher l'état GPU » ne le demande pas (`:125`).

**Ce que le redémarrage fait vraiment** (`manager_runtime.rs:12-23`) : il arrête le moteur avec un budget de **10 secondes**, puis relance la séquence complète de la section 3 — détection d'un moteur externe sur 11434, puis lancement sur un port libre.

**Les trois issues, et leur message :**

| Ce qui s'est passé | Message affiché | Source |
|---|---|---|
| Le moteur de Beaver a redémarré | **« Ollama redémarré »** | `fr.json:1074` |
| Un moteur externe a été retrouvé | **« Ollama externe réutilisé »** | `fr.json:1075` |
| L'arrêt ou le lancement a échoué | Le message du tableau des erreurs | `ollama-settings-section.tsx:56-59` |

⚠️ **« Ollama externe réutilisé » veut dire que rien n'a changé.** Sur un moteur externe, l'arrêt ne fait rien du tout (`manager_stop.rs:21-23`) et le redémarrage se contente de le retrouver. Les réglages modifiés **ne sont pas appliqués**, et la notification est présentée comme un succès. C'est le piège numéro un de cette page — voir « Anomalies relevées ».

**Si l'arrêt échoue**, le message est **« Ollama n'a pas pu s'arrêter correctement. »** (`fr.json:706`). Il n'y a alors pas d'action possible depuis l'application : fermer Beaver et le relancer.

### 10. « Connexion à Ollama perdue » pendant une conversation

**Symptôme** — au milieu d'une réponse, la conversation s'interrompt sur **« Connexion à Ollama perdue »** (`fr.json:1376` ; code produit en `ollama_stream_request.rs:195`, `ollama_stream.rs:200`, `ollama_collect.rs:56`, traduit via `src/lib/agent-error-codes.ts:2`).

**Cause** — la requête vers le moteur n'a pas abouti, soit parce que la connexion a échoué, soit parce qu'elle a expiré (`ollama_stream_request.rs:193`). C'est le symptôme de la section 2, rencontré depuis une conversation plutôt que depuis les réglages.

**À distinguer de « Erreur serveur Ollama »** (`fr.json:1377` ; code produit en `ollama_stream_request.rs:167`) : là, le moteur a répondu, mais par une erreur. Ce message n'arrive **qu'après épuisement des reprises**.

**Les reprises automatiques, à écrire parce qu'elles sont invisibles et généreuses.** Sur une erreur temporaire du moteur — codes HTTP 500, 502, 503, 504 — Beaver réessaie **jusqu'à dix fois** (`route_profile/policy_types.rs:74-83`), avec une attente croissante de **350 millisecondes multipliées par le numéro d'essai**, plafonnée au sixième (`ollama_retry_indicator.rs:40-42`). Deux autres reprises existent, plus discrètes : le retrait d'une capacité que le modèle ne prend pas en charge, et jusqu'à un certain nombre de reprises après un défaut d'analyse des appels d'outils (`ollama_stream_request.rs:109-141`).

**Cette générosité est propre au moteur local.** Les fournisseurs distants n'ont **aucune reprise automatique** (`policy_types.rs:74-79` : la valeur est zéro pour toutes les autres routes) — voir *Dépannage › Fournisseurs et clés*, qui explique pourquoi.

**Résolution** — reprendre la conversation après avoir vérifié l'état du moteur (section 2). La conversation elle-même est conservée ; seule la réponse en cours est perdue.

---

## Tableaux

### Tableau — Les messages d'erreur du moteur

Les vingt codes du moteur (`services/ollama_manager/error.rs:11-32`) et leur message français (`src/i18n/fr.json:689-709`). Ceux qui ne concernent que la première installation renvoient à *Dépannage › Installation*.

| Message affiché | Ce qui s'est passé | Ce qu'il faut faire |
|---|---|---|
| « Ollama est actuellement indisponible. » | Aucun moteur joignable | Section 2 |
| « Ollama n'a pas pu démarrer. » | Le lancement a échoué, ou le moteur n'a pas répondu en 10 s, ou sa version ne correspond pas à celle installée | Section 2, puis section 9 |
| « Ollama n'a pas pu s'arrêter correctement. » | L'arrêt a dépassé son budget | Fermer et relancer Beaver |
| « Ollama n'a pas répondu avant la fin du délai. » | Le moteur n'a pas répondu dans les **45 secondes** de l'installation (`commands/ollama_setup.rs:166-186`, échéance posée en `:166`) | Relancer ; fréquent au premier démarrage sur une machine lente |
| « Une autre opération Ollama est déjà en cours. » | Deux opérations lancées en même temps ; une seule est permise (`constants.rs:3`) | Attendre la fin de la première |
| « L'opération Ollama a été annulée. » | Bouton « Annuler » | Relancer quand vous voulez |
| « Beaver est en cours de fermeture. » | Opération demandée pendant l'arrêt de l'application | Relancer Beaver |
| « Le dossier des modèles chevauche les fichiers d'installation d'Ollama. » | Le dossier des modèles recouvre un dossier de travail du moteur (`spawn_profile.rs:162-169`) | Voir « Points à confirmer », point 3 |
| « Le stockage d'Ollama est inaccessible. Vérifie l'espace libre et les autorisations. » | Écriture impossible dans le dossier de données | *Dépannage › Installation*, section 11 |
| « L'installation d'Ollama est endommagée ou incompatible. » | Le contenu installé n'est pas celui attendu | Relancer l'installation depuis Réglages › Ollama |
| « Ollama n'est pas installé par Beaver. » | Aucune installation présente | Lancer le téléchargement depuis Réglages › Ollama |
| « L'installation d'Ollama doit être réparée avant de continuer. » | Une installation précédente s'est interrompue | Bouton « Réessayer » |
| « La réparation d'Ollama est temporairement impossible. Réessaie. » | Une autre opération occupe le dossier | Attendre quelques secondes |
| « L'état de mise à jour d'Ollama est illisible et doit être réparé. » | Le journal d'installation est corrompu | Bouton « Réessayer » |
| « La mise à jour est installée, mais son nettoyage doit être repris. » | Nettoyage post-mise à jour inachevé | Redémarrer Beaver |
| « La validation d'Ollama n'a pas pu se terminer. Réessaie. » | Contrôle final non abouti, ou aucun port libre obtenu (`port.rs:47-63`) | Réessayer |
| « Le téléchargement d'Ollama a échoué… », « …vérification d'intégrité », « L'extraction d'Ollama a échoué. » | Première installation | *Dépannage › Installation*, sections 8 à 10 |
| « Beaver a rencontré une erreur interne avec Ollama. » | Défaut interne | Redémarrer Beaver ; signaler si ça persiste |

**Un message ne figure dans aucun de ces cas** : **« Ollama ne peut pas terminer cette opération. Réessaie. »** (`fr.json:689`). C'est le repli affiché quand le code reçu n'est pas reconnu (`src/lib/ollama-runtime-error.ts:4`, `:26-31`).

### Tableau — Ce que chaque état du moteur permet

Reprise de la section 4, à publier telle quelle : c'est le tableau que les lecteurs consulteront.

### Tableau — Les délais et limites du moteur

| Ce qui est borné | Valeur | Source |
|---|---|---|
| Intervalle entre deux mesures d'état | **2 secondes** | `polling.rs:9` |
| Délai de la sonde de santé | **2 secondes** | `polling.rs:10` |
| Délai de détection d'un moteur externe | **250 millisecondes** | `constants.rs:24` |
| Port testé pour un moteur externe | **11434** | `constants.rs:22` |
| Budget de lancement du moteur | **10 secondes** | `constants.rs:15` |
| Budget d'arrêt lors d'un redémarrage | **10 secondes** | `manager_runtime.rs:16` |
| Attente du moteur à l'installation | **45 secondes** | `commands/ollama_setup.rs:166` |
| Téléchargement d'un modèle | **3600 secondes** | `ollama_registry.rs:80` |
| Téléchargements de modèles simultanés | **1** | `model_downloads_store.rs:15` |
| Modèles en file d'attente | **16** | `model_downloads_types.rs:3` |
| Opérations simultanées sur le moteur | **1** | `constants.rs:3` |
| Reprises sur erreur temporaire du moteur | **10** | `route_profile/policy_types.rs:76` |
| Attente entre deux reprises | **350 ms × numéro d'essai**, plafonné au 6ᵉ | `ollama_retry_indicator.rs:41` |
| Création d'un modèle personnalisé | **600 secondes** | `manager_process.rs:164-165` |

---

## Encadrés

> **⚠ Un moteur externe, ce n'est pas le même produit.** À placer en tête de la section 4.
> Si Réglages › Ollama affiche **« Ollama externe »**, Beaver se sert d'un moteur qu'il n'a pas lancé. Discuter avec un modèle fonctionne ; **télécharger, supprimer ou créer un modèle, non**, et les réglages d'accélération n'ont aucun effet. Pour retrouver toutes les fonctions, arrêtez le moteur externe puis redémarrez celui de Beaver.

> **ℹ Beaver n'utilise pas le port 11434.** À placer dans la section 3.
> Le port 11434 est celui qu'il **regarde** pour savoir si un moteur tourne déjà. Le sien démarre sur un port libre tiré au hasard, et n'écoute que sur votre machine. Un port 11434 occupé ne bloque donc jamais Beaver.

> **⚠ Annuler un téléchargement le supprime.** À placer dans la section 6.
> Les morceaux déjà obtenus sont effacés et le modèle est retiré du moteur. Relancer repart du début. La seule exception est la **mise à jour** d'un modèle déjà installé : annuler y laisse la version précédente intacte.

> **ℹ Il n'y a pas de journal du moteur.** À placer dans la section 2.
> Contrairement à ce que suggèrent d'anciennes documentations, le moteur lancé par Beaver n'écrit aucun fichier de traces. Il n'y a pas de journal à consulter.

> **⚠ Quand la mémoire n'est pas mesurable, l'avertissement de taille ne s'affiche jamais.** À placer dans la section 8.
> Le contrôle répond « ça tient » quand il ne sait pas mesurer. Sur une machine où la détection échoue, aucun modèle ne sera jamais signalé comme trop lourd — et le contexte restera bloqué au palier minimal de 8 192 jetons.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le téléchargement a échoué » sans autre explication | Un code unique couvre toutes les causes | Vérifier d'abord l'état du moteur, puis l'espace disque |
| Le bouton d'installation ne fait rien d'utile | Le moteur est externe : le téléchargement est impossible | Arrêter le moteur externe, redémarrer celui de Beaver |
| « Ollama externe réutilisé » après avoir changé un réglage | Le réglage n'a pas été appliqué : Beaver ne contrôle pas ce moteur | Section 4 |
| Le moteur passe indisponible puis revient tout seul | La sonde de santé a dépassé 2 secondes sur une machine chargée | Comportement attendu, rien à faire |
| Je cherche Beaver sur le port 11434 et ne trouve rien | Son moteur écoute sur un port tiré au hasard | Section 3 |
| Mon téléchargement a disparu après un redémarrage de Beaver | La file d'attente n'est pas enregistrée sur le disque | Le relancer à la main |
| Le contexte reste à 8 192 jetons | La mémoire graphique n'a pas pu être mesurée | Non réglable depuis l'application |
| Aucun avertissement sur un modèle manifestement trop gros | Mémoire non mesurée : le contrôle répond « ça tient » | Se fier à la table de *Modèles › Matériel et VRAM* |
| Le réglage processeur/carte graphique est absent | Il n'est pas affiché sur Mac | Comportement voulu |
| Les modèles locaux ont disparu de la liste | Le moteur est indisponible, ou n'a aucun modèle installé | Section 2 ; voir aussi « Anomalies relevées », point 4 |

---

## Renvois

- *Dépannage › Installation*, sections 8 à 10 — la première installation du moteur et sa reprise
- *Dépannage › Installation*, section 12 — la détection du constructeur de la carte graphique
- *Dépannage › Fournisseurs et clés* — les modèles distants, qui n'ont besoin d'aucun moteur local
- *Modèles › Matériel et VRAM* — quelle taille de modèle pour quelle machine
- *Modèles › Le moteur Ollama* — les réglages en fonctionnement nominal
- *Modèles › Installer un modèle* — le parcours nominal de téléchargement
- *Référence › Stockage local* — où vivent les modèles

---

## Anomalies relevées

Constatées en écrivant cette page, **non corrigées**. À arbitrer par l'équipe.

1. **Un redémarrage sans effet est annoncé comme un succès.** Sur un moteur externe, l'arrêt ne fait rien (`manager_stop.rs:21-23`), le redémarrage retrouve le même moteur, et l'interface affiche **« Ollama externe réutilisé »** en notification de **succès** (`ollama-settings-section.tsx:60-63`). L'utilisateur qui vient de changer son réglage d'accélération croit l'avoir appliqué. Le message est exact et l'issue est trompeuse.

2. **Le bouton de redémarrage n'est pas traduit en français.** La clé `settings.advanced.hardwareAccelRestart` vaut **« Restart Ollama »** dans `fr.json:1073`, alors que les six autres langues sont traduites — allemand « Ollama neu starten », espagnol « Reiniciar Ollama », italien « Riavvia Ollama », japonais « Ollama を再起動 », chinois « 重启 Ollama ». Seul le français porte le texte anglais.

3. **Rien ne vérifie que ce qui répond sur le port 11434 est bien Ollama.** La détection est une simple ouverture de connexion TCP (`port.rs:71-77`) ; le contrôle de version n'a lieu que pour un moteur lancé par Beaver (`manager_process.rs:114-117`). Un autre logiciel occupant ce port serait déclaré « Ollama externe ». La conséquence est bornée — la sonde de santé le repasserait à « indisponible » deux secondes plus tard (`polling.rs:96-105`) — mais l'état affiché serait faux dans l'intervalle, et l'utilisateur verrait un moteur apparaître et disparaître sans explication.

4. **Le moteur local disparaît de la liste des modèles sans message, contrairement aux fournisseurs distants.** Quand le catalogue d'un fournisseur distant échoue, il reste visible avec un compteur à zéro et la raison s'affiche à l'intérieur (`model-selector-list.tsx:107`, `:138-142`). Quand la liste des modèles locaux est vide ou en échec, l'entrée « ollama » est purement **supprimée** de la liste (`use-available-models.ts:57-59`, `:97-99`). L'utilisateur voit un fournisseur s'évanouir sans savoir pourquoi.

5. **Trois messages du moteur sont impossibles à distinguer d'un défaut de la machine.** « Le dossier des modèles chevauche les fichiers d'installation d'Ollama. » (`fr.json:693`) décrit une situation qui ne peut naître que d'une variable d'environnement `OLLAMA_MODELS` posée à la main (`spawn_profile_paths.rs:9-57`) : le message ne le dit pas, et l'utilisateur n'a aucun moyen de deviner l'action à faire.

---

## Points à confirmer

**Écarts à arbitrer avant publication**

1. **Faut-il documenter l'écart de la section 4 comme un comportement ou comme un défaut ?** Le tableau des fonctions indisponibles avec un moteur externe est exact, mais il décrit une limite que l'application n'annonce nulle part à l'écran. Décider si le site le présente comme un choix assumé (« Beaver ne pilote que le moteur qu'il a lancé ») ou comme une limite temporaire.

2. **Le nettoyage à l'annulation a-t-il le temps de s'exécuter à la fermeture de l'application ?** Quand Beaver se ferme pendant un téléchargement, l'annulation est déclenchée puis le travail est attendu (`model_downloads.rs:104-109`), ce qui déclenche la suppression des morceaux partiels et du modèle. Mais la fermeture a un budget global, et rien dans le code lu ne garantit que ce nettoyage tient dedans. **À provoquer et à observer** avant d'écrire que l'état est toujours propre après une fermeture.

3. **Quelle est l'action réelle pour « Le dossier des modèles chevauche les fichiers d'installation d'Ollama » ?** Le contrôle est vérifié (`spawn_profile.rs:162-169`), la cause aussi, mais **aucune procédure de résolution n'existe dans le code ni dans l'interface**. La ligne du tableau renvoie ici volontairement. Trancher avec l'équipe : documenter la variable d'environnement en cause, ou déclarer le cas hors périmètre.

4. **La taille d'un modèle avant téléchargement.** L'avertissement de la section 8 compare une taille annoncée par le catalogue (`model-install-button.tsx:39-41`) ; sa provenance et sa fiabilité n'ont pas été remontées jusqu'à la source. À vérifier avant d'écrire quoi que ce soit sur sa précision.

5. **Le comportement d'un moteur externe lancé par une autre installation de Beaver.** Non exploré. Deux instances de Beaver sur la même machine se verraient probablement l'une l'autre par la détection du port 11434 — sauf que celui de Beaver n'écoute jamais sur 11434, donc a priori non. À confirmer.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

6. **L'indicateur de matériel** : son emplacement exact dans la fenêtre, ce qu'il affiche quand le moteur est indisponible, et si « CPU · RAM » est lisible dans les deux thèmes.
7. **La ligne d'erreur sous le bouton d'installation** (`model-install-button.tsx:115-119`) : sa persistance, et s'il existe un moyen de la faire disparaître autrement qu'en relançant.
8. **L'écran Réglages › Ollama dans l'état « externe »** : voir si quelque chose y signale les fonctions indisponibles, ou si le tableau de la section 4 est la seule source de cette information.
9. **La notification d'avertissement de mémoire** : quatre secondes, position à l'écran, et ce qui se passe si plusieurs installations sont lancées à la suite.
10. **Les indicateurs de reprise** pendant les dix reprises de la section 10 (`ollama_retry_indicator.rs`) : ce que l'utilisateur voit exactement pendant qu'elles s'enchaînent, et si l'attente cumulée est perceptible.
