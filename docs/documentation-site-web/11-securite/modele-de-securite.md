# Modèle de sécurité

**Emplacement site** — Sécurité › Modèle de sécurité (page d'entrée de la section Sécurité)
**Répond à** — « Qu'est-ce qui sort de ma machine, qu'est-ce qui n'en sort jamais, qu'est-ce qui me protège, et où sont les limites ? »
**Sources** — `SECURITY.md` (racine) ; `src-tauri/src/storage_migration.rs` ; `src-tauri/src/models/config.rs` ; `src-tauri/src/services/agent_local/directory_access.rs`, `directory_access_scope.rs` ; `src-tauri/src/services/agent_local/shell_sandbox/` (`launch.rs`, `macos.rs`, `linux.rs`, `windows.rs`, `windows/windows_profile.rs`) ; `src-tauri/src/services/vault.rs` ; `src-tauri/src/services/api_keys.rs` ; `src-tauri/src/commands/api_keys.rs` ; `src-tauri/src/services/provider_connections/qwen.rs` ; `src-tauri/src/services/secure_http.rs` ; `src-tauri/src/services/llm/mod.rs`, `services/llm/litellm_catalog_refresh.rs` ; `src-tauri/src/commands/agent_chat_task/common.rs` ; `src-tauri/src/services/agent_local/stream_events.rs` ; `src-tauri/src/services/gateway/agent_bridge.rs`, `agent_bridge_run.rs`, `security/audit.rs`, `security/allowlist.rs`, `security/rate_limit.rs` ; `src-tauri/src/services/scheduler/agentic.rs` ; `src-tauri/src/services/mcp_bridge/stdio_cmd.rs`, `process_spawn.rs` ; `src-tauri/src/services/agent_local/agent_work_supervision.rs`, `session_limits.rs`, `session_store_document.rs`, `chat_message.rs` ; `src-tauri/src/services/terminal/manager.rs` ; `src-tauri/src/services/private_store.rs` ; `src-tauri/src/commands/app_update_source.rs` ; `src-tauri/src/services/brand.rs` ; `src-tauri/src/services/ollama_manager/spawn_gate_unix_support.rs`, `port.rs` ; `src-tauri/src/services/searxng/process.rs`, `lifecycle.rs`
**Vérification** — Vérifié dans le code, ligne par ligne, le 9 septembre 2026, sur la version **1.2.2**. Les points d'affichage et les décisions de rédaction sont en fin de fichier. Deux écarts avec `SECURITY.md` et un écart avec les notes d'audit internes sont signalés dans « Points à confirmer ».

---

## Avertissement au rédacteur

C'est la page qu'un utilisateur prudent lit **avant** d'installer. Elle décide de sa confiance. Deux règles non négociables :

1. **Aucune protection n'est annoncée sans sa condition d'activation.** Le bac à sable du shell, en particulier, ne s'active pas avec le réglage par défaut. L'annoncer sans cette condition serait un mensonge par omission.
2. **Le réglage par défaut est le plus permissif.** Beaver démarre en **Accès complet** sur **tout le disque**. La page le dit dans ses premières lignes, pas en note de bas de page.

Une page qui promet plus que ce que le code garantit est pire qu'une page absente : l'utilisateur cesse de se protéger lui-même.

---

## Plan de page proposé

1. Le principe : une application de bureau, pas un service
2. Ce qui sort de votre machine — et vers qui
3. Ce qui n'en sort jamais
4. Les cinq couches de protection
5. Couche 1 — Les permissions de l'agent
6. Couche 2 — Le bac à sable du système, et sa condition
7. Couche 3 — Le coffre des clés
8. Couche 4 — Les bornes
9. Couche 5 — L'isolation des extensions et des connecteurs
10. Les limites assumées
11. Ce qui est hors de portée de Beaver
12. Les cinq réglages qui changent tout

---

## Contenu

### 1. Le principe : une application de bureau, pas un service

Beaver s'exécute **sur votre ordinateur, avec les droits de votre compte utilisateur**. Il n'y a pas de serveur Beaver, pas de compte à créer, pas de synchronisation.

Trois conséquences à énoncer d'emblée :

- **Vos données restent des fichiers chez vous.** Tout vit sous `~/.local/share/cl-go-dash/`, identique sur macOS, Linux et Windows (`SECURITY.md`, « Local data location » ; chemin centralisé dans `services/paths.rs:10-14`, appelé partout).
- **Ce qui part vers un service extérieur part parce que vous l'avez demandé** — en choisissant un modèle cloud, un connecteur, une recherche web ou un canal de messagerie.
- **Beaver ne peut pas vous protéger de ce que vous autorisez.** Un agent en Accès complet fait ce que le modèle décide, avec vos droits.

Le moteur local **Ollama** n'écoute que sur la boucle locale : la variable d'environnement transmise au processus est `OLLAMA_HOST=127.0.0.1:<port>` (`services/ollama_manager/spawn_gate_unix_support.rs:112`), et le port est réservé sur `127.0.0.1` (`services/ollama_manager/port.rs:48`). **Aucune machine du réseau ne peut l'interroger.**

Le méta-moteur de recherche **SearXNG**, quand il est utilisé, est lui aussi lié à la boucle locale : `SEARXNG_BIND_ADDRESS=127.0.0.1` (`services/searxng/process.rs:49`), interrogé sur `http://127.0.0.1:<port>` (`services/searxng/lifecycle.rs:66`).

### 2. Ce qui sort de votre machine — et vers qui

C'est la question à laquelle la page doit répondre **en premier**, avec un tableau (voir la section Tableaux). Les points à développer en prose :

**Ce que voit un fournisseur de modèle cloud.** Quand vous choisissez un modèle cloud, la conversation entière part chez son fournisseur : vos messages, mais aussi **le contenu que les outils ont rapporté**. Un fichier lu par l'agent devient un message de rôle `tool` dans la conversation (`services/agent_local/chat_message.rs:37`), et cette conversation est ce qui est envoyé au modèle à chaque tour. Formulation juste pour le site : *ce que l'agent lit, le fournisseur le lit aussi.*

**Ce que voit un modèle local.** Rien ne sort. Ollama tourne sur votre machine et n'écoute que la boucle locale (section 1). C'est la réponse à donner à qui manipule des données confidentielles.

**Les requêtes que Beaver fait de lui-même.** La liste complète, avec ce que chacune transmet, est l'objet de la page *Confidentialité des données* — elle en fait autorité. Résumé (aligné le 9 septembre 2026 sur cette page) :

| Requête | Destination | Ce qui part |
|---|---|---|
| Vérification de mise à jour de l'application, **toutes les heures** (`src/hooks/use-update-checker.ts:13`, `:110-112`) | `api.github.com`, dépôt `Kevin-hDev/Beaver` (`commands/app_update_source.rs:15`, `:29-33`, `:43-49`) | Rien d'autre que l'en-tête `User-Agent`, qui vaut `<produit>/<version>` (`commands/app_update_source.rs:154-156` ; `services/brand.rs:8-10`) |
| Vérification de mise à jour du moteur Ollama, toutes les heures | `github.com`, dépôt public d'Ollama | Rien de personnel |
| Vérification de mise à jour des modèles installés, toutes les heures | `ollama.com` | **Le nom des familles de modèles installées** sur la machine |
| Catalogue des modèles et de leurs tarifs, au démarrage | `raw.githubusercontent.com`, fichier public du projet LiteLLM (`services/llm/litellm_catalog_refresh.rs:9`) | Une requête conditionnelle, mise en cache localement dans `litellm-models.json` (`:12`) |
| Aperçu des liens — dès qu'une adresse s'affiche dans une conversation, **sans clic**, activé par défaut (`src/components/agent-local/chat-markdown.tsx:78`, `models/config.rs:54`) | Le site de chaque lien affiché | Une requête vers ce site ; désactivable dans les réglages |
| Téléchargement du moteur Ollama — déclenché par un bouton au premier lancement, pas automatique | Dépôt public d'Ollama | Rien de personnel |

**Aucune télémétrie, aucune analyse d'usage.** Vérifié par recherche exhaustive dans `src/` et `src-tauri/src/` : aucune occurrence de service d'analyse (`posthog`, `sentry`, `mixpanel`, `amplitude`), aucun code de télémétrie. Les seules occurrences du mot *analytics* appartiennent au module Forecast et désignent des calculs statistiques locaux sur vos propres séries. C'est une **absence vérifiée**, à formuler comme telle : Beaver n'envoie ni statistique d'usage, ni identifiant d'installation, ni rapport de plantage.

### 3. Ce qui n'en sort jamais

Quatre choses, toutes vérifiées :

- **Vos clés d'API.** Elles ne quittent le coffre que pour la requête HTTPS qui les utilise. L'interface, elle, ne peut pas les lire (section 7).
- **Le contenu de vos messages dans le journal d'audit du gateway.** Une ligne de ce journal contient l'horodatage, le canal, l'identifiant de compte, une **empreinte** de l'identifiant utilisateur, le type d'action, un code de décision et une catégorie d'erreur — et rien d'autre (`services/gateway/security/audit.rs:14-23`). L'empreinte est un HMAC-SHA256 tronqué à 8 octets, calculé avec une clé aléatoire propre à votre installation et rangée dans le coffre (`:74-88`). **Le texte du message n'est jamais écrit.**
- **Le corps brut des réponses des fournisseurs.** Avant toute écriture dans les traces, il passe par une fonction qui masque les formats de secret reconnus, remplace les caractères de contrôle et **tronque à 200 caractères** (`services/llm/mod.rs:146-154`).
- **Les demandes d'approbation.** Même quand une conversation est pilotée depuis Telegram, Slack ou Discord, une demande d'approbation **ne part pas dans la messagerie** : elle est envoyée à la fenêtre Beaver (section 5).

### 4. Les cinq couches de protection

Présenter la sécurité de Beaver comme cinq couches indépendantes, chacune avec ce qu'elle garantit **et ce qu'elle ne garantit pas**. C'est la structure de la page ; les cinq sections qui suivent la déroulent.

| Couche | Ce qu'elle contrôle | Sa limite |
|---|---|---|
| 1 — Permissions de l'agent | Ce que l'agent fait sans vous demander | Par défaut : **rien n'est demandé** |
| 2 — Bac à sable du système | Ce qu'une commande shell peut atteindre | **Inactif** tant que l'accès disque n'est pas restreint |
| 3 — Coffre des clés | Où vivent vos clés d'API | Ne chiffre **que** les clés, pas vos conversations |
| 4 — Bornes | La mémoire et le disque que Beaver peut consommer | Ne s'appliquent pas au code d'une extension |
| 5 — Isolation des extensions et connecteurs | Les pannes, l'identité, l'environnement | **N'est pas un bac à sable** pour le code approuvé |

### 5. Couche 1 — Les permissions de l'agent

Le détail vit dans *Agent › Modes de permission*. Ici, seulement ce qui compte pour juger le produit.

**Le mode par défaut est Accès complet.** À la première installation, Beaver écrit `agent-settings.json` avec `{"permissionMode":"auto"}` (`storage_migration.rs:83`). C'est aussi la valeur de repli quand le fichier est absent, illisible ou porte une valeur inconnue. En Accès complet, **l'agent exécute ses outils sans jamais demander** : écrire un fichier, lancer une commande, modifier une branche Git.

**La recommandation à donner, sans détour :** passez en **Demande d'approbation** si vous n'avez pas une raison précise de rester en Accès complet.

**Un plafond protège les canaux externes.** Quand une conversation arrive de Telegram, Slack ou Discord, le pont demande le mode `auto` (`services/gateway/agent_bridge_run.rs:110-112`), mais cette demande est **plafonnée par votre réglage** : si le mode demandé est plus permissif que le vôtre, c'est le vôtre qui est retenu (`commands/agent_chat_task/common.rs:49-58`, avec la comparaison en `:202-204`). Un canal externe ne peut donc jamais obtenir plus de droits que ceux que vous avez consentis dans l'application.

**Et personne ne peut approuver à distance.** Le pont gateway ne fournit aucun émetteur de permission (`agent_bridge_run.rs:113`). Sans émetteur, une demande d'approbation est publiée dans la fenêtre de l'application (`services/agent_local/stream_events.rs:78-93`). Conséquence à écrire noir sur blanc : **une action sensible demandée depuis une messagerie attend devant votre écran** ; elle ne s'approuve pas depuis le téléphone.

**L'exception à connaître : les réveils programmés.** Une exécution déclenchée par un réveil demande le mode `FullAccess` (`services/scheduler/agentic.rs:112`), et ce mode-là **n'est pas plafonné** par votre réglage (`commands/agent_chat_task/common.rs:48`). Un commentaire du code assume ce choix : un réveil est traité comme une session manuelle en accès complet (`:46-47`). **Décision produit confirmée le 9 septembre 2026**, et la raison se donne telle quelle sur la page : un réveil s'exécute par définition sans personne devant l'écran ; en Demande d'approbation, la demande s'afficherait dans une fenêtre que personne ne regarde et l'exécution resterait bloquée — la fonctionnalité n'existerait pas. Le consentement est donné en amont, au moment où vous créez le réveil et écrivez la tâche qu'il exécutera. À dire clairement : **si vous programmez des réveils, ils s'exécutent en Accès complet même si vos conversations sont en Demande d'approbation — c'est ce qui leur permet de travailler en votre absence.**

### 6. Couche 2 — Le bac à sable du système, et sa condition

**C'est l'argument le plus fort de Beaver, et il est conditionnel. La page doit livrer les deux moitiés dans la même phrase.**

**Ce que fait Beaver.** Quand vous restreignez l'accès disque de l'agent à un ou plusieurs dossiers, chaque commande shell n'est plus lancée directement : elle passe par un processus intermédiaire qui applique une politique d'isolation **avant** d'exécuter la commande (`services/agent_local/shell_sandbox/launch.rs:51-83`). Cette politique est appliquée par le **noyau du système d'exploitation**, pas par le code de Beaver :

| Système | Mécanisme | Source |
|---|---|---|
| macOS | **Seatbelt**, via `/usr/bin/sandbox-exec` et un profil de politique généré | `shell_sandbox/macos.rs:7`, `:145` |
| Linux | **Landlock**, règles de lecture et d'écriture posées par chemin | `shell_sandbox/linux.rs:1-2`, `:124-137` |
| Windows | Un **AppContainer** créé pour l'occasion, avec des droits accordés dossier par dossier | `shell_sandbox/windows.rs:62`, `:70-71`, `:92` ; `shell_sandbox/windows/windows_profile.rs:16-33` |

C'est le seul réglage de Beaver dont l'effet **ne dépend pas du bon fonctionnement de Beaver**. Même si l'application se trompait, le noyau refuserait l'accès. À mettre en avant comme tel.

**La condition, à énoncer immédiatement après.** Ce mécanisme ne s'active que si l'accès disque est effectivement restreint. Le code teste d'abord si la portée configurée autorise le disque entier ; si oui, il lance la commande **sans aucune isolation** (`shell_sandbox/launch.rs:42-49`). Or :

- la portée par défaut est **la racine du disque** — `/` sur macOS et Linux, `C:\` sur Windows (`models/config.rs:116-125`, valeur posée par défaut en `:46`) ;
- une portée est considérée comme non restreinte dès qu'une de ses racines est une racine de système de fichiers (`services/agent_local/directory_access_scope.rs:25-32`, variante Windows en `:34-44`).

**Donc, avec les réglages d'origine, il n'y a pas de bac à sable.** La phrase à écrire sur le site, sans l'adoucir : *tant que l'agent a accès à tout le disque, ses commandes s'exécutent sans isolation ; dès que vous réduisez sa portée à vos dossiers de travail, le système d'exploitation lui-même les enferme.*

C'est le meilleur argument pour restreindre la portée : ce n'est pas seulement une limite déclarative, cela **allume** une protection.

**Ce que la restriction contrôle par ailleurs.** Tout chemin venant de l'interface est résolu puis comparé aux racines autorisées : validation de forme d'abord — chemin absolu, sans caractère de contrôle, **sans segment `..`**, au plus 4 096 caractères (`services/agent_local/directory_access.rs:156-170`) — puis résolution des liens symboliques et des raccourcis (`:99-118`), puis appartenance à une racine autorisée (`:94-97`). Au plus **70 dossiers** peuvent être déclarés (`:7`).

### 7. Couche 3 — Le coffre des clés

Le détail vit dans *Sécurité › Le coffre et les clés d'API*. Ici, un paragraphe et un renvoi.

Vos clés d'API sont rangées dans un fichier chiffré, `secrets.enc`, en **XChaCha20-Poly1305** — un chiffrement authentifié, avec un nonce aléatoire tiré d'un générateur cryptographique à chaque écriture (`services/vault.rs:2-4`, `:90`, `:93`). La clé qui ouvre ce coffre **n'est pas sur le disque** : elle vit dans le trousseau du système — Keychain sur macOS, DPAPI sur Windows, Secret Service sur Linux (`:52`, `:74-82`). Elle est lue **une seule fois** au démarrage et conservée en mémoire dans un conteneur qui s'efface (`services/api_keys.rs:47-48`, `:84`).

**L'interface ne peut pas lire une clé.** Les commandes exposées au frontend sont : enregistrer, supprimer, vérifier la présence, lister les fournisseurs configurés, lire les paramètres de connexion, tester (`commands/api_keys.rs:15`, `:62`, `:75`, `:80`, `:85`, `:100`, `:105`). **Aucune ne renvoie une clé** — et celle qui renvoie des paramètres de connexion ne transporte que la région, le mode de point d'accès et un identifiant d'espace de travail (`services/provider_connections/qwen.rs:35-41`). Le code JavaScript de l'application n'a donc jamais une clé entre les mains.

**Ce que le coffre ne couvre pas, à dire dans le même souffle :** il protège vos clés, **pas vos conversations**. Les fichiers `agent-sessions/*.json` sont du JSON en clair (`services/agent_local/session_store_document.rs:75`). Ils sont protégés par les permissions du système de fichiers — `0600` pour les fichiers, `0700` pour les dossiers sur macOS et Linux, une liste d'accès restreinte sur Windows (`services/private_store.rs:169-179`) — mais pas chiffrés. Ce qui veut dire : lisibles par tout programme lancé sous votre compte.

### 8. Couche 4 — Les bornes

Beaver plafonne ce qu'il peut consommer, pour qu'un emballement — boucle d'agent, réponse démesurée, flot de messages entrants — ne finisse ni en saturation de mémoire ni en disque plein. Les valeurs sont dans le tableau de la section Tableaux.

Deux mécanismes méritent une phrase en prose :

- **Les requêtes qui portent un secret sont bridées.** Le client HTTP utilisé pour ces requêtes **refuse toute redirection** (`services/secure_http.rs:82`) et **n'accepte que HTTPS**, la seule exception étant un service local sur la boucle (`:129-134`). Une redirection malveillante ne peut donc pas emmener une clé vers un serveur tiers. Les réponses sont lues avec une taille maximale (`:19`).
- **Les canaux externes sont fermés par défaut aux inconnus.** Un message venant de Telegram, Slack ou Discord n'est traité que si son expéditeur figure dans la liste des utilisateurs autorisés du compte, et **le joker `*` est refusé** à la construction de cette liste (`services/gateway/agent_bridge.rs:70-73`). Un expéditeur absent de la liste est bloqué et journalisé. Une limite de débit par utilisateur s'applique ensuite (`:74-83`, seaux bornés à 10 000 et purgés au bout de 5 minutes, `services/gateway/security/rate_limit.rs:4-5`).

### 9. Couche 5 — L'isolation des extensions et des connecteurs

Deux mondes différents, à ne pas confondre. Les deux ont leur page ; ici, uniquement ce qui change le jugement d'ensemble.

**Les connecteurs MCP sont durcis.** Beaver n'accepte de lancer que trois programmes — `npx`, `uvx`, `deno` (`services/mcp_bridge/stdio_cmd.rs:3`) —, ne passe jamais par un shell, et **efface l'environnement du processus parent** avant de transmettre une liste explicite de variables (`services/mcp_bridge/process_spawn.rs:25`). Vos variables d'environnement et vos secrets ne fuitent donc pas vers un connecteur par simple héritage. Détail complet dans *Intégrations › Les connecteurs MCP*.

**Les extensions ne sont pas dans un bac à sable, et Beaver le dit lui-même.** Une extension que vous approuvez est du **code de confiance** : il s'exécute avec les droits de votre compte, peut lire et écrire vos fichiers, lancer des programmes, utiliser le réseau, et demander vos clés par l'API du kit de développement. L'isolation par processus — un processus hôte par extension tierce — protège la **stabilité** de l'application, pas vos données (`SECURITY.md`, « Plugins and custom extensions » ; détail et sources dans *Intégrations › Le centre d'extensions*).

Ce que Beaver garantit malgré tout : une double décision explicite (installer, puis faire confiance), une empreinte des fichiers qui révoque l'approbation dès qu'ils changent, une identité attribuée par Beaver et non déclarée par l'extension, et un journal de chaque remise de secret. **Renvoyer à la page Extensions, ne pas redétailler ici.**

### 10. Les limites assumées

Cette section n'est pas une clause de non-responsabilité : c'est le cœur de la crédibilité de la page. Six points, chacun avec la conduite à tenir.

1. **Le mode par défaut est le plus permissif.** Accès complet sur tout le disque (`storage_migration.rs:83` ; `models/config.rs:116-125`). *Conduite à tenir :* passer en Demande d'approbation, et restreindre la portée disque.
2. **Sans restriction de portée, aucune commande n'est isolée** (`shell_sandbox/launch.rs:42-49`). *Conduite à tenir :* déclarer vos dossiers de travail — c'est ce qui allume la protection du noyau.
3. **Les réveils programmés s'exécutent en Accès complet**, quel que soit votre réglage (`services/scheduler/agentic.rs:112` ; `commands/agent_chat_task/common.rs:48`). *Conduite à tenir :* n'automatiser que ce que vous accepteriez de laisser tourner seul.
4. **Le code d'une extension approuvée n'est contraint par rien** (`SECURITY.md`, « Plugins and custom extensions »). *Conduite à tenir :* lire le code avant d'approuver ; en cas de doute, révoquer la clé chez son fournisseur, pas seulement désactiver l'extension.
5. **Vos conversations ne sont pas chiffrées** (`session_store_document.rs:75`). *Conduite à tenir :* traiter `~/.local/share/cl-go-dash/` comme un dossier sensible, et relire les journaux avant de les partager.
6. **Il n'y a pas de signature de code sur les binaires macOS** (`SECURITY.md`, « Limitations and known gaps »). *Conduite à tenir :* installer par le script fourni plutôt que par le navigateur, ou compiler depuis les sources.

### 11. Ce qui est hors de portée de Beaver

Trois cas qu'aucune application de bureau ne peut couvrir, à énoncer pour que l'utilisateur sache où s'arrête la promesse (`SECURITY.md`, « Threat model » et « Limitations and known gaps ») :

- **Un accès physique à une machine déverrouillée.** Tout est lisible sous votre session.
- **Un logiciel malveillant tournant déjà sous votre compte.** Il a les mêmes droits que Beaver.
- **Un trousseau système compromis.** La clé maîtresse du coffre y vit ; si le trousseau tombe, le coffre tombe. C'est une propriété inhérente au stockage de secrets sur un poste de travail, pas un choix de Beaver.

### 12. Les cinq réglages qui changent tout

Terminer par de l'actionnable. Cinq gestes, dans l'ordre d'effet décroissant, chacun renvoyant à la page qui l'explique.

1. **Restreindre l'accès disque** à vos dossiers de travail — c'est le seul geste qui active une protection du noyau.
2. **Passer en Demande d'approbation** — voir *Agent › Modes de permission*.
3. **Choisir un modèle local** pour tout contenu sensible — rien ne sort de la machine.
4. **N'approuver une extension qu'après avoir lu son code** — et révoquer la clé, pas seulement l'extension, en cas de doute.
5. **Restreindre la liste des utilisateurs autorisés** sur chaque canal externe — elle est obligatoire, mais c'est vous qui la remplissez.

---

## Tableaux

### Ce qui sort de votre machine, et vers qui

| Fonction | Destination | Ce qui part | Déclenché par |
|---|---|---|---|
| Modèle cloud | Le fournisseur choisi | La conversation entière, **y compris le contenu rapporté par les outils** (`chat_message.rs:37`) | Vous, en choisissant le modèle |
| Modèle local (Ollama) | **Rien ne sort** — boucle locale uniquement (`ollama_manager/spawn_gate_unix_support.rs:112`) | — | — |
| Recherche web | Le moteur configuré (Brave, Exa, Firecrawl) ou SearXNG en local (`searxng/process.rs:49`) | La requête de recherche | L'agent, via son outil de recherche |
| Lecture d'une page web | Le site visé | La requête | L'agent, via son outil de lecture |
| Connecteur MCP | Le service du connecteur | Ce que l'appel d'outil transporte | Vous, en installant le connecteur |
| Canal externe | Telegram, Slack ou Discord | Les réponses de l'agent | Vous, en configurant le canal |
| Forecast cloud | `api.nixtla.io` | Le jeu de données envoyé (`SECURITY.md`, « Safe usage recommendations ») | Vous, en choisissant un modèle cloud |
| Vérifications de mise à jour (application, moteur, modèles) | `api.github.com`, `github.com`, `ollama.com` (`commands/app_update_source.rs:15`, `:29-33` ; `src/hooks/use-update-checker.ts:13`) | L'en-tête `User-Agent` (`:154-156`) ; `ollama.com` reçoit le nom des familles de modèles installées | Automatique, toutes les heures |
| Catalogue des modèles | `raw.githubusercontent.com` (`llm/litellm_catalog_refresh.rs:9`) | Une requête conditionnelle | Automatique, au démarrage |
| Aperçu des liens | Le site de chaque lien affiché dans une conversation (`chat-markdown.tsx:78`) | Une requête, sans clic | Automatique, activé par défaut — désactivable (`models/config.rs:54`) |
| Télémétrie, statistiques d'usage | **Aucune** — absence vérifiée dans tout le code | — | — |

### Les bornes de l'application

| Ressource | Limite | Source |
|---|---|---|
| Conversations d'agent en cours | **32** | `agent_local/agent_work_supervision.rs:5` |
| Sous-agents simultanés | **8** | `agent_local/agent_work_supervision.rs:6` |
| Commandes shell simultanées | **64** | `agent_local/agent_work_supervision.rs:7` |
| Sessions de terminal | **16** | `services/terminal/manager.rs:35` |
| Messages par conversation | **2 000** | `agent_local/session_limits.rs:10` |
| Dossiers autorisés déclarables | **70** | `agent_local/directory_access.rs:7` |
| Longueur d'un chemin | **4 096** caractères | `agent_local/directory_access.rs:9` |
| Corps de réponse authentifiée | **32 Mio** | `services/secure_http.rs:19` |
| Réponse de vérification de mise à jour | **512 Kio** | `commands/app_update_source.rs:20` |
| Corps de fournisseur écrit dans les traces | **200 caractères**, après masquage | `services/llm/mod.rs:146-154` |
| Utilisateurs suivis par la limite de débit | **10 000**, purgés après **5 minutes** | `gateway/security/rate_limit.rs:4-5` |
| Entrées d'une liste d'utilisateurs autorisés | **1 000**, chacune ≤ **128** caractères | `gateway/security/allowlist.rs:3-4` |

Le tableau complet, extensions comprises, vit dans *Référence › Limites et quotas* et dans `SECURITY.md`.

### Chaque couche et sa limite

Reprendre le tableau de la section 4 du contenu. C'est le résumé que le lecteur pressé emporte.

---

## Encadrés

Ces encadrés vont dans le corps du texte, à l'endroit indiqué.

> **⚠ En tête de page — Beaver démarre en accès complet.**
> À l'installation, l'agent peut exécuter ses outils sans rien demander, sur l'ensemble de votre disque. C'est le réglage d'origine, et il est délibéré : Beaver est conçu pour travailler dans vos projets. Si vous préférez valider chaque action, passez en **Demande d'approbation** et **restreignez l'accès disque** à vos dossiers de travail. Ces deux réglages sont les plus importants de l'application.

> **✔ Dans la section Bac à sable — Une protection garantie par le système, pas par Beaver.**
> Dès que vous restreignez l'accès disque, chaque commande lancée par l'agent est enfermée par le système d'exploitation lui-même : **Seatbelt** sur macOS, **Landlock** sur Linux, un **AppContainer** sur Windows. C'est le seul réglage de Beaver dont l'effet ne dépend pas du bon fonctionnement de Beaver : même en cas de défaut de l'application, le noyau refuse l'accès.

> **⚠ Immédiatement après le précédent — Et il ne s'active pas tout seul.**
> Ce bac à sable est **conditionné à la restriction de la portée disque**. Tant que l'agent a accès à la racine du disque — le réglage d'origine — les commandes s'exécutent **sans aucune isolation**. C'est la raison la plus concrète de déclarer vos dossiers de travail : vous ne faites pas que limiter une portée, vous allumez une protection.

> **ℹ Dans la section Ce qui sort — Ce que l'agent lit, le fournisseur le lit.**
> Avec un modèle cloud, la conversation envoyée contient aussi ce que les outils ont rapporté : le contenu des fichiers lus, la sortie des commandes exécutées, les pages web consultées. Pour un contenu que vous ne voulez transmettre à personne, utilisez un modèle local : rien ne quitte alors votre machine.

> **ℹ Dans la section Coffre — Le coffre protège vos clés, pas vos conversations.**
> `secrets.enc` est chiffré. Vos conversations, elles, sont enregistrées en clair dans votre dossier personnel, protégées par les seules permissions du système de fichiers. Traitez `~/.local/share/cl-go-dash/` comme un dossier sensible, et relisez un journal avant de le partager.

> **⚠ Dans la section Permissions — Personne n'approuve à votre place, à distance.**
> Une conversation pilotée depuis Telegram, Slack ou Discord ne peut jamais obtenir plus de droits que ceux réglés dans l'application, et **aucune demande d'approbation n'est envoyée dans la messagerie** : elle s'affiche dans la fenêtre de Beaver, sur votre ordinateur. Une action sensible demandée à distance attend donc devant votre écran.

> **⚠ Dans la section Limites — Les réveils programmés font exception.**
> Une conversation déclenchée par un réveil s'exécute en **Accès complet**, même si vos conversations manuelles sont en Demande d'approbation. C'est un choix assumé du produit : un réveil travaille sans personne devant l'écran, et une demande d'approbation sans personne pour y répondre bloquerait l'exécution au lieu de la protéger. Votre consentement, vous le donnez au moment de créer le réveil. N'automatisez que ce que vous accepteriez de laisser tourner sans surveillance.

> **ℹ Dans la section Ce qui sort — Aucune télémétrie.**
> Beaver n'envoie ni statistique d'usage, ni identifiant d'installation, ni rapport de plantage. Les seules requêtes qu'il émet de lui-même sont la vérification de mise à jour et le rafraîchissement du catalogue public des modèles. Vérifié dans l'intégralité du code, pas seulement annoncé.

---

## Pièges et erreurs fréquentes

| Idée reçue | Ce qui est vrai | Conduite à tenir |
|---|---|---|
| « Beaver est installé, donc il est en mode sécurisé » | Le mode d'origine est **Accès complet** sur tout le disque | Changer les deux réglages dès le premier lancement |
| « Mes commandes sont dans un bac à sable » | Seulement si la portée disque est restreinte | Déclarer les dossiers de travail |
| « Restreindre l'accès disque n'est qu'une gêne » | C'est le geste qui **active** l'isolation du noyau | Le faire d'abord |
| « Mes conversations sont chiffrées comme mes clés » | Seul `secrets.enc` est chiffré ; les conversations sont en clair | Protéger le dossier de données ; préférer un modèle local pour le sensible |
| « Le modèle ne voit que ce que je tape » | Il voit aussi tout ce que les outils rapportent | Choisir un modèle local pour les contenus confidentiels |
| « Je peux approuver une action depuis Telegram » | Aucune demande d'approbation ne part dans la messagerie | Répondre depuis la fenêtre Beaver |
| « Mes réveils respectent mon mode de permission » | Ils s'exécutent en Accès complet | N'automatiser que du travail acceptable sans surveillance |
| « Une extension est isolée dans son processus » | Le processus séparé protège la stabilité, pas vos données | Lire le code avant d'approuver |
| « Désactiver une extension récupère la clé qu'elle a obtenue » | Non : une copie a pu être conservée | Révoquer la clé chez son fournisseur |
| « N'importe qui sur mon canal Telegram peut parler à l'agent » | Faux : la liste des utilisateurs autorisés est obligatoire et le joker `*` est refusé | Vérifier tout de même son contenu |
| « Beaver envoie des statistiques d'usage » | Non, aucune télémétrie dans le code | — |

---

## Renvois

- `11-securite/vault-et-cles-api.md` — le coffre chiffré, la clé maîtresse dans le trousseau du système, la zéroïsation, et pourquoi l'interface ne peut jamais lire une clé
- `11-securite/acces-fichiers.md` — la portée des dossiers, la protection contre la traversée de chemin, les différences par système
- `11-securite/durcissement.md` — les bornes, le client HTTP sécurisé, le durcissement MCP, le navigateur isolé, les traces filtrées
- `11-securite/mises-a-jour-verifiees.md` — la vérification des métadonnées, les téléchargements bornés, l'échec fermé
- `11-securite/confidentialite-des-donnees.md` — ce que voient les fournisseurs, l'absence de télémétrie, l'effacement des données locales
- `11-securite/signaler-une-vulnerabilite.md` — la procédure, le périmètre, les délais
- `04-agent/permissions.md` — les trois modes, ce qui déclenche une demande, la règle particulière des commandes shell
- `07-integrations/extensions-centre.md` — installer, approuver et révoquer une extension ; ce que l'approbation engage
- `07-integrations/mcp-connecteurs.md` — les connecteurs externes et leur durcissement
- `09-automatisation/` — les réveils programmés, et leur mode de permission
- `12-reference/limites-et-quotas.md` — le tableau complet des bornes
- `12-reference/stockage-local.md` — l'arborescence de `~/.local/share/cl-go-dash/`

---

## Points à confirmer

**Écarts relevés entre `SECURITY.md` et le code — à arbitrer avant publication**

1. **`SECURITY.md` écrit « all inbound messages are hashed and logged » pour le journal du gateway.** Le code ne hache pas le message : il hache **l'identifiant de l'utilisateur** (`gateway/security/audit.rs:74-88`), et le contenu du message n'est écrit nulle part (`:14-23`). La réalité est donc **meilleure** que ce qu'annonce le document, mais la formulation actuelle laisse croire que le message transite dans le journal. À corriger dans `SECURITY.md`, et à ne pas reprendre telle quelle sur le site.
2. **`SECURITY.md` ne mentionne nulle part le bac à sable du shell** — ni Seatbelt, ni Landlock, ni AppContainer, ni sa condition d'activation. C'est la protection la plus solide du produit et elle est absente du document de sécurité officiel. À signaler à l'équipe : `SECURITY.md` mérite une section, dans les deux moitiés (le mécanisme **et** sa condition).

**Écart relevé avec les notes d'audit internes**

3. **Les notes internes décrivent le mécanisme Windows comme un « profil restreint ».** Le code crée en réalité un **AppContainer** dédié — `CreateAppContainerProfile` (`shell_sandbox/windows/windows_profile.rs:16-33`) — puis accorde des droits dossier par dossier au SID obtenu (`shell_sandbox/windows.rs:70-88`) avant de lancer le processus dans ce conteneur (`:92`). Retenir **AppContainer** sur le site : c'est le nom que Microsoft emploie et il est vérifiable.

**Décisions produit et de rédaction**

4. **Faut-il recommander explicitement de changer les deux réglages par défaut dès l'installation ?** La page le fait actuellement. Cela revient à écrire, sur le site officiel, que la configuration d'origine n'est pas la plus sûre. C'est honnête et c'est la position recommandée ici — mais c'est une décision produit, pas une décision de rédaction. Alternative : faire changer le défaut dans l'application, auquel cas cette page se réécrit.
5. ~~L'exception des réveils programmés mérite-t-elle un réglage ?~~ **Tranché le 9 septembre 2026 : non, c'est le comportement voulu et il restera tel quel.** Le but d'un réveil est précisément de s'exécuter sans demande d'approbation — l'utilisateur n'est pas devant l'écran dans la quasi-totalité des cas, et une demande sans personne pour y répondre bloquerait l'exécution. La page l'explique désormais avec cette raison, sans le présenter comme un défaut.
6. **La formulation « aucune télémétrie » repose sur une absence.** Elle a été vérifiée par recherche exhaustive dans `src/` et `src-tauri/src/` sur la version 1.2.2, mais une absence ne se prouve pas aussi solidement qu'une ligne de code. Deux options : la publier telle quelle en datant la vérification, ou la reformuler en « les seules requêtes émises par Beaver de lui-même sont… », qui est une affirmation positive et vérifiable. Recommandation : les deux, dans cet ordre.
7. **Le vocabulaire « bac à sable ».** Le terme est technique pour un public non développeur. À décider : le garder en l'expliquant à sa première occurrence (« le système d'exploitation enferme la commande dans un espace dont elle ne peut pas sortir »), ou employer uniquement la périphrase. Recommandation : garder le terme, l'expliquer une fois — il est trop répandu pour être évité.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

8. **L'écran de réglage de l'accès disque** : où il se trouve exactement, comment on y ajoute un dossier, ce qui est affiché quand la portée est encore la racine du disque. Le code est établi ; l'écran n'a pas été observé. **Point important** : vérifier si l'application signale à l'utilisateur que la portée par défaut désactive l'isolation. Si elle ne le signale pas, c'est une demande d'évolution à ouvrir.
9. **L'indication du mode de permission dans une conversation venue d'un canal externe** : l'utilisateur voit-il, dans la fenêtre Beaver, que la demande d'approbation en attente provient de Telegram, de Slack ou de Discord ? Le mécanisme est vérifié (`stream_events.rs:78-93`), sa présentation ne l'est pas.
10. **La page n'a pas de capture prévue pour les états d'erreur de sécurité** — accès refusé par la portée, isolation du shell indisponible, redirection refusée pendant une mise à jour. Les provoquer suppose de casser volontairement l'environnement. Ils sont documentés d'après le code ; c'est un cas prévu par la convention de ce dossier.
