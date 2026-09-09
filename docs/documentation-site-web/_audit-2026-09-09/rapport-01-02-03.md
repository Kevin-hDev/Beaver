# Audit de fraîcheur — sections 01, 02 et 03

**Date** — 9 septembre 2026
**Périmètre** — les 23 fichiers `.md` de `01-decouverte/` (4), `02-installation/` (8) et `03-interface/` (11)
**Méthode** — chaque affirmation chiffrée, chaque tableau de référence et chaque « Point à confirmer » a été confronté au code du dépôt. Aucun verdict n'est donné sans avoir ouvert le fichier de code cité.
**Version de l'application au moment de l'audit** — **1.2.2** (`package.json:3`, `src-tauri/Cargo.toml:4`, `src-tauri/tauri.conf.json:4`)

---

## Tableau récapitulatif

| Fichier | Verdict | Écarts | Points à confirmer tranchés / restants |
|---|---|---|---|
| `01-decouverte/presentation.md` | DÉCALÉ | 1 | 1 / 1 |
| `01-decouverte/concepts-cles.md` | DÉCALÉ | 5 | 1 / 2 |
| `01-decouverte/local-vs-cloud.md` | DÉCALÉ | 3 | 3 / 1 |
| `01-decouverte/tour-des-fonctionnalites.md` | DÉCALÉ | 4 | 4 / 0 |
| `02-installation/prerequis.md` | À JOUR | 0 | 2 / 4 |
| `02-installation/installation-macos.md` | À JOUR | 0 | 1 / 3 |
| `02-installation/installation-linux.md` | À JOUR | 0 | 2 / 4 |
| `02-installation/installation-windows.md` | À JOUR | 0 | 2 / 4 |
| `02-installation/premier-lancement.md` | DÉCALÉ | 2 | 3 / 3 |
| `02-installation/onboarding.md` | DÉCALÉ | 1 | 2 / 3 |
| `02-installation/import-depuis-un-autre-assistant.md` | À JOUR | 0 | 0 / 6 |
| `02-installation/mise-a-jour.md` | DÉCALÉ | 1 | 2 / 4 |
| `03-interface/vue-densemble.md` | DÉCALÉ | 2 | 0 / 5 |
| `03-interface/panneau-lateral.md` | À JOUR | 0 | 2 / 3 |
| `03-interface/conversations-et-onglets.md` | DÉCALÉ | 3 | 7 / 1 |
| `03-interface/cloner-une-conversation.md` | DÉCALÉ | 1 | 0 / 6 |
| `03-interface/raccourcis-clavier.md` | **OBSOLÈTE** | 4 | 2 / 3 |
| `03-interface/navigateur-integre.md` | À JOUR | 0 | 3 / 3 |
| `03-interface/terminal-integre.md` | DÉCALÉ | 2 | 2 / 3 |
| `03-interface/arbre-de-fichiers-et-previews.md` | À JOUR | 0 | 1 / 4 |
| `03-interface/langues.md` | À JOUR | 0 | 2 / 3 |
| `03-interface/themes-et-apparence.md` | À JOUR | 0 | 2 / 3 |
| `03-interface/mascotte.md` | DÉCALÉ | 1 | 3 / 3 |

**Total** — 10 fichiers à jour, 12 décalés, 1 obsolète.

---

# Détail des fichiers DÉCALÉS et OBSOLÈTE

## 01-decouverte/presentation.md — DÉCALÉ

### Écart 1 — la version courante

- **Le brief dit** (§1) : « Version courante au moment de la rédaction : **1.1.2** (`package.json`). Une v1.1.3 est décrite au CHANGELOG, et une section *Unreleased* couvre le changement de licence. »
- **Le code dit** : la version est **1.2.2** dans les trois fichiers qui font autorité — `package.json:3`, `src-tauri/Cargo.toml:4`, `src-tauri/tauri.conf.json:4`. La 1.1.3 et la section *Unreleased* sont publiées depuis longtemps.

### Ce qui reste exact

La section Licence est confirmée mot pour mot : bascule AGPL v3 à partir de la v1.1.3, versions **jusqu'à 1.1.2 incluse** restées sous Apache 2.0 (`CHANGELOG.md:322`, `README.md:423`), licence commerciale disponible (`README.md:417`). Les libellés des trois modes de permission sont confirmés (`services/agent_local/agent_settings.rs`, modes `auto` / `manual` / `chat`).

---

## 01-decouverte/concepts-cles.md — DÉCALÉ

C'est le fichier le plus touché de la section 01 : la façon dont les outils sont rangés a changé en profondeur.

### Écart 1 — la liste des groupes d'outils est entièrement fausse

- **Le brief dit** : « Groupes réels : `terminal`, `files`, `file_search`, `web`, `mcp`, `skills`, `automations`, `user_choice`, `subagents`, `plan_mode`, `todo_list`, `git_branches`, `forecast`, `spreadsheet`, `document`, `images`. » (seize groupes)
- **Le code dit** (`src-tauri/src/services/agent_local/tool_catalog.rs:29-85`) : **onze groupes**, et sept des seize noms cités n'existent plus.
  - Groupes verrouillés : `core`, `web`, `mcp`, `extensions`
  - Groupes optionnels : `workflow`, `automation`, `subagents`, `todo`, `git`, `forecast`, `office`

### Écart 2 — les groupes verrouillés

- **Le brief dit** : « Les cinq groupes verrouillés : `terminal`, `files`, `file_search`, `web`, `mcp`. »
- **Le code dit** (`tool_catalog.rs:29-50`) : **quatre** groupes verrouillés (`core`, `web`, `mcp`, `extensions`) portant **quatorze** outils. Le groupe `extensions` est nouveau et n'apparaît nulle part dans le brief.

### Écart 3 — le nombre de fournisseurs LLM

- **Le brief dit** : « **Dix** fournisseurs LLM sont gérés. »
- **Le code dit** (`src-tauri/src/services/llm/route_profile/catalog_api.rs:46-179`) : **onze** — Google Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek, xAI, Moonshot Kimi, Z.ai GLM, **Anthropic Claude**, **Qwen**. Groq a été **retiré** (`src-tauri/src/services/api_keys_retired.rs:1` : `RETIRED_PROVIDER_ID: &str = "groq"`).

### Écart 4 — la notion d'Extension est sous-décrite

- **Le brief dit** : « **Extension** — un module installable qui étend l'application elle-même, distribué et mis à jour depuis une source Git. »
- **Le code dit** : les extensions dépassent largement cette définition. Elles contribuent désormais des onglets de réglages (`src/components/settings/settings-sections.ts:78-96`), des entrées de navigation (`src/features/extension-ui/core-occupants.tsx`), des thèmes (`src/lib/app-themes.ts:35` : `ExtensionThemeChoice`) et trois outils verrouillés accessibles à l'agent (`tool_catalog.rs:41-49`). Le glossaire doit dire qu'une extension modifie l'interface elle-même.

### Écart 5 — le nombre maximal d'outils optionnels n'est plus une marge

- **Le brief dit** : « Maximum **32** outils optionnels actifs simultanément (`MAX_OPTIONAL_TOOLS`). »
- **Le code dit** : la valeur 32 est exacte (`tool_catalog.rs:16`), mais le catalogue compte **exactement 32 outils optionnels** (`tool_catalog.rs:52-85`). Le plafond n'écarte donc plus rien aujourd'hui : le présenter comme une limite qui gêne serait trompeur.

### Ce qui reste exact

Neuf outils de sous-agent (`tool_catalog.rs:17-27`), cinq outils de todos (`tool_catalog.rs:66-70`), les trois modes de permission plus `subagent`, les plans dans `plans/<session_id>/` (`services/agent_local/tool_plan_storage.rs:14`), les worktrees dans `subagent-worktrees/` (`services/agent_local/subagent_worktree.rs:36`), la mémoire en portée `global` et `projects` (`services/agent_local/memory_paths.rs:33-35`, `memory_prompt.rs:112`), les trois types de réveil `once`/`daily`/`weekly` (`src-tauri/src/models/config.rs:152-155`), les trois canaux Telegram/Slack/Discord (`services/gateway/service_channels.rs:97-99`), deux jeux de prompts système en variantes compacte et détaillée (`services/agent_local/system_prompt_defaults.rs:7-16`), le coffre XChaCha20-Poly1305 et `secrets.enc`.

---

## 01-decouverte/local-vs-cloud.md — DÉCALÉ

### Écart 1 — Groq figure encore au tableau des fournisseurs

- **Le brief dit** (Tableau 2, ligne 1) : « LLM | Groq | Clé API ».
- **Le code dit** : Groq a été retiré et une migration efface activement sa clé du coffre (`src-tauri/src/services/api_keys_retired.rs:1-4` et `:43-44`, qui crée un marqueur `.retired-provider-groq-v1` et une sauvegarde `secrets.enc.pre-groq-removal.bak`). Une commande de réveil référençant `groq` est explicitement rejetée (`src-tauri/src/commands/heartbeat_validation.rs:168-172`). **Cette ligne doit disparaître du tableau.**

### Écart 2 — deux fournisseurs manquent au tableau

- **Le brief dit** : dix lignes LLM.
- **Le code dit** (`catalog_api.rs:166` et `:179`) : **Anthropic Claude** (clé API Console, en-tête `x-api-key`) et **Qwen** (clé API, point d'accès résolu via Alibaba Model Studio) sont absents du tableau.

### Écart 3 — le champ *Vérification* cite un fichier qui n'existe plus

- **Le brief dit** : « Liste des fournisseurs vérifiée dans le code (`services/llm/catalog.rs`…) ».
- **Le code dit** : l'autorité est aujourd'hui `src-tauri/src/services/llm/route_profile/catalog_api.rs` pour les clés API et `catalog_oauth.rs` pour les comptes web. Le champ *Sources* doit être repointé.

### Points à confirmer tranchés

- **Le statut expérimental du compte web Kimi** — **toujours d'actualité**. `src-tauri/src/services/oauth_providers/mod.rs:57` : `experimental: id == ProviderId::Moonshot`. OpenAI/Codex et xAI ne portent pas ce drapeau (`mod.rs:22` et `:48-57`).
- **Les trois comptes web** — confirmés : OpenAI (Codex), xAI, Moonshot AI (`src-tauri/src/services/llm/route_profile/catalog_oauth.rs:28-56`).
- **Les identifiants `google`/`gemini` et `zai`/`glm`** — l'identifiant de secret est `google` (`catalog_api.rs:48`) et le nom affiché « Google Gemini » ; côté prévision, `google` désigne TimesFM, collision documentée en tête de `src-tauri/src/services/forecast/catalog.rs:1-7`. Sans incidence utilisateur, comme le brief le supposait.

### Ce qui reste exact

Les quatre fournisseurs de recherche Brave, Exa, Firecrawl et le repli local SearXNG (`src-tauri/src/services/search/mod.rs:5` et `:74-76`), et la position du sélecteur de modèle dans la barre de saisie.

---

## 01-decouverte/tour-des-fonctionnalites.md — DÉCALÉ

### Écart 1 — dix fournisseurs annoncés (domaine 9)

- **Le brief dit** : « **Dix** fournisseurs LLM par clé API, dont **trois** acceptant aussi une connexion par compte web. »
- **Le code dit** : **onze** par clé API (`catalog_api.rs:46-179`). Le « trois comptes web » reste exact (`catalog_oauth.rs:28-56`).

### Écart 2 — neuf familles de modèles Forecast (domaine 8)

- **Le brief dit** : « **Neuf familles** de modèles locaux, plus TimeGPT en cloud. »
- **Le code dit** (`src-tauri/src/services/forecast/catalog_specs/mod.rs:16-41`, champ `family_id` de chaque modèle) : **dix familles locales** — `chronos-bolt`, `chronos-2`, `timesfm-2-5`, `toto-2`, `moirai-2`, `flowstate`, `tabpfn-ts`, `tirex`, `kairos`, `sundial` — plus `timegpt-2` en cloud, soit onze familles au total.

### Écart 3 — les extensions ne figurent dans aucun des seize domaines

- **Le brief dit** : seize domaines, dont le 10 couvre « Connecteurs MCP et canaux ».
- **Le code dit** : les extensions sont devenues un domaine à part entière — onglet de réglages dédié (`src/features/extension-ui/core-occupants.tsx:58`), trois outils verrouillés pour l'agent (`tool_catalog.rs:41-49`), contributions d'onglets, de thèmes et de skills (`src-tauri/src/services/extensions/`, ~40 fichiers). Un panorama qui les omet sous-vend l'application.

### Écart 4 — le mot « Unreleased » du piège n° 3 n'a plus d'objet

- **Le brief dit** (Pièges) : « Ce qui figure au CHANGELOG sous *Unreleased* n'y a pas sa place. »
- **Le code dit** : la section *Unreleased* de l'époque (changement de licence) est publiée depuis la v1.1.3 (`CHANGELOG.md:322`). Le conseil reste bon dans son principe, l'exemple est périmé.

### Points à confirmer tranchés

- **Sept langues et six thèmes** — **confirmé, tous proposés**. Sept fichiers de traduction (`src/i18n/{en,fr,de,es,it,zh,ja}.json`) et sept entrées dans le sélecteur (`src/components/settings/general-settings-options.ts:14-22`) ; six thèmes plus l'option Système (`src/lib/app-themes.ts:1-30`).
- **Le navigateur intégré sur Linux** — **limitation par conception, pas temporaire**. `src-tauri/src/services/browser/cef_runtime_policy.rs:72-75` : hors macOS et Windows, la capacité vaut `BrowserCapability::Hidden`, c'est-à-dire **masquée** et non affichée en panne. L'interface ne montre pas l'entrée (`src/components/agent-local/mode-selector.tsx:97`).
- **Les neuf applications sources d'import** — **confirmé** : `claude`, `codex`, `agents`, `hermes`, `qwen`, `zcode`, `openclaw`, `opencode`, `kimi` (`src-tauri/src/services/agent_import/source_specs.rs:49-101`), plafond `MAX_SOURCES = 9` (`agent_import/limits.rs:1`). Le registre est `source_specs.rs`, pas `registry.rs` comme le supposait le brief.
- **Le nombre de familles Forecast** — tranché ci-dessus : dix locales.

---

## 02-installation/premier-lancement.md — DÉCALÉ

### Écart 1 — `terminal-tabs.json` n'est plus créé au premier lancement

- **Le brief dit** (§3, tableau des fichiers créés) : « `terminal-tabs.json` | `[]` ».
- **Le code dit** (`src-tauri/src/storage_migration.rs:81-97`) : la liste des fichiers créés avec leur valeur par défaut ne contient plus `terminal-tabs.json`. Le fichier existe toujours et vit dans le dossier de données (`src-tauri/src/services/terminal/tab_store.rs:181`), mais il est créé à la première utilisation du terminal, pas à l'initialisation.

### Écart 2 — cinq fichiers créés vides ne sont pas listés

- **Le brief dit** : rien sur eux.
- **Le code dit** (`storage_migration.rs:101-115`) : `AGENTS.md`, `memory/core/identity.md`, `memory/core/principles.md`, `memory/core/user.md` et `inbox/idea-discovery.md` sont créés vides au premier lancement. Ces cinq fichiers expliquent le contenu de `personality-injection.json` (quatre entrées, toutes à `false`) et méritent d'être mentionnés.

### Points à confirmer tranchés

- **La migration Windows depuis `%APPDATA%`** — **toujours active**. `storage_migration.rs:49-53` : marqueur `.migrated-from-appdata`, copie depuis l'ancien emplacement, témoin écrit après coup. Les trois migrations du tableau du brief sont confirmées (`:19-23`, `:37-41`, `:49-53`).
- **Le dossier `inbox/`** — sa fonction se lit dans le code : il contient `pending.json` (`storage_migration.rs:88`) et `idea-discovery.md` (`:110`), ce dernier étant l'un des quatre fichiers de personnalité listés dans `personality-injection.json` (`:92-97`). Il **n'est donc pas** lié aux messages entre agent parent et sous-agents comme le supposait le brief.
- **Le comportement du bouton de fermeture** — **confirmé** : `src-tauri/src/app_events.rs:13-19` (`main_window_close_action`) masque la fenêtre sur macOS et quitte ailleurs ; la réouverture par le Dock est gérée en `app_events.rs:42-48`.

### Ce qui reste exact

Le chemin de données unique `~/.local/share/cl-go-dash/` (`src-tauri/src/services/paths.rs:9-14`), les sept dossiers créés (`storage_migration.rs:69-78`), le mode de permission par défaut `auto` (`storage_migration.rs:83`), les archives Ollama par système, dont la variante ROCm sur Linux (`src-tauri/src/services/ollama_manager/release_source.rs:15-25`), et l'instance unique (`src-tauri/src/app_build.rs:59-60`).

---

## 02-installation/onboarding.md — DÉCALÉ

### Écart 1 — « dix fournisseurs » dans les Points à confirmer

- **Le brief dit** : « La limite de 32 fournisseurs affichés. Sans effet aujourd'hui puisqu'il y en a dix. »
- **Le code dit** : onze (`catalog_api.rs:46-179`). La marge reste large, l'observation tient, le chiffre est faux.

### Points à confirmer tranchés

- **La limite de 32 fournisseurs affichés** — **confirmée et bien à 32** : `src/components/onboarding/onboarding-api.tsx:39` — `items.filter((item) => item.category === "llm").slice(0, 32)`. Seule la catégorie `llm` est affichée, comme le dit le brief.
- **L'ordre des étapes** — **confirmé exactement tel que décrit** : bienvenue, préférences, import, fournisseur, puis Ollama en dernier et seulement si nécessaire (`src/components/onboarding/onboarding-screen.tsx:55-89`). L'enchaînement s'adapte : valider l'étape fournisseur termine le parcours quand l'étape Ollama n'est pas nécessaire (`onboarding-screen.tsx:47-53`).
- **La clé de stockage de la langue** — `clgo-language` confirmée (`src/components/onboarding/onboarding-preferences.tsx:23`, `src/components/settings/general-settings.tsx:81`, `src/i18n/index.ts:33`).

---

## 02-installation/mise-a-jour.md — DÉCALÉ

### Écart 1 — la contradiction sur le statut de la release est résolue, le brief la présente encore comme ouverte

- **Le brief dit** (Points à confirmer) : « `CLAUDE.md` affirme que la release CI est créée directement en non-draft. Le workflow `.github/workflows/release.yml` utilise pourtant `--draft`. L'une des deux affirmations est fausse. »
- **Le code dit** (`.github/workflows/release.yml:439-443`) : la release est créée sans aucun `--draft`, avec `--verify-tag --latest`. Et si une release en brouillon existe déjà pour ce tag, le workflow la bascule explicitement en publiée (`release.yml:436` : `gh release edit "$RELEASE_TAG" --draft=false --prerelease=false --latest`). **`CLAUDE.md` a raison, la contradiction n'existe plus.** Ce point doit être retiré, et l'exemple correspondant dans `00-comment-utiliser-ces-fichiers.md:74` est lui aussi périmé.

### Points à confirmer tranchés

- **La fréquence de vérification** — **confirmée à une heure** : `src/hooks/use-update-checker.ts:13` — `const CHECK_INTERVAL_MS = 60 * 60 * 1000;`, appliquée en `:112`.
- **Les fichiers cités existent tous** — `app_update.rs`, `app_update_install.rs`, `app_update_manifest.rs`, `app_update_source.rs`, `app_update_notes.rs`, `app_update_helper.rs`, `update_health.rs` sont bien présents dans `src-tauri/src/commands/` et `src-tauri/src/services/`, et le lot s'est étoffé (`app_update_assets.rs`, `app_update_download.rs`, `app_update_release.rs`, `app_update_source_validation.rs`, `update_handoff.rs`).

### Réserve

Le champ *Vérification* du brief dit lui-même que le processus n'a pas été relu ligne à ligne et s'appuie sur `CROSS-PLATFORM.md`. Cet audit n'a pas comblé cette lacune : les affirmations sur le retour arrière macOS et son absence sous Linux et Windows **restent non vérifiées**. C'est le sujet le plus coûteux en cas d'erreur ; il mérite une passe dédiée avant publication.

---

## 03-interface/vue-densemble.md — DÉCALÉ

### Écart 1 — seize onglets de réglages annoncés, dix-sept existent

- **Le brief dit** (§4) : « **Cinq sections, seize onglets.** […] Application | Conversations archivées, À propos ».
- **Le code dit** (`src/features/extension-ui/core-occupants.tsx:29-65`) : **dix-sept onglets**. La section Application en compte **trois** : **Mises à jour** (`core-occupants.tsx:61`, clé `settings.tabs.updates`), Conversations archivées (`:63`), À propos (`:65`). L'onglet « Mises à jour » est absent du brief. Les quatre autres sections sont exactes.

### Écart 2 — les onglets ne sont plus une liste figée

- **Le brief dit** : un tableau fixe de cinq sections et seize onglets.
- **Le code dit** (`src/components/settings/settings-sections.ts:33-40` et `:78-96`) : les sections sont construites à partir d'emplacements de slots, et une extension peut contribuer son propre onglet de réglages (`subTabFromOccupant`, branche `occupant.source.kind === "extension"`). Le nombre d'onglets dépend donc des extensions installées. La page doit dire « dix-sept onglets d'origine, davantage si vous installez des extensions ».

### Ce qui reste exact

Les quatre destinations de navigation — Sessions, Réveils, Personnalité, Réglages (`core-occupants.tsx:20-26`) — et le comportement du bouton de fermeture par système (`src-tauri/src/app_events.rs:13-19`).

---

## 03-interface/conversations-et-onglets.md — DÉCALÉ

Le fond du fichier est juste — les conversations ne s'ouvrent plus en onglets, seuls les clones en ont — mais il manque plusieurs fonctions qui existent aujourd'hui, et sept de ses huit incertitudes se tranchent par lecture du code.

### Écart 1 — l'épinglage n'est pas mentionné

- **Le brief dit** (Points à confirmer) : « **Les favoris.** Le README mentionne des conversations favorites ; aucun mécanisme correspondant n'a été relevé dans le code lu. Vérifier si la fonction existe encore. »
- **Le code dit** : la fonction existe, sous le nom d'**épinglage** — `pin_agent_session` et `unpin_agent_session` (`src-tauri/src/commands/agent_sessions.rs:197` et `:202`), avec un ordre propre aux conversations épinglées (`reorder_pinned_agent_sessions`, `:23`).

### Écart 2 — l'export Markdown d'une conversation n'est pas mentionné

- **Le brief dit** : rien.
- **Le code dit** : `export_agent_session_markdown` (`src-tauri/src/commands/agent_sessions.rs:207`). Une conversation s'exporte en Markdown ; c'est une fonction visible qui manque à la page.

### Écart 3 — le mode rapide (« fast mode ») n'est pas mentionné

- **Le brief dit** : le tableau « Ce qui est propre à chaque conversation » ne le liste pas.
- **Le code dit** : `set_session_fast_mode` (`src-tauri/src/commands/agent_sessions.rs:96`) est un réglage par conversation, au même titre que le modèle ou le mode de permission.

### Points à confirmer tranchés

- **Ce sur quoi porte la recherche ⌘G** — **sur le nom de la conversation et le nom du projet, jamais sur le contenu des messages** (`src/components/layout/search-dialog.tsx:41-51`). Les sous-agents et les clones sont exclus de la liste (`:36`). C'est décisif pour la rédaction : la page ne doit pas laisser croire à une recherche plein texte.
- **Renommer une conversation** — **possible** : `rename_agent_session` (`agent_sessions.rs:102`), refusée sur une session enfant.
- **Supprimer définitivement** — **possible** : `delete_agent_session` (`agent_sessions.rs:170`), en plus de l'archivage (`archive_agent_session`, `:177`).
- **Désarchiver** — **possible** : `restore_agent_session` (`agent_sessions.rs:192`).
- **Le tri de la barre latérale** — **manuel, et c'est une autorité unique**. `reorder_agent_sessions` (`agent_sessions.rs:13`) écrit dans `session-order.json` (`src-tauri/src/services/agent_local/session_order.rs:40`) ; les conversations elles-mêmes ne portent pas leur rang.
- **Le comportement au-delà de 2 000 messages** — **refus d'envoi**, ni troncature ni compression : `src-tauri/src/services/agent_local/conversation_admission.rs:151-153` retourne une erreur dès que `session.messages.len() >= MAX_MESSAGES_PER_SESSION`. La valeur 2 000 est confirmée (`src-tauri/src/services/agent_local/session_limits.rs:10`).
- **Les limites du tableau** — toutes confirmées : 2 000 messages (`session_limits.rs:10`), 3 onglets par groupe de clones (`session_tabs_state.rs:6`), 10 onglets de navigateur par conversation (`services/browser/session_types.rs:3`), 16 terminaux (`services/terminal/manager.rs:35`). S'y ajoutent deux bornes non citées : 4 096 fichiers de conversation et 32 Mo par fichier (`session_limits.rs:8-9`).

---

## 03-interface/cloner-une-conversation.md — DÉCALÉ

Le mécanisme décrit est exact ; un seul chiffre du tableau des limites ne correspond plus à ce que fait le code.

### Écart 1 — le délai de 180 secondes n'est pas ce que le brief décrit

- **Le brief dit** (§7, tableau) : « Délai de génération du résumé (modèle local) | **180 secondes** ».
- **Le code dit** : le résumé ne passe plus par un chemin Ollama avec son propre délai. Il passe par le transport LLM commun (`src-tauri/src/services/agent_local/clone_session.rs:145-151` : `llm::stream::collect_chat_silent_for_compression`), qui applique **180 secondes d'inactivité et 180 secondes de requête à tous les fournisseurs**, locaux comme distants (`src-tauri/src/services/llm/timeouts.rs:3-4`). Le nombre survit, sa portée a changé : ce n'est plus une particularité des modèles locaux. Aucune constante de 180 secondes propre au clonage n'existe plus dans le code.

### Ce qui reste exact

Les deux modes et leur refus sur le dernier message (`clone_session.rs:77-79`), l'interdiction de cloner une conversation de sous-agent (`:67-69`), l'autorisation explicite de cloner un clone avec son motif écrit en commentaire (`:63-66`), la suppression du clone en cas d'échec du résumé (`:99-104`), et toutes les autres limites : 3 072 jetons (`clone_session.rs:12`), 120 000 caractères analysés, 2 000 caractères par résultat d'outil, 200 fichiers suivis (`clone_summary.rs:5-7`), 64 ancêtres (`clone_roots.rs:4`), 3 onglets par groupe (`session_tabs_state.rs:6`).

---

## 03-interface/raccourcis-clavier.md — OBSOLÈTE

Le mécanisme décrit par ce fichier n'existe plus. Les raccourcis n'étaient auparavant écrits nulle part en un seul endroit : chaque gestionnaire testait ses touches lui-même et l'écran des réglages affichait sa propre liste, d'où le raccourci faux que le brief signalait. Le code a été refondu autour d'une table unique. **Ce fichier est à réécrire, pas à corriger.**

### Écart 1 — sept raccourcis annoncés, vingt existent

- **Le brief dit** (§2) : « **Sept raccourcis au total.** »
- **Le code dit** (`src/lib/app-shortcuts.ts:28-49`) : **vingt raccourcis**. Les sept du brief sont tous là et exacts ; s'y ajoutent treize absents :
  - `⌘,` — ouvrir les réglages (`app-shortcuts.ts:36`)
  - `⌘F` — chercher dans la conversation (`:37`)
  - `⌘L` — placer le curseur dans la zone de message (`:38`)
  - `⌘1` à `⌘9` — choisir un onglet de la conversation (`:39`)
  - `Maj+Tab` — changer le mode de permission (`:40`)
  - `Entrée` — envoyer (`:41`), `Maj+Entrée` — retour à la ligne (`:42`)
  - `Échap Échap` — arrêter la réponse (`:43`)
  - `⌘Entrée` — valider une modification (`:44`), `Échap` — l'annuler (`:45`)
  - `⌘+`, `⌘-`, `⌘0` — agrandir, réduire, réinitialiser le texte (`:46-48`)

### Écart 2 — l'avertissement en tête du fichier n'a plus d'objet

- **Le brief dit** : « **L'écran Réglages › Raccourcis de l'application affiche un raccourci faux.** Il annonce ⌥⌘J […]. Publier le raccourci du code, pas celui de l'écran des réglages. »
- **Le code dit** : le défaut est corrigé, et il l'est par construction. `src/lib/app-shortcuts.ts:26-27` porte le commentaire « Autorité unique : l'écran des réglages et tous les gestionnaires lisent les mêmes touches, pour qu'un changement ne puisse plus diverger côté interface », et `src/components/settings/shortcuts-settings.tsx:5` et `:22` affichent directement `APP_SHORTCUTS`. Le raccourci d'affichage de la prévisualisation est bien `⌥⌘B` (`app-shortcuts.ts:35`). **Tout le bloc « Avertissement au rédacteur » est à supprimer.**

### Écart 3 — la référence de code citée n'existe plus sous cette forme

- **Le brief dit** : « Le code implémente ⌥⌘B (`use-agent-local-shortcuts.ts:29` : `mod && event.altKey && event.code === "KeyB"`). »
- **Le code dit** : cette ligne n'existe plus. Le gestionnaire appelle `matchesAppShortcut(event, "togglePreview")` (`src/hooks/use-agent-local-shortcuts.ts:24`), et le même gestionnaire de mise en page appelle `matchesAppShortcut` pour ses six actions (`src/components/layout/use-app-layout-effects.ts:89-95`).

### Écart 4 — la portée du raccourci de prévisualisation

- **Le brief dit** (Tableau) : « Afficher ou masquer la prévisualisation | ⌥⌘B | Conversation active uniquement ».
- **Le code dit** : c'est exact, mais l'ordre des tests a changé — la prévisualisation est traitée **avant** le terminal et sort immédiatement (`use-agent-local-shortcuts.ts:26-30`), et les deux sont ignorés sans conversation active ou dans un champ de saisie (`:25`). Le comportement décrit tient ; la mention « le gestionnaire ignore volontairement ⌘B quand Alt est enfoncé » est désormais assurée par la comparaison des modificateurs de la table (`app-shortcuts.ts:73-78`), pas par un test ad hoc.

### Points à confirmer tranchés

- **Les raccourcis sont-ils personnalisables ?** — **non**. `APP_SHORTCUTS` est une constante figée (`app-shortcuts.ts:28`) et l'écran des réglages ne fait que l'afficher, sans champ de saisie (`shortcuts-settings.tsx:22`).
- **Existe-t-il d'autres raccourcis non recensés ?** — **oui, treize**, listés à l'écart 1. Le brief avait raison de poser la question ; la réponse est dans la table unique.

---

## 03-interface/terminal-integre.md — DÉCALÉ

### Écart 1 — la neutralisation du mode vi n'existe plus

- **Le brief dit** (§3) : « Détail à mentionner pour les utilisateurs de zsh : Beaver neutralise le passage en mode vi que déclenche une variable `EDITOR` contenant « vi ». »
- **Le code dit** : c'est l'inverse aujourd'hui. `EDITOR` est **transmis tel quel**, et un test le verrouille explicitement — `src-tauri/src/services/terminal/shell_environment_tests.rs:50-55` pose `EDITOR=/usr/bin/vim` et vérifie que la commande le conserve. Aucune référence à `EDITOR` n'existe hors des tests dans `src-tauri/src/services/terminal/`.
- **Ce qui l'a remplacé** : une autorité unique de l'environnement du shell (`src-tauri/src/services/terminal/shell_environment.rs`). Elle **retire** six variables posées par le lanceur — `NO_COLOR`, `NODE_DISABLE_COLORS`, `FORCE_COLOR`, `COLOR`, `CLICOLOR`, `CLICOLOR_FORCE` (`:29-35`) — et **impose** `TERM=xterm-256color` et `COLORTERM=truecolor` (`:23`). Le motif est écrit dans le fichier : sans cela, un lanceur sans couleur rendait la sortie de `vite`, `cargo` et la CLI Tauri uniformément grise. C'est ce comportement-là qui mérite le paragraphe.

### Écart 2 — « 16 terminaux » recouvre deux limites différentes

- **Le brief dit** (§5) : « Terminaux ouverts simultanément | **16** », présenté comme une limite unique, et en Points à confirmer : « La limite de seize est-elle globale ou par conversation ? »
- **Le code dit** : deux limites distinctes portent la valeur 16.
  - `MAX_PTY_SESSIONS = 16` (`src-tauri/src/services/terminal/manager.rs:35`) — les **processus** de terminal réellement vivants, **globalement**.
  - `MAX_TABS_PER_GROUP = 16` (`src-tauri/src/services/terminal/limits.rs:6`) — les **onglets** enregistrés par groupe, avec en plus `MAX_GROUPS = 128` et `MAX_TOTAL_TABS = 256` (`limits.rs:5` et `:7`).
  - La question du brief se tranche donc ainsi : **seize processus au total**, seize onglets par conversation, deux cent cinquante-six onglets au total.

### Points à confirmer tranchés

- **La limite de seize est globale** — voir ci-dessus, `manager.rs:35`.
- **Le shell sous Windows** — `powershell.exe` est résolu par un chemin système validé, sans réglage pour préférer `cmd.exe` ou PowerShell 7 (`src-tauri/src/services/terminal/pty_session_windows.rs:162-164`).

### Ce qui reste exact

`$SHELL` avec repli `/bin/bash` et validation du chemin (`src-tauri/src/services/terminal/shell_helper.rs:73-74`), l'écriture plafonnée à 65 536 octets (`terminal/limits.rs:1` : `MAX_PTY_WRITE_BYTES = 64 * 1024`), les onglets conservés dans `terminal-tabs.json` (`terminal/tab_store.rs:181`), et le jeton de session comparé en temps constant (`terminal/mod.rs:92-95`).

---

## 03-interface/mascotte.md — DÉCALÉ

### Écart 1 — la mascotte est désactivée par défaut, le brief la présente comme active

- **Le brief dit** (§5) : deux réglages seulement, le personnage et la taille, et en Points à confirmer : « **Peut-on désactiver complètement la mascotte ?** Aucun réglage d'activation n'a été repéré. »
- **Le code dit** : un réglage d'activation existe et vaut **faux par défaut** — `src/services/mascot.ts:12` (`enabled: false` dans l'état par défaut), champ déclaré en `:29`, accepté en écriture en `:56`, relu en `:83`. **La mascotte ne s'affiche donc pas tant que l'utilisateur ne l'active pas.** C'est le fait le plus important de la page, et il l'inverse : elle ne dit pas comment enlever la mascotte, elle doit dire comment l'allumer.

### Points à confirmer tranchés

- **Peut-on la désactiver ?** — oui, c'est même l'état par défaut (`src/services/mascot.ts:12`).
- **Les bornes de la taille** — **70 % à 140 %** (`src/services/mascot.ts:8-9`), valeur ramenée dans l'intervalle si elle en sort (`:104`).
- **Le comportement au repos et les états** — **huit animations** : `Idle`, `Thinking`, `ExploreBook`, `WorkLaptop`, `Waiting`, `Success`, `Failed`, `Alert` (`src-tauri/src/services/mascot/activity.rs:9-17`). Une durée non citée par le brief s'y ajoute : l'alerte dure **1,8 seconde** (`src-tauri/src/services/mascot/event_mapping.rs:7`).

### Ce qui reste exact

Les huit personnages et leur identifiant, castor `cl-go-beaver` en tête (`src/components/settings/mascot-settings.tsx:25-62`), le suivi de trente-deux conversations (`src-tauri/src/services/mascot/activity.rs:5`), 2,2 secondes pour la réussite et 2,6 pour l'échec (`src-tauri/src/services/mascot/event_mapping.rs:5-6`).

---

# Fichiers À JOUR — ce qui a été revérifié

## 02-installation/prerequis.md

Toutes les valeurs tiennent. `install.sh` n'appelle bien que `apt-get` (`install.sh:153`) et n'accepte que `Darwin:arm64`, `Darwin:aarch64`, `Linux:x86_64`, `Linux:amd64` (`install.sh:167-170`) : l'avertissement Fedora/RHEL en tête du fichier reste **entièrement valide**.

**Points tranchés** — la version minimale de macOS n'est toujours pas déclarée : le bloc `macOS` de `src-tauri/tauri.conf.json:69-80` ne contient pas `minimumSystemVersion`. L'installeur Windows est en mode « utilisateur courant » (`tauri.conf.json:64-67` : `"installMode": "currentUser"`), ce qui explique le défaut `%LOCALAPPDATA%\Beaver` et retire le besoin de droits administrateur.

**Reste ouvert** — Fedora/RHEL, WebView2 et VC++ sur machine vierge, GPU AMD sous Linux, fenêtre transparente sous Linux : aucun ne se tranche par lecture de code, tous demandent un essai sur machine.

## 02-installation/installation-macos.md

Chaque étape du script est confirmée : architecture (`install.sh:167-170`), 512 Kio pour l'API et 2 Gio pour l'asset (`install.sh:5`), trois redirections au maximum vers l'hôte autorisé (`install.sh:40-44`), vérification du bundle — `Info.plist`, `Contents/MacOS/cl-go-dash`, identifiant `com.clgo.dash`, absence de liens symboliques (`install.sh:109-116`), copie vers un dossier temporaire à nom aléatoire puis déplacement (`install.sh:129-138`), refus si `Beaver.app` **ou** `CL-GO.app` existe (`install.sh:107` et `:134-135`), et validation du dossier saisi — 1 024 caractères, chemin absolu, pas de caractère de contrôle, pas de segment `..` (`install.sh:102-106`), avec développement de `~` (`install.sh:122`).

**Point tranché** — la version minimale de macOS n'est toujours pas définie (`tauri.conf.json:69-80`).

## 02-installation/installation-linux.md

Confirmé : refus si le paquet `beaver` ou `cl-go` est installé (`install.sh:144-145`), contrôle des six métadonnées du `.deb` — `Package=beaver`, `Architecture=amd64`, version identique, `Provides`/`Conflicts`/`Replaces` valant `cl-go` (`install.sh:146-151`), installation par `apt-get install -y` (`install.sh:153`), vérification que `/usr/bin/cl-go-dash` existe et n'est pas un lien (`install.sh:154`).

**Points tranchés** — le nom du binaire est bien `/usr/bin/cl-go-dash` (`install.sh:154`) ; l'avertissement Fedora/RHEL reste fondé (aucune occurrence de `dnf` dans `install.sh`).

## 02-installation/installation-windows.md

Confirmé : `AMD64` exigé (`install.ps1:141`), TLS 1.2 et redirections non suivies automatiquement (`install.ps1:143-145`), 512 Kio pour l'API et 64 Kio pour le manifeste (`install.ps1:5`), nom d'asset `Beaver_<version>_x64-setup.exe` exactement (`install.ps1:98`), taille comparée deux fois puis SHA-256 comparé au manifeste (`install.ps1:160-166`), défaut `%LOCALAPPDATA%\Beaver` (`install.ps1:166-168`), installeur silencieux `/S /D=` en fenêtre masquée (`install.ps1:175`), vérification que `cl-go-dash.exe` existe et n'est pas un point d'analyse (`install.ps1:178-179`), et contraintes du chemin — `X:\`, 1 024 caractères, caractères `* ? < > | "` interdits, deux-points interdits au-delà des trois premiers caractères, aucun segment `..` (`install.ps1:133-138`).

**Points tranchés** — le script Windows ne contient effectivement **aucun** contrôle d'installation existante, contrairement à `install.sh` (comparer `install.ps1:140-181` et `install.sh:107`, `:144-145`). La migration depuis `%APPDATA%\cl-go-dash` est **toujours active** (`src-tauri/src/storage_migration.rs:49-53`), et le dossier de données sous Windows est bien `~/.local/share/cl-go-dash` (`src-tauri/src/services/paths.rs:9-14`).

## 02-installation/import-depuis-un-autre-assistant.md

**Le fichier le plus solide des trois sections.** Les onze limites du tableau sont exactes au chiffre près (`src-tauri/src/services/agent_import/limits.rs:1-12`), y compris les 256 Ko du manifeste qui viennent d'un autre fichier (`src-tauri/src/services/skill_manifest_policy.rs:3`) et le mégaoctet du registre (`src-tauri/src/services/agent_import/registry.rs:10`). Les neuf sources, leurs chemins et leurs particularités sont confirmés (`source_specs.rs:49-101`) : Hermes sans document (`:56-63`), OpenClaw et ses espaces de travail avec le premier `AGENTS.md` trouvé (`:76-85` et `source_specs.rs:37-40`), OpenCode sous `$XDG_CONFIG_HOME/opencode` (`source_specs.rs:22`), Kimi sous `$KIMI_CODE_HOME` avec trois emplacements complémentaires (`source_specs.rs:23`, `:91-99`). Les sauvegardes vont bien dans `agent-import-backups/` avec cinq versions conservées (`agent_import/document_io.rs:44-57` et `:88`), et le registre est `external-agent-sources.json` (`agent_import/documents.rs:24`).

Une précision utile pour la rédaction, non citée par le brief : les espaces de travail OpenClaw sont plafonnés à **trois** (`agent_import/source_paths.rs:4`).

## 03-interface/panneau-lateral.md

Les trois modes sont confirmés (`src/types/forecast-panel.ts:2`), le mode par défaut est bien la prévisualisation (`src/types/forecast-panel.ts:17`), et le navigateur disparaît du sélecteur sur Linux au lieu d'apparaître en panne (`src/components/agent-local/mode-selector.tsx:97`, condition `browserStatus !== "hidden"`).

**Points tranchés** — le mode est mémorisé **par conversation** : il vit dans l'espace de travail de la session (`src/hooks/use-forecast-panel.ts:14` et `:47-48`, champ `panelMode` de `workspace`). Le raccourci ⌥⌘B est confirmé (`src/lib/app-shortcuts.ts:35`) ; en revanche l'avertissement du §3 sur l'écran des réglages qui annoncerait un raccourci faux **est périmé** — voir le fichier `raccourcis-clavier.md` ci-dessus.

## 03-interface/navigateur-integre.md

Toutes les limites sont exactes : dix onglets (`src-tauri/src/services/browser/session_types.rs:3`), titres à 80 caractères (`session_types.rs:5`), 2 048 caractères d'URL et schémas `http`/`https` seuls (`services/browser/url_policy.rs:1` et `:29`), 128 Ko par fichier de session (`services/browser/session_store.rs:7`, avec en plus 64 Ko de contenu déchiffré, `:8`), 256 Mo pour le magasin de cookies (`services/browser/cookie_store_probe.rs:10`), et pour la détection de serveurs locaux : 128 candidats, 32 résultats, 64 Ko lus, 80 caractères de titre (`services/browser/local_site_types.rs:3-6`), 8 sondages simultanés (`services/browser/local_site_scanner.rs:12`), redirections bornées et restreintes aux adresses locales (`services/browser/local_site_probe.rs:72-79`).

**Points tranchés** — les **64 sessions sont bien globales** : `MAX_LIVE_BROWSER_SESSIONS = 64` dans un registre unique (`services/browser/live_session_registry.rs:3` et `:18`). Le **comportement sur Linux est un masquage propre**, pas une erreur : `BrowserCapability::Hidden` (`services/browser/cef_runtime_policy.rs:72-75`). Le magasin de cookies de 256 Mo est vérifié sur macOS seulement, la sonde étant compilée sous `cfg(any(test, target_os = "macos"))` (`services/browser/cef_cookie_gate_cleanup.rs:4`).

## 03-interface/arbre-de-fichiers-et-previews.md

Toutes les valeurs sont exactes : 2 Mo pour le texte, 4 096 caractères de chemin, 500 vérifications d'existence par appel (`src-tauri/src/commands/file_preview.rs:6-8`) ; 50 Mo pour les documents et les tableurs, 500 lignes par défaut et 5 000 au maximum (`src-tauri/src/commands/file_preview_office.rs:4-7`) ; les extensions de tableur `csv`, `tsv`, `xlsx`, `xls`, `ods`, `xlsm` (`file_preview_office.rs:55-63`) ; les documents limités à `docx` et `pdf` (`file_preview_office.rs:71` : `BINARY_EXTENSIONS`).

**Point tranché** — le regroupement de **200 ms s'applique bien aux deux surveillances** : `src-tauri/src/commands/file_tree_watcher.rs:34` pour l'arborescence de travail et `src-tauri/src/services/file_watcher.rs:15` pour les fichiers de configuration. Ce dernier borne en outre à 256 le nombre de chemins regroupés (`file_watcher.rs:16`).

## 03-interface/langues.md

Les deux listes sont exactes : sept langues d'interface (`src/components/settings/general-settings-options.ts:14-22`) et les sept mêmes plus l'option vide « — » pour les réponses (`:24-33`). La clé `clgo-language` est confirmée aux trois endroits où elle est écrite ou lue (`src/components/onboarding/onboarding-preferences.tsx:23`, `src/components/settings/general-settings.tsx:81`, `src/i18n/index.ts:33`).

**Points tranchés** — **la langue par défaut est l'anglais, sans détection du système** : `src/i18n/index.ts:33` retourne `"en"` quand rien n'est stocké. Le stockage est bien côté navigateur (`localStorage`, même ligne) et non dans le dossier de données : le réglage **ne survit donc pas** à une réinstallation qui vide les données du webview, ce que la page peut dire franchement.

**À corriger sans que ce soit un écart de fond** — les tailles de fichiers citées en Points à confirmer ont vieilli : le français fait aujourd'hui 143 Ko, le japonais 158 Ko, le chinois 123 Ko. L'écart entre langues persiste ; la question de la complétude des traductions reste ouverte.

## 03-interface/themes-et-apparence.md

Les trois tableaux sont exacts. Six thèmes plus l'option Système, avec leur classement clair/sombre : `light` clair, `dark` sombre, `emerald-night` sombre, `cobalt-frost` **clair**, `astral-mist` sombre, `crimson-eclipse` sombre (`src/lib/app-themes.ts:1-30`). Le mode Système ne bascule qu'entre `dark` et `light` (`app-themes.ts:63-66` : `resolveTheme`). Taille du texte de 10 à 24 pixels, défaut 18, valeur ramenée dans l'intervalle et repli sur 18 en cas de saisie invalide (`src/hooks/use-settings.ts:3-5` et `:43-56`). Sept polices, dont quatre à chasse fixe et deux manuscrites (`use-settings.ts:17-25`). Cinq thèmes de coloration du code, chacun en variante claire et sombre choisie automatiquement (`use-settings.ts:29-39`, avec le motif écrit en commentaire aux lignes 29-32).

**Points tranchés** — **les thèmes ne sont pas personnalisables par l'utilisateur**, mais une **extension peut en apporter un** : `src/lib/app-themes.ts:35` définit `ExtensionThemeChoice` et `isThemeChoice` (`:47-52`) accepte les choix contribués. La formulation de la page doit en tenir compte. Les noms des thèmes passent par des clés de traduction (`app-themes.ts`, champs `labelKey`) : vérifier à l'écran ce que donne `settings.emeraldNight` en français avant de figer le vocabulaire de la page.

---

# Points à confirmer tranchés — récapitulatif

| Fichier | Question du brief | Réponse | Référence |
|---|---|---|---|
| local-vs-cloud | Le compte web Kimi est-il toujours expérimental ? | Oui | `services/oauth_providers/mod.rs:57` |
| local-vs-cloud | Trois comptes web ? | Oui : OpenAI/Codex, xAI, Moonshot | `llm/route_profile/catalog_oauth.rs:28-56` |
| local-vs-cloud | Identifiants `google`/`gemini`, `zai`/`glm` | Sans incidence ; collision `google` documentée | `forecast/catalog.rs:1-7` |
| tour | Sept langues et six thèmes réellement proposés ? | Oui, tous | `general-settings-options.ts:14-22`, `app-themes.ts:1-30` |
| tour | Navigateur Linux : limite définitive ? | Oui, par conception, état masqué | `browser/cef_runtime_policy.rs:72-75` |
| tour | Neuf sources d'import ? | Oui, plafonnées à neuf | `agent_import/source_specs.rs:49-101`, `limits.rs:1` |
| tour | Combien de familles Forecast ? | Dix locales + TimeGPT cloud | `forecast/catalog_specs/mod.rs:16-41` |
| prerequis / macos | Version minimale de macOS déclarée ? | Non, toujours absente | `tauri.conf.json:69-80` |
| prerequis | Mode d'installation Windows | Utilisateur courant, sans administrateur | `tauri.conf.json:64-67` |
| linux | Nom du binaire | `/usr/bin/cl-go-dash` | `install.sh:154` |
| linux | Fedora/RHEL dans le code ? | Non, `apt-get` uniquement | `install.sh:153` |
| windows | Migration `%APPDATA%` encore active ? | Oui | `storage_migration.rs:49-53` |
| windows | Refus d'installation existante côté Windows ? | Non, aucun contrôle | `install.ps1:140-181` |
| premier-lancement | Rôle du dossier `inbox/` | `pending.json` + `idea-discovery.md`, un des quatre fichiers de personnalité | `storage_migration.rs:88`, `:92-97`, `:110` |
| premier-lancement | Bouton de fermeture par système | Masque sur macOS, quitte ailleurs | `app_events.rs:13-19` |
| onboarding | Limite de 32 fournisseurs affichés | Confirmée, catégorie `llm` seule | `onboarding-api.tsx:39` |
| onboarding | Ordre des étapes | Conforme au brief, Ollama en dernier | `onboarding-screen.tsx:55-89` |
| mise-a-jour | Release CI en brouillon ? | Non, publiée directement | `.github/workflows/release.yml:436-443` |
| mise-a-jour | Fréquence de vérification | Une heure | `use-update-checker.ts:13` |
| panneau-lateral | Mode mémorisé par conversation ? | Oui | `use-forecast-panel.ts:14`, `:47-48` |
| conversations | Portée de la recherche ⌘G | Nom de conversation et nom de projet, jamais le contenu | `search-dialog.tsx:41-51` |
| conversations | Renommer une conversation ? | Oui | `commands/agent_sessions.rs:102` |
| conversations | Supprimer définitivement ? | Oui | `commands/agent_sessions.rs:170` |
| conversations | Désarchiver ? | Oui | `commands/agent_sessions.rs:192` |
| conversations | Les favoris existent-ils encore ? | Oui, sous forme d'épinglage | `commands/agent_sessions.rs:197`, `:202` |
| conversations | Tri de la barre latérale | Manuel, autorité `session-order.json` | `agent_local/session_order.rs:40` |
| conversations | Au-delà de 2 000 messages ? | Envoi refusé | `conversation_admission.rs:151-153` |
| raccourcis | Personnalisables ? | Non, table figée | `app-shortcuts.ts:28`, `shortcuts-settings.tsx:22` |
| raccourcis | D'autres raccourcis non recensés ? | Oui, treize | `app-shortcuts.ts:36-48` |
| navigateur | 64 sessions : global ou par conversation ? | Global | `browser/live_session_registry.rs:3` |
| navigateur | Comportement sur Linux | Masquage propre, pas d'erreur | `browser/cef_runtime_policy.rs:72-75` |
| navigateur | Magasin de cookies | 256 Mo, sonde macOS seulement | `browser/cookie_store_probe.rs:10` |
| terminal | Limite de seize : globale ? | Oui, processus ; 16 onglets par groupe, 256 au total | `terminal/manager.rs:35`, `terminal/limits.rs:5-7` |
| terminal | Choix du shell Windows ? | Non, `powershell.exe` en dur | `terminal/pty_session_windows.rs:162-164` |
| fichiers | Le délai de 200 ms couvre-t-il les deux surveillances ? | Oui | `file_tree_watcher.rs:34`, `file_watcher.rs:15` |
| langues | Langue par défaut au premier lancement | Anglais, sans détection système | `i18n/index.ts:33` |
| langues | Stockage du choix de langue | `localStorage`, ne survit pas à un vidage des données | `i18n/index.ts:33` |
| themes | Thèmes personnalisables ? | Pas par l'utilisateur, mais par une extension | `app-themes.ts:35`, `:47-52` |
| mascotte | Peut-on la désactiver ? | Oui, et elle est éteinte par défaut | `services/mascot.ts:12` |
| mascotte | Bornes de la taille | 70 % à 140 % | `services/mascot.ts:8-9` |
| mascotte | États | Huit animations, alerte à 1,8 s | `mascot/activity.rs:9-17`, `mascot/event_mapping.rs:7` |
| concepts-cles | Liste des dix fournisseurs à recouper | Onze, Groq retiré | `catalog_api.rs:46-179`, `api_keys_retired.rs:1` |

---

# Points restant ouverts, et pourquoi

## Ce qui ne se tranche que sur une machine

Aucune lecture de code ne peut y répondre. Ils demandent un essai réel, et resteront ouverts jusque-là.

- **Fedora, RHEL et les distributions hors famille Debian** (`prerequis`, `installation-linux`) — la décision est produit, pas technique : ajouter le support ou corriger `CROSS-PLATFORM.md`. Le code, lui, est sans ambiguïté (`install.sh:153`).
- **WebView2 et le runtime Visual C++ sur une machine Windows vierge** (`prerequis`, `installation-windows`) — l'installeur NSIS ne les mentionne pas (`tauri.conf.json:64-67`), mais l'absence d'une déclaration ne prouve pas l'absence du composant : Tauri peut en inclure un amorceur. Seul un essai tranche.
- **Le GPU AMD sous Linux et sous Windows** (`prerequis`, `installation-linux`, `installation-windows`) — l'archive ROCm est bien sélectionnée (`ollama_manager/release_source.rs:22-25`), la suite est inconnue.
- **La fenêtre au fond transparent sous Linux** (`prerequis`, `installation-linux`) — à reproduire avant de pouvoir écrire quoi que ce soit.
- **La désinstallation complète sur les trois systèmes** (`installation-macos`, `installation-linux`, `installation-windows`) — rien dans le dépôt ne la décrit. Le point le plus sensible reste la clé maîtresse laissée dans le trousseau après désinstallation.
- **Le retour arrière des mises à jour** (`mise-a-jour`) — l'affirmation « macOS restaure seul, Linux et Windows non » vient de `CROSS-PLATFORM.md` et n'a été vérifiée ni dans le code ni sur machine. C'est l'affirmation la plus coûteuse en cas d'erreur de toute la section 02.

## Ce qui se tranche à l'écran, lors de la passe d'interface prévue

Conformément à `00-comment-utiliser-ces-fichiers.md:93-102`, ces points ne sont pas bloquants.

- La position et l'aspect du sélecteur de mode du panneau latéral, les seuils de repli de la barre latérale, les largeurs minimales (`panneau-lateral`, `vue-densemble`).
- Les libellés français des deux modes de clonage (le code les nomme `Cut` et `Summary`), la présentation du champ d'axe de résumé, la lisibilité du résumé caché (`cloner-une-conversation`).
- Les noms affichés des huit personnages de la mascotte et leur apparence (`mascotte`).
- Les noms des thèmes tels qu'ils s'affichent en français (`themes-et-apparence`).
- La liste complète des éléments de la barre d'outils et le comportement de la barre latérale par section (`vue-densemble`).
- Ce qui reste en anglais dans l'interface (`langues`) — l'établir par observation plutôt que par déduction.

## Ce qui demande une lecture de code que cet audit n'a pas faite

Volontairement écarté du périmètre : ces questions relèvent des sections 04 à 07, auditées en parallèle.

- Le réimport, l'annulation d'un import, les conflits de noms de skills, le sort du document d'instructions quand plusieurs sources en fournissent un (`import-depuis-un-autre-assistant`) — six questions qui appellent une lecture de `agent_import/documents.rs` et `discovery.rs`, et probablement un essai.
- L'emplacement exact de l'assistant d'import dans les réglages (`onboarding`, `import-depuis-un-autre-assistant`) — le tableau de correspondance annonce « Réglages › Extensions », ce qui est douteux maintenant que l'onglet Extensions concerne les modules installables.
- Le comportement du changement de modèle en cours de conversation quand la nouvelle fenêtre de contexte est plus petite que l'historique (`local-vs-cloud`) — relève du transport LLM et de la compression.
- La couverture réelle des informations d'usage par fournisseur (`local-vs-cloud`) — relève de la section 06.
- Les téléchargements et le nettoyage des cookies du navigateur intégré (`navigateur-integre`).
- La possibilité de désactiver la vérification automatique des mises à jour (`mise-a-jour`).
- Le rôle du dossier `translations/` créé au premier lancement (`premier-lancement`) — toujours non documenté.
- La taille réelle du téléchargement d'Ollama par système (`premier-lancement`) — les noms d'archives sont connus (`ollama_manager/release_source.rs:15-25`), leur poids non.

## Décisions éditoriales, non techniques

Elles n'appartiennent pas au code et ne se trancheront jamais par lecture.

- Le nom français de « worktree », et le choix entre « fournisseur » et « provider » (`concepts-cles`).
- Le mot retenu entre « cloner », « forker » et « brancher » (`cloner-une-conversation`).
- La version de l'application affichée sur le site et sa mise à jour (`presentation`).
- L'adresse de contact commercial à exposer publiquement (`presentation`).

---

# Deux remarques hors périmètre

**1. Le fichier `00-comment-utiliser-ces-fichiers.md` contient un exemple périmé.** Sa ligne 74 cite comme erreur relevée : « `CLAUDE.md` affirme que la release CI est publiée directement, alors que le workflow la crée en brouillon ». Le workflow crée aujourd'hui la release directement en publiée (`.github/workflows/release.yml:439-443`) et bascule en publiée toute release trouvée en brouillon (`:436`). Cet exemple, qui sert à illustrer la hiérarchie des sources, ne tient plus. Le fichier 00 n'étant pas dans mon périmètre, je le signale sans le corriger.

**2. Le retrait de Groq touche plus que deux tableaux.** Le fournisseur apparaît encore dans le champ *Vérification* de `local-vs-cloud.md` et dans son tableau 2, mais aussi potentiellement dans les sections 06 et 07 que d'autres relisent. Une recherche du mot « Groq » sur l'ensemble de `docs/documentation-site-web/` avant publication éviterait qu'il en survive une occurrence.
