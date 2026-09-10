# Connecteurs, extensions et canaux externes qui ne répondent pas

**Emplacement site** — Dépannage › Connecteurs, extensions et canaux
**Répond à** — « J'ai branché un connecteur, une extension ou un bot de messagerie, et ça ne marche pas — ou ça marchait et ça s'est arrêté. Où est le problème ? »
**Sources** — `src-tauri/src/services/mcp_bridge/` : `config.rs`, `registry.rs`, `registry_probe.rs`, `process_manager.rs`, `process_pool.rs`, `stdio_cmd.rs`, `stdio_catalog.rs`, `stdio_session.rs`, `token_validation.rs`, `trusted.rs`, `transport.rs`, `http.rs`, `work_supervision.rs` ; `src-tauri/src/services/mcp_oauth/` : `flow.rs`, `callback_server.rs`, `storage.rs`, `trusted_oauth.rs` ; `src-tauri/src/services/extensions/` : `error_codes.rs`, `manifest.rs`, `runtime_host_load.rs`, `runtime_restart.rs`, `extension_recovery.rs`, `types.rs` ; `src-tauri/resources/extension-host/contract.json` ; `src-tauri/src/services/gateway/` : `service_runtime.rs`, `supervisor.rs`, `reconnect_policy.rs`, `agent_bridge.rs`, `agent_bridge_support.rs`, `service_consumer.rs`, `refusal_audit.rs`, `security/audit.rs`, `security/audit_store.rs`, `security/allowlist.rs`, `security/rate_limit.rs`, `security/rate_state.rs`, `config_validation.rs`, `channels/telegram_support.rs`, `channels/slack_support.rs`, `channels/discord_support.rs` ; `src-tauri/src/services/extensions/host_channel.rs` ; `src-tauri/src/models/gateway_config.rs` ; `src-tauri/src/services/agent_local/tool_mcp_call.rs`, `tool_mcp.rs` ; `src/components/connectors/mcp-oauth-dialog.tsx`, `mcp-config-dialog.tsx` ; `src/components/channels/channel-error.ts`, `channels-detail.tsx`, `__tests__/channel-error.test.ts` ; `src/components/extensions/extensions-host-panel.tsx` ; `src/types/extension-ui-contract.generated.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026. Aucune panne n'a été provoquée à l'écran : les messages sont documentés d'après le code, avec leur source.

> **Trois systèmes distincts sur une même page.** Un **connecteur MCP** est un service externe qui prête des outils à l'agent. Une **extension** est du code qui tourne dans un processus séparé de Beaver. Un **canal externe** est un bot de messagerie qui parle à l'agent depuis Telegram, Slack ou Discord. Ils tombent en panne différemment, mais un utilisateur ne fait pas la différence tant qu'on ne la lui a pas montrée.

---

## Plan de page proposé

1. Reconnaître lequel des trois est en cause
2. Un connecteur MCP qui ne démarre pas
3. Un outil MCP qui échoue en pleine conversation
4. La connexion OAuth d'un connecteur a échoué
5. Le jeton d'un connecteur a expiré
6. L'hôte d'extensions est indisponible
7. Une extension refusée ou incompatible
8. Un chargement d'extension interrompu
9. Un canal externe silencieux
10. Un canal externe en erreur
11. Le journal d'audit des canaux

---

## Contenu

### 1. Reconnaître lequel des trois est en cause

| Ce que vous observez | Système concerné | Où regarder |
|---|---|---|
| Un outil promis par un service externe ne répond pas | **Connecteur MCP** | Réglages › Connecteurs, statut « Connecté » / « Déconnecté » (`fr.json:1590-1591`) |
| Un plugin officiel ou une extension personnalisée est absent, en erreur ou incompatible | **Extension** | Réglages › Extensions › Hôte, section **« Dernières erreurs de chargement »** (`extensions-host-panel.tsx:54-73`) |
| Un bot Telegram, Slack ou Discord ne répond plus | **Canal externe** | Réglages › Canaux, statut du compte (`fr.json:143-147`) |

Les trois ont un point commun : **une panne de l'un n'arrête pas Beaver.** L'agent continue de fonctionner sans les outils manquants.

### 2. Un connecteur MCP qui ne démarre pas

**Le premier point à écrire, parce qu'il élimine la moitié des questions : Beaver n'accepte pas un serveur MCP quelconque.** La liste des connecteurs installables localement est **fermée**, inscrite dans le code (`stdio_catalog.rs:13-21`, `:1-11`) :

| Connecteur | Ce que Beaver lance |
|---|---|
| `context7` | `npx @upstash/context7-mcp@2.2.5` |
| `huggingface` | `npx @llmindset/hf-mcp-server@0.3.13` |
| `producthunt` | `uvx product-hunt-mcp==0.1.0` |
| `reddit` | `npx reddit-mcp-server@1.4.5` |
| `imessage` | `deno run --allow-read --allow-env --allow-sys --allow-ffi jsr:@wyattjoh/imessage-mcp@0.4.2` — **macOS uniquement** (`registry.rs:44-46`) |

Trois programmes seulement sont autorisés à démarrer : **`npx`, `uvx`, `deno`** (`stdio_cmd.rs:3`). Chaque argument doit correspondre au motif `[a-zA-Z0-9@/_.:=-]` (`stdio_cmd.rs:5-6`, `:27-32`), et la commande doit être **exactement** celle du catalogue — même programme, même nombre d'arguments, mêmes valeurs (`stdio_cmd.rs:52-74`).

Pour les connecteurs distants, la liste est également fermée : **treize points d'accès approuvés** (`trusted.rs:1-15`) — Gmail, Google Drive, Google Agenda, Canva, Figma, Notion, Slack, Linear, Lucid, Sentry, Vercel, Apify, GitHub. L'adresse doit être en **HTTPS**, sans paramètre de requête, sans port explicite, et le chemin doit correspondre au caractère près (`trusted.rs:17-32`).

**Les causes de non-démarrage, dans l'ordre où le code les rencontre :**

| Cause | Message technique | Source |
|---|---|---|
| Le programme n'est pas installé sur la machine | « runtime requis non trouvé dans le PATH » | `process_manager.rs:116-117` |
| Le connecteur n'est pas au catalogue | « connecteur stdio non autorisé » | `stdio_cmd.rs:61` |
| La commande ne correspond pas au catalogue | « commande MCP non autorisée » | `stdio_cmd.rs:64`, `:71` |
| Un argument contient un caractère interdit | « argument invalide » | `stdio_cmd.rs:29` |
| Huit processus MCP tournent déjà | « connecteur MCP indisponible » | `process_pool.rs:37-40`, `:55-68` ; limite `work_supervision.rs:10` |
| Beaver est en cours de fermeture | La même phrase | `process_pool.rs:42-44` |

**`npx` et `uvx` téléchargent le paquet au premier lancement.** C'est la cause la plus banale d'un démarrage qui semble bloqué : la machine télécharge. Le code ne pose aucun délai sur le démarrage lui-même — seulement sur la première interrogation, à **20 secondes** (`registry.rs:12`, `registry_probe.rs:7-12`).

**Ce que dit l'interface**, quel que soit le motif ci-dessus : **« Impossible de valider le connecteur. Vérifiez le token puis réessayez. »** (`fr.json:1628`). Le message accuse le jeton dans tous les cas — voir « Anomalies relevées ».

**Trois autres bornes utiles :**

- **32 connecteurs** configurés au maximum ; au-delà : « limite de connecteurs atteinte » (`config.rs:9`, `:41-43`) ;
- un connecteur inactif est **arrêté au bout de 10 minutes** sans usage (`process_manager.rs:13` ; `process_pool.rs:130-141`) — son redémarrage à l'appel suivant est normal ;
- le catalogue d'outils d'un connecteur est **gardé en mémoire 5 minutes** (`registry.rs:11`). Un outil ajouté côté serveur peut donc mettre jusqu'à cinq minutes à apparaître.

### 3. Un outil MCP qui échoue en pleine conversation

L'agent n'appelle pas un connecteur directement : il passe par un outil unique, en mode `call` (`tool_mcp.rs:13`). L'échec porte alors un code précis côté moteur, mais **vous ne lisez que la phrase de sa catégorie** (voir `13-depannage/agent-et-outils.md`, section 4).

| Ce qui a échoué | Code interne | Phrase affichée |
|---|---|---|
| Le connecteur n'est plus actif, ou l'outil n'existe plus | `mcp_tool_unavailable` | L'outil est temporairement indisponible. |
| Les arguments ne respectent pas le schéma de l'outil | `invalid_mcp_arguments` | La demande transmise à l'outil est invalide. |
| L'appel a dépassé **60 secondes** | `mcp_call_timeout` | L'outil n'a pas répondu à temps. |
| Le service est injoignable | `mcp_service_unavailable` | L'outil est temporairement indisponible. |
| Le serveur a renvoyé une erreur | `mcp_server_error` | Un service externe a empêché l'opération. |
| La réponse du serveur est illisible | `mcp_invalid_response` | Un service externe a empêché l'opération. |
| Le transport a échoué | `mcp_transport_failed` | Un service externe a empêché l'opération. |
| L'outil lui-même signale une erreur | `mcp_tool_error` | Un service externe a empêché l'opération. |

Sources : `tool_mcp_call.rs:26-32`, `:34-41`, `:8` et `:51-58`, `:62-87`, `:106-120`.

**Un conseil accompagne chacun**, et il est différent selon que l'appel a pu partir ou non : **« Aucun appel d'outil n'a été envoyé ; une nouvelle tentative est sûre. »** quand le service était injoignable, **« Vérifier l'état du service avant de relancer : l'action a pu être exécutée. »** dans tous les autres cas (`tool_mcp_call.rs:78-86`). C'est une distinction qui mérite d'être expliquée sur le site : elle dit si relancer est sans risque.

**La sortie d'un outil MCP est coupée à 4 096 caractères** et marquée comme tronquée (`tool_mcp_call.rs:118`, `:124-136`). Les caractères de contrôle bidirectionnels sont retirés (test `:169-177`).

### 4. La connexion OAuth d'un connecteur a échoué

Le parcours, tel que le code l'exécute (`mcp_oauth/flow.rs:53-136`) : Beaver interroge le service pour trouver son serveur d'autorisation, vérifie que les adresses obtenues font partie des adresses approuvées, ouvre un port local sur `127.0.0.1`, ouvre votre navigateur, attend le retour, échange le code contre des jetons, teste le connecteur, puis enregistre.

**Sept causes d'échec distinctes**, toutes vérifiées :

| Cause | Message technique | Source |
|---|---|---|
| Le serveur d'autorisation est introuvable ou non approuvé | Renvoyé par la découverte | `flow.rs:60-61` |
| Le port local n'a pas pu être ouvert | « callback OAuth indisponible » | `callback_server.rs:31-38` |
| L'adresse d'enregistrement n'est pas en HTTPS | « endpoint d'enregistrement non HTTPS » | `flow.rs:76-78` |
| Le service ne fournit ni identifiants statiques ni enregistrement | « pas de credentials disponibles pour ce service » | `flow.rs:82` |
| **Vous n'avez pas terminé dans les 5 minutes** | « délai d'attente dépassé » | `callback_server.rs:11`, `:53` |
| 50 requêtes sont arrivées sur le port sans callback valide | « trop de requêtes sans callback valide » | `callback_server.rs:14`, `:66-68` |
| Le connecteur refuse le jeton obtenu au test final | « test MCP échoué » ou « test MCP expiré » | `registry_probe.rs:55-58` |

**Ce que vous lisez, dans les sept cas : « Échec de la connexion. Réessayez. »** (`fr.json:1642`). Le motif est bien transmis à l'interface, qui ne l'utilise pas — voir « Anomalies relevées », point 2.

**Trois détails à connaître :**

- **Cinq connexions OAuth au maximum en parallèle** ; au-delà : « trop de flows OAuth en cours » (`flow.rs:16`, `:143-145`).
- **Relancer la même connexion pendant qu'elle tourne ne fait rien** : la seconde tentative est écartée en silence, sans message d'erreur (`flow.rs:18`, `:33`, `:140-142`). Le bouton **« Cela n'a pas fonctionné ? Relancez. »** (`fr.json:1638`) n'a donc d'effet qu'une fois la première tentative terminée ou annulée.
- La page qui s'affiche dans le navigateur après une connexion réussie dit **« Authentification en cours — Vous pouvez fermer cet onglet et retourner dans l'application. »** Elle est **écrite en dur dans le code, en français uniquement** (`callback_server.rs:16-23`).

### 5. Le jeton d'un connecteur a expiré

Beaver renouvelle le jeton **30 secondes avant son échéance** (`mcp_oauth/storage.rs:46-48`). Le renouvellement est protégé par un verrou par connecteur, et l'état est relu après l'avoir obtenu : deux appels simultanés ne renouvellent pas deux fois (`storage.rs:51-58`).

**Deux situations sans issue automatique :**

- **Le service n'a pas fourni de jeton de renouvellement** : « token expiré et pas de refresh_token » (`storage.rs:59-62`). Il faut refaire la connexion.
- **Le renouvellement échoue** — service injoignable, jeton révoqué de votre côté : « échec du rafraîchissement du token » (`storage.rs:95-98`). Beaver **ne réessaie pas** et ne prévient pas : l'échec remonte comme une indisponibilité de l'outil.

Le point d'accès du renouvellement est **revérifié contre la liste approuvée** avant chaque appel (`storage.rs:73`) : un service qui changerait d'adresse casserait le renouvellement plutôt que d'envoyer le jeton ailleurs. C'est voulu.

Les jetons vivent dans le coffre chiffré, pas dans un fichier de configuration — voir `11-securite/vault-et-cles-api.md`.

### 6. L'hôte d'extensions est indisponible

Les extensions ne tournent pas dans Beaver : elles tournent dans un **processus Node.js séparé**, l'hôte. Son état est visible dans Réglages › Extensions › Hôte : **Arrêté**, **Démarrage**, **En cours**, **Erreur** (`fr.json:1872-1875` ; `contract.json` → `hostStates`).

**Node.js 20 au minimum** est exigé (`types.rs:6`). Une version inférieure fait échouer la validation de démarrage (`runtime_host_load.rs:46`).

**Les messages que vous pouvez lire** (`fr.json:1895`, `:1901-1902`, `:1930`, `:1940`) :

| Message affiché | Ce qu'il signifie |
|---|---|
| « L'hôte d'extensions est indisponible. » | Le processus ne tourne pas ou ne répond plus |
| « L'hôte d'extensions traite trop de demandes. Réessaie dans un instant. » | Plus de **64 demandes** en attente (`host_channel.rs:49-51` ; `contract.json` → `maxPendingRequests`) |
| « L'hôte d'extensions n'a pas répondu à temps. » | Dépassement d'un des délais du contrat |
| « L'hôte d'extensions n'a pas confirmé son arrêt. » | Suivi de **« Quittez complètement Beaver, puis relancez-le avant de réactiver cette extension. »** (`extensions-host-panel.tsx:29`, `:51`) |
| « L'hôte d'extensions n'est pas compatible avec cette version de Beaver. » | Le protocole annoncé ne correspond pas (`runtime_host_load.rs:43-45`) |

**Les délais du contrat**, tous en millisecondes (`src-tauri/resources/extension-host/contract.json` → `timeouts`) :

| Opération | Délai |
|---|---|
| Appel d'un outil d'extension | **55 000** |
| Requête vers l'hôte | **60 000** |
| Appel d'un outil MCP depuis une extension | **25 000** |
| Requête d'une extension vers Beaver | **30 000** |
| Action d'interface | **15 000** |
| Gestionnaire d'événement | **5 000** |
| Arrêt de l'hôte | **5 000** |

**L'hôte ne redémarre pas indéfiniment.** Beaver s'autorise **3 redémarrages automatiques par fenêtre de 300 secondes** (`contract.json` → `maxHostRestartsPerWindow`, `hostRestartWindowSeconds` ; appliqué dans `runtime_restart.rs:12-27`). Passé ce budget, plus aucun redémarrage automatique n'a lieu jusqu'à une action manuelle. C'est exactement le cas où le bouton **« Redémarrer l'hôte »** sert à quelque chose (`fr.json:1723`).

**Le dernier recours** est le bouton **« Désactiver les extensions hébergées »** (`fr.json:1724`), décrit dans l'application sous le titre **« Mode de récupération »** par **« Désactive les plugins Beaver et les extensions locales afin de retrouver un Beaver fonctionnel. »** (`fr.json:1877-1878` ; `extensions-host-panel.tsx:80-89`).

### 7. Une extension refusée ou incompatible

**Une extension est refusée avant d'être chargée** quand son manifeste annonce une version d'API différente de celle de Beaver — actuellement **1** (`manifest.rs:49-54` ; `contract.json` → `apiVersion`). Message affiché : **« Cette extension n'est pas compatible avec cette version de Beaver. »** (`fr.json:1919`), état **« Incompatible »** dans la liste (`fr.json:1732`).

**Il n'y a aucun mode de compatibilité** : la comparaison est stricte, une extension écrite pour une autre version ne se charge pas. La seule issue est une mise à jour de l'extension, ou de Beaver.

**Les autres refus au moment de l'installation**, avec leur message exact (`fr.json:1907-1931`) : dépôt Git non téléchargeable, dépôt Git qui n'a pas répondu à temps, adresse ou référence Git invalide, nom ou version npm invalide, paquet qui n'est pas une extension Beaver, manifeste ou point d'entrée invalide, lien symbolique non pris en charge, identifiant déjà installé, nombre maximal atteint, dépendances non installables, environnement d'installation invalide ou trop volumineux.

**Les limites qui déclenchent « Le nombre maximal d'extensions est atteint. »** (`contract.json` → `limits`) : **128 extensions personnelles**, **132 au total** plugins officiels compris, **64 outils par extension**, **256 outils en tout**.

**Une extension activée reçoit la confiance, pas une permission limitée.** La fenêtre d'activation le dit sans détour : **« Cette extension aura un accès total aux données, secrets et fonctions de votre compte. Beaver ne l'a pas auditée. »** avec le bouton **« Faire confiance et activer »** (`fr.json:1806-1810`). Après une mise à jour, l'extension est **désactivée** et doit être réapprouvée : **« La mise à jour désactivera l'extension. Examinez sa nouvelle version avant de lui refaire confiance. »** (`fr.json:1766`). Si les fichiers changent hors de Beaver, l'extension est arrêtée avec **« Les fichiers de l'extension ont changé et doivent être vérifiés à nouveau. »** (`fr.json:1928`).

**Les erreurs de chargement individuelles** sont listées sous **« Dernières erreurs de chargement »**, avec l'identifiant de l'extension, la phrase du diagnostic et, quand le code la connaît, la position dans le fichier (`extensions-host-panel.tsx:54-73`). Les plus utiles (`fr.json:1955-1964`) : « Module ou dépendance introuvable », « Erreur de syntaxe », « L'activation a échoué », « L'import du module a échoué », « Le point d'entrée de l'extension est indisponible », « L'hôte n'a pas confirmé le chargement de cette extension ».

**Les erreurs d'interface d'extension, en revanche, sont toutes ramenées à une seule phrase** — voir « Anomalies relevées », point 4.

### 8. Un chargement d'extension interrompu

Si Beaver s'est arrêté pendant le chargement d'une extension, il le sait au démarrage suivant grâce à un fichier témoin, et il **s'arrête avant de recharger** cette extension. Une fenêtre s'ouvre : **« Chargement d'extension interrompu — Beaver s'est arrêté pendant le chargement d'une extension. Examinez-la avant de choisir la suite. »** (`fr.json:1813-1814`).

Cinq réponses possibles (`fr.json:1816-1820` ; `extension_recovery.rs:42-62`) : ouvrir la fiche, **garder désactivée**, réessayer le chargement, supprimer les informations de reprise, **restaurer le registre précédent**.

**Le cas des interfaces d'extension est traité séparément et plus sévèrement** : après une interruption pendant le chargement d'une interface, Beaver démarre en mode sûr et **n'affiche aucun élément d'interface tiers** (`fr.json:1823-1832`). Le bandeau dit alors : **« Interfaces d'extension désactivées — Ce lancement reste en mode sûr et n'affiche aucune interface tierce. »** (`fr.json:1831-1832`).

C'est cohérent avec la règle du projet : une fonctionnalité est disponible et fonctionne, ou elle est invisible — jamais visible et cassée.

### 9. Un canal externe silencieux

**Le cas le plus déroutant, et celui qu'il faut expliquer le mieux : le canal affiche « Actif », et le bot ne répond pas.**

Un message venu de l'extérieur traverse cinq contrôles avant d'atteindre l'agent (`agent_bridge.rs:57-98`). **Quatre d'entre eux le refusent en silence** : rien n'est répondu à l'expéditeur, rien n'apparaît dans Beaver, le canal reste « Actif ».

| Contrôle | Ce qui le déclenche | Source |
|---|---|---|
| Validation du message | Identifiants malformés, message vide ou trop long | `agent_bridge_support.rs:27-41` |
| Compte non configuré | Le compte ne correspond à aucune configuration | `agent_bridge.rs:68-69` |
| **Liste des utilisateurs autorisés** | L'expéditeur n'y figure pas | `agent_bridge.rs:70-73` |
| **Limite de débit** | Trop de messages sur la minute écoulée | `agent_bridge.rs:74-83` |
| Fournisseur restreint | Le modèle choisi n'est pas autorisé hors du chat interactif | `agent_bridge.rs:92-98` |

**Les limites de débit par défaut** (`models/gateway_config.rs:44-46`) : **12 messages par minute et par utilisateur**, **120 par minute et par canal**, **300 par minute au total**. Le compteur est une fenêtre glissante de 60 secondes par utilisateur, canal et compte (`security/rate_state.rs:15-17` ; `security/rate_limit.rs:40-61`).

**La liste des utilisateurs autorisés est obligatoire.** Un canal dont la liste est vide ne répond à personne : `contains` ne renvoie vrai que sur une entrée présente, et le joker `*` est refusé à la construction (`security/allowlist.rs:29-37`, `:61-66` ; `agent_bridge.rs:70`). La comparaison ignore la casse et les espaces autour (`allowlist.rs:78-80`). La liste est bornée à **1 000 entrées** de **128 caractères**, avec éviction de la plus ancienne (`allowlist.rs:3-4`, `:39-43`) — champ **« Utilisateurs autorisés »** dans l'interface (`fr.json:127`).

**Deuxième cause de silence, propre aux groupes** : l'option **« Mention requise en groupe »** (`fr.json:130`). Quand elle est active, un message de groupe qui ne mentionne pas le bot est ignoré avant même d'entrer dans la file (`channels/telegram_support.rs:91` ; `channels/slack_support.rs:82` ; `channels/discord_support.rs:25`).

**À écrire tel quel sur le site** : ce silence est un choix de sécurité, pas un défaut. Un bot qui répondrait « vous n'êtes pas autorisé » confirmerait son existence à n'importe qui.

### 10. Un canal externe en erreur

Cinq états sont affichés : **« Arrêté »**, **« Démarrage… »**, **« Actif »**, **« Erreur »**, **« Arrêt… »** (`fr.json:143-147`).

**La reconnexion est automatique et bornée** (`supervisor.rs:3-6`, `:36-63`) : le délai part de **1 seconde** et double à chaque échec jusqu'à **60 secondes**. Il repart à 1 seconde si la connexion précédente a tenu plus de **5 minutes**. Après **10 tentatives**, Beaver abandonne et passe le canal en **« Erreur »**.

**Une erreur d'authentification arrête tout immédiatement**, sans aucune tentative : un jeton révoqué ou invalide met le canal en erreur du premier coup (`supervisor.rs:36-42`). C'est la bonne réaction — réessayer avec un mauvais jeton dix fois de suite ne mène nulle part, et certains services en tiennent rigueur.

**Les trois phrases affichées** (`fr.json:150-152`) :

| Code renvoyé par le moteur | Phrase affichée |
|---|---|
| `invalidConfig` | La configuration du canal est invalide. |
| `unavailable` | Le canal est indisponible. |
| tout autre code | L'opération a échoué. Réessayez. |

Le bouton de test affiche **« Connexion réussie »** ou **« Connexion échouée »** (`fr.json:133-134`).

**Ce que le moteur vérifie avant de démarrer un compte** (`service_runtime.rs:15-30`) : identifiants de canal et de compte valides, fournisseur et modèle renseignés — sinon « provider ou modèle manquant » —, identifiants d'utilisateurs autorisés valides, et **présence de tous les jetons requis** pour ce canal — sinon « token manquant ». Slack en demande deux : **« Token bot Slack »** et **« Token app Slack »** (`fr.json:124-125`).

**Une reconnexion interne, distincte de celle du superviseur**, existe pour la connexion permanente d'un canal : de **1 à 30 secondes**, remise à zéro après **60 secondes** de connexion stable (`reconnect_policy.rs:4-6`, `:19-29`).

### 11. Le journal d'audit des canaux

**`~/.local/share/cl-go-dash/logs/gateway-audit.jsonl`** (`security/audit.rs:143-145`). C'est le seul endroit où retrouver ce qui est arrivé aux messages refusés en silence.

Une ligne JSON par événement, sept champs : horodatage, canal, compte, **empreinte de l'utilisateur**, action, décision de sécurité, erreur (`audit.rs:14-23`).

**Huit actions** (`audit.rs:27-36`) : `message_received`, `message_sent`, `blocked`, `rate_limited`, `agent_error`, `channel_started`, `channel_stopped`, `auth_failed`.

**L'identifiant de l'expéditeur n'est jamais écrit en clair** : il est remplacé par une empreinte HMAC-SHA256 tronquée à 8 octets, calculée avec une clé tirée au hasard et rangée dans le coffre (`audit.rs:74-88`). Conséquence pratique à écrire sur le site : **vous ne pouvez pas lire dans ce journal qui a été refusé** ; vous pouvez seulement voir que deux lignes concernent la même personne.

**Bornes** (`audit_store.rs:7-8`) : **10 000 entrées**, **2 048 octets par ligne**, et une purge par ancienneté selon la rétention configurée — **30 jours** par défaut, ramenée de force entre 1 et 365 (`audit.rs:12`, `:69-72`).

**Le journal peut être désactivé** dans la configuration (`audit.rs:11`, `:91-93`) ; il est alors muet, sans erreur.

**Quand le journal ne peut pas être écrit, le message est refusé.** L'échec d'audit n'est pas ignoré : il remonte comme une erreur qui bloque le traitement (`agent_bridge_support.rs:43-48`, `:50-64` ; `agent_bridge.rs:66`). Un canal peut donc devenir muet pour une raison qui n'a rien à voir avec le canal — un disque plein, par exemple.

---

## Encadrés

> **⚠ Beaver n'accepte pas n'importe quel serveur MCP.**
> Cinq connecteurs installables localement, treize points d'accès distants approuvés, trois programmes autorisés à démarrer. La commande lancée doit correspondre au caractère près à celle du catalogue. Ce n'est pas une limite provisoire : c'est ce qui empêche un connecteur d'exécuter du code arbitraire sous votre compte.

> **⚠ Un canal qui affiche « Actif » peut ne répondre à personne.**
> Quatre contrôles refusent les messages **en silence** : liste des utilisateurs autorisés, limite de débit, compte non configuré, message invalide. Rien n'est répondu à l'expéditeur, rien n'apparaît dans Beaver. Le seul endroit où le voir est le journal d'audit.

> **⚠ La liste des utilisateurs autorisés n'est pas facultative.**
> Vide, elle bloque tout le monde. Le joker `*` est refusé. C'est le premier réglage à vérifier quand un bot ne répond pas.

> **ℹ Un connecteur qui redémarre après dix minutes d'inactivité, c'est normal.**
> Beaver arrête les processus MCP inutilisés au bout de 10 minutes et les relance au besoin. Le premier appel après une pause est plus lent.

> **ℹ Une extension activée a un accès total, et l'application vous le dit.**
> « Cette extension aura un accès total aux données, secrets et fonctions de votre compte. Beaver ne l'a pas auditée. » Après chaque mise à jour, l'extension est désactivée et la confiance doit être redonnée.

> **ℹ L'hôte d'extensions ne redémarre que trois fois en cinq minutes.**
> Passé ce budget, plus rien ne se relance tout seul. C'est le moment d'utiliser « Redémarrer l'hôte », ou « Désactiver les extensions hébergées » si Beaver reste inutilisable.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause vérifiée | Ce qu'on peut faire |
|---|---|---|
| « Impossible de valider le connecteur. Vérifiez le token puis réessayez. » | **Message unique pour toutes les causes** : programme absent, commande refusée, service injoignable, délai dépassé (`fr.json:1628`) | Vérifier que `npx`, `uvx` ou `deno` est installé avant de soupçonner le jeton. Voir « Anomalies relevées », point 1 |
| « Je ne peux pas ajouter mon propre serveur MCP » | Catalogue fermé : cinq connecteurs locaux, treize adresses distantes (`stdio_catalog.rs:13-21` ; `trusted.rs:1-15`) | Comportement voulu, aucun contournement |
| Le premier lancement d'un connecteur prend très longtemps | `npx` / `uvx` téléchargent le paquet ; seule la première interrogation est bornée, à 20 s (`registry.rs:12`) | Attendre, puis réessayer |
| Un outil du connecteur n'apparaît pas | Catalogue d'outils gardé **5 minutes** en mémoire (`registry.rs:11`) | Attendre, ou retirer et rajouter le connecteur |
| « Échec de la connexion. Réessayez. » après un OAuth | Sept causes possibles, **un seul message** (`fr.json:1642`) | Refaire la connexion sans quitter la fenêtre ; ne pas dépasser 5 minutes |
| Le bouton « Relancez » ne fait rien | Une connexion est déjà en cours pour ce connecteur, la seconde est écartée en silence (`flow.rs:140-142`) | Fermer la fenêtre, puis recommencer |
| Un connecteur autrefois connecté ne répond plus | Jeton expiré sans jeton de renouvellement, ou renouvellement en échec (`storage.rs:59-62`, `:95-98`) | Se déconnecter du connecteur et refaire la connexion |
| « Cette extension n'est pas compatible avec cette version de Beaver. » | Version d'API du manifeste ≠ **1** (`manifest.rs:49-54`) | Mettre à jour l'extension ou Beaver. Aucun mode de compatibilité |
| L'hôte d'extensions repasse en erreur en boucle, puis s'arrête | Budget de **3 redémarrages / 300 s** épuisé (`runtime_restart.rs:22`) | « Redémarrer l'hôte » ; si ça recommence, « Désactiver les extensions hébergées » |
| « L'hôte d'extensions n'a pas confirmé son arrêt. » | L'arrêt a dépassé 5 secondes | Quitter complètement Beaver puis le relancer — l'application le dit (`fr.json:1879`) |
| Aucune interface d'extension ne s'affiche après un plantage | Mode sûr après une interruption de chargement (`fr.json:1831-1832`) | Réessayer l'interface ou écarter l'incident depuis la fenêtre de reprise |
| Le bot Telegram ne répond à personne | Liste des utilisateurs autorisés vide ou mal remplie (`agent_bridge.rs:70-73`) | Ajouter l'identifiant. Le canal reste « Actif » malgré tout |
| Le bot répond en privé mais pas en groupe | « Mention requise en groupe » est active (`fr.json:130`) | Mentionner le bot, ou désactiver l'option |
| Le bot répond puis s'arrête après quelques messages | **12 messages par minute et par utilisateur** (`gateway_config.rs:44`) | Attendre la minute suivante, ou relever la limite dans la configuration |
| Canal en « Erreur » immédiatement, sans tentative | Erreur d'authentification : arrêt définitif dès le premier échec (`supervisor.rs:36-42`) | Vérifier le jeton du bot |
| Canal en « Erreur » après plusieurs minutes | **10 tentatives** épuisées (`supervisor.rs:6`, `:44-49`) | Vérifier le réseau et le service, puis relancer le canal |
| « L'opération a échoué. Réessayez. » sur un canal | Message par défaut pour tout code non reconnu, **y compris une panne du journal d'audit** (`channel-error.ts:1-5`) | Vérifier l'espace disque. Voir « Anomalies relevées », point 3 |
| Un canal devient muet sans erreur affichée | L'écriture du journal d'audit échoue et bloque chaque message (`agent_bridge.rs:66`) | Vérifier le disque et les droits sur `logs/` |

---

## Renvois

- `07-integrations/mcp-connecteurs.md` — la liste des connecteurs et comment les ajouter
- `07-integrations/extensions-centre.md` — installer, approuver et mettre à jour une extension
- `07-integrations/canaux-externes.md` — brancher un bot Telegram, Slack ou Discord
- `11-securite/vault-et-cles-api.md` — où vivent les jetons OAuth et les jetons de bots
- `13-depannage/agent-et-outils.md` — les catégories d'erreur d'outil, dont celles des appels MCP
- `12-reference/emplacement-des-donnees.md` — `mcp-connectors.json`, `logs/gateway-audit.jsonl`

---

## Anomalies relevées

Relevées en lisant le code, **non corrigées** — à arbitrer avec l'équipe avant de publier la page.

1. **Le message d'échec d'un connecteur accuse le jeton dans tous les cas.** « Impossible de valider le connecteur. Vérifiez le token puis réessayez. » (`fr.json:1628`) s'affiche que le problème soit un jeton, un programme absent du PATH, une commande refusée par le catalogue, un service injoignable ou un délai de 20 secondes dépassé (`mcp-config-dialog.tsx:51`, `:56`, qui ne lit pas le motif renvoyé). Le moteur distingue pourtant six causes (`process_manager.rs:116-117` ; `stdio_cmd.rs:27-32`, `:61`, `:64` ; `registry_probe.rs:55-58`). Même structure que le message du coffre relevée dans `11-securite/vault-et-cles-api.md` : le code sait, l'interface ne le dit pas.

2. **L'échec d'une connexion OAuth affiche toujours la même phrase, alors que le motif traverse la frontière.** Le backend envoie un événement contenant un champ `error` porteur du motif exact (`mcp_oauth/flow.rs:39-46`) ; l'interface le lit, teste `success`, et ignore `error` pour afficher « Échec de la connexion. Réessayez. » (`mcp-oauth-dialog.tsx:43-54`). Un délai de 5 minutes dépassé et un service non approuvé produisent donc le même écran.

3. **Le code d'erreur `auditUnavailable` n'a pas de traduction.** Le moteur le renvoie quand le journal d'audit ne peut pas être écrit (`service_runtime.rs:63`, `:93`, `:127-129`), mais l'interface ne reconnaît que `invalidConfig` et `unavailable` (`channel-error.ts:1`). Le message affiché est donc « L'opération a échoué. Réessayez. » — qui n'oriente pas vers la vraie cause, un problème de disque ou de droits. Le test du fichier vérifie les trois clés existantes et ne remarque pas l'absence de la quatrième (`__tests__/channel-error.test.ts:26-28`).

4. **Dix-huit messages de diagnostic d'interface d'extension sont traduits en sept langues et ne s'affichent jamais.** L'interface remplace le message précis par le générique « La contribution d'interface de cette extension a été refusée » pour tout code figurant dans la liste des diagnostics d'interface (`extensions-host-panel.tsx:98-100` ; liste dans `extension-ui-contract.generated.ts:16`). Les dix-huit phrases précises existent pourtant dans `fr.json:1967-1984` — « Le contrat d'interface est invalide », « L'interface avancée utilise un import Node.js interdit », « Les fichiers d'interface ont changé après l'approbation »… Seul `ui_manifest_legacy` échappe à la règle (`extensions-host-panel.tsx:95-97`). Le choix est peut-être volontaire — ne pas exposer le détail d'un refus d'interface — mais il n'est écrit nulle part.

5. **L'état « Validation du connecteur en cours… » n'est jamais atteint.** La clé `connectors.oauth.testing` (`fr.json:1637`) correspond à un état `testing` que le composant déclare (`mcp-oauth-dialog.tsx:10`) et affiche (`:87`), mais qu'aucun chemin ne définit : la validation finale a lieu côté Rust, entre l'échange du code et l'enregistrement (`flow.rs:126-130`), sans événement intermédiaire. L'utilisateur reste sur « Complétez les étapes de connexion dans le nouvel onglet du navigateur. » (`fr.json:1636`) pendant la validation.

6. **La page de retour OAuth affichée dans le navigateur est en français uniquement**, écrite en dur dans le code Rust (`callback_server.rs:16-23`), y compris ses couleurs. C'est le seul écran de Beaver qui ne suit ni le thème ni la langue choisie.

7. **Le motif d'un refus disparaît du journal d'audit.** Le code écrit une décision de sécurité, mais ne conserve la valeur que si elle est faite uniquement de minuscules ASCII et de tirets bas (`audit.rs:130-140`). Or les motifs passés sont des phrases anglaises avec espaces — `"user not in allowlist"`, `"account not configured"`, `"provider restricted to interactive chat"` (`agent_bridge.rs:69`, `:72`, `:97`). Toutes deviennent `"blocked"`. Seul `"rate_limited"` survit (`agent_bridge.rs:76-81`). **Conséquence directe pour le dépannage : le journal d'audit ne permet pas de distinguer un utilisateur non autorisé d'un compte mal configuré.**

---

## Points à confirmer

**Non vérifié — hors de portée d'une lecture du code**

1. **Aucune de ces pannes n'a été provoquée à l'écran.** Faire expirer un jeton OAuth, saturer la file de l'hôte d'extensions ou épuiser dix tentatives de reconnexion suppose de fabriquer la panne. Les faits viennent du code et sont sûrs ; l'enchaînement visible ne l'est pas.
2. **Le comportement réel de `npx` et `uvx` au premier lancement** — durée du téléchargement, apparence côté Beaver pendant ce temps — n'a pas été observé. À mesurer une fois sur chaque système avant d'écrire un ordre de grandeur sur le site.
3. **La rétention par défaut du journal d'audit** est fixée à 30 jours dans le code (`audit.rs:12`), mais la valeur effective vient de la configuration. Reste à confirmer ce que contient `config.json` à l'installation, et si l'utilisateur peut la changer depuis l'interface ou seulement dans le fichier.
4. **Le journal d'audit est-il consultable depuis l'application ?** Le code ne montre qu'une écriture ; aucun écran de lecture n'a été trouvé. Si la seule voie est d'ouvrir le fichier à la main, la page doit le dire et donner le chemin.
5. **Où se règlent les limites de débit des canaux.** Les valeurs par défaut sont dans le code (`gateway_config.rs:44-46`) et validées à l'enregistrement (`config_validation.rs:16-18`), mais l'écran qui les expose n'a pas été identifié.
6. **La formulation dans les six autres langues** n'a pas été relue. Chaque message cité ici l'a été en français seulement.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

7. La section « Dernières erreurs de chargement » : combien de diagnostics sont affichés à la fois, et si la liste défile — la borne du moteur est de quatre diagnostics par extension (`types.rs:11`).
8. Le statut d'un canal : le libellé texte existe (`channels-detail.tsx:157`) mais le bouton d'état ne porte qu'une pastille colorée (`:112-116`). À vérifier à l'écran : deux états voisins se distinguent-ils sans la couleur ?
9. La fenêtre de connexion OAuth : ce qu'elle montre pendant les cinq minutes d'attente, et si l'utilisateur comprend qu'il doit agir dans son navigateur.
10. La fenêtre de reprise après un chargement interrompu : ses cinq boutons tiennent-ils sur une fenêtre étroite (`fr.json:1816-1820`) ?
