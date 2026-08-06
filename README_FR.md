# Beaver

Beaver est un espace de travail agentique pour les modèles locaux via Ollama et les modèles cloud via une clé API ou un compte web. L'application réunit les conversations, les outils, la planification, les sous-agents, la mémoire persistante, un navigateur intégré, Git, les prévisions, les connecteurs MCP, les réveils automatiques, les aperçus de fichiers et un terminal.

## Fonctionnalités

- **Agent local et outils** : utilise les modèles locaux ou cloud avec les fichiers, les commandes système, la recherche web, les documents Office, Git, MCP, Forecast, les diagnostics, les todos et les choix interactifs
- **Planification et permissions** : explore en sécurité avec le mode Plan, enregistre des plans Markdown, valide leur mise en œuvre et choisis des permissions automatiques, manuelles ou propres à chaque chat
- **Conversations et projets** : gère les discussions en onglets, les pièces jointes, les favoris, les messages en attente, les branches de conversation, les archives, les résumés cachés et les dossiers de projet
- **Sous-agents contrôlés par le parent** : coordonne des sessions enfant isolées, suis leur activité, corrige-les ou réutilise-les, examine leurs changements et nettoie leurs worktrees en sécurité
- **Mémoire persistante** : conserve une mémoire globale et une mémoire par projet, avec modes manuel ou automatique, résumés limités, fichiers par sujet, activité visible et accès en lecture seule pour les sous-agents
- **Navigateur intégré** : navigue dans dix onglets maximum par conversation, conserve les connexions web, détecte les sites locaux et partage le panneau latéral avec les aperçus et Forecast. Disponible sur macOS et Windows
- **Workflow Git complet** : crée, change, fusionne et supprime des branches ou worktrees ; crée des commits et pousse-les ; parcours les changements et consulte les différences récentes ou historiques
- **Forecast V2** : contrôle les séries temporelles, sélectionne les modèles manuellement ou automatiquement, lance des prévisions locales ou cloud, compare les backtests, crée des ensembles, explore les analyses avancées et exporte les résultats
- **Fournisseurs et consommation** : connecte OpenAI/Codex, Grok et Kimi avec un compte web, utilise les fournisseurs par clé API et consulte les limites, crédits, tokens, requêtes et coûts estimés disponibles
- **Connecteurs MCP et canaux** : active des connecteurs MCP locaux ou cloud par conversation et relie éventuellement la Gateway à Telegram, Slack ou Discord
- **Réveils** : programme des demandes ponctuelles, quotidiennes ou hebdomadaires avec le scheduler interne et conserve chaque résultat dans une conversation dédiée
- **Ollama géré par Beaver** : télécharge Ollama au premier lancement, réutilise un service existant, parcourt et installe les modèles, modifie les modelfiles et configure les paramètres ou instructions de chaque modèle
- **Espace de travail desktop** : utilise le terminal à onglets, l'arbre de fichiers, les aperçus enrichis et Office, les liens, le détail du contexte, six thèmes visuels et le compagnon Beaver interactif
- **Démarrage et migration guidés** : configure Beaver au premier lancement et importe des instructions, skills ou règles depuis Claude Code, Codex, Agents, Hermes, Qwen Code, ZCode, OpenClaw, OpenCode et Kimi Code
- **Stockage local sécurisé** : conserve les identifiants dans un coffre chiffré XChaCha20-Poly1305 dont la clé maître reste dans le trousseau du système ; les secrets bruts ne sont jamais envoyés à l'interface

## Fournisseurs compatibles

| Type | Fournisseur | Connexion |
|---|---|---|
| LLM | [Groq](https://console.groq.com/keys) | Clé API |
| LLM | [Google Gemini](https://aistudio.google.com/app/apikey) | Clé API |
| LLM | [Mistral](https://console.mistral.ai/api-keys) | Clé API |
| LLM | [Cerebras](https://cloud.cerebras.ai/) | Clé API |
| LLM | [OpenRouter](https://openrouter.ai/settings/keys) | Clé API |
| LLM | [OpenAI](https://platform.openai.com/api-keys) | Clé API ou compte web OpenAI/Codex |
| LLM | [DeepSeek](https://platform.deepseek.com/api_keys) | Clé API |
| LLM | [xAI](https://console.x.ai) | Clé API ou compte web Grok |
| LLM | [Moonshot Kimi](https://platform.kimi.ai/console/api-keys) | Clé API ou compte web Kimi expérimental |
| LLM | [Z.ai GLM](https://z.ai/manage-apikey/apikey-list) | Clé API |
| Recherche | [Brave Search](https://api-dashboard.search.brave.com/app/keys) | Clé API |
| Recherche | [Exa](https://dashboard.exa.ai/api-keys) | Clé API |
| Recherche / extraction | [Firecrawl](https://www.firecrawl.dev/app/api-keys) | Clé API |
| Recherche | SearXNG | Solution locale de secours sans clé API |
| Prévisions | [Nixtla TimeGPT](https://dashboard.nixtla.io/) | Clé API |

Les modèles, quotas et prix peuvent changer chez les fournisseurs. Beaver affiche les informations actuelles du compte lorsque le fournisseur les rend disponibles.

## Modèles Forecast

Beaver inclut un espace Forecast dédié à l'analyse des séries temporelles :

- **Familles locales** : Amazon Chronos / Chronos-Bolt, Google TimesFM, Datadog Toto 2.0, Salesforce MOIRAI 2.0, IBM FlowState, PriorLabs TabPFN-TS, NX-AI TiRex, Kairos et THUML Sundial
- **Famille cloud** : Nixtla TimeGPT-2 / TimeGPT-2.1
- **Sélection et qualité des données** : choisis un modèle ou laisse Beaver le sélectionner selon les données, le matériel, l'horizon, la fréquence, le besoin d'incertitude et les capacités du modèle
- **Évaluation et analyse** : lance des backtests glissants, compare les références et les modèles, puis examine MASE, sMAPE, MAE, couverture, anomalies, dérive, décomposition, importance des variables et ensembles pondérés
- **Espace de travail et exports** : explore les vues Données, Prévisions, Évaluation, Comparaison, Scénarios, Notes et Rapport, puis exporte en CSV, Excel, JSON, PNG, SVG, PDF ou vers le presse-papiers

## Stack technique

- **Backend** : Rust + Tauri 2
- **Frontend** : React 19 + TypeScript + Vite
- **Runtime LLM local** : Ollama géré et téléchargé par Beaver
- **Forecast runtime** : sidecar local Forecast plus API Nixtla optionnelle
- **Navigateur** : Chromium Embedded Framework isolé sur macOS et Windows
- **Recherche** : Brave, Exa et Firecrawl avec SearXNG comme solution locale de secours
- **Connector runtime** : bridge MCP, stockage OAuth et service Gateway pour les channels
- **Sécurité** : vault XChaCha20-Poly1305, master key dans keyring OS (macOS Keychain / Windows DPAPI / Linux Secret Service)
- **File watching** : crate `notify` (FSEvents macOS, inotify Linux, ReadDirectoryChangesW Windows)

## Prérequis

- macOS (Apple Silicon), Linux, ou Windows
- Node.js 20+
- Rust (via `rustup`)

## Installation

### macOS / Linux (une commande)

```bash
curl -fsSL https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.sh | bash
```

Télécharge la dernière release, installe l'app et la lance automatiquement.
- **macOS** : installe dans `/Applications/`
- **Linux** : installe le paquet Debian via `apt-get` (Ubuntu/Debian uniquement)

L'installateur Linux utilise le fichier `.deb` de la release pour rendre l'app visible dans le menu système.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.ps1 | iex
```

Télécharge la dernière release et lance l'installeur Windows NSIS `-setup.exe` automatiquement.

> **Windows Defender** : au premier lancement, l'« Accès contrôlé aux dossiers » peut bloquer `ollama.exe`. Clique sur « Autoriser » dans la notification — ça ne redemande plus ensuite.

### Mises à jour

Les mises à jour sont automatiques : une notification apparaît dans l'app quand une nouvelle version est disponible. Un clic et l'app se met à jour toute seule.

### De CL-GO à Beaver

Beaver est le nouveau nom de CL-GO. Les utilisateurs existants passent par la version-pont CL-GO 1.0.2 et conservent leurs conversations, réglages, identifiants, connecteurs MCP, mémoire, modèles Forecast, données Ollama et sessions du navigateur. Les anciens identifiants internes et le dossier de données décrit plus bas sont volontairement conservés pour assurer la compatibilité.

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
npm run lint            # Vérifications du frontend et des limites React
npm test                # Tests du frontend et du navigateur intégré
npx tsc --noEmit        # Check TypeScript
cd src-tauri && cargo check    # Check Rust
cd src-tauri && cargo clippy --all-targets  # Lint strict
cd src-tauri && cargo test     # Tests unitaires
```

## Architecture

```
src-tauri/                # Backend Rust + Tauri
├── src/
│   ├── commands/         # Commandes Tauri organisées par domaine
│   ├── services/
│   │   ├── agent_local/  # Sessions, outils, permissions, plans, mémoire, sous-agents
│   │   ├── agent_import/ # Import guidé depuis d'autres applications agentiques
│   │   ├── browser/      # Sessions Chromium isolées et vues natives
│   │   ├── llm/          # Client cloud unifié, catalogue, raisonnement, streaming
│   │   ├── codex_client/ et *_oauth/  # Connexions web OpenAI, Grok, Kimi et MCP
│   │   ├── provider_usage/  # Limites, historique d'usage et coûts estimés
│   │   ├── search/ et searxng/  # Recherche cloud et solution locale
│   │   ├── forecast/     # Contrôles des données, modèles, analyses et exports
│   │   ├── mcp_bridge/ et mcp_oauth/  # Connecteurs MCP locaux et cloud
│   │   ├── gateway/      # Canaux Telegram, Slack et Discord en arrière-plan
│   │   ├── git/          # Branches, worktrees, commits, push, fusions, différences
│   │   ├── scheduler/ et terminal/  # Réveils et terminal cross-platform
│   │   ├── paths.rs      # Chemin de données centralisé
│   │   ├── vault.rs      # Coffre chiffré XChaCha20-Poly1305
│   │   └── private_store/  # Fichiers privés et permissions système
│   ├── tray.rs           # Intégration dans la barre système
│   ├── storage_migration.rs  # Initialisation et compatibilité du stockage
│   └── ollama_polling.rs # Surveillance de l'état d'Ollama
└── resources/              # Icônes et ressources statiques

src/                      # Frontend React
├── components/
│   ├── agent-local/ et agent-side-panel/  # Chat et panneau latéral partagé
│   ├── agent-import/     # Assistant de migration depuis d'autres agents
│   ├── internal-browser/ # Interface du navigateur intégré
│   ├── forecast/         # Espace de travail, graphiques, évaluations, notes, modèles
│   ├── providers/        # Connexions API/web et détails de consommation
│   ├── connectors/ et channels/  # Configuration MCP et Gateway
│   ├── ollama/           # Catalogue et personnalisation des modèles locaux
│   ├── heartbeat/        # Planification et historique des réveils
│   ├── file-tree/ et file-preview/  # Navigation et aperçus enrichis
│   ├── mascot/           # Compagnon Beaver interactif
│   ├── onboarding/ et settings/  # Démarrage et préférences
│   └── terminal/ et ui/  # Terminal intégré et composants partagés
├── hooks/                # Logique extraite par domaine
├── lib/                  # Outils partagés et détection du système
├── types/                # Types TS alignés sur Rust
└── i18n/                 # 7 langues (FR, EN, DE, ES, IT, JA, ZH)
```

## Stockage local

Données dans `~/.local/share/cl-go-dash/` sur les 3 OS. Ce dossier conserve son
identifiant historique pour rester compatible avec les installations existantes :

| Chemin | Contenu |
|---|---|
| `secrets.enc` | Identifiants API et OAuth chiffrés |
| `configured-providers.json`, `provider-usage.json` | Fournisseurs connectés et historique local de consommation |
| `config.json`, `heartbeat-runtime.json` | Réglages de l'application et état des réveils |
| `agent-sessions/*.json` | Conversations de l'Agent local |
| `agent-settings.json`, `session-tabs.json` | Permissions et onglets de conversation ouverts |
| `projects.json`, `favorite-models.json`, `terminal-tabs.json` | Projets, modèles favoris et onglets du terminal |
| `AGENTS.md`, `external-agent-sources.json`, `agent-import-backups/` | Instructions importées, sources externes et sauvegardes |
| `plans/`, `skills/`, `tool-results/` | Plans de l'agent, skills locales et gros résultats d'outils |
| `subagent-changes/`, `subagent-worktrees/` | Changements et worktrees isolés des sous-agents |
| `memory/core/` | Fichiers Markdown de personnalité et de contexte |
| `memory/global/`, `memory/projects/`, `memory-settings.json` | Mémoire persistante globale et par projet |
| `browser/` | Sessions chiffrées et profil Chromium privé |
| `mcp-connectors.json`, `mcp-runtime/` | Configuration et données des connecteurs MCP |
| `gateway-session-map.json`, `logs/gateway-audit.jsonl` | Liens des sessions Gateway et historique d'audit |
| `forecast-*` | Analyses, profils de données, modèles, réglages, brouillons, notes et exports Forecast |
| `ollama-*` | Runtime Ollama, métadonnées des modèles et instructions personnalisées |
| `searxng-sidecar/` | Runtime local de recherche SearXNG |
| `logs/` | Journaux limités des réveils, de Gateway, Ollama, SearXNG et des outils |

## Ollama — runtime géré

Beaver gère **Ollama** localement pour éviter une installation manuelle séparée :

- Au premier lancement, un écran de setup télécharge Ollama automatiquement dans `~/.local/share/cl-go-dash/ollama-bundle/`
- Au démarrage, l'app vérifie si un daemon Ollama tourne déjà sur `localhost:11434`
- Si oui (Ollama.app déjà installée), elle l'utilise tel quel
- Si non, elle lance son propre binaire téléchargé
- À la fermeture, le sidecar est arrêté proprement (SIGTERM Unix / kill Windows + grace period 3s)
- Sur Linux, détection GPU automatique (AMD → archive ROCm, Nvidia → archive standard avec CUDA)
- Les paramètres, instructions système et modelfiles complets peuvent être personnalisés dans Beaver

**Les modèles sont partagés** avec Ollama.app si elle est installée (`~/.ollama/models/`).

## Sécurité

- **Vault chiffré** : clés API chiffrées XChaCha20-Poly1305, master key dans le keyring OS natif (Keychain / DPAPI / Secret Service)
- **JS ne voit jamais une clé** : aucune commande Tauri n'expose `get_api_key` ; les secrets restent côté Rust et sont zéroïsés après usage
- **Protection des chemins** : les chemins demandés par l'interface sont validés, normalisés et maintenus dans leurs dossiers autorisés
- **Collections bornées** : ActiveStreams (32), PTY sessions (16), messages par session (2000), profondeur/taille JSON MCP limitées
- **HTTP sécurisé pour les credentials** : redirections bloquées, HTTPS imposé, messages d'erreur sanitizés
- **Durcissement MCP** : allowlist de programmes, pas de shell, validation des arguments, isolation de l'environnement
- **Navigateur protégé** : processus isolés, navigation limitée, permissions sensibles bloquées, profil privé et onglets restaurés sous forme chiffrée
- **Mises à jour vérifiées** : métadonnées strictes, téléchargements limités, manifestes SHA-256, contrôle de santé et installation bloquée en cas d'échec
- **Logs filtrés** : body HTTP providers tronqué à 200 chars, formats de credentials connus masqués

Pour le modèle de menace complet, la politique de signalement de vulnérabilité et les recommandations d'usage sûr, voir **[SECURITY.md](SECURITY.md)**.

Pour consulter l'historique complet des versions, voir **[CHANGELOG.md](CHANGELOG.md)**.

## Licence

Beaver est distribué sous **[GNU Affero General Public License v3.0](LICENSE)**.

Copyright © 2026 Kevin Huynh

Vous êtes libre d'utiliser, d'étudier, de modifier et de redistribuer Beaver.
En contrepartie, toute version distribuée ou accessible via un réseau —
modifiée ou non — doit être publiée sous AGPL v3 et fournir l'intégralité de
son code source.

Les contributions sont bienvenues et nécessitent la signature du CLA décrit
dans **[CONTRIBUTING.md](CONTRIBUTING.md)**.

Pour une licence commerciale vous dispensant des obligations de l'AGPL,
contactez huynh.kevin7@outlook.fr.

Les composants tiers conservent leur propre licence — voir
**[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)**.

> Les versions publiées jusqu'à la v1.1.2 incluse l'ont été sous Apache
> License 2.0 et restent disponibles selon ces termes.
