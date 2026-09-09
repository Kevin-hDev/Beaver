# Audit de fraîcheur — sections 06, 07 et fichiers transverses

**Date** — 9 septembre 2026
**Périmètre** — `06-modeles/` (9 fichiers), `07-integrations/` (4 fichiers), `differents-points-a-traiter.md`, `_geles/README.md`, `_geles/plan-mode.md.gele` — **16 fichiers**
**Version de l'application au moment de l'audit** — **1.2.2** (`package.json`, `src-tauri/Cargo.toml`)
**Méthode** — chaque affirmation chiffrée ou nominative a été relue dans le code. Aucun verdict n'est déduit sans lecture. Les briefs n'ont pas été modifiés.

---

## Tableau récapitulatif

| Fichier | Verdict | Écarts | Points à confirmer tranchés / restants |
|---|---|---|---|
| `06-modeles/catalogue-et-favoris.md` | DÉCALÉ | 1 | 2 tranchés / 3 restants |
| `06-modeles/materiel-et-vram.md` | DÉCALÉ | 2 | 2 tranchés / 3 restants |
| `06-modeles/ollama-modeles.md` | DÉCALÉ | 1 | 2 tranchés / 3 restants |
| `06-modeles/ollama-personnalisation.md` | À JOUR | 0 | 1 tranché / 4 restants |
| `06-modeles/ollama-runtime.md` | **OBSOLÈTE** | 6 | 2 tranchés / 2 restants |
| `06-modeles/providers-api.md` | **DÉCALÉ (grave)** | 5 | 3 tranchés / 2 restants |
| `06-modeles/providers-comptes-web.md` | À JOUR | 0 | 2 tranchés / 3 restants |
| `06-modeles/raisonnement.md` | DÉCALÉ | 2 | 2 tranchés / 3 restants |
| `06-modeles/usage-et-couts.md` | À JOUR | 0 | 1 tranché / 4 restants |
| `07-integrations/channels-gateway.md` | À JOUR | 0 | 2 tranchés / 4 restants |
| `07-integrations/mcp-connecteurs.md` | À JOUR | 0 | 2 tranchés / 4 restants |
| `07-integrations/mcp-oauth.md` | À JOUR | 0 | 1 tranché / 4 restants |
| `07-integrations/recherche-web.md` | DÉCALÉ | 3 | 3 tranchés / 2 restants |
| `differents-points-a-traiter.md` | DÉCALÉ | 6 | voir la liste dédiée |
| `_geles/README.md` | DÉCALÉ | 1 | — |
| `_geles/plan-mode.md.gele` | DÉCALÉ | 3 | 1 tranché / 5 restants |

**Total : 16 fichiers, 16 verdicts.** Trois fichiers portent des écarts graves : `ollama-runtime.md`, `providers-api.md`, `differents-points-a-traiter.md`.

---

# Fichiers DÉCALÉS ou OBSOLÈTES

## 1. `06-modeles/providers-api.md` — DÉCALÉ (grave)

Le catalogue des fournisseurs a changé de composition **et** de fichier d'autorité. C'est l'écart le plus visible du périmètre : le titre de la page, son tableau principal et son fichier source sont tous les trois faux.

### Écart 1 — « Les dix fournisseurs » : il y en a onze

Le brief annonce dix fournisseurs (titre de section, plan de page, tableau).

Réalité : **onze** profils déclarés dans `src-tauri/src/services/llm/route_profile/catalog_api.rs:42-191`, et confirmés côté interface dans `src/i18n/fr.json:41-52`.

### Écart 2 — Groq a été retiré du produit

Le brief liste **Groq** en tête de son tableau (`providers-api.md:37`).

Réalité : Groq n'existe plus comme fournisseur. Il a été retiré, avec une procédure de nettoyage dédiée :
- `src-tauri/src/services/api_keys_retired.rs:1` — `const RETIRED_PROVIDER_ID: &str = "groq"` ;
- `src-tauri/src/services/api_keys_retired.rs:43-44` — un marqueur `.retired-provider-groq-v1` et une sauvegarde `secrets.enc.pre-groq-removal.bak` accompagnent le retrait de la clé du coffre ;
- trois tests interdisent son retour : `src-tauri/src/services/llm/route_tests.rs:42-43`, `src-tauri/src/services/api_keys_validate_tests.rs:26-27`, `src-tauri/src/commands/heartbeat_validation.rs:168-172`.

Le nom « groq » subsiste uniquement comme préfixe de modèles revendus par OpenRouter (`src-tauri/src/services/llm/route_profile/tool_policies.rs:17-19`) — ce n'est pas un fournisseur configurable.

### Écart 3 — Deux fournisseurs manquent : Anthropic et Qwen

| Fournisseur | Nom affiché | Où créer la clé | Référence |
|---|---|---|---|
| **Anthropic Claude** | `Anthropic Claude` | `console.anthropic.com/settings/keys` | `catalog_api.rs:163-175` |
| **Qwen** | `Qwen` | `modelstudio.console.alibabacloud.com` | `catalog_api.rs:176-190` |

Anthropic n'utilise pas le même format d'authentification que les autres : en-tête `x-api-key` et en-tête de version `anthropic-version: 2023-06-01` (`catalog_api.rs:17-25`). Qwen n'a pas d'adresse fixe : son point d'entrée est résolu à l'exécution depuis la connexion configurée (`catalog_api.rs:183-185`).

### Écart 4 — Le fichier source cité n'est plus l'autorité

Le brief cite `services/llm/catalog.rs` comme source (`providers-api.md:5`).

Réalité : `src-tauri/src/services/llm/catalog.rs:1` porte désormais le commentaire « Vue publique du catalogue, dérivée de l'autorité privée `route_profile` ». Ce fichier ne contient plus aucune donnée de fournisseur : il ne fait que transformer les profils de `route_profile/catalog_api.rs`. **L'autorité à citer pour une revérification future est `services/llm/route_profile/catalog_api.rs`.**

### Écart 5 — Le tableau des adresses est incomplet et contient une erreur

Adresses actuellement dans le code (`catalog_api.rs`, champ `catalog: public(...)`) :

| Fournisseur | Adresse dans le code | Ligne | État du brief |
|---|---|---|---|
| Google Gemini | `aistudio.google.com/app/apikey` | 55 | correct |
| Mistral | `console.mistral.ai/api-keys` | 68 | correct |
| Cerebras | `cloud.cerebras.ai/` | 81 | correct |
| OpenRouter | `openrouter.ai/settings/keys` | 94 | correct |
| OpenAI | `platform.openai.com/api-keys` | 107 | correct |
| DeepSeek | `platform.deepseek.com/api_keys` | 120 | correct |
| xAI | `console.x.ai` | 133 | correct |
| Moonshot Kimi | `platform.kimi.ai/console/api-keys` | 146 | correct |
| Z.ai GLM | `z.ai/manage-apikey/apikey-list` | 159 | correct |
| **Anthropic Claude** | `console.anthropic.com/settings/keys` | 172 | **absent du brief** |
| **Qwen** | `modelstudio.console.alibabacloud.com/` | 187 | **absent du brief** |
| ~~Groq~~ | — | — | **à retirer du brief** |

### Ce qui reste juste dans ce fichier

Toute la partie sécurité est confirmée et n'a pas bougé : coffre chiffré, clé maîtresse dans le trousseau de l'OS, aucune commande de lecture exposée à l'interface. Le tableau « Ce que fait chaque commande » (`providers-api.md:119-125`) correspond au code (`src-tauri/src/commands/api_keys.rs`, `src-tauri/src/services/vault.rs`). La phrase « Trois de ces fournisseurs proposent aussi une connexion par compte » reste exacte.

---

## 2. `06-modeles/ollama-runtime.md` — OBSOLÈTE

Le module qui gère le moteur Ollama a été entièrement réécrit. Ce n'est pas un ajustement de chiffres : le mécanisme décrit par le brief n'existe plus sous cette forme. Le brief doit être réécrit à partir du code, pas corrigé.

### Écart 1 — Quatre des cinq fichiers sources cités ont disparu

Le brief cite (`ollama-runtime.md:5`) : `services/ollama_lifecycle.rs`, `services/ollama_port.rs`, `services/ollama_env.rs`, `services/ollama_kill.rs`, `ollama_polling.rs`.

**Aucun de ces cinq fichiers n'existe.** Le cycle de vie complet vit désormais dans `src-tauri/src/services/ollama_manager/` (plus de 150 fichiers). `CLAUDE.md` note que `ollama_lifecycle.rs` a été supprimé le 15 août 2026 et que deux tests interdisent son nom.

Seuls `services/gpu_detect.rs` et `services/gpu_vram.rs` subsistent.

### Écart 2 — La plage de ports 11500–11599 n'existe plus

Le brief écrit (`ollama-runtime.md:51`) : « Il cherche un port libre entre **11500 et 11599** ».

Réalité : le port du moteur lancé par Beaver est **choisi par le système d'exploitation**, pas dans une plage fixée. `src-tauri/src/services/ollama_manager/port.rs:48` demande un port au système (`TcpListener::bind(("127.0.0.1", 0))`) et retient celui qui lui est attribué, avec au plus trois tentatives (`constants.rs:18`, `MAX_PROBE_PORT_ATTEMPTS = 3`).

Ce qui reste vrai : le port **11434** est celui qui est sondé pour détecter un moteur déjà en cours (`constants.rs:22`, `PROBE_DEFAULT_PORT = 11_434`, utilisé par `port.rs:66-84`). Mais ce n'est plus un « dernier recours » : c'est uniquement l'adresse de la détection.

### Écart 3 — Le tableau « Les réglages imposés au moteur » est en grande partie faux

Le brief donne treize réglages (`ollama-runtime.md:104-117`). Le code n'en applique que **quatre**, plus l'adresse d'écoute :

| Variable réellement posée | Valeur | Référence |
|---|---|---|
| `OLLAMA_MODELS` | dossier des modèles vérifié | `ollama_manager/spawn_profile.rs:116-119` |
| `OLLAMA_NO_CLOUD` | `1` | `ollama_manager/spawn_profile.rs:120` |
| `OLLAMA_MAX_LOADED_MODELS` | `1`, ou `0` si le multi-modèle est activé | `ollama_manager/spawn_settings.rs:26-29` |
| `OLLAMA_LLM_LIBRARY` | `cpu` en mode processeur, vide sinon | `ollama_manager/compute_mode.rs:18-31` |
| `OLLAMA_HOST` | `127.0.0.1:<port>` | `ollama_manager/spawn_gate_unix_support.rs:112`, `spawn_gate_windows_support.rs:29` |

**Ne figurent nulle part dans le code** : l'attention optimisée, la compression du cache, « une seule requête à la fois », le délai de chargement de dix minutes, l'accélération Vulkan sous Windows, et **le gigaoctet de mémoire vidéo réservé au système**. Une recherche sur `FLASH_ATTENTION`, `KV_CACHE_TYPE`, `NUM_PARALLEL`, `LOAD_TIMEOUT`, `GPU_OVERHEAD` et `VULKAN` dans tout `src-tauri/src/` ne renvoie aucun résultat en dehors des tests.

Le seul de ces réglages qui subsiste sous une autre forme est la **persistance en mémoire** : elle n'est plus imposée au moteur mais transmise requête par requête, dans le corps de l'appel (`src-tauri/src/services/agent_local/ollama_wire.rs:48`), depuis la configuration (`src-tauri/src/models/config.rs:22` et `:45`, valeur par défaut `"5m"`), avec le cas « indéfiniment » traité à `src-tauri/src/services/agent_local/agent_loop_support.rs:29-32`.

### Écart 4 — Le gigaoctet de mémoire vidéo réservé n'est pas dans le code

Affirmé deux fois (`ollama-runtime.md:65` et `:115`) et repris dans `materiel-et-vram.md:89`. Aucune trace dans `src-tauri/src/services/ollama_manager/` ni dans `src-tauri/src/services/gpu_vram.rs`. **À retirer des deux fichiers tant que la ligne qui l'implémente n'a pas été trouvée.**

### Écart 5 — Le mot « une douzaine de valeurs » du point à confirmer

`ollama-runtime.md:183` dit « Le code applique une douzaine de valeurs ». Il en applique **quatre** (voir écart 3).

### Écart 6 — Le fichier de traces

Le brief situe les traces dans `logs/ollama-sidecar.log`, écrasé à chaque démarrage (`ollama-runtime.md:92-94`). `CLAUDE.md` le confirme toujours, mais la réécriture du module n'a pas été recoupée sur ce point précis. **À revérifier avant réécriture** — c'est le seul point du fichier que je n'ai pas pu trancher.

### Ce qui reste vrai

- **Les paliers de contexte selon la mémoire sont inchangés et exacts** : `src-tauri/src/services/gpu_vram.rs:44-48` (seuils 24 000 Mo et 12 000 Mo, contextes 32 768 / 24 576 / 8 192), vérifiés par les tests `gpu_vram.rs:176-180`. Repli sur 8 192 quand la mesure échoue (`gpu_vram.rs:180`).
- **La réutilisation d'un moteur existant** subsiste : `ollama_manager/adoption.rs` et la sonde de `port.rs:66-84`.
- **L'écoute uniquement locale** : `OllamaEndpoint::loopback` partout (`ollama_manager/types_tests.rs:32-33` montre l'adresse produite, `http://127.0.0.1:<port>`).
- **Sur Mac Intel la mémoire n'est pas mesurée** : `src-tauri/src/services/gpu_vram/macos.rs:4-5` sort immédiatement hors architecture `aarch64`. Le contexte tombe donc au palier de 8 192 jetons. **Point à confirmer tranché : c'est bien le comportement du code.**

---

## 3. `06-modeles/raisonnement.md` — DÉCALÉ

### Écart 1 — Deux fichiers sources cités n'existent plus

Le brief cite (`raisonnement.md:5`) `services/reasoning_google.rs` et `services/llm/providers/groq.rs`. Aucun des deux n'existe. Le dossier `src-tauri/src/services/llm/providers/` ne contient plus que `mod.rs`, `moonshot.rs`, `openai.rs`, `xai.rs`.

Les sources encore valides : `src-tauri/src/services/reasoning.rs`, `reasoning_effort.rs`, `reasoning_profile.rs`, `reasoning_ollama.rs`, `src-tauri/src/services/llm/stream_reasoning.rs`.

### Écart 2 — Le budget de réflexion Google

Le brief affirme (`raisonnement.md:85`) que « l'effort se traduit par un budget de réflexion chiffré » chez Google. Le fichier qui portait cette logique (`reasoning_google.rs`) a disparu ; la notion de budget se retrouve dans `src-tauri/src/services/llm/stream_reasoning.rs`. **L'affirmation reste plausible mais son emplacement a changé — à revérifier avant publication.**

### Ce qui reste vrai

**Les huit paliers du tableau sont exacts** : `off`, `auto`, `low`, `medium`, `high`, `xhigh`, `max`, `ultra` — `src-tauri/src/services/reasoning.rs:10`. La correspondance avec les libellés du brief (Désactivé, Automatique, Faible, Moyen, Élevé, Très élevé, Maximum, Ultra) est cohérente. **Le point à confirmer sur l'existence réelle du palier Ultra est tranché : il existe bien dans le code** ; reste à vérifier qu'un modèle l'atteint depuis l'interface.

Le repli automatique quand un palier n'est pas supporté est confirmé (`reasoning.rs:75-81` : le code retombe sur `medium`, puis `auto`, puis sur le premier mode différent de `off`).

---

## 4. `06-modeles/catalogue-et-favoris.md` — DÉCALÉ

### Écart 1 — Un fichier source cité n'existe plus

Le brief cite `services/llm/tool_capable.rs` (`catalogue-et-favoris.md:5`). Ce fichier n'existe pas. La capacité d'un modèle à utiliser des outils est désormais dans **`src-tauri/src/services/llm/provider_model_capabilities.rs`**.

Les autres sources citées existent toutes : `services/favorite_models.rs`, `commands/favorite_models.rs`, `services/llm/litellm_catalog_search.rs`, `provider_model_registry.rs`, `openai_compat_models.rs`, `model_metadata.rs`, `vision.rs`.

### Points à confirmer tranchés

- **« Le nombre maximal de favoris n'est pas borné »** (`catalogue-et-favoris.md:114`) — **confirmé**. `src-tauri/src/services/favorite_models.rs` ne contient aucune constante de plafond ni aucun `truncate` ; la fonction d'ajout ne compare que la longueur avant et après pour détecter un doublon (`favorite_models.rs:58-60`). C'est bien une exception à la règle des collections bornées appliquée partout ailleurs. **À signaler à l'équipe, comme le brief le demandait.**
- **« Ajouter un favori déjà présent est sans effet »** (`catalogue-et-favoris.md:59`) — **confirmé** par la même comparaison `favorite_models.rs:58-60`.

### Points restant ouverts

L'écran d'exploration des modèles, l'emplacement des favoris et la mise en avant des modèles gratuits relèvent de la passe d'interface, non tranchables depuis le code.

---

## 5. `06-modeles/materiel-et-vram.md` — DÉCALÉ

### Écart 1 — Un fichier source cité n'existe plus

`services/ollama_env.rs` (`materiel-et-vram.md:5`) n'existe pas. Voir l'écart 1 de `ollama-runtime.md`.

### Écart 2 — Le gigaoctet de mémoire vidéo réservé au système

`materiel-et-vram.md:89` : « Il réserve par ailleurs **un gigaoctet de mémoire vidéo** au système ». Introuvable dans le code (voir l'écart 4 de `ollama-runtime.md`). **À retirer.**

### Points à confirmer tranchés

- **« La table de mémoire est codée en dur dans l'interface »** (`materiel-et-vram.md:158`) — **confirmé, et les valeurs du brief sont exactes au chiffre près**. `src/components/settings/vram-table.tsx:4-14` : cinq tailles (`3B`, `7B`, `13B`, `30B`, `70B`), quatre compressions (`Q4_K_M`, `Q5_K_M`, `Q8_0`, `f16`), et les vingt valeurs correspondent une à une au tableau du brief (`materiel-et-vram.md:46-52`). La formule affichée sous la table vient bien d'une clé de traduction (`vram-table.tsx:38`, clé `settings.advanced.vramFormula`).
- **Le cas Mac Intel** (`materiel-et-vram.md:102`) — **confirmé** : `src-tauri/src/services/gpu_vram/macos.rs:4-5`.

### Ce qui reste vrai

Les paliers de contexte (`materiel-et-vram.md:81-85`) sont exacts : `src-tauri/src/services/gpu_vram.rs:44-48` et tests `:176-180`.

---

## 6. `06-modeles/ollama-modeles.md` — DÉCALÉ

### Écart 1 — Un fichier source cité n'existe plus

`services/ollama_lifecycle.rs` (`ollama-modeles.md:5`). Voir l'écart 1 de `ollama-runtime.md`. Toutes les autres sources existent : `commands/ollama_setup.rs`, `commands/ollama_updates.rs`, `services/agent_local/ollama_registry_details.rs`, et les composants d'interface cités.

### Points à confirmer tranchés

- **« La limite de 100 familles »** (`ollama-modeles.md:129`) — **confirmée, et silencieuse**. `src-tauri/src/commands/ollama_updates.rs:26` arrête la collecte à cent familles et `:36` n'en examine que cent (`.take(100)`). Rien n'informe l'utilisateur au-delà. **Le signalement à l'équipe reste justifié.**
- **La ligne du tableau des pièges « Au-delà de 100 familles installées, toutes ne sont pas vérifiées »** (`ollama-modeles.md:110`) est donc exacte.

### Points restant ouverts

La reprise d'un téléchargement partiel, la suppression d'un modèle et l'existence d'une liste de modèles recommandés n'ont pas été tranchées faute de temps ; elles demandent une lecture de `commands/ollama_setup.rs` et de `services/ollama_manager/download*.rs`.

---

## 7. `07-integrations/recherche-web.md` — DÉCALÉ

### Écart 1 — La recherche de Python a changé de règle

Le brief écrit (`recherche-web.md:83`) : « Beaver cherche les versions 3.13, 3.12, 3.11, 3.10, puis les commandes génériques ».

Réalité : Beaver cherche **une seule version exacte**, déclarée dans un fichier manifeste livré avec le moteur.

- Le manifeste `.runtime.json` déclare `major` et `minor` (`src-tauri/src/services/searxng/runtime_manifest.rs:11-25`), avec `major` obligatoirement 3 et `minor` entre 10 et 99 (`:43-46`).
- Les commandes essayées sont, dans l'ordre : `python<major>.<minor>`, puis `python3`, puis `python` — et sous Windows, `py -<major>.<minor>` en premier (`src-tauri/src/services/searxng/python_runtime.rs:75-95`).
- **Mais chaque candidat est interrogé et rejeté s'il ne rapporte pas exactement la version du manifeste** (`python_runtime.rs:137-152` : l'implémentation doit être `cpython`, et `major` et `minor` doivent correspondre au manifeste).

Conséquence à écrire sur le site : ce n'est pas « n'importe quel Python 3 récent » qui convient, c'est **une version précise**. Un utilisateur avec un Python 3 installé peut donc quand même se voir refuser le moteur local. C'est plus contraignant que ce que le brief annonce, et cela renforce le point remonté à l'équipe.

Le fichier cité par `differents-points-a-traiter.md:345` (`services/searxng/runtime.rs:106`) ne contient plus ce code : `runtime.rs` ne fait plus que 885 octets.

### Écart 2 — Le délai d'attente au démarrage est de 12 secondes, pas 10

Le brief écrit **10 secondes** (`recherche-web.md:59` et `:161`).

Réalité : `src-tauri/src/services/searxng/start_lifecycle.rs:8` — `SIDECAR_START_TIMEOUT: Duration = Duration::from_secs(12)`.

Deux autres délais du même fichier, absents du brief et utiles : la récupération d'un processus orphelin dispose de 3 secondes (`start_lifecycle.rs:7`), et la stabilité d'identité de 250 ms (`:9`).

### Écart 3 — Le tableau des limites cite un fichier source éclaté

Le brief cite `services/searxng/lifecycle.rs` et `runtime.rs` comme sources principales. Le module a été découpé : le démarrage est dans `start_lifecycle.rs`, l'échec dans `startup_failure.rs`, l'environnement Python dans `python_runtime.rs` / `runtime_environment.rs`, le manifeste dans `runtime_manifest.rs`. `lifecycle.rs` et `runtime.rs` existent encore mais ne portent plus l'essentiel.

### Points à confirmer tranchés

- **Les limites communes du tableau (`recherche-web.md:152-162`) sont exactes** : `src-tauri/src/services/search/common.rs:4-9` — 10 résultats, 512 caractères de requête, 160 de titre, 300 d'extrait, 2 048 d'adresse, 512 Ko de réponse.
- **Le délai d'une recherche locale est bien de 15 secondes** : `src-tauri/src/services/searxng/client.rs:5`.
- **Le délai de 30 secondes avant une nouvelle tentative après échec est confirmé** : `src-tauri/src/services/searxng/startup_failure.rs:4` — `START_FAILURE_COOLDOWN: Duration = Duration::from_secs(30)`.
- **L'ordre fixe Brave → Exa → Firecrawl → moteur local est confirmé** : `src-tauri/src/services/search/mod.rs:73-77` fixe l'ordre des fournisseurs par clé, et `mod.rs:82-98` n'appelle le moteur local qu'après eux. L'agrégation des causes d'échec et leur expurgation sont visibles à `mod.rs:106-135` (`common::sanitize_error`).

### Points restant ouverts

- **« Le moteur local est-il arrêté quand il ne sert pas ? »** — non tranché. `stop_lifecycle.rs` existe mais je n'ai pas vérifié s'il est appelé sur inactivité ou seulement à la fermeture. Reste ouvert.
- **La taille du téléchargement et la durée de la première installation** restent non mesurées.

### Ce qui reste vrai

Les trois services par clé (Brave, Exa, Firecrawl) sont inchangés : `src-tauri/src/services/search/` contient `brave.rs`, `exa.rs`, `firecrawl.rs`, et `mod.rs:178-197` les route et les teste.

---

## 8. `_geles/README.md` — DÉCALÉ

### Écart 1 — Les extensions ne sont plus gelées

Le fichier gèle « Extensions — quatre briefs non écrits » (`_geles/README.md:23-34`) au motif que « l'implémentation a été interrompue en cours de route et doit être finalisée ».

Ce n'est plus le cas au 9 septembre 2026. Les extensions sont implémentées et testées :

- `EXTENSIONS.md` à la racine du dépôt (56,6 Ko) fait autorité sur le sujet ;
- le module `src-tauri/src/services/extensions/` existe et est largement couvert de tests ;
- les outils de découverte d'extensions sont intégrés au mode Plan, avec leurs constantes publiées : `src-tauri/src/services/extensions/mod.rs:192-193` (`LIST_EXTENSIONS_TOOL_NAME`, `INSPECT_EXTENSIONS_TOOL_NAME`) ;
- la politique de permission par effet d'extension est en place et testée : `src-tauri/src/services/agent_local/tool_plan_guard.rs:46-48` et son test `:89-98` (seules les extensions en lecture seule passent en mode Plan).

**Les quatre briefs Extensions sont donc à sortir de la liste des gels** et à écrire, avec `EXTENSIONS.md` comme source principale. La note du README sur le niveau d'exigence attendu reste entièrement valable.

### Ce qui reste vrai

- **Le mode Plan reste gelé** — et la raison du gel est vérifiée : sa liste d'outils a effectivement changé depuis la rédaction du brief (voir la fiche suivante).
- **La compression du contexte reste gelée** — `src-tauri/src/services/llm/agent_loop_compression.rs` et `compress_hook.rs` existent, et des fichiers de test « fast mode » récents (`agent_loop_compression_fast_mode_tests.rs`) montrent un chantier encore actif. Le gel est justifié.

---

## 9. `_geles/plan-mode.md.gele` — DÉCALÉ

Le brief a été gelé exactement pour la bonne raison : la liste d'outils a bougé. Le reste du fichier tient.

### Écart 1 — Dix-neuf outils autorisés → vingt et un

Le brief annonce **dix-neuf** outils (`plan-mode.md.gele:35`). La liste actuelle en compte **vingt et un** : `src-tauri/src/services/agent_local/tool_plan_guard.rs:3-25`.

### Écart 2 — `search_extension_tools` a disparu, remplacé par trois entrées

Le brief liste `search_extension_tools` (`plan-mode.md.gele:42`) et s'interroge à son sujet dans ses points à confirmer (`:168`).

**Point tranché : cet outil n'existe plus du tout dans le code.** Une recherche sur `search_extension_tools` dans tout `src-tauri/src/` ne renvoie rien. Il a été remplacé par trois entrées distinctes dans la liste du mode Plan :

- `list_extensions` — `tool_plan_guard.rs:10`, constante définie à `services/extensions/mod.rs:192` ;
- `inspect_extensions` — `tool_plan_guard.rs:11`, constante définie à `services/extensions/mod.rs:193` ;
- l'outil de lecture d'une ressource d'extension — `tool_plan_guard.rs:12` (`super::tool_extension_resource::NAME`).

### Écart 3 — Une nouvelle règle conditionnelle, absente du brief

En plus des trois outils conditionnels que le brief décrit correctement (`bash`, `transform_image`, `search_mcp_tools` — confirmés à `tool_plan_guard.rs:38-43`), le code applique désormais une règle par **effet d'extension** : tout outil fourni par une extension est autorisé en mode Plan **si et seulement si son effet est « lecture seule »** (`tool_plan_guard.rs:35-37` et `:46-48`, test `:89-98`).

### Ce qui reste vrai — et qui est confirmé au chiffre près

| Affirmation du brief | Vérification |
|---|---|
| Sept états du parcours | `src-tauri/src/services/agent_local/types_plan.rs:18-24` — `NeedsContext`, `CollectingQuestions`, `PlanPublished`, `AwaitingApproval`, `Approved`, `Rejected`, `Cancelled` |
| Quatre corrections automatiques avant échec | `src-tauri/src/services/agent_local/plan_mode_controller.rs:5` — `MAX_REPAIRS: usize = 4` |
| Titre limité à 120 caractères | `src-tauri/src/services/agent_local/tool_plan.rs:12` |
| Contenu limité à 40 000 caractères | `src-tauri/src/services/agent_local/tool_plan.rs:13` |
| Vingt plans conservés par conversation | `src-tauri/src/services/agent_local/types_plan.rs:45` et `tool_plan.rs:95` |
| Tests et compilation autorisés | `tool_plan_guard.rs:161-171` — `git status`, `cargo test`, `cargo check`, `npm run build`, `npm test`, `npx tsc --noEmit` passent ; `rm file` est refusé |
| `write_file`, `edit_file`, `todo_write`, `create_branch`, `delegate_task` interdits | `tool_plan_guard.rs:81-87` |
| Le mode Plan protège même en Accès complet | `tool_plan_guard.rs:50-55` — la garde ne consulte pas le mode de permission |

L'argument central du brief — « le mode Plan protège même en Accès complet » — est donc intact.

---

## 10. `differents-points-a-traiter.md` — DÉCALÉ

Voir la section dédiée « Points en suspens : encore d'actualité / réglés » plus bas. Les écarts structurels du fichier lui-même :

| Écart | Réalité |
|---|---|
| « Référence projet : Beaver v1.1.2 » (ligne 11) | La version est **1.2.2** (`package.json`, `src-tauri/Cargo.toml`) |
| « `06-modeles/providers-api.md` — table des 10 providers » (ligne 94) | Il y en a **onze** (`catalog_api.rs:42-191`) |
| « `06-modeles/ollama-runtime.md` — … port 11434 » (ligne 91) | Le port du moteur possédé est attribué par le système (`ollama_manager/port.rs:48`) ; 11434 n'est plus que le port de détection (`constants.rs:22`) |
| Les quatre pages Extensions cochées « en attente » (lignes 106-119) | Le chantier est terminé — voir la fiche `_geles/README.md` ci-dessus |
| « Fichiers gelés : 6 (4 Extensions + Mode Plan + Compression) » (ligne 191) | **Deux** restent gelés : Mode Plan et Compression |
| « `services/searxng/runtime.rs:106` cherche `python3.13` à `python` » (ligne 345) | Le code est ailleurs et la règle a changé — `python_runtime.rs:75-95` et `:137-152` |

---

# Points à confirmer tranchés

Résumé de tous les points que le code permet de trancher, avec leur référence.

| Fichier | Point à confirmer | Verdict | Référence |
|---|---|---|---|
| `catalogue-et-favoris.md:114` | Le nombre de favoris est-il borné ? | **Non borné** — anomalie confirmée | `services/favorite_models.rs` (aucune constante de plafond ; `:58-60`) |
| `catalogue-et-favoris.md:59` | Ajouter un favori en double est-il sans effet ? | **Oui** | `services/favorite_models.rs:58-60` |
| `materiel-et-vram.md:158` | La table VRAM est-elle codée en dur ? | **Oui**, et ses vingt valeurs sont exactes | `src/components/settings/vram-table.tsx:4-14` |
| `materiel-et-vram.md:102` / `ollama-runtime.md:186` | Mac Intel : mémoire non mesurée, contexte minimal | **Confirmé** | `services/gpu_vram/macos.rs:4-5` + `gpu_vram.rs:180` |
| `ollama-modeles.md:129` | La limite de 100 familles est-elle silencieuse ? | **Oui** | `commands/ollama_updates.rs:26` et `:36` |
| `ollama-personnalisation.md` (tableau des onze paramètres) | Les onze paramètres et leurs valeurs par défaut | **Exacts, un à un** | `src/components/ollama/model-parameter-catalog.ts:43-53` |
| `ollama-personnalisation.md:69-70` | Recette limitée à 2 Mo, création abandonnée à 10 minutes | **Confirmé** | `services/agent_local/ollama_modelfile_create.rs:4-5` |
| `providers-api.md:174` | Les paliers gratuits Google et Mistral sont-ils encore publiés ? | **Réglé côté produit** : les textes ne portent plus de chiffres | `src/i18n/fr.json:41-42` |
| `providers-api.md:177` | Z.ai GLM a-t-il une configuration particulière ? | **Oui** : son point d'entrée n'a pas de chemin de catalogue (`""`) | `route_profile/catalog_api.rs:157` |
| `providers-comptes-web.md` (limites) | 15 minutes, 4 096 caractères, marge d'une minute | **Confirmés** | `llm_oauth/device_flow.rs:10` (900 s), `llm_oauth/store.rs:8` (4 096), `llm_oauth/types.rs:51` (60 s) |
| `providers-comptes-web.md:157` | Kimi est-il toujours marqué expérimental ? | **Oui** | `src/i18n/fr.json:20`, clé `experimental` |
| `providers-comptes-web.md:36-43` | Trois fournisseurs par compte | **Confirmé** : xAI, Moonshot, Codex (OpenAI) | `route_profile/catalog_oauth.rs:28,41,54` |
| `raisonnement.md:163` | Le palier Ultra existe-t-il ? | **Oui, dans le code** | `services/reasoning.rs:10` |
| `usage-et-couts.md` | Rejet d'un catalogue de moins de cent entrées | **Confirmé** | `services/llm/litellm_catalog_refresh.rs:58` |
| `usage-et-couts.md:54` | Garde-fou du coût aberrant | **Confirmé** : rejet si non fini ou hors de 0 à 1 000 000 $ | `services/provider_usage/pricing.rs:72` |
| `channels-gateway.md:206` | **Quel mode de permission s'applique aux messages du gateway ?** | **Tranché — voir ci-dessous** | `services/gateway/agent_bridge_run.rs:110-113` + `commands/agent_chat_task/common.rs:44-59` |
| `channels-gateway.md` (tableau des limites) | 16 / 100 / 1 000 / 12 000 / 10 000 / 365 / 128 | **Tous confirmés** | `gateway/work_supervision.rs:11` (16) ; `gateway/config_validation.rs:7-12` (les six autres) |
| `channels-gateway.md:116` | Les connexions par compte sont-elles refusées au gateway ? | **Oui**, refus explicite | `services/gateway/agent_bridge.rs:97` |
| `mcp-connecteurs.md` (les dix-huit connecteurs) | 13 distants + 5 locaux | **Confirmés, nom par nom** | `mcp_bridge/trusted.rs:2-14` (13) ; `mcp_bridge/stdio_catalog.rs:14-21` + `:1-3` (5) |
| `mcp-connecteurs.md` (tableau des limites) | 32 / 8 / 10 min / 128 / 250 / 64 / 15 | **Tous confirmés** | `mcp_bridge/config.rs:9` (32) ; `work_supervision.rs:10` (8) ; `process_manager.rs:13` (600 s) ; `transport.rs:13-15` (128/250/64) ; `agent_local/tool_mcp.rs:8` (15) |
| `mcp-connecteurs.md:118` | Trois programmes autorisés seulement | **Confirmé** : `npx`, `uvx`, `deno` | `mcp_bridge/stdio_catalog.rs:3` et `:15-18` |
| `mcp-connecteurs.md:223` | Les connecteurs locaux se mettent-ils à jour ? | **Non** : versions figées au caractère près | `mcp_bridge/stdio_catalog.rs:3,15-18` (`@2.2.5`, `@0.3.13`, `==0.1.0`, `@1.4.5`, `@0.4.2`) |
| `mcp-oauth.md` (tableau des limites) | 5 min / 5 / 30 s / 4 Ko / 15 s | **Tous confirmés** | `mcp_oauth/callback_server.rs:11` (300 s), `:13` (4 096) ; `mcp_oauth/flow.rs:16` (5) ; `mcp_oauth/storage.rs:46-55` (30 s) ; `mcp_oauth/discovery.rs:6` (15 s) |
| `mcp-oauth.md:163` | Les identifiants d'application Google et GitHub | **Confirmés présents** dans le coffre ; leur origine reste à clarifier avec l'équipe | `mcp_oauth/static_credentials.rs:11-14` |
| `recherche-web.md:152-162` | Les neuf limites communes | **Sept confirmées, une fausse (12 s ≠ 10 s), une confirmée ailleurs** | `search/common.rs:4-9` ; `searxng/client.rs:5` ; `searxng/start_lifecycle.rs:8` ; `searxng/startup_failure.rs:4` |
| `plan-mode.md.gele:168` | `search_extension_tools` : outil réel ou reliquat ? | **Reliquat — l'outil n'existe plus** | absence totale dans `src-tauri/src/` ; remplacé par `tool_plan_guard.rs:10-12` |

## Le point de sécurité du gateway — tranché en détail

C'était le point le plus important à trancher du périmètre (`channels-gateway.md:206`, repris dans `differents-points-a-traiter.md:356-360`). Voici ce que dit le code.

1. **Le gateway demande le mode « Accès complet »** pour la conversation qu'il lance : `src-tauri/src/services/gateway/agent_bridge_run.rs:110-112` passe `StreamPermissionMode::Bounded(Some("auto"))`.
2. **Mais cette demande est plafonnée par le réglage de l'utilisateur** : `src-tauri/src/commands/agent_chat_task/common.rs:49-56` — si le mode demandé est plus permissif que celui enregistré, c'est le mode enregistré qui l'emporte. Un utilisateur en « Demande d'approbation » n'est donc **pas** basculé en « Accès complet » par un message Telegram.
3. **En revanche, le mode par défaut de l'application est « Accès complet »** : `src-tauri/src/storage_migration.rs:83` crée `agent-settings.json` avec `{"permissionMode":"auto"}`. Sur une installation où l'utilisateur n'a rien changé, un message reçu par messagerie déclenche donc un agent en accès complet.
4. **Le mode Plan est explicitement désactivé** pour ces conversations : `agent_bridge_run.rs:116` passe `plan_mode: Some(false)`.
5. **Aucune demande d'approbation ne part vers la messagerie** : `agent_bridge_run.rs:113` passe `permission_emitter: None`, et sans émetteur dédié, la demande retombe sur l'émission d'événement ordinaire de l'application (`src-tauri/src/services/agent_local/stream_events.rs:78-95`). Autrement dit, **une approbation demandée par un message Telegram s'affiche dans la fenêtre de Beaver sur la machine**, pas dans la messagerie.

**Formulation proposée pour le site** : *un message reçu par messagerie lance un agent avec le mode de permission que vous avez réglé dans Beaver — jamais plus permissif. Mais si ce mode demande des approbations, celles-ci s'affichent sur votre ordinateur, pas dans la messagerie : personne ne peut y répondre à distance.* L'encadré existant de `channels-gateway.md:170-171` reste juste et devient plus fort ainsi complété.

---

# Points restant ouverts (et pourquoi)

| Point | Fichier | Pourquoi il reste ouvert |
|---|---|---|
| L'écran d'exploration des modèles, l'emplacement des favoris, la mise en avant des modèles gratuits | `catalogue-et-favoris.md:112-115` | Relève de la passe d'interface ; le code ne dit pas comment l'écran se présente |
| L'emplacement de l'écran d'usage et de coûts | `usage-et-couts.md:172` | Idem |
| La rétention du journal des requêtes | `usage-et-couts.md:173` | Non lu ; demande une lecture de `services/provider_usage/request_journal.rs` |
| Quels fournisseurs renvoient un coût exact | `usage-et-couts.md:174` | Demande la lecture complète de `services/provider_usage/remote.rs`, non faite |
| « Les jetons de raisonnement comptés comme de la sortie » | `usage-et-couts.md:175` | Non vérifié fournisseur par fournisseur |
| Le fichier de traces `logs/ollama-sidecar.log` | `ollama-runtime.md:92-94` | Le module a été réécrit ; la ligne qui écrit ce fichier n'a pas été retrouvée |
| Le moteur de recherche local s'arrête-t-il sur inactivité ? | `recherche-web.md:215` | `stop_lifecycle.rs` existe mais son déclencheur n'a pas été tracé |
| La taille et la durée du premier téléchargement du moteur local | `recherche-web.md:214` | Non mesurable depuis le code |
| L'origine des identifiants d'application OAuth Google et GitHub | `mcp-oauth.md:163` | Le code les lit dans le coffre (`static_credentials.rs:11-14`), il ne dit pas qui les y met |
| La reprise d'un téléchargement de modèle interrompu | `ollama-modeles.md:128` | Demande une lecture de `services/ollama_manager/download*.rs` |
| Les libellés d'interface (paliers de raisonnement, paramètres Ollama, modes de permission) | plusieurs fichiers | Relèvent de la passe d'interface et des fichiers de traduction |
| Le budget de réflexion Google | `raisonnement.md:85` | Le fichier `reasoning_google.rs` a disparu ; la logique existe ailleurs mais n'a pas été relue |

---

# `differents-points-a-traiter.md` — points en suspens : encore d'actualité ou réglés

## Encore d'actualité

| Point | Vérification |
|---|---|
| **`CROSS-PLATFORM.md` annonce un support Fedora/RHEL que `install.sh` n'implémente pas** | Toujours vrai. `install.sh:153` n'appelle que `apt-get`, et `:169` ne construit que le suffixe `_amd64.deb` |
| **Le mode de permission par défaut est « Accès complet »** | Toujours vrai. `src-tauri/src/storage_migration.rs:83` — `{"permissionMode":"auto"}`. Devient d'autant plus important avec le point gateway tranché ci-dessus |
| **Deux dossiers créés au premier lancement ne sont documentés nulle part** | Toujours vrai. `storage_migration.rs:70` (`inbox`), `:74` (`translations`), `:87` (`inbox/pending.json`), et un fichier supplémentaire non mentionné dans le brief : `:110` (`inbox/idea-discovery.md`) |
| **Le journal `logs/permission-diagnostics.jsonl` n'est documenté nulle part** | Toujours vrai. `services/agent_local/permission_gate.rs:116`, avec rotation vers `.jsonl.1` au-delà de `MAX_DIAGNOSTIC_LOG_BYTES` (`:117-123`) |
| **Les onglets de clones sont limités à 3 par groupe** | Toujours vrai. `services/agent_local/session_tabs_state.rs:6` — `MAX_TABS_PER_SESSION: usize = 3`, appliqué à `:40` et `:88` |
| **`read_document` annonce un filtrage par pages qu'il n'applique pas** | Toujours vrai. `services/agent_local/tool_document_read.rs:12` reçoit toujours le paramètre sous le nom `_pages` et ne s'en sert pas |
| **La limite d'outils optionnels vaut exactement le nombre d'outils optionnels** | Toujours vrai, et au chiffre près. `services/agent_local/tool_catalog.rs:16` — `MAX_OPTIONAL_TOOLS = 32` ; la liste `OPTIONAL_TOOLS` (`:52-84`) compte exactement **32** entrées (13 activées par défaut, 19 désactivées). La troncature reste silencieuse (`:140`, `.take(MAX_OPTIONAL_TOOLS)`) |
| **L'écran des outils dit « Tools essentiels » et « Tools optionnels »** | Toujours vrai en français. `src/i18n/fr.json:977-978` |
| **Les sous-agents portent des noms visibles fixes « Claudiator » et « Geminitor »** | Toujours vrai. `services/agent_local/tool_definitions_subagent.rs:38` |
| **Deux paramètres de `delegate_task` sont marqués « legacy »** | Toujours vrai. Même ligne, `tool_definitions_subagent.rs:38` (« Legacy mission label ») |
| **La limite de 15 outils MCP par connecteur dans une recherche est silencieuse** | Toujours vrai. `services/agent_local/tool_mcp.rs:8` et `:61` |
| **Aucune borne sur le nombre de modèles favoris** | Toujours vrai. `services/favorite_models.rs` |
| **La vérification des mises à jour de modèles s'arrête à 100 familles sans le signaler** | Toujours vrai. `commands/ollama_updates.rs:26` et `:36` |
| **Sur Mac Intel la mémoire n'est pas mesurée** | Toujours vrai. `services/gpu_vram/macos.rs:4-5` |
| **Le nom « Codex » désigne le mécanisme de connexion OpenAI** | Toujours vrai, et désormais visible dans le nom affiché lui-même : `route_profile/catalog_oauth.rs:56` — `display_name: "Codex"`. L'arbitrage éditorial reste entier |
| **Les connecteurs locaux ne se mettent pas à jour** | Toujours vrai. `mcp_bridge/stdio_catalog.rs:3,15-18` — versions figées |
| **Le connecteur iMessage donne accès aux messages personnels** | Toujours vrai. `mcp_bridge/stdio_catalog.rs:1-11` |
| **Le prérequis Python de la recherche web n'est documenté nulle part** | Toujours vrai, et **aggravé** : ce n'est plus « un Python 3 récent » mais une version exacte imposée par le manifeste (`services/searxng/python_runtime.rs:137-152`). Reste le point le plus urgent de la section 7 |
| **Le bac à sable du shell ne s'active que si l'accès disque est restreint** | Toujours vrai. `services/agent_local/shell_sandbox/launch.rs:42` — `roots_allow_full_disk(&configured)` court-circuite l'isolation. Argument à mettre en avant, comme le note le brief |
| **Aucun tarif de fournisseur sur le site** | Décision inchangée et toujours fondée. Le catalogue de tarifs est bien téléchargé, pas codé en dur : `services/llm/litellm_catalog_refresh.rs:8` (source unique) et `:58` (rejet d'un catalogue de moins de cent entrées) |
| **Les libellés des modes de permission** (`auto` / `manual` / `chat`, plus `subagent` interne) | Inchangés. `commands/agent_chat_task/common.rs:50` énumère exactement `"auto" \| "manual" \| "chat" \| "subagent"` |
| **Les définitions d'agents spécialisés réutilisables (`.beaver/agents/<nom>.md`) ne sont documentées nulle part** | Non revérifié dans cet audit ; reste ouvert |
| **« Authentification GitHub requise » sur une création de branche locale** | Non revérifié dans cet audit ; reste ouvert |

## Réglés depuis

| Point | Ce qui a changé |
|---|---|
| **`CLAUDE.md` affirme que la release CI est en non-draft, alors que le workflow utilise `--draft`** | **Réglé.** `.github/workflows/release.yml:439-443` crée la release avec `--verify-tag --latest` et **sans** `--draft` ; le chemin de reprise force explicitement `--draft=false` (`:436`). `CLAUDE.md` dit désormais vrai |
| **Les paliers gratuits de Google et Mistral affichés dans l'application ne sont plus publiables** | **Réglé côté produit.** Les textes ont été réécrits sans chiffre : `src/i18n/fr.json:41` (« Niveau gratuit pour certains modèles, selon le compte et les limites ») et `:42` (« Mode gratuit et plan payant ; disponibilité selon le compte »). Les commentaires datés du 30 juillet 2026 dans `catalog.rs` ont disparu avec la réécriture du module |
| **`search_extension_tools` est verrouillé mais n'appartient à aucun groupe** | **Réglé.** L'outil n'existe plus. Aucune occurrence dans `src-tauri/src/`. Il a été remplacé par `list_extensions` et `inspect_extensions` (`services/extensions/mod.rs:192-193`) |
| **Les quatre pages Extensions sont gelées** | **Réglé.** Le chantier est terminé — `EXTENSIONS.md` fait autorité, le module `services/extensions/` est implémenté et testé, et la politique de permission par effet est en place (`tool_plan_guard.rs:46-48`) |
| **Le mode de permission appliqué aux messages du gateway n'est pas déterminé** | **Tranché.** Voir la section détaillée plus haut : `gateway/agent_bridge_run.rs:110-116` et `commands/agent_chat_task/common.rs:44-59` |
| **`services/searxng/runtime.rs:106` cherche `python3.13` à `python`** | **Périmé.** Le code a été déplacé et la règle a changé : version exacte imposée par le manifeste (`services/searxng/python_runtime.rs:75-95` et `:137-152`). Le fond du problème — le prérequis non documenté — reste, mais la description technique du point est fausse |
| **Rien ne signale que Beaver réutilise un moteur Ollama existant (`ollama_lifecycle.rs:69-79`)** | **La référence est périmée** : le fichier n'existe plus. Le constat produit lui-même n'a pas été revérifié dans le nouveau module `ollama_manager/adoption.rs` — à reprendre |
| **Référence projet v1.1.2** | **Périmé.** La version courante est **1.2.2** |
| **Compteurs de suivi (94 prévus / 63 rédigés / 6 gelés / 88 rédigeables)** | **Périmés.** Deux fichiers restent gelés, pas six ; quatre briefs Extensions passent de « gelés » à « à écrire » |

---

# Ce que je recommande de faire en premier

1. **Réécrire `06-modeles/ollama-runtime.md` de zéro** à partir de `src-tauri/src/services/ollama_manager/`. C'est le seul fichier OBSOLÈTE du périmètre : quatre de ses cinq sources ont disparu, sa plage de ports est fausse et son tableau principal décrit neuf réglages qui n'existent pas.
2. **Corriger `06-modeles/providers-api.md`** : onze fournisseurs, retirer Groq, ajouter Anthropic et Qwen, et changer le fichier d'autorité pour `route_profile/catalog_api.rs`.
3. **Dégeler les quatre briefs Extensions** dans `_geles/README.md` et dans `differents-points-a-traiter.md`, et mettre les compteurs de suivi à jour.
4. **Retirer partout l'affirmation du gigaoctet de mémoire vidéo réservé** (`ollama-runtime.md:65` et `:115`, `materiel-et-vram.md:89`) tant que la ligne qui l'implémente n'est pas trouvée.
5. **Compléter `07-integrations/channels-gateway.md`** avec la réponse au point de sécurité, maintenant tranchée.
