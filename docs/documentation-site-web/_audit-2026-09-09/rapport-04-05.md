# Audit de fraîcheur — sections 04-agent et 05-outils

**Date** — 9 septembre 2026
**Périmètre** — les 12 fichiers de `docs/documentation-site-web/04-agent/` et les 14 fichiers de `docs/documentation-site-web/05-outils/`, soit 26 briefs.
**Méthode** — chaque affirmation chiffrée, chaque nom d'outil, chaque libellé et chaque tableau de référence a été comparé au code actuel du dépôt. Aucun verdict n'est posé sans lecture du fichier cité.
**Règle appliquée** — le code fait foi.

Aucun brief n'a été modifié. Ce fichier est le seul écrit.

---

## Tableau récapitulatif

| Fichier | Verdict | Écarts | Points à confirmer tranchés / restants |
|---|---|---|---|
| `04-agent/contexte.md` | À JOUR | 0 | 2 / 4 |
| `04-agent/diagnostics-et-erreurs.md` | À JOUR | 0 | 1 / 5 |
| `04-agent/fonctionnement.md` | DÉCALÉ | 2 | 4 / 2 |
| `04-agent/memoire-persistante.md` | À JOUR | 0 | 1 / 6 |
| `04-agent/permissions.md` | DÉCALÉ | 4 | 3 / 3 |
| `04-agent/personnalite-et-agents-md.md` | DÉCALÉ | 2 | 3 / 3 |
| `04-agent/pieces-jointes.md` | À JOUR | 0 | 1 / 5 |
| `04-agent/prompts-systeme.md` | À JOUR | 0 | 1 / 5 |
| `04-agent/repertoire-de-travail.md` | À JOUR | 0 | 2 / 4 |
| `04-agent/skills-locaux.md` | DÉCALÉ | 1 | 2 / 4 |
| `04-agent/sous-agents.md` | DÉCALÉ | 2 | 3 / 4 |
| `04-agent/todos.md` | À JOUR | 0 | 1 / 4 |
| `05-outils/choix-interactif.md` | À JOUR | 0 | 0 / 4 |
| `05-outils/documents.md` | À JOUR | 0 | 2 / 2 |
| `05-outils/fichiers.md` | À JOUR | 0 | 0 / 4 |
| `05-outils/forecast-outils.md` | À JOUR | 0 | 0 / 5 |
| `05-outils/git.md` | À JOUR | 0 | 0 / 4 |
| `05-outils/images.md` | À JOUR | 0 | 0 / 4 |
| `05-outils/mcp.md` | À JOUR | 0 | 0 / 4 |
| `05-outils/recherche-fichiers.md` | À JOUR | 0 | 0 / 3 |
| `05-outils/skills-et-automatisations.md` | **OBSOLÈTE** | 6 | 1 / 3 |
| `05-outils/sous-agents-outils.md` | DÉCALÉ | 2 | 2 / 3 |
| `05-outils/tableurs.md` | À JOUR | 0 | 0 / 4 |
| `05-outils/terminal-et-shell.md` | À JOUR | 0 | 0 / 5 |
| `05-outils/vue-densemble.md` | DÉCALÉ | 2 | 2 / 3 |
| `05-outils/web.md` | À JOUR | 0 | 0 / 4 |

**26 fichiers audités. 18 à jour, 7 décalés, 1 obsolète.**

Le constat général est que les valeurs chiffrées ont extrêmement bien tenu : sur plusieurs centaines de nombres, de chemins et de noms d'outils vérifiés, un seul est faux (le compte des motifs shell sûrs). Le décalage réel porte sur trois mécanismes qui ont changé de nature depuis la rédaction : les automatisations, les outils d'extension, et le mode Chatbot.

---

# Les fichiers décalés ou obsolètes

## 1. `05-outils/skills-et-automatisations.md` — OBSOLÈTE (partie automatisations)

La partie « skills » du fichier est exacte. La partie « automatisations » décrit un mécanisme qui n'existe plus : le modèle de sécurité a été **inversé**.

### Écart 1 — Une automatisation ne fige plus de liste d'outils

> Brief : « l'automatisation fige […] **la liste exacte des outils** dont elle a besoin — au plus **12** ; **la liste exacte des skills** à charger — au plus **8** ».

**Réalité** : ces champs n'existent plus. La structure enregistrée sur le disque ne contient que `id`, `name`, `model`, `provider`, `prompt`, `schedule`, `description`, `project_id`, `active`, `paused_by_global`, `created_at` — `src-tauri/src/models/config.rs:134-149`. La définition de l'outil n'expose que `action`, `id`, `name`, `description`, `prompt`, `schedule`, `active`, `confirm` — `src-tauri/src/services/agent_local/tool_definitions_automation.rs:7-27`.

### Écart 2 — Une automatisation s'exécute désormais en accès complet, avec tous les outils activés

> Brief : « **une automatisation ne dispose que des outils qui lui ont été explicitement donnés**. […] c'est une tâche à portée réduite, décidée à l'avance. »

**Réalité, exactement l'inverse.** La description de l'outil l'énonce noir sur blanc : « Every automation runs through the complete Agent Local engine in **full-access mode, with all currently enabled tools and skills** » — `src-tauri/src/services/agent_local/tool_definitions_automation.rs:6`. C'est confirmé à l'exécution : le planificateur lance la conversation avec `StreamPermissionMode::FullAccess` — `src-tauri/src/services/scheduler/agentic.rs:112`.

**C'est le point le plus important de tout cet audit.** Une tâche programmée s'exécute sans aucune demande d'approbation, avec la totalité des outils activés dans l'application, y compris ceux qui écrivent des fichiers et lancent des commandes. Le site doit le dire ; le brief affirme aujourd'hui le contraire.

### Écart 3 — Il n'y a plus d'outils interdits dans une automatisation

> Brief : « **Douze outils sont refusés** dans une automatisation », avec un tableau les listant (choix interactif, mode Plan, gestion d'automatisations, délégation et les huit outils de sous-agents).

**Réalité** : aucune liste de refus n'existe dans le code. `src-tauri/src/services/agent_local/tool_automation_validation.rs` ne contient que trois fonctions — analyse du déclencheur, lecture d'un champ texte, erreur interne (lignes 5-19). Le fichier entier fait 34 lignes, tests compris. Le tableau « Les outils refusés dans une automatisation » du brief est à supprimer.

### Écart 4 — La règle anti-récursion a disparu

> Brief : « Ce dernier point est le plus important : sans lui, une automatisation pourrait en créer d'autres, qui en créeraient d'autres. La règle coupe la récursion à la racine. »

**Réalité** : `manage_automation` étant disponible en mode accès complet, une automatisation peut en créer une autre. Rien dans `tool_automation.rs` ni dans `scheduler/agentic.rs:112` ne l'empêche. La seule borne restante est le plafond global de réveils.

### Écart 5 — La confirmation porte sur trois points, pas cinq

> Brief : « la définition de l'outil lui impose de ne créer ou modifier qu'après confirmation de l'utilisateur sur **les cinq points** : le déclencheur, l'instruction, les outils, les skills, et l'état actif ».

**Réalité** : « Use create or update only after the user confirms **the trigger, instruction, and active state** » — trois points — `tool_definitions_automation.rs:6`. Les deux points disparus sont ceux qui n'existent plus.

### Écart 6 — Les messages d'erreur listés n'existent plus

> Brief, tableau « Les erreurs » : « Skill d'automatisation introuvable », « Outil d'automatisation non autorisé », « Liste d'automatisation trop longue ».

**Réalité** : les codes actuels sont `automation_action_invalid` (`tool_automation.rs:14`), `automation_schedule_required` et `automation_schedule_invalid` (`tool_automation_validation.rs:7-10`), `automation_invalid` (`tool_automation.rs:54`), `automation_create_failed` (`:77`), `automation_update_failed` (`:122`), `automation_confirmation_required` (`:127`), `automation_unavailable` (`tool_automation_validation.rs:18`).

### Ce qui reste exact dans ce fichier

Toute la partie skills (256 Ko : `skill_manifest_policy.rs:3` ; 768 octets : `models/agent_turn_contract.rs:12` ; 120 caractères : `tool_skill_loader.rs:5` ; les trois erreurs de chargement : `tool_skill_loader.rs:20-26`). Les trois formes de déclencheur et le refus d'une syntaxe cron (`tool_automation_validation.rs:29-32`). La suppression exigeant `confirm=true` (`tool_automation.rs:126-127`). Le modèle, le fournisseur et le projet figés à la création (`tool_automation.rs:43-48`). L'avertissement immédiat du planificateur (`tool_automation.rs:72`). La mise en pause automatique quand les réveils sont globalement en pause (`tool_automation.rs:63-66`).

---

## 2. `04-agent/permissions.md` — DÉCALÉ (4 écarts)

C'est la page la plus sensible du site ; deux de ces écarts touchent à ce que l'agent a le droit de faire.

### Écart 1 — Le mode Chatbot n'est pas « aucun outil »

> Brief, tableau des trois modes : « **Chatbot** | `chat` | Aucun outil : réponses en texte uniquement ».

**Réalité** : en mode Chatbot, l'agent reçoit **deux outils** — `web_search` et `web_fetch`. Le catalogue envoyé au modèle est réduit aux définitions web (`src-tauri/src/services/agent_local/tool_definitions_chat.rs:3-5`, avec un test nommé `chat_exposes_only_web_search_and_fetch` ligne 12), aussi bien pour les fournisseurs distants (`src-tauri/src/commands/agent_chat_task/api_tools.rs:15-16`) que pour Ollama (`src-tauri/src/commands/agent_chat_task/ollama_setup.rs:26-27`). Tout autre outil appelé est refusé à l'exécution avec « Outil indisponible dans ce mode. » (`src-tauri/src/services/agent_local/tool_dispatcher_entry.rs:66-70`), la liste des outils autorisés étant `web_search` et `web_fetch` (`src-tauri/src/services/agent_local/tool_dispatcher_route.rs:15-17`).

Formulation juste pour le site : *en mode Chatbot, l'agent ne peut ni lire ni écrire sur la machine ; il garde l'accès à la recherche web et à la lecture d'une page.*

Second effet, non mentionné par le brief : en mode Chatbot, ni `AGENTS.md` ni les fichiers de personnalité ne sont injectés (`src-tauri/src/commands/agent_chat_task/common.rs:105-107`).

### Écart 2 — Les motifs shell sûrs sont vingt, pas vingt et un

> Brief : « **Les motifs reconnus comme sûrs** — vingt et un au total ».

**Réalité** : la liste compte **vingt expressions** — `src-tauri/src/services/agent_local/permission_bash.rs:4-30`. Le contenu du tableau du brief est exact (`ls`, `cat`, `head`, `tail`, `wc`, `grep`, `find`, `git status|log|diff|show|remote|tag`, `git branch` seul, `pwd`, `echo`, `which`, `cargo check|test|clippy|build`, `npx tsc`, `npm run|test`, `tree`, `file`, `stat`, `du`, `df`) ; seul le décompte est faux. Le plus sûr est d'écrire « une vingtaine de familles de commandes » plutôt qu'un chiffre exact, puisque cette liste est appelée à bouger.

### Écart 3 — Les outils fournis par une extension échappent aux deux listes

> Brief : « **Tous les autres outils passent sans confirmation**, y compris en mode Demande d'approbation ».

**Réalité** : un outil venant d'une extension ne passe ni par la liste des douze, ni par les quatre conditionnels. Sa nécessité de confirmation est décidée par la politique de son effet déclaré — `src-tauri/src/services/agent_local/permission_gate.rs:84-89`. Un commentaire du code précise même que l'effet « external-read » réutilise la décision et le dialogue de `web_fetch`. La phrase du brief est donc trop absolue depuis que les extensions sont livrées.

### Écart 4 — Deux caractères de contrôle manquent à la liste

> Brief : les opérateurs qui disqualifient une commande, « retour à la ligne » compris.

**Réalité** : le code teste séparément `\n` **et** `\r` (`permission_bash.rs:49-50`), soit quinze contrôles au total. C'est un détail sans portée pratique, signalé pour l'exactitude.

### Points tranchés

- **« `git branch` seul est sûr, mais `git branch -d` ? »** → **Tranché : `git branch -d` déclenche une confirmation.** Le motif est `^git\s+branch\s*$`, ancré en fin de chaîne : toute option le fait échouer — `permission_bash.rs:14`.
- **« Le comportement en mode Chatbot : les outils sont-ils absents du catalogue ou refusés à l'exécution ? »** → **Tranché : les deux.** Le catalogue est réduit à `web_search` et `web_fetch` avant l'envoi (`tool_definitions_chat.rs:3-5`), et un appel hors de ces deux outils est refusé à l'exécution (`tool_dispatcher_entry.rs:66-70`).
- **« Le mode par défaut est-il bien Accès complet ? »** → **Tranché : oui.** `default_permission_mode()` renvoie `"auto"`, y compris quand le fichier de réglages est absent, illisible ou porte une valeur inconnue — `src-tauri/src/services/agent_local/agent_settings.rs:37-40, 67-69, 75-85`.

### Points restant ouverts

Les libellés exacts des trois réponses, l'aspect de la demande de confirmation, et l'existence d'un réglage de mode par défaut dans l'écran des réglages — tous trois sont des questions d'interface, à traiter dans la passe d'écran de fin de parcours.

---

## 3. `05-outils/vue-densemble.md` — DÉCALÉ (2 écarts)

Le fichier est remarquablement exact sur les chiffres. Ce qui a bougé, ce sont les outils d'extension.

### Écart 1 — Le catalogue verrouillé compte quatorze outils, pas douze

> Brief, dans les points à confirmer : « **Un douzième outil essentiel existe dans le catalogue mais n'appartient à aucun groupe** : `search_extension_tools`. »

**Réalité** : `search_extension_tools` **n'existe plus**. Trois outils d'extension le remplacent, tous verrouillés et tous absents de l'écran des réglages : `list_extensions`, `inspect_extensions` et `load_extension_resource` — déclarés dans `src-tauri/src/services/agent_local/tool_catalog.rs:41-49`, nommés dans le contrat généré `DISCOVERY_TOOL_NAMES` et dans `src-tauri/src/services/agent_local/tool_extension_resource.rs:5`.

Le décompte exact aujourd'hui : **14 outils verrouillés** — les 11 répartis dans les 5 groupes essentiels, plus ces 3 outils d'extension hors groupe.

### Écart 2 — Le remplacement d'un outil natif par une extension n'est plus un chantier gelé

> Brief : « Le comportement quand une extension remplace un outil natif désactivé est implémenté […] mais relève du chantier gelé Extensions. Ne pas documenter maintenant. »

**Réalité** : le filtrage tient compte des outils dynamiques et des remplacements en production — `src-tauri/src/services/agent_local/tool_catalog_filter.rs:13-21`, qui appelle `tool_availability::available` avec `is_dynamic_tool` et `is_replacement`. Les extensions étant livrées, la consigne « ne pas documenter » est à réexaminer avec le propriétaire.

### Ce qui reste exact — et c'est l'essentiel du fichier

- 5 groupes essentiels / 11 outils, 11 groupes optionnels / 32 outils, 5 groupes actifs par défaut — `src-tauri/src/services/agent_local/tool_group_catalog.rs:12-81`.
- Les listes d'outils de chaque groupe, une par une, sont exactes — même fichier.
- La dépendance du groupe Sous-agents à `delegate_task` — `tool_catalog.rs:81-85`.
- `MAX_OPTIONAL_TOOLS = 32`, égal au nombre d'outils optionnels, avec troncature silencieuse par `.take(32)` — `tool_catalog.rs:16` et `:89`. **L'avertissement du brief à l'équipe produit reste entièrement valable.**
- Tous les plafonds de troncature : `web_fetch` 50 000, `bash`/`bash_control` 30 000, `grep`/`web_search`/`list_dir` 10 000, `glob` 5 000, erreurs 30 000, aperçu 2 000 — `src-tauri/src/services/agent_local/tool_result_truncate.rs:4-11`.
- Budget cumulé 100 000 caractères et nettoyage à 24 heures — `src-tauri/src/services/agent_local/tool_result_budget.rs:3` et `:9-14`.
- Le filtrage du prompt système, y compris le retrait des deux sections entières — `src-tauri/src/services/agent_local/tool_prompt_filter.rs:9-18, 37`.
- `plan_mode` est bien un outil optionnel actif par défaut — `tool_catalog.rs:65`.

---

## 4. `04-agent/personnalite-et-agents-md.md` — DÉCALÉ (2 écarts)

### Écart 1 — Le quatrième fichier de personnalité n'est pas dans `memory/core/`

> Brief : « Des fichiers Markdown rangés dans `memory/core/` […] Quatre existent à l'installation », dont `idea-discovery.md`.

**Réalité** : trois fichiers seulement vivent dans `memory/core/` — `identity.md`, `principles.md`, `user.md`. Le quatrième, `idea-discovery.md`, est créé dans **`inbox/`** — `src-tauri/src/storage_migration.rs:107-110`. La séparation est reprise côté commandes : `CORE_FILES` d'un côté, `INBOX_FILES` de l'autre — `src-tauri/src/commands/personality.rs:32-42`.

Le reste de l'affirmation est juste : les quatre sont bien désactivés par défaut (`storage_migration.rs:91-94`, tous à `false`) et leur activation est enregistrée dans `personality-injection.json`.

### Écart 2 — Les instructions permanentes ne sont pas injectées partout

Le brief ne le mentionne pas : ni `AGENTS.md`, ni la personnalité ne sont injectés **en mode Chatbot** ni **dans une session de sous-agent** — `src-tauri/src/commands/agent_chat_task/common.rs:105-107`. C'est une information utile pour l'utilisateur qui constate qu'une consigne permanente semble ignorée.

### Points tranchés

- **« Peut-on ajouter ses propres fichiers de personnalité dans `memory/core/` ? »** → **Tranché : non.** Les trois listes sont des constantes du code — `commands/personality.rs:30, 32-37, 39-42`. Un fichier déposé à la main dans `memory/core/` n'apparaîtra pas dans l'écran Personnalité et ne sera pas injecté.
- **« L'édition de `AGENTS.md` depuis l'application — quel fichier est modifié ? »** → **Tranché : le fichier global**, celui du dossier de données. Il est déclaré comme fichier racine de l'écran Personnalité — `commands/personality.rs:30`, avec `data_root()` comme base.
- **« Le contenu par défaut des quatre fichiers de personnalité »** → partiellement tranché : chaque fichier porte une clé de description i18n (`personality.descriptions.identity`, `.principles`, `.user`, `.ideaDiscovery`) — `commands/personality.rs:33-41`. Le texte affiché à l'utilisateur vient donc des traductions, pas du fichier.

### Ce qui reste exact

L'ordre d'assemblage des six sources est **exactement** celui du brief — `src-tauri/src/services/agent_local/agent_md.rs:33-111` : global, documents importés activés, règles externes triées par source, `AGENTS.md` du projet, `.cl-go/AGENTS.md`, puis `.cl-go/rules/*.md` triés par nom. La limite de 200 Ko (`agent_md.rs:3`), le message d'omission mot pour mot (`agent_md.rs:4-5`), l'en-tête annonçant que les instructions de projet sont plus spécifiques (`agent_md.rs:75-77`) et le filtrage sur l'extension `.md` (`agent_md.rs:103`) sont tous exacts.

---

## 5. `04-agent/sous-agents.md` — DÉCALÉ (2 écarts)

Toutes les valeurs chiffrées de ce fichier sont justes — c'est un fichier solide. Les deux écarts sont d'une autre nature.

### Écart 1 — Référence de ligne périmée

> Brief : « la cascade est interdite explicitement (`tool_delegate.rs:55-60`) ».

**Réalité** : le refus est aujourd'hui aux lignes **45-50** de `src-tauri/src/services/agent_local/tool_delegate.rs`, avec le message inchangé : « Les sous-agents ne peuvent pas lancer d'autres sous-agents. » Le fait est confirmé, seule la référence a glissé.

### Écart 2 — Une restriction manque à la section « Ce qu'un sous-agent ne peut pas faire »

Le brief liste quatre restrictions. Il en manque une, vérifiée : **un sous-agent ne reçoit ni `AGENTS.md` ni la personnalité** — `src-tauri/src/commands/agent_chat_task/common.rs:105-107`. C'est cohérent avec le principe posé par la page (« un sous-agent ne voit rien de la conversation parente »), et cela mérite d'y figurer : quelqu'un qui a écrit des conventions de projet dans `AGENTS.md` doit savoir que ses sous-agents ne les ont pas.

### Points tranchés

- **« La portée d'accès disque d'un sous-agent — celle du parent, ou restreinte à son espace isolé ? »** → **Tranché : restreinte à son espace isolé.** Chaque chemin passé à un outil est vérifié comme confiné dans le répertoire de travail du sous-agent, avec refus explicite de tout `..` — `src-tauri/src/services/agent_local/subagent_tool_guard.rs:107-115` (chemins des outils fichiers), `:131-140` (`validate_confined_path`), `:175-177` (le `workdir` d'une commande shell). Message de refus : « Chemin hors du dossier autorisé. »
- **« La liste complète des types de sous-agents et de leurs profils d'outils »** → **Tranché.** Deux types seulement, `explorer` et `coder` ; tout autre valeur est refusée avec « Type de sous-agent invalide. » — `tool_delegate.rs:22-36` et `src-tauri/src/services/agent_local/subagent_tool_profile.rs:14-20`. Les profils exacts sont donnés au point 6 ci-dessous.
- **« Un sous-agent peut-il écrire en mémoire ? »** → **Tranché : non, et le message est explicite** : « Les sous-agents peuvent seulement lire une mémoire sélectionnée et suggérer une modification au parent. » — `subagent_tool_guard.rs:96-104`. Seul `read_file` passe sur un fichier de mémoire.

---

## 6. `05-outils/sous-agents-outils.md` — DÉCALÉ (2 écarts)

### Écart 1 — Le tableau des outils par type de sous-agent est incomplet

> Brief, tableau « Les outils par type de sous-agent » : Explorateur = lire un fichier, lister un dossier, chercher par motif, chercher par nom, chercher sur le web, ouvrir une page. Codeur = lire un fichier, créer un fichier, modifier un fichier, travailler dans un espace Git isolé.

**Réalité** — `src-tauri/src/services/agent_local/subagent_tool_profile.rs:22-50` :

- **Explorateur** : `bash`, `read_file`, `list_dir`, `grep`, `glob`, `web_search`, `web_fetch` — **sept outils**. Le brief en oublie un, et pas le moindre : **l'explorateur dispose de `bash`.**
- **Codeur** : `bash`, `bash_control`, `read_file`, `write_file`, `edit_file`, `list_dir`, `grep`, `glob`, `web_search`, `web_fetch`, plus `load_skill` si le groupe Skills est activé — **dix ou onze outils**. Le brief en cite quatre.

**Nuance importante à ne pas perdre** : le `bash` de l'explorateur n'est pas le `bash` ordinaire. C'est une liste blanche stricte de commandes de lecture — `pwd`, `ls`, `tree` (avec `-L` obligatoire, profondeur 1 à 8), `file`, `stat`, `wc`, `du`, `df`, `git` — sans aucun opérateur de shell (`;`, `|`, `>`, `<`, backtick, `$(`, `&&`, `||`, guillemets, antislash tous refusés), au plus 32 mots, et tous les chemins confinés au dossier de travail — `src-tauri/src/services/agent_local/subagent_explorer_bash.rs:45-110`.

La phrase du brief « Explorateur — Écrit dans le projet : **Non** » reste donc **vraie**. Mais le tableau doit dire que l'explorateur peut lancer des commandes d'exploration, sans quoi le lecteur croit qu'il n'a aucun accès au shell.

### Écart 2 — Le groupe Skills conditionne l'accès du codeur aux skills

Le brief n'en parle pas : `load_skill` n'est ajouté au profil du codeur **que si les skills sont activés**, et **jamais** au profil de l'explorateur — `subagent_tool_profile.rs:46-48`. Un utilisateur qui coupe le groupe Skills prive aussi ses sous-agents codeurs de leurs guides.

### Points tranchés

- **« Les sous-agents portent-ils des noms visibles fixes — "Claudiator", "Geminitor" ? »** → **Tranché : oui, ces noms sont toujours dans le code**, et ils sont imposés : `default_name("coder")` renvoie `"Claudiator"`, `default_name("explorer")` renvoie `"Geminitor"` — `src-tauri/src/services/agent_local/subagent_profile.rs:1-2, 10, 70-72`. Mieux : un nom fourni par l'agent est **écrasé** par ces noms par défaut (`clean_name`, testé lignes 78-82). **La question posée par le brief à l'équipe produit reste entièrement ouverte et devient plus urgente : ces noms sont bien affichés, et ils évoquent deux produits concurrents.**
- **« Le nombre maximal de projets isolés simultanés »** → **Tranché indirectement** : la borne réelle est le nombre de sous-agents actifs, 4 par conversation et 8 au total — `src-tauri/src/services/agent_local/subagent_registry.rs:9-10, 162-166` et `agent_work_supervision.rs:6`. Il n'existe pas de plafond distinct de « projets isolés ».

### Ce qui reste exact

Les neuf outils et leur ordre (`tool_catalog.rs:17-27`), le fait qu'`apply_subagent_changes` soit le seul soumis à approbation (`permission_gate.rs:76`), l'interdiction de la cascade (`tool_delegate.rs:45-50`), la contrainte du type `coder` sur un dossier valide (`tool_delegate.rs:51-56`), et les deux plafonds 4 / 8.

---

## 7. `04-agent/fonctionnement.md` — DÉCALÉ (2 écarts)

Les trois valeurs de la page sont exactes : 200 tours (`src-tauri/src/services/agent_local/agent_loop_limits.rs:1`), 6 appels identiques (`circuit_breaker.rs:1, 21`), 10 outils de lecture en parallèle (`tool_executor_parallel_batch.rs:17, 38`).

### Écart 1 — Le message du garde-fou anti-boucle n'est pas traduit

Le brief le range dans les points à confirmer. Le voici, en dur dans le code : **« Circuit breaker : {N} appels identiques consécutifs détectés. Boucle probable, arrêt. »** — `src-tauri/src/services/agent_local/circuit_breaker.rs:22-25`.

Deux remarques pour le site. D'abord, ce message n'est pas une clé de traduction : un utilisateur en espagnol ou en japonais lira du français mêlé d'anglais. **C'est un manquement à la règle i18n du projet, à remonter à l'équipe.** Ensuite, le terme « circuit breaker » n'est pas compréhensible sans explication ; la page doit décrire le comportement plutôt que de citer le message tel quel.

### Écart 2 — La décharge du GPU n'est pas une fonction de fin de boucle

> Brief : « **La décharge du GPU en fin de boucle.** Une fonction s'en occupe pour les modèles locaux. »

**Réalité** : aucune fonction de déchargement n'existe côté agent. Le mécanisme est le paramètre `keep_alive` envoyé à Ollama **à chaque requête** — `src-tauri/src/services/agent_local/agent_loop_support.rs:29-36` et `ollama_wire.rs:48`. Il vient des réglages avancés, vaut **5 minutes par défaut**, et la valeur « forever » est traduite en `-1m`, ce qui demande à Ollama de ne jamais décharger le modèle.

Réponse à la question du brief : **oui, le modèle est libéré de la mémoire vidéo**, mais après cinq minutes d'inactivité, pas en fin de boucle — et le message suivant paie alors le temps de rechargement. C'est un réglage utilisateur, à documenter comme tel.

### Points tranchés

- **« Le mécanisme de pré-dispatch »** → **Tranché : il existe et il fait bien ce que le brief soupçonnait.** Les outils **en lecture seule** demandés par le modèle sont lancés **pendant que sa réponse est encore en train d'arriver**, sans attendre la fin du flux — `src-tauri/src/services/agent_local/eager_dispatch.rs:1-40`, avec le même plafond de dix appels simultanés que le traitement par lots (`eager_dispatch.rs:11`). Le mécanisme est activé en production et désactivé seulement dans les rejeux de test (`agent_loop.rs:53-62`). **C'est effectivement un point de performance à documenter** : sur une exploration, la lecture des fichiers commence avant que le modèle ait fini d'écrire.
- **« Le comportement en mode Chatbot : la boucle se réduit-elle à un seul tour ? »** → **Tranché : non.** La boucle est la même ; c'est le catalogue d'outils qui est réduit à `web_search` et `web_fetch` (voir le point 2 de ce rapport). Un tour de Chatbot peut donc enchaîner une recherche, la lecture d'une page, puis la réponse.
- **« Le message affiché à 200 tours »** → non trouvé sous forme de message dédié ; la boucle est un `for turn in 0..MAX_TURNS` (`agent_loop.rs:30`) et la fin de parcours est signalée au modèle avant le dernier tour. À vérifier à l'écran avec le reste de la passe d'interface.
- **La taille de la file d'attente de messages** reste ouverte.

---

## 8. `04-agent/skills-locaux.md` — DÉCALÉ (1 écart)

Toutes les limites sont exactes : 2 048 skills (`skill_catalog.rs:9`), 256 Ko (`skill_manifest_policy.rs:3`), en-tête 32 Ko (`skill_parser.rs:7`), description 250 caractères (`skill_parser.rs:8`), nom 120 caractères (`tool_skill_loader.rs:5`), identifiant 768 octets (`models/agent_turn_contract.rs:12`), `SKILL.md` et `skill.md` acceptés (`skill_catalog.rs:8`), tri insensible à la casse (`skill_catalog.rs:24-30`).

### Écart — Une quatrième source de skills existe désormais

Le brief décrit deux sources : les skills locaux et les skills importés d'un autre assistant. Le code en connaît une troisième : les **skills d'extension**, dont l'identifiant est préfixé par `extension:`. Ils sont **délibérément retirés du catalogue global** — « This namespace is session-authorized and must never enter the global catalog » — `src-tauri/src/services/agent_local/skill_catalog.rs:22-23, 35-42`. Ils sont autorisés session par session.

Sans conséquence directe pour l'utilisateur aujourd'hui, mais la page « Les skills importés » est incomplète.

### Points tranchés

- **« Le champ `command` associé à chaque skill — correspond-il à une façon de l'invoquer explicitement ? »** → **Tranché : oui, et c'est une fonctionnalité à documenter.** Chaque skill reçoit une commande construite depuis sa source et son nom, rendue unique en cas de doublon — `skill_catalog.rs:141-154` et `:156-161`. Côté interface, taper `/` ouvre une liste d'autocomplétion et la sélection insère la commande du skill dans le message — `src/hooks/use-active-skills.ts:35, 43` et `src/components/agent-local/slash-autocomplete.tsx`. Un skill local s'invoque donc par `/nom-du-skill` ; un skill importé par `/source:nom`.
- **« L'accès des sous-agents aux skills »** → **Tranché.** Seul un sous-agent de type **codeur** reçoit `load_skill`, et seulement si le groupe Skills est activé. Un sous-agent explorateur n'y a jamais accès — `subagent_tool_profile.rs:46-48`.
- **« Le comportement en cas de noms identiques entre un skill local et un skill importé »** → **Tranché** : les commandes sont rendues uniques après le tri (`make_commands_unique`, `skill_catalog.rs:31, 156-161`) ; les identifiants restent distincts puisqu'ils intègrent la source.

---

# Points à confirmer tranchés — récapitulatif

Ces points étaient ouverts dans les briefs ; le code permet de les fermer. Chaque ligne donne la réponse et sa preuve.

| Fichier | Question | Réponse | Preuve |
|---|---|---|---|
| contexte | Quel fournisseur ne compte pas le raisonnement ? | Le compte web **Codex** (identifiant `codex-oauth`). L'indicateur `reasoning_included` est faux pour lui seul | `context_usage_buckets.rs:37` + `services/codex_client/mod.rs:34` |
| contexte | Le méta-contexte occupe-t-il une catégorie propre ? | Oui, l'une des sept, distincte de « Skills » | `context_usage_buckets.rs:8, 19-27` |
| diagnostics | Valeurs exactes de troncature et de masquage | 200 caractères pour le texte, 1 000 pour les structures, `[redacted]` et `[path]` | `diagnostic_args.rs:4-5`, `diagnostic_redaction.rs:3-4` |
| fonctionnement | Le pré-dispatch existe-t-il ? | Oui : les outils de lecture démarrent pendant le flux de réponse, dix au plus | `eager_dispatch.rs:11-40`, `agent_loop.rs:53-62` |
| fonctionnement | La décharge du GPU | Paramètre `keep_alive`, 5 minutes par défaut, « forever » = jamais | `agent_loop_support.rs:29-36` |
| fonctionnement | Message du garde-fou anti-boucle | « Circuit breaker : N appels identiques consécutifs détectés. Boucle probable, arrêt. » — non traduit | `circuit_breaker.rs:22-25` |
| fonctionnement / permissions | Comportement en mode Chatbot | Boucle inchangée ; catalogue réduit à `web_search` et `web_fetch` ; tout autre appel refusé | `tool_definitions_chat.rs:3-5`, `tool_dispatcher_route.rs:15-17`, `tool_dispatcher_entry.rs:66-70` |
| mémoire | Budget minimum d'injection | 256 jetons, valeur bornée à l'enregistrement comme à la lecture | `memory_types.rs:45-48` |
| permissions | `git branch -d` est-il sûr ? | Non : le motif exige la fin de chaîne, toute option déclenche une confirmation | `permission_bash.rs:14` |
| permissions | Le mode par défaut est-il Accès complet ? | Oui, y compris fichier absent, illisible ou valeur inconnue | `agent_settings.rs:37-40, 67-69, 75-85` |
| personnalité | Peut-on ajouter ses propres fichiers de personnalité ? | Non, les listes sont des constantes du code | `commands/personality.rs:30-42` |
| personnalité | Quel `AGENTS.md` l'application édite-t-elle ? | Le global, celui du dossier de données | `commands/personality.rs:30` |
| prompts système | Seuil entre Compact et Détaillé | **25 milliards de paramètres**. Les métadonnées d'Ollama font autorité, le nom du modèle sert de repli | `model_size.rs:3, 15-17, 30-34` |
| répertoire de travail | Quelles sont les trois racines internes ? | L'espace de travail géré d'une session (`session-workspaces/<…>/work`) et le dossier de sorties de session, ajoutés aux racines configurées | `directory_access_scope.rs:5-22`, `session_workspace.rs:104-134` |
| répertoire de travail / sous-agents | Portée disque d'un sous-agent | Confinée à son dossier de travail, `..` refusé, y compris pour le `workdir` d'une commande shell | `subagent_tool_guard.rs:107-115, 131-140, 175-177` |
| skills locaux | À quoi sert le champ `command` ? | À invoquer le skill par `/nom` dans la zone de saisie | `skill_catalog.rs:141-161`, `src/hooks/use-active-skills.ts:35, 43` |
| skills locaux / sous-agents | Quels sous-agents ont accès aux skills ? | Le codeur seulement, et seulement si le groupe Skills est activé | `subagent_tool_profile.rs:46-48` |
| sous-agents | Types existants | `explorer` et `coder`, tout autre valeur refusée | `tool_delegate.rs:22-36`, `subagent_tool_profile.rs:14-20` |
| sous-agents | Un sous-agent peut-il écrire en mémoire ? | Non, lecture seule, avec un message dédié | `subagent_tool_guard.rs:96-104` |
| todos | Qu'est-ce qu'un « tour » dans le compteur de négligence ? | **Un message de l'utilisateur** — le compteur est incrémenté par `record_user_turn` | `tool_todo_neglect.rs:15-23` |
| documents | Le filtrage par page des PDF fonctionne-t-il ? | **Toujours pas.** Le paramètre est reçu sous le nom `_pages` et jamais utilisé, alors que la définition l'annonce au modèle | `tool_document_read.rs:12` et `tool_definitions_office.rs:33` |
| documents | La limite d'un million de caractères protège-t-elle le PDF ? | **Non.** Elle n'est appliquée que sur le chemin `.docx`. L'extraction PDF n'est bornée par rien | `tool_document_read.rs:47-70` (PDF, sans borne) vs `:166` (docx) |
| automatisations | Nombre maximal d'automatisations | **64 réveils au total**, agentiques et non agentiques confondus | `commands/heartbeat_validation.rs:4` |
| vue d'ensemble | Le douzième outil essentiel | Il y en a trois : `list_extensions`, `inspect_extensions`, `load_extension_resource` ; `search_extension_tools` n'existe plus | `tool_catalog.rs:41-49` |
| sous-agents outils | Les noms Claudiator / Geminitor sont-ils affichés ? | Oui, et ils sont imposés : un nom fourni par l'agent est écrasé | `subagent_profile.rs:1-2, 10, 78-82` |

---

# Points restant ouverts, et pourquoi

## Ce qui demande la passe d'interface prévue en fin de parcours

Ces points sont ouverts par construction : les briefs ont été écrits depuis le code, et le code ne dit pas à quoi ressemble un écran. Ils forment la liste de contrôle de la passe d'interface, exactement comme le prévoit `00-comment-utiliser-ces-fichiers.md`. Aucun ne remet en cause un fait.

Où se trouve l'écran d'usage du contexte et ce qu'il affiche ; où l'utilisateur voit les diagnostics ; l'aspect d'une demande de confirmation et les libellés de ses trois réponses ; l'existence d'un réglage de mode par défaut dans l'écran des réglages ; l'affichage de la liste de tâches, des sous-agents, de l'écran d'inspection des changements, d'une question interactive, d'un résultat de recherche, d'une commande en cours, d'un document ou d'un tableur lu ; le libellé exact des deux titres de l'écran Outils (le code y écrit « Tools essentiels » et « Tools optionnels », en anglais — l'incohérence signalée par le brief `vue-densemble.md` est confirmée et reste à remonter).

## Ce qui demande un essai réel, non lisible dans le code

- **Le comportement de la capture d'environnement shell avec `.zprofile`** (`terminal-et-shell.md`). Le shell est lancé en mode connexion, ce qui exécute d'autres fichiers que ceux nommés dans le brief. Seul un essai tranchera.
- **L'effet de l'arrêt après fermeture d'une question interactive** — l'agent s'arrête-t-il net ou conclut-il par une phrase ? (`choix-interactif.md`)
- **Le comportement sur un GIF animé** (`images.md`) et sur un **CSV dont la première ligne n'a pas de séparateur** (`tableurs.md`).
- **Le comportement quand on change de modèle vers une fenêtre plus petite que l'historique** (`contexte.md`).
- **Ce qui déclenche une écriture en mode Automatique** dans la mémoire, et le moment où le résumé de portée est produit (`memoire-persistante.md`) : lisible dans le code, mais demande une lecture du module d'extraction que cet audit n'a pas faite.

## Ce qui relève d'une décision produit, pas d'une vérification

- **Les noms Claudiator et Geminitor.** Confirmés dans le code et imposés. La question posée à l'équipe produit reste entière.
- **La liste des motifs shell bloqués** : la publier documente aussi ce qui n'est pas bloqué. La recommandation du brief — donner les catégories, pas la liste — tient.
- **La liste des ports réseau bloqués** : même raisonnement.
- **Le paramètre `pages` des PDF** : soit l'implémenter, soit le retirer de la définition. Il ment aujourd'hui au modèle.
- **L'extraction PDF non bornée** : à corriger côté produit avant que le site ne promette quoi que ce soit sur les gros PDF.
- **La troncature silencieuse à 32 outils optionnels** : sans conséquence aujourd'hui, piège certain au prochain outil ajouté.
- **Le message du garde-fou anti-boucle non traduit** : manquement à la règle i18n du projet.
- **Le mode Plan** : la page `04-agent/permissions.md` y renvoie et le groupe `plan_mode` est actif par défaut (`tool_catalog.rs:65`), alors que le mécanisme doit être revu. Les outils autorisés pendant un plan sont aujourd'hui au nombre de vingt et un, plus les commandes shell reconnues comme sûres et la recherche MCP sans appel — `tool_plan_guard.rs:3-44`. Ce périmètre changera ; ne rien figer sur le site avant la refonte.

---

# Annexe — ce qui a été vérifié et trouvé exact

Cette liste sert à éviter qu'un futur relecteur refasse le travail. Toutes ces valeurs ont été lues dans le code le 9 septembre 2026 et correspondent au brief.

**Boucle et contexte** — 200 tours, 6 appels identiques, 10 outils en parallèle ; réserve de réponse 15 % / 4 096 / 16 384 jetons ; sept catégories d'usage ; code `context_capacity_exceeded`.

**Permissions** — les douze outils de la liste dure, dans l'ordre ; les quatre outils conditionnels et leur condition exacte ; les seize marqueurs de chemin sensible ; les quatre fichiers d'application protégés ; le retrait du corps des heredocs avant analyse ; validité d'une heure, 64 sessions, 16 outils par session ; les trois outils jamais mémorisés ; 64 demandes en attente au maximum ; rotation du journal de permissions à 2 Mo.

**Mémoire** — 3 000 jetons par défaut et au maximum, 256 au minimum ; 256 sujets par portée ; 48 Ko par sujet ; 16 Ko par résumé ; 8 étiquettes ; mode désactivé par défaut ; portées `memory/global/` et `memory/projects/` ; les trois vues globale / projet actif / autres projets.

**Instructions permanentes** — l'ordre des six sources ; 200 Ko ; le message d'omission mot pour mot ; le tri alphabétique des règles de projet ; le filtrage sur `.md`.

**Prompts système** — deux modes, deux niveaux, quatre états, trois provenances ; un prompt personnalisé vide vaut désactivation.

**Pièces jointes** — 15 pièces, 20 Mo, chemin de 4 096 octets ; autorisation signée par HMAC avec une clé du coffre ; code `attachment_access_denied` ; restauration au redémarrage.

**Accès disque** — 70 dossiers configurables, 73 racines d'espace de travail, 4 096 caractères ; message « Accès au dossier refusé par les réglages. » ; défaut `/` sur macOS et Linux, `C:\` sur Windows ; liste vide ramenée au défaut.

**Liste de tâches** — 20 séries conservées, relance après 2 tours, pause automatique après 4 ; trois statuts de tâche, trois états de série ; groupe éteint par défaut ; quatre des cinq outils permis en mode Plan.

**Sous-agents** — 4 par session, 8 au total, avec les messages exacts ; 50 000 caractères par consigne, 8 consignes en file ; 128 chemins, 512 caractères par chemin, 256 lots, 128 Ko par descriptif ; instantané de dossier 20 000 fichiers / 512 Mo / 64 Mo par fichier ; quatre états de lot ; deux natures d'espace de travail.

**Outils, vue d'ensemble** — 5 groupes essentiels et leurs 11 outils ; 11 groupes optionnels et leurs 32 outils ; les 5 groupes actifs par défaut ; la dépendance à `delegate_task` ; tous les plafonds de troncature ; budget de 100 000 caractères ; nettoyage à 24 heures ; filtrage du prompt système et de ses deux sections.

**Fichiers** — 20 Mo, 2 000 lignes par défaut, 50 000 au maximum ; `list_dir` à 3 niveaux et 500 entrées, masquant les fichiers cachés, `node_modules` et `target` ; 1 000 chemins mémorisés comme vus, 100 évincés à la fois.

**Recherche** — 250 correspondances, 100 fichiers, motif de 500 caractères, délai de 600 secondes.

**Terminal** — 64 sessions ; commande de 512 Ko sur Unix et 24 Ko sur Windows ; entrée de 64 Ko avec 5 secondes d'écriture ; attente de 250 ms à 30 s, 10 s par défaut ; 1 Mo conservé, ~28 Ko par passage, 2 000 lignes ou 50 Ko au modèle ; 500 chemins suivis ; 64 profils d'environnement en cache, 128 Ko, 5 secondes ; 300 ms de grâce à la fermeture.

**Web** — requête de 512 caractères, 10 résultats, titre 160, extrait 300, adresse 2 048 ; corps de 5 Mo, délai de 15 secondes, 3 redirections ; ordre Brave → Exa → Firecrawl → moteur local.

**MCP** — 15 outils par connecteur, nom de 64 caractères, réponse de 4 096 caractères, appel abandonné à 60 secondes.

**Bureautique** — archive : 4 096 entrées, compression 100 pour 1, 200 Mo décompressés, source 50 Mo ; tableur : 500 lignes par défaut, 5 000 au maximum, 1 000 colonnes, 5 millions de cellules, 10 000 opérations ; document : 1 million de caractères extraits, 5 000 blocs ; image : 8 000 pixels de côté, 50 millions de pixels, 128 opérations, qualité de 1 à 100.

**Choix interactif** — 1 à 5 questions, 2 à 4 options, intitulé 30, question 500, libellé 80, description 1 500, aperçu 1 500, réponse libre 1 500.

**Forecast** — 5 000 lignes, 256 colonnes, 256 séries, 64 variables explicatives, horizon 5 000, 100 000 prédictions, 5 Mo en direct, 50 Mo en tableur, 20 profils, 500 analyses, 200 prédictions par page, 5 modèles et 5 fenêtres de validation, 1 validation à la fois, ensemble de 2 à 4 modèles, 5 candidats en mode automatique.

**Git** — deux outils, groupe éteint par défaut, tous deux soumis à approbation.
