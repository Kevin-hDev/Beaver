# Confidentialité des données

**Emplacement site** — Sécurité › Confidentialité des données
**Répond à** — « Qui voit mes conversations ? Quelles connexions Beaver ouvre-t-il tout seul ? Comment j'efface tout ? »
**Sources** — `src-tauri/src/services/paths.rs`, `paths/ollama.rs` ; `services/vault.rs` ; `services/api_keys.rs`, `commands/api_keys.rs` ; `services/private_store.rs` ; `services/app_log.rs` ; `services/llm/mod.rs`, `llm/litellm_catalog.rs`, `llm/litellm_catalog_refresh.rs` ; `services/llm_oauth/headers.rs`, `llm_oauth/xai_headers.rs` ; `services/agent_local/tool_hooks.rs`, `session_security.rs`, `sensitive_data_redaction.rs`, `agent_loop_ollama_request.rs`, `conversation_attachments.rs`, `ollama_registry_details.rs`, `tool_result_truncate.rs`, `tool_plan_storage.rs` ; `services/llm/agent_loop_tools.rs`, `agent_loop_request.rs` ; `services/ollama_manager/types.rs`, `release_source.rs`, `release_fetch.rs`, `release_cache.rs`, `spawn_profile.rs`, `spawn_profile_paths.rs` ; `services/search/mod.rs`, `brave.rs`, `exa.rs`, `firecrawl.rs` ; `services/searxng/client.rs`, `settings.rs`, `lifecycle.rs`, `startup.rs`, `wheels.rs` ; `src-tauri/resources/searxng-sidecar/settings.template.yml` ; `services/gateway/security/audit.rs`, `gateway/channels/telegram.rs`, `slack.rs`, `slack_support.rs`, `discord_http.rs` ; `services/mcp_bridge/trusted.rs`, `token_validation.rs` ; `services/forecast/client_nixtla.rs`, `forecast/model_manager/download.rs`, `forecast/dev_updates.rs`, `forecast/sidecar_spawn.rs`, `forecast/sidecar_process_env.rs`, `forecast/model_manager/smoke.rs` ; `services/brand.rs` ; `services/link_preview/mod.rs`, `link_preview/providers.rs`, `link_preview/security.rs` ; `services/provider_usage/remote.rs` ; `services/agent_local/tool_dispatcher.rs`, `tool_web_fetch.rs` ; `src-tauri/src/models/config.rs` ; `commands/app_update.rs`, `app_update_source.rs`, `ollama_updates.rs`, `ollama_version.rs`, `ollama_setup.rs`, `link_preview.rs`, `agent_sessions.rs`, `projects.rs`, `agent_ollama.rs`, `heartbeat.rs`, `mcp_oauth.rs` ; `src-tauri/src/app_build.rs`, `runtime_startup.rs` ; `src/hooks/use-update-checker.ts`, `use-forecast-dev-updates.ts`, `use-provider-usage.ts` ; `src/App.tsx` ; `src/components/ollama/ollama-setup-screen.tsx` ; `src/components/agent-local/chat-markdown.tsx`, `link-preview-card.tsx` ; `src-tauri/Cargo.toml`, `package.json` ; `src-tauri/scripts/download-ollama.sh`
**Vérification** — Vérifié dans le code, ligne par ligne, le **9 septembre 2026**, sur la version **1.2.2**. Le verdict d'absence de télémétrie est établi par une recherche dont la méthode complète est donnée en section 12 : c'est le seul fait de cette page qui ne peut pas porter de `fichier:ligne`, par nature. Deux écarts avec le fichier voisin `modele-de-securite.md` sont signalés en « Points à confirmer ».

---

## Avertissement au rédacteur

Cette page est lue par quelqu'un qui se demande s'il peut faire confiance à Beaver avec des données qui comptent. Trois règles de rédaction, non négociables :

1. **On ne promet jamais plus que ce que le code garantit.** Chaque protection est écrite avec sa condition et sa limite. Une phrase rassurante et fausse coûte davantage qu'une phrase inquiétante et vraie.
2. **On ne parle jamais à la place des fournisseurs tiers.** Ce que Beaver envoie chez OpenAI, Brave ou Telegram est vérifiable dans le code, donc affirmable. Ce que ces sociétés font ensuite de ces données relève de **leurs** politiques : on renvoie vers elles, on ne les paraphrase pas, on ne les résume pas.
3. **Une absence se démontre par une méthode, pas par une affirmation.** « Beaver n'envoie aucune télémétrie » n'est publiable qu'accompagné de la façon dont on l'a cherchée. La section 12 existe pour ça.

**Ce que cette page n'est pas.** Elle ne redit ni `11-securite/modele-de-securite.md` (les couches de protection, les permissions, le bac à sable) ni `11-securite/vault-et-cles-api.md` (le chiffrement du coffre). Elle répond à une question différente et plus étroite : **où vont mes données, et comment je les reprends**.

---

## Plan de page proposé

1. La question en une phrase
2. Qui voit mes conversations ? — le principe
3. Avec un modèle cloud : tout ce que l'agent lit part aussi
4. Avec un modèle local : ce qui reste vraiment sur la machine
5. Le filet automatique : les secrets retirés avant l'envoi
6. La recherche web : quatre chemins, quatre expositions
7. Les connecteurs externes et les extensions
8. Les canaux Telegram, Slack et Discord
9. Quelles connexions Beaver ouvre-t-il tout seul ?
10. Les connexions qui n'ont lieu que si vous le demandez
11. Les identifiants transmis : la règle et son exception
12. Télémétrie, statistiques d'usage, rapports de plantage
13. Où vivent vos données sur le disque
14. Ce qui est écrit dans les traces — et ce qui en est retiré
15. Comment tout effacer

---

## Contenu

### 1. La question en une phrase

Beaver est une application de bureau : il n'existe pas de serveur Beaver, pas de compte Beaver, pas de synchronisation. **L'essentiel de ce qui quitte votre machine, ce sont les données que vous envoyez à un service que vous avez vous-même configuré** — plus une courte liste de connexions que Beaver ouvre seul, détaillée et chiffrée en section 9.

Cette phrase est le résumé de la page. Le reste en donne la preuve et les nuances — et il y en a, notamment l'aperçu des liens (section 9), qui contacte des sites sans que vous ayez cliqué. Ne pas écrire « rien ne sort sans votre action » : ce serait faux.

### 2. Qui voit mes conversations ? — le principe

La réponse dépend d'une seule chose : **le modèle choisi pour la conversation**.

- Modèle **local** (Ollama) → la conversation ne quitte pas la machine.
- Modèle **cloud** (clé API ou compte web) → la conversation entière part chez ce fournisseur, à chaque tour.

Le choix se fait conversation par conversation et se change en cours de route : voir `01-decouverte/local-vs-cloud.md`. Ce qui suit approfondit ce que « la conversation entière » recouvre exactement, parce que c'est la partie qui surprend.

### 3. Avec un modèle cloud : tout ce que l'agent lit part aussi

**La formule à retenir, et à mettre en encadré : *ce que l'agent lit, le fournisseur le lit aussi*.**

Une conversation Beaver n'est pas seulement vos messages. C'est une liste de messages qui contient aussi :

- **vos messages** ;
- **les pièces jointes** : un fichier texte est inséré dans la conversation sous forme de texte, une image sous forme d'image encodée (`conversation_attachments.rs:28-37`, `ResolvedAttachmentContent::Text` et `Image` avec son contenu en base64) ;
- **le résultat de chaque outil que l'agent utilise** — le contenu d'un fichier lu, la sortie d'une commande de terminal, la liste d'un dossier, le résultat d'une recherche ;
- **le prompt système**, dont les réglages de personnalité et la liste des skills disponibles (`chat_prompt_sections.rs:9-23`).

Cet ensemble est renvoyé au modèle **à chaque tour** de la conversation, pas seulement au premier. Un fichier lu une fois reste dans le contexte et repart à chaque échange suivant.

Conséquence à énoncer franchement sur le site : **si vous demandez à un agent piloté par un modèle cloud de lire un dossier, le contenu de ce dossier part chez ce fournisseur.** Ce n'est pas un défaut, c'est le fonctionnement d'un agent ; mais l'utilisateur doit le savoir avant, pas après.

**Ce que Beaver n'envoie pas au fournisseur, en plus des messages** : aucun identifiant d'installation, aucun nom de machine, aucune statistique. L'exception — une seule — est traitée en section 11.

**Ce que le fournisseur fait ensuite de ces données** — conservation, entraînement de modèles, durée de rétention — **ne dépend pas de Beaver et n'est pas décrit ici.** La page renvoie aux politiques de confidentialité et aux conditions d'utilisation de chaque fournisseur, par un lien, sans les résumer. Un résumé vieillit et engage Beaver sur des pratiques qu'il ne contrôle pas.

### 4. Avec un modèle local : ce qui reste vraiment sur la machine

Quand la conversation utilise un modèle Ollama, la requête part vers **une adresse de la machine elle-même**, et le code le garantit par construction :

- l'adresse du moteur est fabriquée uniquement sous la forme `http://127.0.0.1:<port>` (`ollama_manager/types.rs:126-133`) ;
- toute adresse fournie autrement est **refusée** si elle n'est pas exactement en `http` sur `127.0.0.1`, sans identifiant, sans mot de passe, sans paramètre ni fragment (`types.rs:145-156`) ;
- les modèles « cloud » d'Ollama sont désactivés explicitement : Beaver lance le moteur avec `OLLAMA_NO_CLOUD=1` (`ollama_manager/spawn_profile.rs:120`).

**Aucune requête de conversation ne sort donc de la machine avec un modèle local.** C'est la réponse à donner à quelqu'un qui manipule des données confidentielles.

**Trois nuances à ne pas escamoter :**

1. **Le modèle doit d'abord être téléchargé**, et ce téléchargement, lui, passe par Internet (section 10).
2. **Une vérification de mise à jour des modèles a lieu automatiquement** et envoie le nom des familles de modèles installées à `ollama.com` (section 9). C'est le seul point où « modèle local » ne veut pas dire « silence radio total », et il doit être écrit.
3. **Les outils de l'agent, eux, peuvent sortir**. Un agent local à qui on demande une recherche web ouvre une connexion (section 6). Le modèle est local ; ses outils ne le sont pas tous.

### 5. Le filet automatique : les secrets retirés avant l'envoi

Beaver retire les valeurs qui ressemblent à des secrets **du résultat des outils, avant que ce résultat n'entre dans la conversation** — donc avant qu'il ne parte chez un fournisseur cloud.

Le mécanisme, vérifié : après chaque appel des outils `bash`, `bash_control`, `read_file`, `grep`, `glob` et `list_dir`, le contenu du résultat passe par une fonction de caviardage qui remplace les valeurs reconnues par `[REDACTED]` (`agent_local/tool_hooks.rs:83-92`).

Ce que la fonction reconnaît (`sensitive_data_redaction.rs:7-52`) :

- les paires du type `api_key=…`, `token: …`, `password=…`, `client_secret`, `access_token`, `refresh_token`, `authorization` ;
- les formats de jeton connus : `sk-…` (OpenAI), `gsk_…`, `xai-…`, `csk-…`, `hf_…`, `ghp_/gho_/ghu_/ghs_/ghr_…`, `github_pat_…`, `glpat-…`, `xoxb-…`/`xapp-…` (Slack), les clés AWS `AKIA…`/`ASIA…`, les clés Google `AIza…`, les jetons de robot Telegram, les jetons JWT, les URL de webhook Slack ;
- les en-têtes `Bearer …` ;
- les blocs de clé privée et de certificat au format PEM.

Ce même caviardage s'applique aussi au journal de la conversation enregistré sur le disque et à ce qui est affiché dans l'interface (`session_security.rs:10-29`).

**Ce filet a des limites, et la page doit les écrire :**

- il reconnaît des **formats connus**. Un secret qui ne ressemble à aucun de ces motifs — un mot de passe ordinaire dans un fichier texte, un numéro de dossier médical, une adresse — **n'est pas caviardé**. Ce n'est pas un filtre de données personnelles, c'est un filtre de jetons techniques ;
- il s'applique **à six outils nommés**, pas à tous. Le résultat d'un connecteur externe ou d'une extension n'y passe pas ;
- **il ne remplace pas votre jugement** sur le dossier que vous ouvrez à l'agent.

### 6. La recherche web : quatre chemins, quatre expositions

Quand l'agent utilise l'outil de recherche web, Beaver essaie les fournisseurs configurés dans un ordre fixe — **Brave, puis Exa, puis Firecrawl** — et se rabat sur **SearXNG local** si aucun n'est configuré ou si tous échouent (`search/mod.rs:73-77` et `:92-98`).

**Avec Brave, Exa ou Firecrawl** : votre requête part chez ce fournisseur, accompagnée de votre clé API — donc rattachée à votre compte chez lui.

| Fournisseur | Adresse contactée | Source |
|---|---|---|
| Brave | `api.search.brave.com` | `search/brave.rs:10` |
| Exa | `api.exa.ai` | `search/exa.rs:12` |
| Firecrawl | `api.firecrawl.dev` | `search/firecrawl.rs:15-16` |

Ce qui part exactement : la requête de recherche, et rien d'autre que les paramètres de format — nombre de résultats, longueur d'extrait (`exa.rs:20-26`, `firecrawl.rs:24-27`). Ce que ces sociétés conservent relève de leurs politiques ; renvoyer vers elles.

**Avec SearXNG**, le repli local, la situation est différente et **doit être expliquée précisément, sans exagérer l'avantage** :

- SearXNG s'exécute **sur votre machine**. Il n'écoute que la boucle locale : le fichier de configuration fixe `bind_address: "127.0.0.1"` (`resources/searxng-sidecar/settings.template.yml:16`), et Beaver l'interroge sur `http://127.0.0.1:<port>` (`searxng/lifecycle.rs:65-67`, `searxng/client.rs:7-16`) ;
- il n'y a **aucune clé API**, donc votre requête n'est rattachée à aucun compte ;
- il n'y a **aucun cache partagé** : la configuration désactive Valkey (`settings.template.yml:24-25`) et les métriques (`:6`) ;
- **mais SearXNG est un métamoteur.** Son rôle est d'interroger des moteurs de recherche publics pour vous et d'en agréger les résultats. **Votre requête sort donc de la machine**, vers ces moteurs, depuis votre adresse IP. Ce qui change par rapport à Brave ou Exa, ce n'est pas que la requête reste chez vous — c'est qu'elle n'est **pas associée à un compte** et qu'elle n'est pas concentrée chez un seul fournisseur.

**Cette nuance est le point le plus facile à rater de toute la page.** Écrire « la recherche SearXNG est locale » sans la préciser serait faux. Formulation juste : *SearXNG évite d'associer vos recherches à un compte ; il ne les empêche pas de sortir.*

La liste des moteurs interrogés vient des réglages par défaut de SearXNG lui-même — la configuration de Beaver active `use_default_settings: true` (`settings.template.yml:1`) — et non d'un choix inscrit dans le dépôt Beaver. Voir « Points à confirmer ».

### 7. Les connecteurs externes et les extensions

Deux mécanismes que l'utilisateur ajoute lui-même, et qui déplacent la frontière :

**Les connecteurs externes (MCP).** Un connecteur peut être un programme local ou un service distant. Ceux qui sont distants reçoivent les arguments des outils que l'agent leur passe — c'est-à-dire, potentiellement, du contenu de votre travail. Les adresses des connecteurs reconnus par Beaver sont écrites dans `mcp_bridge/trusted.rs:50-66`. Rien n'est contacté tant que vous n'avez pas configuré et activé le connecteur.

**Les extensions.** Une extension approuvée s'exécute avec les droits de votre compte et peut utiliser le réseau librement. Beaver ne surveille pas ce qu'elle envoie. C'est traité en détail dans `07-integrations/extensions-centre.md` ; cette page-ci se contente de renvoyer vers lui, avec une phrase : **une extension approuvée sort du périmètre décrit ici.**

### 8. Les canaux Telegram, Slack et Discord

Si vous branchez un canal externe, vos messages et les réponses de l'agent **transitent par les serveurs de la plateforme concernée**, comme n'importe quelle conversation sur ces services :

| Canal | Adresse contactée | Source |
|---|---|---|
| Telegram | `api.telegram.org` | `gateway/channels/telegram.rs:41` |
| Slack | `slack.com/api/…` | `gateway/channels/slack.rs:173`, `slack_support.rs:29`, `:51` |
| Discord | `discord.com/api/v10/…` | `gateway/channels/discord_http.rs:30` |

**Le journal d'audit local est le point rassurant de cette section, et il est solide.** Beaver tient un journal des actions du gateway dans `~/.local/share/cl-go-dash/logs/gateway-audit.jsonl` (`gateway/security/audit.rs:143-145`). Une ligne contient exactement sept champs : horodatage, canal, identifiant de compte, **empreinte** de l'identifiant utilisateur, type d'action, code de décision, catégorie d'erreur (`audit.rs:14-23`).

Trois faits à énoncer :

- **le texte des messages n'y figure pas.** Les actions possibles sont des étiquettes — message reçu, message envoyé, bloqué, limité, erreur d'agent, canal démarré, canal arrêté, authentification échouée (`audit.rs:25-36`) — jamais du contenu ;
- **l'identifiant de la personne n'y figure pas en clair** : c'est un HMAC-SHA256 tronqué à 8 octets, calculé avec une clé aléatoire propre à votre installation et rangée dans le coffre chiffré (`audit.rs:74-88`) ;
- **les erreurs sont réduites à cinq catégories** avant écriture — configuration invalide, limite atteinte, authentification échouée, canal indisponible, opération échouée (`audit.rs:38-56`, `:124-128`) — donc aucun détail interne ne s'y retrouve.

Le journal est conservé **30 jours par défaut**, réglable entre 1 et 365 jours (`audit.rs:12`, `:69-72`), et il peut être désactivé (`audit.rs:69-71`, `:91-93`).

### 9. Quelles connexions Beaver ouvre-t-il tout seul ?

**C'est la section que cherche l'utilisateur soucieux de sa vie privée. Elle doit être exhaustive, chiffrée et vérifiable.** Tableau complet en section Tableaux ; voici ce qu'il faut en dire en prose.

Beaver ouvre **six connexions sans que vous ne lui demandiez rien**. Quatre sont une vérification de mise à jour, la cinquième est un catalogue de modèles, et la sixième — la moins évidente, et celle qui mérite le plus d'attention — est l'aperçu des liens.

**Au démarrage de l'application, puis toutes les heures**, un même bloc de vérifications part ensemble (`src/hooks/use-update-checker.ts:13` pour l'intervalle d'une heure, `:110-112` pour le déclenchement au lancement et la répétition, `:78-84` pour la liste). Ce bloc tourne dès l'ouverture de la fenêtre, parce qu'il est monté à la racine de l'application (`src/App.tsx:108`).

1. **Nouvelle version de Beaver** → `api.github.com`, dépôt `Kevin-hDev/Beaver` (`commands/app_update_source.rs:15`, `:29-33`, `:43-49`). Si une version plus récente existe, Beaver va aussi lire le manifeste sur `github.com` et les notes de version sur `raw.githubusercontent.com` (`app_update_source.rs:16-17`, `:63-83` ; `commands/app_update.rs:42-55`). **Ce qui part : l'en-tête `User-Agent`, qui vaut `Beaver/<version>`** (`app_update_source.rs:154-156` ; `services/brand.rs:8-10`). Rien d'autre.
2. **Nouvelle version du moteur Ollama** → `github.com/ollama/ollama/releases/latest` (`ollama_manager/release_fetch.rs:10`, `:13` ; `commands/ollama_version.rs:41-54`). Le résultat est mis en cache localement dans `ollama-release-cache.json` (`release_cache.rs:16-25`).
3. **Nouvelles versions de vos modèles Ollama** → `ollama.com` (`agent_local/ollama_registry_details.rs:8`, `:32-35`). **C'est la seule vérification automatique qui envoie une information vous concernant** : Beaver liste vos modèles installés, en extrait les noms de familles — la partie avant le `:` — et interroge la page publique de chacune, **jusqu'à 100 familles** (`commands/ollama_updates.rs:19-40`). Autrement dit, `ollama.com` apprend quels modèles sont installés chez vous. Aucun compte, aucune clé, aucun identifiant n'accompagne ces requêtes : seul l'en-tête `User-Agent` `Beaver/<version>` est envoyé (`ollama_registry_details.rs:37-45`). **À écrire noir sur blanc : c'est le point qu'un utilisateur soucieux de sa vie privée voudra connaître.**
4. **Catalogue public des modèles et de leurs tarifs** → `raw.githubusercontent.com`, un fichier du projet LiteLLM (`llm/litellm_catalog_refresh.rs:8-9`). Lancé une fois au démarrage, en tâche de fond (`runtime_startup.rs:92-97`, appelé par `app_build.rs:214` ; `llm/litellm_catalog.rs:87-90`). La requête est **conditionnelle** : elle envoie la date du fichier déjà en cache et ne rapatrie rien s'il n'a pas changé (`litellm_catalog_refresh.rs:41`, `:50`). Le résultat est écrit dans `litellm-models.json` (`:11-13`). Le code refuse toute réponse venant d'un autre hôte que `raw.githubusercontent.com` (`litellm_catalog.rs:92-93`, appliqué en `litellm_catalog_refresh.rs:56`). **Ce qui part : la date du fichier en cache. Rien de personnel.** Un catalogue de secours est de toute façon inclus dans l'application (`litellm_catalog.rs:8`, `:80-81`).
5. **L'aperçu des liens** → **n'importe quel site**, dès qu'un lien apparaît dans une conversation. **C'est la connexion automatique la plus facile à ne pas voir, et elle mérite un encadré.** Quand un message affiché contient une adresse web, Beaver contacte ce site pour en récupérer le titre, la description et l'image d'aperçu — **sans que vous ayez cliqué sur quoi que ce soit** (`src/components/agent-local/chat-markdown.tsx:78` monte une carte d'aperçu par lien ; `link-preview-card.tsx:33-41` lance la requête à l'affichage). Trois conséquences à écrire :
   - **cela vaut pour les liens que le modèle produit**, pas seulement pour ceux que vous écrivez. Un modèle qui cite une adresse fait contacter cette adresse par votre machine ;
   - le site contacté voit votre **adresse IP** et sait qu'un lien vers lui a été affiché chez vous ;
   - **c'est activé par défaut** (`src-tauri/src/models/config.rs:33`, `:54`), et **cela se désactive** dans les réglages avancés — la commande refuse alors toute requête (`commands/link_preview.rs:6-9`). Le résultat est mis en cache pour ne pas redemander le même lien (`link-preview-card.tsx:34-36`).
   Cas particulier : un lien YouTube est traité par une route dédiée qui contacte `youtube.com` et `img.youtube.com` (`services/link_preview/providers.rs:21`, `:48`, `:57`). Beaver refuse par ailleurs de contacter une adresse privée ou locale par ce chemin (`link_preview/security.rs:55-72`, liste des hôtes bloqués ; `:73` et suivantes, vérification de l'adresse IP résolue).
6. **Démarrage du moteur de recherche local**, si SearXNG est installé (`app_build.rs:153` ; `searxng/lifecycle.rs:49-63`). **Ce n'est pas une connexion sortante** : le processus est lancé et interrogé sur la boucle locale uniquement. Le mentionner justement pour dire qu'il ne compte pas.

**Deux cas intermédiaires, à ne pas ranger dans la liste précédente mais à ne pas taire non plus :**

- **L'usage et le solde de vos comptes fournisseurs.** Rien n'est demandé au démarrage, mais **ouvrir l'écran des fournisseurs** déclenche une requête vers chaque fournisseur configuré, sans clic supplémentaire (`src/hooks/use-provider-usage.ts:32-38`). Destinations : `chatgpt.com/backend-api/wham/usage`, `api.deepseek.com/user/balance`, `api.moonshot.ai/v1/users/me/balance`, `openrouter.ai/api/v1/key` et `/credits` (`services/provider_usage/remote.rs:7`, `:57-58`, `:72`, `:91`). Ce sont des services auxquels vous avez déjà confié une clé : l'enjeu est mince, mais l'affirmation « rien ne part sans un clic » serait fausse sans cette nuance.
- **Les canaux de messagerie interrogent en continu.** Un canal Telegram actif interroge `api.telegram.org` en boucle pour recevoir les messages ; un canal Discord maintient une connexion permanente. Cela ne démarre qu'avec les deux réglages de la section 10, mais une fois démarré, c'est du trafic continu, pas ponctuel.

**Une septième vérification existe dans le code mais ne s'exécute jamais dans l'application livrée** : la mise à jour des composants de prévision est conditionnée à `import.meta.env.DEV`, c'est-à-dire au mode développement (`src/hooks/use-forecast-dev-updates.ts:19` et `:28-33`). Le signaler évite qu'un lecteur qui parcourt le code croie à une omission.

### 10. Les connexions qui n'ont lieu que si vous le demandez

Par symétrie, et pour que la section 9 soit crédible, la page liste ce qui ne part **jamais** sans un geste de votre part. Tableau complet en section Tableaux. Les points saillants :

- **le moteur Ollama n'est pas téléchargé au démarrage.** Il l'est quand vous cliquez sur le bouton de l'écran d'installation (`src/components/ollama/ollama-setup-screen.tsx:49` et `:138`, qui appelle la commande `download_ollama` de `commands/ollama_setup.rs:39-40`). L'archive vient de `github.com/ollama/ollama/releases/…` et le nom de fichier est vérifié contre une liste fermée de quatre archives autorisées (`ollama_manager/release_source.rs:11`, `:33-51`, `:170-182` ; `release_fetch.rs:106`) ;
- **aucun modèle n'est téléchargé tout seul** ;
- **aucune requête n'est envoyée à un fournisseur cloud tant qu'une conversation n'utilise pas ce fournisseur.** Configurer une clé API n'envoie rien — sauf si vous cliquez sur le bouton de test, qui fait un appel au fournisseur pour vérifier la clé (`commands/api_keys.rs:100`, `:105`) ;
- **les canaux Telegram, Slack et Discord ne démarrent au lancement que si vous avez coché les deux réglages** correspondants : le gateway activé **et** le démarrage avec l'application (`app_build.rs:192-193`).

### 11. Les identifiants transmis : la règle et son exception

**La règle.** Beaver ne fabrique aucun identifiant d'installation et n'en envoie à personne. La seule signature qu'il ajoute à ses propres requêtes est l'en-tête `User-Agent`, qui vaut `Beaver/<version>` — le nom du produit et son numéro de version, rien d'autre (`services/brand.rs:3`, `:8-10`).

**L'exception, unique, et qui doit figurer sur la page.** La connexion par **compte web Kimi (Moonshot)** transmet une identité de machine, parce que le protocole de ce fournisseur l'exige. Beaver envoie alors, en en-têtes (`llm_oauth/headers.rs:103-109`) :

- un **identifiant stable** tiré au hasard au premier usage et conservé dans `~/.local/share/cl-go-dash/oauth-providers/kimi-device-id` (`headers.rs:54-65`) ;
- le **nom de votre machine** tel que le système le donne (`headers.rs:86-88`) ;
- la version de votre système et son architecture (`headers.rs:90-101`) ;
- le nom et la version de Beaver.

Ces en-têtes ne sont ajoutés **que** pour ce fournisseur : le code les conditionne explicitement (`headers.rs:16-20`). Aucune autre connexion de Beaver ne les transporte.

**Une seconde exception, plus légère.** La connexion par compte web xAI transmet l'identifiant de votre compte xAI (`llm_oauth/xai_headers.rs:13-15`) et une signature de client — nom et version de Beaver (`xai_headers.rs:5-11`). C'est l'identité de votre compte chez eux, pas celle de votre machine : aucun nom d'ordinateur n'y figure.

Rappel du rappel : le **compte web Kimi est marqué expérimental** dans le code (voir `01-decouverte/local-vs-cloud.md`).

### 12. Télémétrie, statistiques d'usage, rapports de plantage

**Verdict : Beaver n'embarque aucun outil de télémétrie, d'analyse d'usage ou de rapport de plantage.** Aucune statistique, aucun identifiant d'installation, aucune trace de plantage n'est envoyée à qui que ce soit.

**Ce verdict est une absence, donc il se démontre par une méthode. La page publie la méthode.** C'est ce qui distingue une affirmation d'un argument commercial.

**La recherche menée le 9 septembre 2026 sur la version 1.2.2 :**

1. **Recherche par mot-clé, insensible à la casse, dans tout le code source** — `src-tauri/src/` (backend Rust) et `src/` (interface) — sur les motifs : `sentry`, `telemetry`, `analytics`, `posthog`, `crashlytics`, `mixpanel`, `amplitude`, `datadog`, `bugsnag`, `rollbar`, `opentelemetry`, `gtag`, `matomo`, `plausible`, `umami`, `google-analytics`, `segment.io`, `tracking`, `beacon`.
2. **Recherche dans les déclarations de dépendances** — `src-tauri/Cargo.toml` pour les bibliothèques Rust, `package.json` pour les bibliothèques JavaScript — sur les mêmes motifs. **Zéro résultat dans les deux fichiers.** C'est le contrôle décisif : aucun outil de ce genre ne peut fonctionner sans être déclaré comme dépendance.
3. **Examen un par un de toutes les correspondances textuelles**, pour écarter les faux positifs.

**Le résultat de l'examen, à publier tel quel** — parce que c'est ce qui rend la démonstration vérifiable par un lecteur méfiant :

| Motif trouvé | Ce que c'est en réalité |
|---|---|
| `sentry` | La chaîne `PROCESSENTRY32W`, une structure de l'API Windows (`services/process_tree_windows.rs:3`), **et** le connecteur externe Sentry proposé au catalogue MCP (`mcp_bridge/trusted.rs:58`), que vous configurez vous-même s'il vous sert. Ce n'est pas du rapport de plantage Beaver. |
| `analytics` | Le module de prévision : `AdvancedAnalytics` désigne des calculs statistiques faits **localement** sur vos propres séries de données (`services/forecast/advanced/types.rs:14`). |
| `datadog` | **Datadog Toto 2.0**, un modèle de prévision de séries temporelles proposé au catalogue (`forecast/catalog_specs/providers.rs:29-30`). Un nom de modèle, pas un service de supervision. |
| `rollbar` | La sous-chaîne de **`scrollbar`**, dans les feuilles de style. |
| `amplitude` | Le mot « amplitude » au sens de l'analyse de signal, dans les textes du module de prévision. |
| `gtag` | La sous-chaîne de `HeadingTag`, un composant d'affichage de titres (`components/file-preview/document-preview.tsx:59`). |
| `plausible` | L'adjectif « plausible » dans des commentaires et des noms de tests. |
| `telemetry` | **Trois occurrences, et elles vont dans le bon sens** : Beaver désactive la télémétrie des composants tiers qu'il lance, en leur imposant `TABPFN_DISABLE_TELEMETRY=1` (`forecast/sidecar_spawn.rs:82`, `forecast/model_manager/smoke.rs:64`) et `HF_HUB_DISABLE_TELEMETRY=1` (`forecast/sidecar_process_env.rs:31`). |
| `posthog`, `crashlytics`, `mixpanel`, `bugsnag`, `opentelemetry`, `matomo`, `umami`, `google-analytics`, `segment.io` | **Aucune occurrence.** |

**Le point le plus intéressant à faire ressortir** : la seule chose que Beaver fasse en matière de télémétrie, c'est **l'éteindre** chez les composants tiers qu'il exécute pour vous.

**Ce que la méthode ne prouve pas, et qu'il faut dire** : elle porte sur le code de Beaver. Elle ne dit rien de ce que font les services que vous configurez vous-même, ni du code d'une extension que vous approuvez.

### 13. Où vivent vos données sur le disque

**Un seul dossier, identique sur les trois systèmes** : `~/.local/share/cl-go-dash/` (`services/paths.rs:10-13`). C'est vrai sur macOS, sur Linux et sur Windows — Beaver n'utilise pas les emplacements habituels de chaque système, il en impose un unique.

Le détail des fichiers est dans `12-reference/fichiers-de-donnees.md` ; cette page-ci ne reprend que ce qui touche à la confidentialité :

| Contenu | Emplacement | Source |
|---|---|---|
| Conversations | `agent-sessions/` | `private_store.rs:98` |
| Clés API, chiffrées | `secrets.enc` | `vault.rs:41-42` |
| Journaux d'application | `logs/beaver.log` (rotation) | `app_log.rs:82-89` |
| Journal d'audit du gateway | `logs/gateway-audit.jsonl` | `gateway/security/audit.rs:143-145` |
| Sorties d'outils trop longues | `tool-results/<conversation>/` | `agent_local/tool_result_truncate.rs:84` |
| Plans du mode Plan | `plans/` | `agent_local/tool_plan_storage.rs:14` |
| Identifiant machine Kimi | `oauth-providers/kimi-device-id` | `llm_oauth/headers.rs:55` |
| Catalogue de modèles en cache | `litellm-models.json` | `llm/litellm_catalog_refresh.rs:11-13` |
| Moteur Ollama | `ollama-bundle/` | `paths/ollama.rs:38` |

**Les protections d'accès sur le disque**, vérifiées :

- sur macOS et Linux, les fichiers créés par Beaver le sont en **`0600`** — lisibles par votre seul compte — et les dossiers en **`0700`** (`private_store.rs:141-149`, `:169-174`) ;
- sur Windows, une liste de contrôle d'accès équivalente est posée (`private_store.rs:176-179`) ;
- à chaque démarrage, Beaver **restaure** ces droits sur le dossier des conversations, celui des notes de prévision, celui des journaux, ainsi que sur le coffre, le fichier des connecteurs, le fichier d'usage des fournisseurs et le journal d'audit (`private_store.rs:91-113`).

**La limite à écrire** : ces droits protègent vos données des **autres comptes** de l'ordinateur. Ils ne les protègent pas d'un programme lancé sous **votre** compte — y compris une extension que vous avez approuvée.

**Le point qui compte pour l'effacement, et qui surprend :** les **modèles Ollama téléchargés ne sont pas dans ce dossier**. Ils vont dans `~/.ollama/models`, sauf si la variable `OLLAMA_MODELS` en décide autrement (`ollama_manager/spawn_profile_paths.rs:17-33`). Ils pèsent souvent plusieurs dizaines de gigaoctets. Voir section 15.

### 14. Ce qui est écrit dans les traces — et ce qui en est retiré

Beaver écrit un journal technique dans `~/.local/share/cl-go-dash/logs/`, sous le nom `beaver` (`app_log.rs:82-89`). Il **reste sur votre machine** : rien n'est envoyé nulle part.

Les garde-fous, tous vérifiés :

- **chaque ligne passe par un caviardage** avant d'être écrite (`app_log.rs:44-50`) ;
- **chaque ligne est bornée à 2 048 caractères** (`app_log.rs:8`, `:50`) ;
- **le fichier est borné à 2 Mio** et **4 fichiers** sont conservés en rotation (`app_log.rs:10-11`, `:93-94`) ;
- **le corps des réponses des fournisseurs n'est jamais écrit brut** : il passe par une fonction qui caviarde les secrets, remplace les caractères de contrôle et **tronque à 200 caractères** (`services/llm/mod.rs:146-152`) ;
- **les messages d'erreur des moteurs de recherche** sont caviardés et tronqués à 240 caractères (`search/common.rs:77`).

**Vos clés API ne sont jamais écrites dans les traces**, et l'interface ne peut de toute façon pas les lire : les commandes exposées à l'interface sont `set_api_key`, `delete_api_key`, `has_api_key`, `list_configured_providers`, `get_provider_connection`, `test_api_key` et `test_api_key_with_value` (`commands/api_keys.rs:15`, `:62`, `:75`, `:80`, `:85`, `:100`, `:105`). **Aucune ne renvoie la valeur d'une clé.** Côté Rust, la lecture d'une clé passe par une fonction qui rend une valeur effacée de la mémoire dès qu'elle n'est plus utilisée (`api_keys.rs:122`). Le détail est dans `11-securite/vault-et-cles-api.md`.

**Un défaut à ne pas cacher** : le journal d'application n'a **aucun bouton d'effacement dans l'interface**. Il s'efface en supprimant le dossier `logs/`. Voir « Points à confirmer ».

### 15. Comment tout effacer

**La procédure est manuelle. Beaver ne propose aucune commande « tout effacer », et la page doit le dire au lieu de laisser croire le contraire.**

Vérifié : aucune commande exposée à l'interface ne supprime l'ensemble des données. Les suppressions disponibles sont partielles et ciblées — une clé API (`commands/api_keys.rs:62`), une conversation (`commands/agent_sessions.rs:170`), un projet (`commands/projects.rs:26`), un modèle Ollama (`commands/agent_ollama.rs:80`), un réveil programmé (`commands/heartbeat.rs:101`), un jeton de connecteur (`commands/mcp_oauth.rs:39`), une note ou une analyse de prévision. **Aucune ne couvre l'ensemble.**

La procédure complète comporte **quatre étapes**, et **oublier la troisième ou la quatrième laisse des données derrière soi** :

1. **Désinstaller l'application**, selon la méthode de votre système. Cela retire le programme, pas vos données.
2. **Supprimer le dossier de données** : `~/.local/share/cl-go-dash/`. Il contient les conversations, le coffre chiffré, les journaux, les plans, les sorties d'outils, les réglages, le moteur Ollama téléchargé par Beaver et l'identifiant machine Kimi. **C'est l'étape qui efface le plus.**
3. **Supprimer l'entrée du trousseau du système.** La clé qui déchiffre le coffre n'est pas dans le dossier de données : elle est rangée dans le trousseau de votre système, sous le service **`cl-go-dash`** et le compte **`master-key`** (`vault.rs:16`, `:18`, `:50-52`). Supprimer le dossier sans supprimer cette entrée laisse une clé orpheline dans votre trousseau. La procédure exacte, système par système, est à écrire pour le site (voir « Points à confirmer »).
4. **Supprimer les modèles Ollama**, s'il y en a, dans `~/.ollama/models` (`ollama_manager/spawn_profile_paths.rs:33`). Ils ne sont **pas** dans le dossier de l'étape 2 et représentent souvent l'essentiel de l'espace occupé.

**Deux mises en garde à placer en encadré :**

- **Ce que l'effacement ne peut pas rattraper.** Ce qui est déjà parti chez un fournisseur cloud, chez un moteur de recherche ou sur une messagerie **n'est pas récupérable en effaçant votre disque**. La seule action utile de ce côté est de **révoquer vos clés API** chez chaque fournisseur, depuis son propre site, et de supprimer les données que ce fournisseur vous permet de supprimer, selon ses règles.
- **Le trousseau peut contenir des entrées anciennes.** Les versions plus anciennes de Beaver rangeaient chaque clé directement dans le trousseau, sous le même service `cl-go-dash` et sous le nom du fournisseur — par exemple `brave`, `openai`, `mistral` (`vault.rs:157-176`). Si vous utilisez Beaver depuis longtemps, vérifiez ces entrées-là aussi.

---

## Tableaux

### Tableau 1 — Qui voit quoi, selon ce que vous utilisez

| Ce que vous utilisez | Ce qui sort de la machine | Qui le reçoit |
|---|---|---|
| Modèle Ollama local | Rien de la conversation | Personne |
| Modèle par clé API | Messages, pièces jointes, résultats d'outils, prompt système | Le fournisseur du modèle |
| Modèle par compte web | Idem, plus l'identité du compte (et de la machine pour Kimi) | Le fournisseur du modèle |
| Recherche Brave / Exa / Firecrawl | La requête de recherche, avec votre clé | Le fournisseur de recherche |
| Recherche SearXNG locale | La requête, vers les moteurs publics, sans compte | Les moteurs interrogés par SearXNG |
| Connecteur externe distant | Les arguments des outils que l'agent lui passe | L'éditeur du connecteur |
| Canal Telegram / Slack / Discord | Vos messages et les réponses de l'agent | La plateforme concernée |
| Un lien affiché dans une conversation | Une requête d'aperçu, sans clic de votre part | Le site du lien |
| Extension approuvée | **Non déterminable par Beaver** | Non déterminable |
| Prévision par modèle local | Rien | Personne |
| Prévision par Nixtla TimeGPT | Vos séries de données | Nixtla (`forecast/client_nixtla.rs:11`) |

### Tableau 2 — Les connexions ouvertes sans action de votre part

| Ce que Beaver vérifie | Destination | Quand | Ce qui part | Source |
|---|---|---|---|---|
| Nouvelle version de Beaver | `api.github.com`, puis `github.com` et `raw.githubusercontent.com` si une version existe | Au lancement, puis **toutes les heures** | `User-Agent: Beaver/<version>` | `use-update-checker.ts:13`, `:79`, `:110-112` ; `app_update_source.rs:15-18`, `:154-156` |
| Nouvelle version du moteur Ollama | `github.com/ollama/ollama` | Au lancement, puis **toutes les heures** | `User-Agent: Beaver/<version>` | `use-update-checker.ts:81` ; `release_fetch.rs:10`, `:21` |
| Nouvelles versions de vos modèles Ollama | `ollama.com` | Au lancement, toutes les heures, et à chaque changement de votre liste de modèles | **Le nom des familles de modèles installées chez vous**, jusqu'à 100 | `use-update-checker.ts:80`, `:113-115` ; `ollama_updates.rs:19-40` ; `ollama_registry_details.rs:8`, `:32-35` |
| Catalogue public des modèles et tarifs | `raw.githubusercontent.com` (projet LiteLLM) | **Une fois** au démarrage | La date du fichier déjà en cache | `litellm_catalog_refresh.rs:8-9`, `:37-46` ; `runtime_startup.rs:92-97` ; `app_build.rs:214` |
| **Aperçu d'un lien** | **N'importe quel site**, celui du lien | **Dès qu'un lien s'affiche dans une conversation**, y compris un lien écrit par le modèle. Activé par défaut, désactivable | Une requête depuis votre adresse IP vers ce site | `chat-markdown.tsx:78` ; `link-preview-card.tsx:33-41` ; `commands/link_preview.rs:6-9` ; `models/config.rs:54` |
| Usage et solde de vos comptes fournisseurs | `chatgpt.com`, `api.deepseek.com`, `api.moonshot.ai`, `openrouter.ai` | **À l'ouverture de l'écran des fournisseurs**, sans clic supplémentaire | Votre clé, pour lire votre propre consommation | `use-provider-usage.ts:32-38` ; `provider_usage/remote.rs:7`, `:57-58`, `:72`, `:91` |
| Canaux Telegram / Slack / Discord | Les plateformes concernées | Au lancement, **uniquement** si le gateway est activé **et** réglé pour démarrer avec l'application. Ensuite : trafic **continu** (interrogation en boucle Telegram, connexion permanente Discord) | Selon le canal | `app_build.rs:192-193` ; `gateway/channels/telegram.rs:41` ; `discord_http.rs:30` |

### Tableau 3 — Les connexions qui exigent une action de votre part

| Action | Destination | Source |
|---|---|---|
| Télécharger le moteur Ollama | `github.com/ollama/ollama/releases/…` | `ollama-setup-screen.tsx:138` → `commands/ollama_setup.rs:39-40` ; `release_fetch.rs:106` |
| Télécharger un modèle Ollama | Registre Ollama | — |
| Envoyer un message à un modèle cloud | Le fournisseur choisi | `services/llm/route_profile/catalog_api.rs` |
| Tester une clé API | Le fournisseur concerné | `commands/api_keys.rs:100`, `:105` |
| Lancer une recherche web | Brave, Exa, Firecrawl, ou les moteurs derrière SearXNG | `search/mod.rs:73-77`, `:92-98` |
| Demander à l'agent d'ouvrir une page web | L'adresse indiquée — l'outil `web_fetch` va la chercher | `agent_local/tool_dispatcher.rs:102` ; `agent_local/tool_web_fetch.rs:16-20` |
| Naviguer dans le navigateur intégré | Les sites que vous visitez | `services/browser/` |
| Installer un modèle de prévision | `huggingface.co` | `forecast/model_manager/download.rs:76-91` |
| Consulter la fiche d'un modèle de prévision | `huggingface.co`, `api.github.com` | `forecast/model_details_huggingface.rs:14-28` ; `forecast/model_details_github.rs:17` |
| Utiliser Nixtla TimeGPT | `api.nixtla.io` | `forecast/client_nixtla.rs:11` |
| Installer une extension depuis Git ou npm | Le dépôt ou le registre indiqué | Voir `07-integrations/extensions-centre.md` |
| Activer un connecteur externe distant | L'adresse du connecteur | `mcp_bridge/trusted.rs:50-66` |
| Télécharger une mise à jour de Beaver | `github.com`, redirection autorisée vers `release-assets.githubusercontent.com` | `app_update_source.rs:16`, `:18`, `:180-193` |

### Tableau 4 — La procédure d'effacement complet

| # | Étape | Ce qui disparaît | Ce qui reste si on l'oublie |
|---|---|---|---|
| 1 | Désinstaller l'application | Le programme | Toutes vos données |
| 2 | Supprimer `~/.local/share/cl-go-dash/` | Conversations, coffre chiffré, journaux, plans, réglages, moteur Ollama, identifiant Kimi | — |
| 3 | Supprimer l'entrée de trousseau `cl-go-dash` / `master-key` | La clé de déchiffrement du coffre | Une clé orpheline dans le trousseau |
| 4 | Supprimer `~/.ollama/models` | Les modèles téléchargés | Souvent plusieurs dizaines de gigaoctets |
| — | Révoquer vos clés chez chaque fournisseur | L'accès à votre compte depuis Beaver | Une clé toujours valable, connue de qui a lu votre disque |

---

## Encadrés

Ces encadrés vont dans le corps du texte, à l'endroit indiqué.

> **⚠ À placer en section 3 — Ce que l'agent lit, le fournisseur le lit aussi.**
> Avec un modèle cloud, ce ne sont pas seulement vos messages qui partent : le contenu de chaque fichier lu par l'agent, la sortie de chaque commande, chaque pièce jointe entrent dans la conversation et repartent **à chaque tour**. Si vous ouvrez à l'agent un dossier qui contient des données confidentielles, ces données partent chez le fournisseur du modèle. Pour ce genre de travail, utilisez un modèle local.

> **⚠ À placer en section 6 — « SearXNG est local » demande une précision.**
> Le moteur s'exécute sur votre machine et n'utilise aucune clé API : vos recherches ne sont donc rattachées à aucun compte. **Mais SearXNG interroge des moteurs publics pour vous** : la requête sort de la machine, depuis votre adresse IP. Ce que SearXNG vous apporte, c'est l'absence de compte et la dispersion des requêtes — pas leur confinement.

> **⚠ À placer en section 9 — Un lien affiché est un lien visité.**
> Dès qu'une adresse web apparaît dans une conversation, Beaver contacte ce site pour en afficher un aperçu — **sans que vous cliquiez**. Cela vaut aussi pour les adresses citées par le modèle : le site voit alors votre adresse IP et sait qu'un lien vers lui a été affiché chez vous. C'est **activé par défaut** et se désactive dans les réglages avancés. Si vous travaillez sur des sujets que vous ne voulez signaler à personne, coupez l'aperçu des liens.

> **ℹ À placer en section 9 — La seule vérification automatique qui parle de vous.**
> Toutes les heures, Beaver demande à `ollama.com` s'il existe une version plus récente de vos modèles. Pour cela, il envoie le **nom des familles de modèles installées chez vous**. Aucun compte, aucune clé, aucun identifiant n'accompagne cette requête — mais `ollama.com` apprend quels modèles vous utilisez. Les quatre autres vérifications automatiques n'envoient que le nom et la version de Beaver.

> **ℹ À placer en section 12 — Une absence, et comment on l'a vérifiée.**
> Beaver n'embarque aucun outil de télémétrie, d'analyse d'usage ni de rapport de plantage. Ce n'est pas une promesse : c'est le résultat d'une recherche par mot-clé menée dans l'intégralité du code et, surtout, dans les fichiers de dépendances — `Cargo.toml` et `package.json` — où un tel outil devrait obligatoirement être déclaré. Ils n'en contiennent aucun. La liste des motifs cherchés et des faux positifs écartés est publiée ci-dessus, pour que vous puissiez refaire la vérification.

> **⚠ À placer en section 11 — Le compte web Kimi transmet le nom de votre machine.**
> Le protocole de ce fournisseur exige une identité d'appareil. Beaver lui envoie donc un identifiant tiré au hasard et conservé chez vous, le nom de votre ordinateur, la version de votre système et son architecture. **C'est la seule connexion de Beaver dans ce cas.** Si cela vous gêne, utilisez la clé API Moonshot plutôt que le compte web.

> **⚠ À placer en section 15 — Supprimer le dossier ne suffit pas.**
> Trois choses vivent hors du dossier de données : la clé qui déchiffre votre coffre, rangée dans le **trousseau du système** ; les **modèles Ollama**, dans `~/.ollama/models` ; et ce que vous avez déjà envoyé à des services tiers, qui n'est pas sur votre disque. Les deux premières se suppriment à la main. La troisième se traite en **révoquant vos clés** chez chaque fournisseur.

> **ℹ À placer en section 5 — Le filet à secrets attrape des formats, pas des sens.**
> Beaver retire automatiquement les jetons et les clés qu'il reconnaît du résultat des outils avant que le modèle ne les voie. Il reconnaît des **formats techniques**. Un mot de passe ordinaire, un nom, un numéro de dossier ne ressemblent à aucun format connu : ils ne sont pas retirés. Ce filet complète votre vigilance, il ne la remplace pas.

---

## Pièges et erreurs fréquentes

| Ce que l'utilisateur croit | Ce qui est vrai | Où le lui dire |
|---|---|---|
| « J'utilise un modèle local, donc rien ne sort jamais » | La conversation ne sort pas. La vérification horaire des modèles envoie le nom de vos familles de modèles à `ollama.com`, et un outil de recherche web sort par définition | Sections 4 et 9 |
| « SearXNG est local, donc mes recherches restent chez moi » | Elles sortent vers les moteurs publics, mais sans compte ni clé | Section 6 |
| « Je n'envoie que mes messages au fournisseur » | Chaque fichier lu, chaque sortie de commande et chaque pièce jointe partent aussi, et repartent à chaque tour | Section 3 |
| « Configurer une clé API envoie des données tout de suite » | Non. Rien ne part avant qu'une conversation n'utilise ce fournisseur — sauf si vous cliquez sur le bouton de test, ou si vous ouvrez l'écran des fournisseurs, qui lit votre solde | Sections 9 et 10 |
| « Beaver ne contacte un site que si je clique sur son lien » | Non : un lien **affiché** dans une conversation est contacté automatiquement pour en faire l'aperçu, y compris un lien produit par le modèle | Section 9 |
| « Beaver télécharge Ollama tout seul au premier lancement » | Non : c'est un bouton sur l'écran d'installation | Section 10 |
| « J'ai désinstallé, donc tout est effacé » | La désinstallation ne touche ni le dossier de données, ni le trousseau, ni les modèles Ollama | Section 15 |
| « Supprimer le dossier de données efface tout » | La clé du coffre reste dans le trousseau, les modèles dans `~/.ollama/models` | Section 15 |
| « Supprimer ma clé API dans Beaver la met hors d'usage » | Non : elle reste valable chez le fournisseur. Seule la révocation chez lui la neutralise | Section 15 |
| « J'ai lu "sentry" dans le code, il y a du rapport de plantage » | Ce sont une structure de l'API Windows et le connecteur MCP Sentry, que vous configurez vous-même | Section 12 |
| « Le journal d'audit du gateway contient mes messages » | Il contient sept champs techniques et une empreinte de l'identifiant utilisateur. Jamais le texte | Section 8 |
| « Beaver caviarde tout ce qui est sensible avant l'envoi » | Il caviarde les **formats de secret connus**, dans le résultat de **six outils** nommés. Pas les données personnelles, pas les résultats des connecteurs ni des extensions | Section 5 |
| « Mes données sont protégées des autres logiciels » | Les droits `0600`/`0700` protègent des autres comptes de l'ordinateur, pas d'un programme lancé sous le vôtre | Section 13 |

---

## Renvois

- `11-securite/modele-de-securite.md` — la vue d'ensemble : les couches de protection, les permissions de l'agent, le bac à sable et sa condition d'activation
- `11-securite/vault-et-cles-api.md` — le chiffrement du coffre, la clé maîtresse, ce que l'interface ne peut pas lire
- `01-decouverte/local-vs-cloud.md` — le choix entre modèle local, clé API et compte web
- `05-outils/recherche-web.md` — le paramétrage des quatre chemins de recherche
- `07-integrations/extensions-centre.md` — pourquoi une extension approuvée sort du périmètre de cette page
- `07-integrations/mcp-connecteurs.md` — les connecteurs externes et ce qu'ils reçoivent
- `07-integrations/gateway-canaux.md` — Telegram, Slack, Discord et le journal d'audit
- `10-reglages/avance.md` — le réglage qui coupe l'aperçu des liens, et les autres options avancées
- `12-reference/fichiers-de-donnees.md` — la liste complète des fichiers du dossier de données
- `02-installation/mise-a-jour.md` — le mécanisme de mise à jour, côté procédure

---

## Points à confirmer

**Écarts relevés avec le fichier voisin `modele-de-securite.md` — à arbitrer avant publication**

1. **Sa liste des requêtes automatiques est incomplète.** Son tableau « Les requêtes que Beaver fait de lui-même » en donne trois : mise à jour de l'application, catalogue LiteLLM, téléchargement d'Ollama. Il manque **la vérification des versions de vos modèles Ollama** (`ollama.com`, toutes les heures, avec le nom de vos familles de modèles), **la vérification de version du moteur Ollama** (`github.com`, toutes les heures) et surtout **l'aperçu des liens**, qui contacte des sites arbitraires sans clic. Il manque aussi la **périodicité horaire**, qui n'apparaît nulle part. Cette page-ci donne la liste que je crois complète ; les deux fichiers doivent être alignés, et je recommande d'aligner `modele-de-securite.md` sur celui-ci.
2. **Il présente le téléchargement d'Ollama comme automatique** (« au premier lancement »). Il est déclenché par un **bouton** de l'écran d'installation (`ollama-setup-screen.tsx:138`). L'expression « au premier lancement » décrit le moment où l'écran apparaît, pas un téléchargement automatique. À corriger dans les deux fichiers avec la même formulation.
3. **Il cite `services/llm/mod.rs:130-137` pour la fonction de caviardage des traces.** Au 9 septembre 2026 elle se trouve en **`:146-152`**. Le fait est inchangé, la référence a glissé. Signaler que les références de ligne des deux fichiers doivent être revérifiées ensemble avant publication, et non l'une après l'autre.

**Éléments à établir avant de pouvoir publier la page**

4. **La procédure exacte de suppression de l'entrée du trousseau, pour les trois systèmes.** Le code donne le service (`cl-go-dash`) et le compte (`master-key`), pas la manipulation utilisateur. Il faut écrire, tester et capturer : le Trousseau d'accès sur macOS, le gestionnaire d'identifiants sur Windows, et le cas Linux — où le trousseau dépend de l'environnement de bureau installé, ce qui n'est pas uniforme. **Sans cette procédure, l'étape 3 de la section 15 est inapplicable pour un utilisateur non technique**, et c'est la principale lacune de cette page.
5. **La liste des moteurs interrogés par SearXNG.** La configuration de Beaver active les réglages par défaut de SearXNG (`settings.template.yml:1`) sans nommer de moteur. La liste vient donc de SearXNG lui-même, à la version embarquée. Décider : publier la liste (et la maintenir), ou renvoyer à la documentation de SearXNG. **Recommandation : renvoyer**, une liste de moteurs recopiée sera périmée à la première mise à jour du composant.
6. **Le comportement des fournisseurs cloud en matière de conservation et d'entraînement.** Volontairement absent de cette page. À décider pour le site : un tableau de liens vers les politiques de confidentialité des quatorze fournisseurs, ou une simple phrase de renvoi. **Recommandation : un tableau de liens seulement, sans une ligne de résumé** — résumer une politique juridique tierce engage Beaver sur des faits qu'il ne contrôle pas et qui changent.
7. **Y a-t-il un moyen d'effacer le journal d'application depuis l'interface ?** Je n'en ai trouvé aucun : les commandes de suppression exposées sont toutes ciblées (clé, conversation, projet, modèle, réveil, note, analyse). À confirmer avec l'équipe, et si l'absence est confirmée, décider si la page la présente comme une limite ou si un bouton est ajouté au produit.
8. **Faut-il un moyen de désactiver les vérifications automatiques ?** La page va révéler qu'un contact horaire avec `ollama.com`, `api.github.com` et `github.com` a lieu **sans réglage possible**, alors que l'aperçu des liens, lui, se désactive (`models/config.rs:33`). Cette asymétrie va se voir dès la publication, et la demande d'évolution qui suivra est prévisible : un réglage « ne rien vérifier automatiquement » à côté de celui de l'aperçu des liens. À trancher avant, pour que la page puisse soit décrire ce réglage, soit assumer franchement son absence.
9. **Le libellé exact du réglage d'aperçu des liens dans l'interface.** La page en fait une recommandation concrète (« coupez l'aperçu des liens ») : il faut le nom affiché et son emplacement exact dans les réglages avancés, relevés à l'écran, pas déduits du nom du champ de configuration (`models/config.rs:33`).
10. **La couverture du caviardage sur le chemin cloud.** Vérifié : le caviardage des résultats d'outils (`tool_hooks.rs:83-92`) s'applique aux deux chemins, puisque les deux passent par le même exécuteur d'outils (`llm/agent_loop_tools.rs:83`). En revanche, la passe de caviardage supplémentaire sur l'ensemble des messages (`session_security::sanitize_chat_messages`) n'est appelée que sur le chemin Ollama (`agent_loop_ollama_request.rs:54`) et **je n'en ai trouvé aucun équivalent sur le chemin cloud**. Cette page ne l'affirme donc nulle part. **À faire confirmer par l'équipe** : est-ce un choix — le chemin cloud étant couvert autrement — ou un manque ? La réponse change ce que la section 5 a le droit d'écrire.
11. **L'identifiant machine Kimi n'a pas de moyen de remise à zéro dans l'interface.** Il se supprime en effaçant le fichier `oauth-providers/kimi-device-id`. À confirmer, et à décider si la page l'indique.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

12. **L'écran d'installation d'Ollama** : vérifier à l'écran que le bouton de téléchargement est bien une action explicite et qu'aucun téléchargement ne démarre à l'affichage de l'écran. Le code le dit ; la page l'affirme ; une capture le prouverait.
13. **Le panneau des mises à jour** : vérifier ce que l'utilisateur voit quand les trois vérifications horaires trouvent quelque chose en même temps, et si l'origine de chaque vérification est lisible.
14. **Les réglages du gateway** : vérifier les libellés exacts des deux cases « activé » et « démarrer avec l'application », dont la page fait dépendre une affirmation de confidentialité. Les nommer avec les mots de l'écran, pas avec ceux du code.
15. **L'aperçu d'un lien, à l'écran** : vérifier à quoi ressemble la carte d'aperçu, ce qu'elle affiche quand le site ne répond pas, et si l'interface indique d'une façon ou d'une autre qu'une requête a été faite. Ce dernier point n'est pas cosmétique : c'est ce qui rend la connexion visible ou invisible à l'utilisateur.
