# Beaver

Application desktop agentique (Tauri 2 + React 19) pour LLM locaux via Ollama et providers cloud (Groq, Gemini, Mistral, OpenAI, OpenRouter, DeepSeek, Cerebras, xAI, Moonshot Kimi, Z.ai GLM). Chat en onglets, outils, subagents, connecteurs MCP, réveils automatisés, forecasting, workflows Git, previews de fichiers et terminal intégré.

## Fonctionnalités

- **Agent Local** : chat avec n'importe quel modèle Ollama ou provider cloud, gestion des conversations en onglets, permissions manuelles/auto/chat sur les outils, réflexion approfondie, modèles favoris, projets et contexte `AGENTS.md`
- **Planification agent** : Plan mode avec exploration en lecture seule, plans Markdown locaux, validation utilisateur obligatoire et passage à l'implémentation
- **Workflow agent UI** : progression todo en temps réel, historique todo caché pour l'agent, choix interactifs et diagnostics sécurisés après erreur de flux ou d'outil
- **Outils** : bash, lecture/écriture de fichiers, web fetch/search, actions Git, arbre de fichiers, preview de fichiers, preview Office, link preview, outils MCP, outils Forecast, diagnostics, todos et choix interactifs
- **Branching de sessions** : clone un chat depuis n'importe quel message dans un onglet d'en-tête, optionnellement avec un résumé caché, et garde les sessions branchées hors de la sidebar principale
- **Chats archivés** : la suppression d'un chat l'archive d'abord ; restaure les sessions ou supprime-les définitivement depuis les Réglages
- **Subagents contrôlés par le parent** : lance des assistants isolés coordonnés par l'agent parent, avec statut en temps réel, injection cachée des rapports, sessions enfant réutilisables et nettoyage plus sûr des runs en file
- **Réveils** : scheduler interne qui prompt un LLM à heure fixe (ponctuel / journalier / hebdomadaire), réponses stockées dans une conversation dédiée par modèle
- **Forecast** : prévisions de séries temporelles avec modèles locaux et cloud, historique, comparaisons, scénarios, notes, exports et analyses appelables par l'Agent Local
- **Connecteurs MCP** : connecteurs cloud et locaux avec OAuth ou tokens d'environnement, test de statut et activation par chat
- **Gateway / channels** : gateway optionnel en arrière-plan pour des canaux externes comme Telegram, Slack et Discord
- **Clés API** : gestion centralisée des identifiants LLM, search, forecast, MCP et gateway. Clés stockées dans un **vault chiffré XChaCha20-Poly1305** (master key dans le keyring OS) — jamais en clair sur disque, jamais exposées au frontend
- **Ollama embarqué** : téléchargé au premier lancement, plus besoin d'installer Ollama séparément
- **Git branch management** : sélecteur de branche dans le chat avec switch, création inline, worktree navigation, file watcher temps réel, dialog de conflit avec commit WIP auto
- **Terminal intégré** : PTY cross-platform avec onglets, raccourci Cmd/Ctrl+J
- **Personnalité et mémoire** : édition des fichiers Markdown de contexte, injection de personnalité et dossiers mémoire locaux
- **Indicateur de contexte** : l'anneau de l'input de chat affiche un résumé par messages, outils, MCP/connecteurs, skills, méta-contexte et system prompt
- **Onboarding** : un écran au premier lancement pour configurer les préférences et Ollama
- **Ollama browser** : recherche de modèles, pull, édition de modelfiles

## Providers supportés

| Type | Provider | Limite / prix affiché |
|---|---|---|
| LLM | [Groq](https://console.groq.com/keys) | 14 400 req/day |
| LLM | [Google Gemini](https://aistudio.google.com/app/apikey) | 1M tokens/min |
| LLM | [Mistral](https://console.mistral.ai/api-keys) | 1B tokens/month |
| LLM | [Cerebras](https://cloud.cerebras.ai/) | 1M tokens/day |
| LLM | [OpenRouter](https://openrouter.ai/settings/keys) | 30+ free models |
| LLM | [OpenAI](https://platform.openai.com/api-keys) | GPT-5.6 Sol / Terra / Luna |
| LLM | [DeepSeek](https://platform.deepseek.com/api_keys) | Low-cost ($0.14/Mtok) |
| LLM | [xAI](https://console.x.ai) | Grok 4.5 / 4.3 / 4.20 |
| LLM | [Moonshot Kimi](https://platform.kimi.ai/console/api-keys) | Low-cost ($0.60/Mtok) |
| LLM | [Z.ai GLM](https://z.ai/manage-apikey/apikey-list) | GLM-4.5-Flash gratuit |
| Search | [Brave Search](https://api-dashboard.search.brave.com/app/keys) | 2 000 req/month |
| Search | [Exa](https://dashboard.exa.ai/api-keys) | 1 000 req/month |
| Scraping | [Firecrawl](https://www.firecrawl.dev/app/api-keys) | 500 crédits |
| Forecast | [Nixtla TimeGPT](https://dashboard.nixtla.io/) | API cloud forecast |

## Modèles Forecast

Beaver inclut un espace Forecast pour analyser des séries temporelles :

- **Familles locales** : Amazon Chronos / Chronos-Bolt, Google TimesFM, Datadog Toto 2.0, Salesforce MOIRAI 2.0, IBM FlowState, PriorLabs TabPFN-TS, NX-AI TiRex, Kairos et THUML Sundial
- **Famille cloud** : Nixtla TimeGPT-2 / TimeGPT-2.1
- **Workflow** : lance des prévisions, garde l'historique, compare les analyses, crée des scénarios, ajoute des notes, exporte les résultats et appelle les outils Forecast depuis l'Agent Local

## Stack technique

- **Backend** : Rust + Tauri 2
- **Frontend** : React 19 + TypeScript + Vite
- **LLM runtime** : Ollama (embarqué comme sidecar)
- **Forecast runtime** : sidecar local Forecast plus API Nixtla optionnelle
- **Connector runtime** : bridge MCP, stockage OAuth et service Gateway pour les channels
- **Sécurité** : vault XChaCha20-Poly1305, master key dans keyring OS (macOS Keychain / Windows DPAPI / Linux Secret Service)
- **File watching** : crate `notify` (FSEvents macOS, inotify Linux, ReadDirectoryChangesW Windows)

## Prérequis

- macOS (Apple Silicon), Linux, ou Windows
- Node.js 20+
- Rust (via `rustup`)

## Installation

### macOS / Linux

```bash
(
  installer="$(mktemp /tmp/beaver-bootstrap.XXXXXXXX)" &&
  trap 'rm -f "$installer"' EXIT &&
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error \
    --output "$installer" \
    https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.sh &&
  bash "$installer"
)
```

Télécharge la dernière release, installe l'app et la lance automatiquement.
- **macOS** : installe dans `/Applications/`
- **Linux** : installe le paquet Debian via `apt-get` (Ubuntu/Debian uniquement)

L'installateur Linux utilise le fichier `.deb` de la release pour rendre l'app visible dans le menu système.

### Windows (PowerShell)

```powershell
$installer = Join-Path ([IO.Path]::GetTempPath()) "beaver-install-$([Guid]::NewGuid().ToString('N')).ps1"
try {
  Invoke-WebRequest https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.ps1 -OutFile $installer -ErrorAction Stop
  & $installer
} finally {
  Remove-Item -LiteralPath $installer -Force -ErrorAction SilentlyContinue
}
```

Télécharge la dernière release et lance l'installeur Windows NSIS `-setup.exe` automatiquement.

> **Windows Defender** : au premier lancement, l'« Accès contrôlé aux dossiers » peut bloquer `ollama.exe`. Clique sur « Autoriser » dans la notification — ça ne redemande plus ensuite.

### Mises à jour

Les mises à jour sont automatiques : une notification apparaît dans l'app quand une nouvelle version est disponible. Un clic et l'app se met à jour toute seule.

---

## Développement

```bash
# 1. Cloner le repo
git clone https://github.com/Kevin-hDev/Beaver.git
cd Beaver

# 2. Installer les dépendances
npm install

# 3. Télécharger le binaire Ollama pour votre OS
cd src-tauri && bash scripts/download-ollama.sh
```

## Commandes

```bash
npm run tauri dev       # Mode dev (hot reload)
npm run tauri build     # Build release (.dmg / -setup.exe / .deb)
npx tsc --noEmit        # Check TypeScript
cd src-tauri && cargo check    # Check Rust
cd src-tauri && cargo clippy --all-targets  # Lint strict
cd src-tauri && cargo test     # Tests unitaires
```

## Architecture

```
src-tauri/                # Backend Rust + Tauri
├── src/
│   ├── commands/         # Tauri IPC (agent_chat, heartbeat, api_keys, llm, forecast, mcp, gateway, ...)
│   ├── services/
│   │   ├── agent_local/  # Sessions, outils, permissions, subagents, todos, diagnostics, Plan mode
│   │   ├── llm/          # Client unifié OpenAI-compat, catalog, streaming SSE
│   │   ├── search/       # Brave, Exa, Firecrawl + routing
│   │   ├── forecast/     # Catalogue Forecast, runs, scénarios, notes, exports, runtime sidecar
│   │   ├── mcp_bridge/   # Config connecteurs MCP, process manager, bridge stdio/HTTP
│   │   ├── mcp_oauth/    # Callback OAuth, stockage tokens, auth connecteurs
│   │   ├── gateway/      # Channels externes, runtime arrière-plan, audit log
│   │   ├── scheduler/    # Scheduler Tokio interne (réveils)
│   │   ├── git/          # Branch ops, status, watcher, worktree listing (git2)
│   │   ├── terminal/     # PTY cross-platform (portable-pty)
│   │   ├── file_preview/ # Previews texte, binaire, image, spreadsheet et documents
│   │   ├── link_preview.rs  # Preview metadata URL
│   │   ├── compress.rs   # Helpers de compression de contexte
│   │   ├── codex_client.rs / codex_oauth.rs  # Client compatible Codex et OAuth
│   │   ├── paths.rs      # Chemin data centralisé cross-platform
│   │   ├── vault.rs      # Vault chiffré XChaCha20-Poly1305
│   │   ├── api_keys.rs   # Gestion clés API (Zeroizing en mémoire)
│   │   ├── favorite_models.rs  # Persistance des modèles favoris
│   │   ├── config.rs     # Lecture/écriture config.json tolérante
│   │   ├── stream_utils.rs  # compute_tps, clean_think_tags partagés
│   │   └── ollama_lifecycle.rs  # Gestion du sidecar Ollama
│   ├── tray.rs           # Tray icon (labels FR/EN dynamiques)
│   ├── storage_migration.rs  # Migration one-shot depuis CL-GO legacy
│   ├── ollama_polling.rs # Polling status Ollama
│   └── models/           # Schémas ScheduledWakeup, HeartbeatConfig
└── resources/              # Icônes et ressources statiques

src/                      # Frontend React
├── components/
│   ├── agent-local/      # Chat, permissions, tabs, outils, file tree, previews, panel Forecast, branching, archives
│   ├── heartbeat/        # Grid réveils, popup création, détails
│   ├── forecast/         # Espace Forecast, charts, scénarios, notes, gestion modèles
│   ├── forecast-docs/    # Documentation Forecast intégrée à l'app
│   ├── ollama/           # Model browser
│   ├── modelfile/        # Éditeur de modelfiles
│   ├── personality/      # Markdown viewer
│   ├── connectors/       # Configuration connecteurs MCP
│   ├── channels/         # Configuration channels Gateway (Telegram, Slack, Discord)
│   ├── onboarding/       # Écran de configuration au premier lancement
│   ├── settings/         # Général, Ollama, connecteurs, channels, API keys, forecast, LLM, avancé, archives
│   ├── terminal/         # Terminal intégré PTY
│   ├── api-keys/         # Configuration clés API
│   ├── file-tree/        # Navigation arbre de fichiers
│   ├── file-preview/     # Previews texte, binaire, image
│   ├── layout/           # Sidebar, toolbar, drag region
│   └── ui/               # Primitives réutilisables
├── hooks/                # Logique extraite par domaine
├── lib/                  # platform.ts (détection OS)
├── types/                # Types TS alignés sur Rust
└── i18n/                 # 7 langues (FR, EN, DE, ES, IT, JA, ZH)
```

## Stockage local

Données dans `~/.local/share/cl-go-dash/` sur les 3 OS. Ce dossier conserve son
identifiant historique pour rester compatible avec les installations existantes :

| Chemin | Contenu |
|---|---|
| `secrets.enc` | Vault chiffré contenant les clés API |
| `configured-providers.json` | Registry des providers configurés |
| `config.json` | Heartbeat config + scheduled_wakeups + advanced settings |
| `agent-sessions/*.json` | Conversations Agent Local |
| `agent-settings.json` | Mode permissions par défaut (auto/manuel/chat) |
| `agent-tabs.json` | État des onglets ouverts |
| `plans/<session_id>/*.md` | Plans Markdown locaux créés par le Plan mode |
| `projects.json` | Projets enregistrés |
| `favorite-models.json` | Liste des modèles favoris |
| `terminal-tabs.json` | Onglets du terminal intégré |
| `personality-injection.json` | Réglages d'injection de personnalité |
| `memory/core/*.md` | Fichiers de personnalité |
| `skills/` | Skills locales |
| `tool-results/` | Sorties complètes persistées pour les gros résultats d'outils |
| `mcp-connectors.json` | Connecteurs MCP configurés |
| `mcp-runtime/` | Données runtime MCP |
| `gateway-session-map.json` | Liens entre Gateway et sessions agent |
| `forecast-analyses/` | Analyses Forecast sauvegardées |
| `forecast-notes/` | Notes Forecast |
| `forecast-models/` | Modèles Forecast locaux installés |
| `forecast-model-configs.json` | Configuration des modèles Forecast |
| `forecast-selected-model.json` | Dernier modèle Forecast sélectionné |
| `forecast-exports/` | Exports Forecast |
| `ollama-custom-models.json` | Métadonnées des modèles Ollama custom |
| `logs/wakeups.jsonl` | Historique d'exécution des réveils (rolling 500 lignes) |
| `logs/gateway-audit.jsonl` | Log d'audit Gateway |
| `logs/ollama-sidecar.log` | Logs stderr du sidecar Ollama (écrasé à chaque démarrage) |

## Ollama — sidecar embarqué

L'application embarque **Ollama** comme sidecar pour éviter toute dépendance externe :

- Au premier lancement, un écran de setup télécharge Ollama automatiquement dans `~/.local/share/cl-go-dash/ollama-bundle/`
- Au démarrage, l'app vérifie si un daemon Ollama tourne déjà sur `localhost:11434`
- Si oui (Ollama.app déjà installée), elle l'utilise tel quel
- Si non, elle lance son propre binaire téléchargé
- À la fermeture, le sidecar est arrêté proprement (SIGTERM Unix / kill Windows + grace period 3s)
- Sur Linux, détection GPU automatique (AMD → archive ROCm, Nvidia → archive standard avec CUDA)

**Les modèles sont partagés** avec Ollama.app si elle est installée (`~/.ollama/models/`).

## Sécurité

- **Vault chiffré** : clés API chiffrées XChaCha20-Poly1305, master key dans le keyring OS natif (Keychain / DPAPI / Secret Service)
- **JS ne voit jamais une clé** : aucune commande Tauri n'expose `get_api_key` ; les secrets restent côté Rust et sont zéroïsés après usage
- **Path traversal** : `canonicalize()` + `starts_with()` sur tout chemin venant du frontend
- **Collections bornées** : ActiveStreams (32), PTY sessions (16), messages par session (2000), profondeur/taille JSON MCP limitées
- **HTTP sécurisé pour les credentials** : redirections bloquées, HTTPS imposé, messages d'erreur sanitizés
- **Durcissement MCP** : allowlist de programmes, pas de shell, validation des arguments, isolation de l'environnement
- **Logs filtrés** : body HTTP providers tronqué à 200 chars, formats de credentials connus masqués

Pour le modèle de menace complet, la politique de signalement de vulnérabilité et les recommandations d'usage sûr, voir **[SECURITY.md](SECURITY.md)**.

## Licence

[Apache License 2.0](LICENSE)
