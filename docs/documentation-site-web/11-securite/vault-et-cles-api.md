# Le coffre et vos clés API

**Emplacement site** — Sécurité › Le coffre et vos clés API
**Répond à** — « Si je confie ma clé API à Beaver, où va-t-elle exactement, qui peut la relire, et qu'est-ce qui se passe si quelque chose tourne mal ? »
**Sources** — `src-tauri/src/services/vault.rs`, `vault_tests.rs` ; `services/api_keys.rs` et ses fichiers inclus (`api_keys_state.rs`, `api_keys_registry.rs`, `api_keys_transactions.rs`, `api_keys_raw.rs`, `api_keys_validate.rs`, `api_keys_mcp.rs`, `api_keys_http.rs`, `api_keys_credential_scope.rs`, `api_keys_credential_scope_wire.rs`) ; `src-tauri/src/commands/api_keys.rs`, `commands/codex.rs`, `commands/oauth_providers.rs` ; `src-tauri/src/invoke_handler.rs`, `invoke_handler_tail.rs` ; `services/private_store.rs` et `services/private_store/` (`atomic_write.rs`, `private_store_windows.rs`, `windows_acl.rs`) ; `services/mcp_oauth/storage.rs`, `mcp_oauth/static_credentials.rs` ; `services/codex_oauth/store.rs` ; `services/llm_oauth/store.rs`, `llm_oauth/types.rs` ; `services/browser/session_store.rs` ; `services/gateway/tokens_account.rs` ; `services/extensions/core_secrets.rs` ; `services/security_cleanup.rs` ; `services/secure_http.rs` ; `services/llm/mod.rs` ; `services/agent_local/sensitive_data.rs` ; `src-tauri/src/app_build.rs`, `src/startup.rs` ; `src-tauri/Cargo.toml` ; `src/App.tsx`, `src/components/layout/ready-app.tsx`, `src/components/layout/vault-error-banner.tsx`, `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 9 septembre 2026, sur la version **1.2.2**. Les points d'affichage et les scénarios de panne non provoqués sont listés en fin de fichier.

> **Cette page décrit le mécanisme, pas le parcours.** La saisie d'une clé fournisseur par fournisseur est dans `06-modeles/providers-api.md` ; la connexion par compte web dans `06-modeles/providers-comptes-web.md`. Cette page-ci répond à la question de confiance : où va le secret, et qui peut le relire.

---

## Plan de page proposé

1. Le problème que le coffre résout
2. Ce qu'est le coffre, concrètement
3. Le chiffrement, expliqué sans jargon
4. La clé maîtresse, et où chaque système la range
5. Le cycle de vie d'une clé, de la saisie à l'appel
6. La garantie centrale : l'interface ne peut pas relire une clé
7. Ce que le coffre contient en plus des clés de modèles
8. Ce qui n'est **pas** dans le coffre
9. Supprimer une clé, se déconnecter d'un compte
10. Quand le trousseau du système est indisponible
11. Ce qui est écrit dans les traces
12. Les limites du coffre

---

## Contenu

### 1. Le problème que le coffre résout

Une clé API est un mot de passe : qui la détient peut dépenser sur votre compte chez le fournisseur. Beaucoup d'applications les rangent dans un fichier de configuration en clair, lisible par n'importe quel programme tournant sous votre session.

Beaver ne fait pas cela. C'est le point à établir en premier sur la page, et le reste en découle.

### 2. Ce qu'est le coffre, concrètement

Un **seul fichier**, à un **seul endroit**, identique sur les trois systèmes :

**`~/.local/share/cl-go-dash/secrets.enc`** (`services/vault.rs:41-43` — le chemin passe par la fonction unique `paths::data_dir()`, il n'est écrit en dur nulle part).

Ce que contient ce fichier, une fois ouvert, avant déchiffrement (`services/vault.rs:34-39`, `:100-105`) :

| Champ | Contenu |
|---|---|
| `version` | Le numéro de format, **1** aujourd'hui (`vault.rs:19`) |
| `nonce` | Un nombre tiré au hasard, différent à chaque écriture |
| `data` | **Tous vos secrets, chiffrés, en un seul bloc illisible** |

Rien d'autre. Aucun nom de fournisseur, aucun indice sur ce qu'il y a dedans : la liste elle-même est à l'intérieur du bloc chiffré.

**Le fichier est réservé à votre compte utilisateur.** Sous macOS et Linux, il est créé avec les permissions `0600` — vous seul pouvez le lire et l'écrire — et les dossiers en `0700` (`services/private_store.rs:141-150`, `:168-174`). Sous Windows, Beaver pose une liste de contrôle d'accès qui ne contient qu'une entrée, la vôtre, refuse l'héritage des droits du dossier parent, **et relit ensuite ce qu'il a posé pour vérifier** (`private_store.rs:176-179` ; `private_store/windows_acl.rs:16-41` pour la pose, `:43-77` pour la vérification). Ces permissions sont revérifiées et réparées à chaque démarrage sur `secrets.enc` et `configured-providers.json` (`private_store.rs:91-115`).

**Le fichier n'est jamais écrit à moitié.** Beaver écrit d'abord un fichier temporaire, le force sur le disque, corrige ses permissions, puis le renomme par-dessus l'ancien — un renommage est une opération indivisible pour le système (`vault.rs:154` → `private_store/atomic_write.rs:37-66`). Une coupure de courant pendant une sauvegarde laisse donc soit l'ancien coffre entier, soit le nouveau, jamais un mélange des deux.

### 3. Le chiffrement, expliqué sans jargon

L'algorithme est **XChaCha20-Poly1305** (`services/vault.rs:2-5`, `:88-106` pour le chiffrement, `:108-132` pour le déchiffrement). Il fait deux choses à la fois, et les deux comptent :

- **XChaCha20** rend le contenu illisible sans la clé. C'est la partie qu'on imagine spontanément en entendant « chiffré ».
- **Poly1305** ajoute une **signature de contrôle** : si un seul octet du fichier est modifié — par un autre programme, par un disque défaillant, par quelqu'un qui essaie de bricoler le fichier — le déchiffrement **échoue** au lieu de renvoyer des données douteuses. C'est vérifié par un test dédié (`services/vault_tests.rs:59`, `decrypt_rejects_tampered_ciphertext`).

Deux propriétés supplémentaires, vérifiées :

- **Chaque écriture utilise un nombre à usage unique différent** — 24 octets tirés d'un générateur aléatoire de qualité cryptographique (`vault.rs:92-94`). Conséquence : sauvegarder deux fois le même contenu produit deux fichiers différents à l'œil, et personne ne peut déduire que « rien n'a changé ». Un test le vérifie (`vault_tests.rs:24`, `nonce_is_random_ciphertext_differs_each_call`).
- **Un coffre écrit par une version future de Beaver est refusé, pas réécrit** (`vault.rs:111-113` ; test `vault_tests.rs:75`). Beaver préfère s'arrêter plutôt que de détruire des données qu'il ne comprend pas.

Une phrase suffit sur le site pour l'essentiel : **copier `secrets.enc` sur une autre machine ne donne accès à rien.** Le fichier ne contient pas de quoi s'ouvrir lui-même.

### 4. La clé maîtresse, et où chaque système la range

La clé qui ouvre le coffre — la **clé maîtresse** — n'est pas dans le coffre. Ce serait mettre la clé de la maison sous le paillasson.

Elle est confiée au **gestionnaire de secrets du système d'exploitation**, celui-là même qui garde les mots de passe de votre navigateur. Beaver passe par la bibliothèque `keyring` version 3.6, configurée pour utiliser le mécanisme natif de chaque système (`src-tauri/Cargo.toml:111`) :

| Système | Où vit la clé maîtresse | Option activée dans le code |
|---|---|---|
| macOS | **Trousseau d'accès** | `apple-native` |
| Windows | **Gestionnaire d'identifiants** | `windows-native` |
| Linux | **Service de secrets du bureau** (GNOME Keyring, KWallet) | `sync-secret-service` |

L'entrée porte le nom de service **`cl-go-dash`** et l'identifiant **`master-key`** (`vault.rs:16-18`, `:51-52`) : c'est sous ce nom qu'on la retrouve dans le Trousseau macOS ou le Gestionnaire d'identifiants Windows.

**Comment elle naît.** Au tout premier lancement, Beaver tire **32 octets** d'un générateur aléatoire de qualité cryptographique (`OsRng`, celui fourni par le système) et les dépose dans le trousseau (`vault.rs:73-81`). Il ne la dérive pas d'un mot de passe, ne la calcule pas à partir de la machine : elle est purement aléatoire.

**Un seul accès au trousseau, au démarrage.** La fonction qui lit la clé maîtresse n'a **qu'un seul appelant dans tout le code**, la routine d'initialisation lancée une fois au lancement (`vault.rs:50` appelée uniquement depuis `services/api_keys.rs:47`). Vous n'êtes donc pas sollicité à chaque requête.

*Nuance à ne pas omettre* : lors du **tout premier lancement après une mise à jour depuis une version ancienne**, Beaver lit en plus jusqu'à dix anciennes entrées du trousseau pour les récupérer et les ranger dans le coffre (`vault.rs:157-177`, liste des fournisseurs `:22-32`). Cette migration ne se produit **qu'une fois** : un fichier marqueur `.vault-migrated` l'empêche de recommencer (`api_keys.rs:49-63`).

**Le cas où la clé maîtresse a disparu mais le coffre existe.** Beaver **refuse explicitement** de fabriquer une nouvelle clé maîtresse dans cette situation (`vault.rs:66-72`). Il pourrait le faire — ce serait plus confortable — mais cela écraserait silencieusement un coffre encore valable. Il s'arrête à la place. Conséquence pour l'utilisateur, à écrire clairement : **si vous videz le trousseau de votre système, vos clés API deviennent définitivement illisibles et devront être ressaisies.**

### 5. Le cycle de vie d'une clé, de la saisie à l'appel

Le parcours complet, en cinq temps, chacun vérifié. C'est le cœur de la page.

**Temps 1 — La saisie.** Vous collez la clé dans le champ de l'écran des clés API. L'interface appelle la commande `set_api_key`. Dès que le travail est fait, **la copie que l'interface a transmise est écrasée en mémoire** (`commands/api_keys.rs:22` : `key.zeroize()`).

**Temps 2 — La validation, une seule fois, à l'entrée.** Beaver vérifie que le fournisseur existe dans son catalogue, que la clé n'est pas vide, qu'elle fait au plus **256 caractères**, et qu'elle ne contient aucun caractère de contrôle (`services/api_keys_validate.rs:1`, `:13-32`). Une clé aberrante est refusée plutôt que rangée.

**Temps 3 — L'écriture dans le coffre.** Elle est **transactionnelle** : Beaver fabrique une copie complète du contenu du coffre, y applique la modification, vérifie l'ensemble, écrit le fichier entier, et seulement ensuite remplace ce qu'il a en mémoire (`services/api_keys_transactions.rs:1-26`). Si l'écriture échoue, l'état précédent reste en place ; il n'y a pas de moment où le coffre serait à moitié modifié. La copie de travail est effacée de la mémoire quoi qu'il arrive, y compris en cas d'erreur (`services/api_keys_state.rs:11-19`).

**Temps 4 — La vie en mémoire.** Le coffre est déchiffré **une fois au démarrage**, et les clés restent en mémoire pour la durée de la session, chacune dans un conteneur qui **efface son contenu quand il est libéré** au lieu de le laisser traîner (`services/api_keys_state.rs:3-6` ; le type `Zeroizing`). La clé maîtresse elle-même vit dans le même type de conteneur.

> **À écrire honnêtement, sans surpromettre.** Les clés ne sont pas rechargées depuis le disque à chaque requête : elles sont en mémoire pendant que Beaver tourne. Ce qui est garanti, c'est que **chaque copie remise à un appel est effacée dès qu'elle a servi**, et que rien n'est laissé en clair dans la mémoire après la fermeture de l'application. Ne pas écrire « la clé n'existe en mémoire qu'au moment de l'appel » : ce n'est pas ce que fait le code.

**Temps 5 — L'appel au fournisseur.** Au moment d'envoyer une requête, Beaver demande une copie protégée de la clé, la pose dans l'en-tête HTTPS, et cette copie est effacée à la fin de l'appel (`services/api_keys.rs:122-131` ; utilisation dans `services/llm/route_authenticated_send.rs:21-24`, `services/search/brave.rs:40`, `services/search/firecrawl.rs:21`). L'en-tête qui porte le secret est de plus **marqué comme sensible** auprès de la bibliothèque HTTP, ce qui l'exclut de ses propres traces (`services/secure_http.rs:144-153`, utilisé dans `services/api_keys_http.rs:107-121`).

### 6. La garantie centrale : l'interface ne peut pas relire une clé

C'est l'argument le plus fort de la page, et il se démontre par une liste exhaustive, pas par une affirmation.

Beaver est une application de bureau : une partie interface, écrite en JavaScript, et un moteur écrit en Rust. Le seul passage entre les deux est une liste de **commandes déclarées une par une**. Ce qui n'y figure pas n'existe pas pour l'interface.

**Les sept commandes liées aux clés API, liste complète** (`src-tauri/src/invoke_handler.rs:137-143`) :

| Commande | Ce qu'elle fait | Ce qu'elle renvoie |
|---|---|---|
| `set_api_key` | Enregistre une clé | Rien (succès ou erreur) |
| `delete_api_key` | Supprime une clé | Rien |
| `has_api_key` | Dit si un fournisseur est configuré | **Oui / non** |
| `list_configured_providers` | Liste les fournisseurs configurés | **Des identifiants**, jamais des clés |
| `get_provider_connection` | Rappelle les réglages de connexion Qwen | Région, mode de point d'accès, identifiant d'espace de travail — **aucun secret** (`services/provider_connections/qwen.rs:35-41`) |
| `test_api_key` | Teste la clé déjà enregistrée | Succès ou message d'erreur |
| `test_api_key_with_value` | Teste une clé saisie, avant de l'enregistrer | Succès ou message d'erreur |

**Aucune commande de lecture n'existe.** Le fichier de commandes le dit lui-même en tête, en commentaire : « aucune commande ne retourne la clé en clair au frontend » (`commands/api_keys.rs:1-4`).

La même règle vaut pour les comptes web (`src-tauri/src/invoke_handler_tail.rs:108-116`) :

- `codex_status` renvoie un booléen « connecté » et **l'adresse e-mail** du compte. L'e-mail est extrait du jeton **côté Rust** ; le jeton, lui, ne franchit jamais la frontière (`commands/codex.rs:39-55`).
- `list_oauth_provider_statuses` renvoie l'identifiant du fournisseur, son nom affiché, l'état de connexion et le nom du compte. Pas de jeton (`services/oauth_providers/types.rs:47-55`).
- `has_mcp_oauth_token` et `delete_mcp_oauth_token` : un booléen, une suppression. Rien à lire (`invoke_handler.rs:148-149`).

**Conséquence à énoncer telle quelle sur le site :** une fois enregistrée, une clé ne peut plus être affichée — **y compris par Beaver lui-même**. Si vous l'avez perdue, il faut en générer une nouvelle chez le fournisseur. Ce n'est pas un oubli d'ergonomie, c'est le prix de la garantie.

### 7. Ce que le coffre contient en plus des clés de modèles

Le coffre n'est pas réservé aux clés API. **Tout ce qui est secret dans Beaver y va**, et c'est vérifiable entrée par entrée.

| Ce qui est rangé | Nom de l'entrée dans le coffre | Source |
|---|---|---|
| Clés API des fournisseurs de modèles, de recherche web et de prévision | L'identifiant du fournisseur | `api_keys.rs:133-140` |
| Compte **OpenAI / Codex** : jeton d'accès, jeton de renouvellement, échéance | `_codex_oauth` | `api_keys_credential_scope_wire.rs:8` ; `services/codex_oauth/store.rs:40-60` |
| Comptes **Grok (xAI)** et **Kimi** | `LLM_OAUTH_XAI_KEY`, `LLM_OAUTH_KIMI_KEY` | `services/llm_oauth/types.rs:26-30` ; `llm_oauth/store.rs:22-46` |
| Jetons OAuth des **connecteurs MCP** | `mcp_oauth_<identifiant du connecteur>` | `services/mcp_oauth/storage.rs:25-41` → `api_keys_mcp.rs:1`, `:14-24` |
| Identifiants d'application OAuth Google et GitHub | `_oauth_google_client_id`, `_oauth_google_client_secret`, `_oauth_github_client_id`, `_oauth_github_client_secret` | `services/mcp_oauth/static_credentials.rs:11-14`, `:43-54` |
| Variables d'environnement secrètes des connecteurs MCP | Une entrée par connecteur et par variable | `services/mcp_bridge/stdio_env.rs:64-65` ; `commands/mcp_connectors.rs:91` |
| Jetons des **canaux externes** (Slack, Discord, Telegram) | Une entrée par canal et par compte | `services/gateway/tokens_account.rs:141-148` |
| Réglages de connexion Qwen | `provider_connection:qwen` | `services/provider_connections/qwen.rs:6` |

S'y ajoutent des **clés techniques que Beaver fabrique lui-même** et range au même endroit, chacune tirée au hasard à sa première utilisation :

| Clé technique | À quoi elle sert |
|---|---|
| `browser-session-key-v1`, 32 octets | Chiffre les sessions du navigateur intégré (`services/browser/session_store.rs:9-10`, `:28-34`) |
| Clé de signature des pièces jointes, 32 octets | Empêche qu'un chemin de fichier autorisé soit falsifié (`services/attachment_access.rs:38`) |
| Clé d'empreinte de continuité de raisonnement, 32 octets | `services/reasoning_continuity/fingerprint.rs:24-31` |
| Clé de signature du journal d'audit de la passerelle, 32 octets | `services/gateway/security/audit.rs:78` |

**Le navigateur intégré mérite une mention à part**, parce que c'est là que vivent les sessions de vos comptes web. Ses sessions sont enregistrées dans `~/.local/share/cl-go-dash/browser/sessions/<identifiant>.enc`, **chiffrées avec la même primitive que le coffre** (`session_store.rs:22-26`, `:150` pour l'écriture, `:128` pour la lecture), et la clé qui les ouvre est elle-même dans le coffre. Le contenu est borné à **64 Kio** en clair et **128 Kio** chiffré ; au-delà, le fichier est refusé (`session_store.rs:7-8`).

### 8. Ce qui n'est **pas** dans le coffre

Aussi important que ce qui y est. Trois points, tous vérifiés.

**`configured-providers.json` ne contient aucun secret.** Ce fichier, à côté du coffre, ne contient qu'une **liste triée d'identifiants de fournisseurs** — `["brave", "openai", ...]` — reconstruite depuis l'état en mémoire à chaque modification (`services/api_keys_registry.rs:5-19`, `:21-36`). Il sert à savoir *quels* fournisseurs sont configurés sans avoir à ouvrir le coffre. Il est malgré tout protégé par les mêmes permissions de fichier et réparé au démarrage (`private_store.rs:104-113`).

**Une extension approuvée peut demander un secret au coffre.** C'est explicite dans le code : un pont dédié remet à une extension approuvée une clé de fournisseur, un jeton OAuth de connecteur MCP, une variable d'environnement de connecteur ou un jeton de canal externe (`services/extensions/core_secrets.rs:4-9`, `:11-18`, `:20-35`, `:37-49`). Le coffre protège vos secrets **du reste du système**, pas du code que vous avez vous-même approuvé. La page doit le dire et renvoyer à `07-integrations/extensions-centre.md`, qui détaille cette décision de confiance.

**Des versions anciennes de Beaver rangeaient certains jetons hors du coffre.** Au démarrage, Beaver supprime définitivement trois fichiers hérités : `oauth-providers/moonshot/credentials/kimi-code.json`, `oauth-providers/xai/auth.json` et `secrets.enc.bak-corrupted` (`services/security_cleanup.rs:26-28`, appelé depuis `app_build.rs:131`). Le nettoyage n'a lieu qu'une fois, un fichier marqueur `.security-hardening-v1` le mémorise (`security_cleanup.rs:4`, `:18-24`, `:30`). **Tranché le 10 septembre 2026 par Kevin : on le dit clairement**, sur la page du coffre (`reference-coffre-et-cles`). Raison : un utilisateur qui retrouve ces fichiers dans une sauvegarde ancienne a besoin de savoir qu'ils contenaient des jetons en clair et qu'il doit révoquer les accès correspondants chez Kimi et Grok ; la transparence sur un défaut corrigé renforce la confiance.

### 9. Supprimer une clé, se déconnecter d'un compte

**Supprimer une clé API** retire son entrée du coffre, réécrit le fichier entier, et met à jour la liste des fournisseurs configurés (`commands/api_keys.rs:61-72` → `api_keys.rs:142-147` → `api_keys_registry.rs:31-36`). Pour Qwen, les réglages de connexion associés partent dans la même opération, pas dans une seconde (`commands/api_keys.rs:64-66` ; `api_keys_transactions.rs:130-139`).

**Se déconnecter d'un compte web** efface l'entrée correspondante : `_codex_oauth` pour OpenAI (`codex_oauth/store.rs:84-86`), l'entrée du fournisseur pour Grok et Kimi (`llm_oauth/store.rs:69-73`). Les autres connexions ne sont pas touchées : chacune a son entrée distincte.

**Trois choses que la suppression ne fait pas**, à écrire sans les adoucir :

1. **Elle ne révoque rien chez le fournisseur.** La clé reste valable sur votre compte OpenAI, Mistral ou Brave : elle n'est simplement plus dans Beaver. Pour la rendre inutilisable, il faut la supprimer sur le site du fournisseur.
2. **Elle n'est pas annulable.** Aucune restauration n'est proposée, et comme aucune commande ne permet de relire une clé, il n'existe aucun moyen de la retrouver. Il faut la ressaisir ou en générer une nouvelle.
3. **Elle ne récupère pas ce qui est déjà parti.** Si une extension approuvée a obtenu la clé pendant qu'elle était enregistrée, supprimer la clé du coffre n'annule pas cette copie.

### 10. Quand le trousseau du système est indisponible

Le cas se produit surtout sous **Linux**, sur une session de bureau minimale ou une connexion en ligne de commande, où aucun service de secrets ne tourne. Le comportement est entièrement déterminé par le code, et il faut le décrire tel quel.

**Ce que fait Beaver**, dans l'ordre :

1. L'accès au trousseau échoue (`vault.rs:84`).
2. L'initialisation du coffre échoue et écrit `[vault] init failed` dans les traces techniques (`app_build.rs:138-140`).
3. **L'application démarre quand même.** Elle n'affiche pas d'écran bloquant.
4. **Deux secondes plus tard**, elle envoie un signal à l'interface (`app_build.rs:141-149` ; `src/startup.rs:4-11`). Le délai laisse à la fenêtre le temps d'apparaître avant que le message ne s'affiche.
5. L'interface affiche un bandeau, qui disparaît quand on clique dessus (`src/App.tsx:49`, `:53-58`, `:116` ; `src/components/layout/ready-app.tsx:88` ; `src/components/layout/vault-error-banner.tsx:8-15`).

**Le texte affiché**, mot pour mot (`src/i18n/fr.json:1380`) :

> « Impossible d'accéder au trousseau de clés. Installe gnome-keyring ou kwallet pour stocker les clés API. »

**Ce qui marche encore et ce qui ne marche plus.** Le coffre reste non initialisé : toute tentative d'enregistrer, de lire ou de supprimer un secret renvoie « coffre indisponible » (`api_keys.rs:126` ; `api_keys_transactions.rs:105-110`). Les modèles locaux via Ollama, eux, n'ont besoin d'aucune clé et continuent de fonctionner. **À vérifier à l'écran avant publication** : voir ce que montrent réellement les écrans de fournisseurs dans cet état.

**Ce que l'utilisateur doit faire**, sur Linux : installer et démarrer `gnome-keyring` ou `kwallet`, puis relancer Beaver. Le message de l'application le dit déjà, la page peut donner la commande d'installation par distribution.

**Écart relevé, à signaler à l'équipe.** Ce bandeau s'affiche pour **toute** panne d'initialisation du coffre — trousseau injoignable, mais aussi coffre illisible, coffre écrit par une version plus récente, ou clé maîtresse disparue alors que le coffre existe (`app_build.rs:138`, qui ne distingue pas les causes). Le message accuse donc le trousseau dans des cas où il n'y est pour rien, et propose d'installer un logiciel qui ne résoudra pas le problème. Voir « Points à confirmer ».

### 11. Ce qui est écrit dans les traces

Trois protections cumulées, toutes vérifiées :

- **Les réponses des fournisseurs ne sont jamais écrites brutes.** Elles passent par une fonction unique qui masque les données sensibles, supprime les caractères de contrôle et **tronque à 200 caractères** (`services/llm/mod.rs:130-137`).
- **Les en-têtes HTTPS qui portent un secret sont marqués sensibles** auprès de la bibliothèque réseau, ce qui les exclut de ses propres traces de débogage (`services/secure_http.rs:144-153`).
- **`secrets.enc` figure dans la liste des fichiers protégés de l'application** : l'agent qui tente de le lire ou de le manipuler par une commande déclenche une alerte (`services/agent_local/sensitive_data.rs:1-6`, `:27-45`). La même liste couvre `config.json`, `agent-settings.json` et `configured-providers.json`.

Les messages d'erreur destinés à l'utilisateur sont génériques : « Clé API invalide ou non autorisée », « Clé valide mais quota dépassé », « test de la clé refusé » (`api_keys_http.rs:132-143`). Le corps de la réponse du fournisseur est lu puis jeté, sans être affiché ni journalisé.

### 12. Les limites du coffre

À présenter en tableau, sans dramatiser : ce sont des bornes de sécurité, pas des restrictions arbitraires.

| Limite | Valeur | Source |
|---|---|---|
| Longueur d'une clé API | **256 caractères** | `api_keys_validate.rs:1` |
| Entrées au total dans le coffre | **500** | `api_keys_raw.rs:3` |
| Taille d'un secret rangé | **8 192 octets** | `api_keys_raw.rs:1` |
| Longueur d'un nom d'entrée | **160 caractères** | `api_keys_raw.rs:2` |
| Secrets écrits en une seule opération | **8** | `api_keys_raw.rs:4` |
| Identifiant de connecteur MCP | **64 caractères**, lettres, chiffres, `-` et `_` seulement | `api_keys_mcp.rs:2-12` |
| Jeton de compte Grok ou Kimi | **4 096 caractères** | `llm_oauth/store.rs:8` |
| Jeton d'accès OpenAI / Codex | **512 Kio** | `codex_oauth/store.rs:7` |
| Session du navigateur intégré | **64 Kio** en clair, **128 Kio** chiffré | `browser/session_store.rs:7-8` |

---

## Encadrés

Ces encadrés portent la valeur de la page. Ils vont dans le corps du texte, à l'endroit indiqué.

> **ℹ À placer en tête de page — Le résumé en quatre phrases.**
> Vos clés vivent dans un seul fichier chiffré, `secrets.enc`, lisible par votre seul compte utilisateur. La clé qui l'ouvre n'est pas dedans : elle est confiée au gestionnaire de secrets de votre système — Trousseau sur macOS, Gestionnaire d'identifiants sur Windows, service de secrets du bureau sur Linux. L'interface de Beaver ne dispose d'aucune commande capable de relire une clé, et il n'en existe pas de version cachée. Copier le fichier sur une autre machine ne donne accès à rien.

> **⚠ À placer dans la section Clé maîtresse — Vider le trousseau de votre système rend vos clés illisibles.**
> Beaver refuse volontairement de fabriquer une nouvelle clé maîtresse quand l'ancienne a disparu mais que le coffre existe encore : cela écraserait silencieusement des données encore valables. Si vous videz le trousseau de votre système, ou si vous changez de machine sans lui, il faudra **ressaisir toutes vos clés**. Aucune sauvegarde du fichier `secrets.enc` ne vous aidera : il ne contient pas de quoi s'ouvrir.

> **⚠ À placer dans la section Supprimer — Supprimer dans Beaver ne révoque rien chez le fournisseur.**
> Une clé retirée du coffre reste parfaitement valable sur votre compte OpenAI, Mistral ou Brave. Si vous pensez qu'elle a fuité, la seule action efficace est de la **supprimer sur le site du fournisseur**. La supprimer dans Beaver ne fait que la retirer de cette machine.

> **⚠ À placer dans la section Ce qui n'est pas dans le coffre — Une extension approuvée peut demander vos secrets.**
> Le coffre protège vos clés du reste du système. Il ne les protège pas du code que vous avez vous-même approuvé : une extension activée peut demander une clé de fournisseur ou un jeton de connecteur, et Beaver la lui remet. C'est pourquoi l'activation d'une extension demande une confirmation explicite.

> **ℹ À placer dans la section Cycle de vie — Une clé enregistrée ne peut plus être affichée.**
> Y compris par Beaver. Il n'existe aucune commande de lecture, et ce n'est pas un oubli : c'est ce qui garantit qu'aucun bout d'interface, aucune extension d'interface, aucun défaut d'affichage ne peut faire apparaître votre clé à l'écran. Notez-la ailleurs si vous en avez besoin, ou générez-en une nouvelle chez le fournisseur.

> **ℹ À placer dans la section Trousseau indisponible — Sur Linux, il faut un service de secrets.**
> Sur une session de bureau minimale ou une connexion sans environnement graphique complet, aucun gestionnaire de secrets ne tourne. Beaver démarre quand même, affiche un bandeau, et refuse toute opération sur les clés jusqu'à ce qu'un service soit disponible. Les modèles locaux, qui n'ont besoin d'aucune clé, continuent de fonctionner normalement.

---

## Tableaux

### Où vit chaque chose

| Élément | Emplacement | Chiffré | Lisible par l'interface |
|---|---|---|---|
| Clés API, jetons de comptes, jetons de connecteurs | `~/.local/share/cl-go-dash/secrets.enc` | **Oui** | **Non** |
| Clé maîtresse du coffre | Trousseau / Gestionnaire d'identifiants / service de secrets, entrée `cl-go-dash` · `master-key` | Par le système | Non |
| Liste des fournisseurs configurés | `~/.local/share/cl-go-dash/configured-providers.json` | Non — **ne contient que des identifiants** | Oui, via `list_configured_providers` |
| Sessions du navigateur intégré | `~/.local/share/cl-go-dash/browser/sessions/*.enc` | **Oui**, clé rangée dans le coffre | Non |

### Ce que chaque commande peut et ne peut pas

| Commande | Écrit un secret | Lit un secret | Renvoie un secret |
|---|---|---|---|
| `set_api_key` | Oui | Non | Non |
| `delete_api_key` | Oui (suppression) | Non | Non |
| `has_api_key` | Non | Existence seulement | Non |
| `list_configured_providers` | Non | Identifiants seulement | Non |
| `get_provider_connection` | Non | Réglages Qwen seulement | Non |
| `test_api_key` | Non | **En interne**, pour l'appel HTTPS | Non |
| `test_api_key_with_value` | Non | Valeur fournie par l'appelant | Non |
| `codex_status` | Non | **En interne**, pour en extraire l'e-mail | Non — e-mail seulement |
| `has_mcp_oauth_token` | Non | Existence seulement | Non |
| `delete_mcp_oauth_token` | Oui (suppression) | Non | Non |

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Impossible d'accéder au trousseau de clés » au démarrage | Aucun service de secrets ne tourne — cas courant sur une session Linux minimale | Installer et démarrer `gnome-keyring` ou `kwallet`, puis relancer Beaver |
| Le même message alors que le trousseau fonctionne | Le bandeau s'affiche pour **toute** panne du coffre, y compris un coffre illisible ou une clé maîtresse disparue | Consulter les traces techniques ; voir « Points à confirmer », point 1 |
| « Je ne retrouve plus ma clé dans Beaver » | Aucune commande ne permet de relire une clé enregistrée | Générer une nouvelle clé chez le fournisseur et la ressaisir |
| Mes clés ont disparu après une réinstallation du système | La clé maîtresse vivait dans le trousseau, pas dans le fichier de coffre | Ressaisir les clés ; sauvegarder `secrets.enc` seul ne sert à rien |
| J'ai copié `~/.local/share/cl-go-dash/` sur une autre machine et rien ne marche | Le coffre est là, sa clé maîtresse est restée dans le trousseau de l'ancienne machine | Ressaisir les clés sur la nouvelle machine |
| « clé API invalide (vide ou trop longue) » | La clé dépasse **256 caractères** ou est vide | Vérifier qu'aucun retour à la ligne ou espace n'a été collé avec |
| « provider inconnu » | L'identifiant du fournisseur n'est pas dans le catalogue de Beaver | Passer par l'écran des clés API plutôt que par une valeur saisie à la main |
| « limite du coffre atteinte » | **500 entrées** au total, toutes catégories confondues | Supprimer des connecteurs ou des canaux devenus inutiles |
| « Clé valide mais quota dépassé » | La clé fonctionne, c'est le fournisseur qui refuse | Vérifier la facturation chez le fournisseur, pas dans Beaver |
| J'ai supprimé ma clé de Beaver, est-elle révoquée ? | Non — elle reste valable chez le fournisseur | La supprimer sur le site du fournisseur |

---

## Renvois

- `06-modeles/providers-api.md` — la liste des fournisseurs, où récupérer une clé et comment la saisir
- `06-modeles/providers-comptes-web.md` — se connecter avec un compte OpenAI, Grok ou Kimi plutôt qu'avec une clé, et le renouvellement automatique des jetons
- `07-integrations/extensions-centre.md` — pourquoi une extension approuvée peut obtenir vos secrets, et ce que vaut la confirmation d'activation
- `07-integrations/mcp-connecteurs.md` — les connecteurs externes et leurs jetons OAuth
- `11-securite/durcissement.md` — la vue d'ensemble : permissions de fichiers, isolation des processus, journaux d'audit
- `12-reference/emplacement-des-donnees.md` — l'inventaire complet de `~/.local/share/cl-go-dash/`
- `13-depannage/` — le dépannage général au démarrage

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **Le message d'erreur du coffre accuse le trousseau dans tous les cas.** Le bandeau « Impossible d'accéder au trousseau de clés. Installe gnome-keyring ou kwallet » s'affiche pour **toute** panne d'initialisation (`app_build.rs:138`, qui ne distingue pas les causes), alors que le code sait distinguer au moins quatre situations : trousseau injoignable (`vault.rs:84`), clé maîtresse de longueur invalide (`vault.rs:60-64`), clé maîtresse absente alors que le coffre existe (`vault.rs:66-72`), coffre écrit par une version plus récente (`vault.rs:111-113`). Trois questions pour l'équipe : le site décrit-il le comportement réel — un message unique — ou attend-on une correction ? Faut-il ouvrir une demande d'évolution pour transmettre la cause à l'interface ? Le message doit-il rester au tutoiement (« Installe ») alors que le reste de ces fichiers vouvoie ?
2. **Ce message est aussi le seul retour visible en cas de coffre corrompu**, situation où la bonne action de l'utilisateur est très différente. À trancher avec l'équipe avant d'écrire quoi que ce soit d'engageant dans la section « Pièges ».
3. ~~**Faut-il mentionner le nettoyage des anciens fichiers de jetons ?**~~ **Tranché le 10 septembre 2026 par Kevin : oui, clairement**, sur la page du coffre — voir la section 8 du Contenu, qui porte la décision et sa raison.
4. **Aucune annulation après la suppression d'une clé**, alors que la règle d'interface du projet demande une annulation plutôt qu'une confirmation pour toute action destructive. Ici, la suppression est irréversible par construction — la clé ne peut pas être relue pour être restaurée. À signaler comme écart assumé, et à décider si la page le mentionne ou reste descriptive.

**Non vérifié — hors de portée d'une lecture du code**

5. **Le comportement du Trousseau macOS au premier lancement.** Le système peut demander une autorisation d'accès au trousseau lors de la création de l'entrée `cl-go-dash`. Le code ne décrit pas cette boîte de dialogue, qui vient du système. À observer sur une machine macOS avant de rédiger la section, et à capturer si elle apparaît.
6. **Le comportement sous Windows** quand le Gestionnaire d'identifiants est indisponible ou restreint par une politique d'entreprise. Le chemin d'erreur est le même dans le code (`vault.rs:84`), mais le message affiché parle de `gnome-keyring` et `kwallet`, deux logiciels qui n'existent pas sous Windows. À confirmer par un essai, et probablement à corriger dans l'application.
7. **Le comportement réel sur une session Linux sans service de secrets** n'a pas été provoqué. Le parcours décrit à la section 10 vient entièrement de la lecture du code ; il est sûr sur le fond, mais l'enchaînement visible à l'écran ne l'est pas.
8. **La formulation dans les six autres langues.** Le message `errors.keyringFailed` a été relu en français seulement (`src/i18n/fr.json:1380`). Si l'équipe le corrige, la correction porte sur sept langues.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

9. **Le bandeau d'erreur du coffre** : sa position dans la fenêtre, son contraste dans les deux thèmes, et le fait qu'il soit **entièrement cliquable pour disparaître** (`vault-error-banner.tsx:11`) sans rien indiquer d'autre. Un bandeau qu'on fait disparaître d'un clic et qui ne revient pas dans la session est un choix à vérifier.
10. **L'écran des clés API quand le coffre est indisponible** : que voit exactement l'utilisateur s'il tente d'enregistrer une clé — le message « coffre indisponible » brut, ou un message traduit ?
11. **L'écran des clés API en fonctionnement normal** : comment est représentée une clé déjà enregistrée, puisqu'elle ne peut pas être affichée. Points, mention « configurée », champ vide ? Non observé.
12. **Les états d'erreur de test de clé** (`api_keys_http.rs:132-143`) ne recevront pas de capture : les provoquer suppose une clé révoquée ou un quota dépassé. Ils sont documentés d'après le code, comme le prévoit la convention de ce dossier.
