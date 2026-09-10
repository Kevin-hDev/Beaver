<div align="center">

# Beaver

**Les autres agents te donnent ce qu'ils ont décidé.<br>Beaver te donne tout, puis les clés.**

Beaver est un agent IA de bureau open source qui arrive complet : modèles cloud par clé API ou compte web connecté, modèles entièrement locaux via Ollama, et les outils standard des grands harnais agentiques, opérationnels dès le premier lancement. Puis il te tend les clés — remplace ses outils, réécris ses prompts système, règle sa compression de contexte, remodèle son interface. Par extensions, sans maintenir un fork.

[![Dernière version](https://img.shields.io/github/v/release/Kevin-hDev/Beaver?label=version)](https://github.com/Kevin-hDev/Beaver/releases/latest)
[![Téléchargements](https://img.shields.io/github/downloads/Kevin-hDev/Beaver/total?label=t%C3%A9l%C3%A9chargements)](https://github.com/Kevin-hDev/Beaver/releases)
[![Licence : AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-orange)](LICENSE)

**[Documentation](https://kevin-hdev.github.io/Beaver/)** · **[Installer](#installation)** · **[Tout est déjà là](#tout-est-déjà-là)** · **[Puis les clés](#puis-les-clés)** · **[Sécurité](SECURITY.md)** · **[English](README.md)**

macOS (Apple Silicon) · Linux · Windows

<!-- TODO : GIF de démonstration et captures d'écran -->

</div>

Beaver fonctionne sur ton ordinateur ; le modèle choisi peut fonctionner dans le cloud ou localement. L'espace Agent prend en charge les deux, sans être réservé à l'IA locale.

## Tout est déjà là

### L'agent

- **Agent et outils** : utilise les modèles cloud ou locaux avec les fichiers, les commandes système, la recherche web, les documents Office, Git, MCP, Forecast, les diagnostics, les todos et les choix interactifs
- **Planification et permissions** : explore en sécurité avec le mode Plan, enregistre des plans Markdown, valide leur mise en œuvre et choisis des permissions automatiques, manuelles ou propres à chaque chat
- **Sous-agents contrôlés par le parent** : coordonne des sessions enfant isolées, suis leur activité, corrige-les ou réutilise-les, examine leurs changements et nettoie leurs worktrees en sécurité
- **Mémoire persistante** : conserve une mémoire globale et une mémoire par projet, avec modes manuel ou automatique, résumés limités, fichiers par sujet, activité visible et accès en lecture seule pour les sous-agents
- **Raisonnement et continuité multimodale** : utilise les réglages de raisonnement et les images validées pour chaque modèle pendant que Beaver conserve le raisonnement natif du fournisseur entre les messages et les appels d'outils sans exposer son état privé

### L'espace de travail

- **Conversations et projets** : gère les discussions en onglets, les pièces jointes, les favoris, les messages en attente, les branches de conversation, les archives, les résumés cachés et les dossiers de projet
- **Navigateur intégré** : navigue dans dix onglets maximum par conversation, conserve les connexions web, détecte les sites locaux et partage le panneau latéral avec les aperçus et Forecast. Disponible sur macOS et Windows
- **Workflow Git complet** : crée, change, fusionne et supprime des branches ou worktrees ; crée des commits et pousse-les ; parcours les changements et consulte les différences récentes ou historiques
- **Espace de travail desktop** : utilise le terminal à onglets, l'arbre de fichiers, les aperçus enrichis et Office, les liens, le détail du contexte, six thèmes visuels et le compagnon Beaver interactif
- **Démarrage et migration guidés** : configure Beaver au premier lancement et importe des instructions, skills ou règles depuis Claude Code, Codex, Agents, Hermes, Qwen Code, ZCode, OpenClaw, OpenCode et Kimi Code

### Modèles et fournisseurs

- **Fournisseurs et consommation** : connecte OpenAI/Codex, Grok et Kimi avec un compte web ; configure Anthropic, Alibaba Cloud Qwen et les autres fournisseurs par clé API ; puis consulte les limites, crédits, tokens, requêtes et coûts estimés disponibles
- **Modèles locaux via Ollama** : Beaver gère le téléchargement du runtime et la réutilisation d'un service disponible ; parcours et installe les modèles locaux, modifie leurs modelfiles et configure leurs paramètres ou instructions
- **Stockage local sécurisé** : conserve les identifiants dans un coffre chiffré XChaCha20-Poly1305 dont la clé maître reste dans le trousseau du système

### Automatisation et connecteurs

- **Connecteurs MCP et canaux** : active des connecteurs MCP locaux ou cloud par conversation et relie éventuellement la Gateway à Telegram, Slack ou Discord
- **Réveils** : programme des demandes ponctuelles, quotidiennes ou hebdomadaires avec le scheduler interne et conserve chaque résultat dans une conversation dédiée
- **Forecast V2** : contrôle les séries temporelles, sélectionne les modèles manuellement ou automatiquement, lance des prévisions locales ou cloud, compare les backtests, crée des ensembles, explore les analyses avancées et exporte les résultats

## Puis les clés

Rien de tout ça n'est soudé. Beaver peut être remodelé sans maintenir une copie modifiée de l'application — le système d'extensions est disponible et continue d'évoluer :

- **Extensions personnalisées** : installe des extensions JavaScript ou TypeScript de confiance depuis une source locale, Git ou npm ; ajoute des outils agentiques, événements, onglets, réglages, actions et thèmes via l'[API d'extensions Beaver](./EXTENSIONS.md) versionnée
- **Contrôle des outils** : active ou désactive les outils intégrés directement dans l'application — le socle fichiers, shell et web reste actif ; les outils natifs explicitement remplaçables peuvent être remplacés par les tiens via l'API d'extensions avancée
- **Prompts système personnalisés** : ajuste le prompt système global ou les prompts propres aux modèles Ollama directement dans l'application
- **Compression du contexte personnalisable** : crée des profils réutilisables, choisis-les globalement ou par conversation, règle les éléments conservés selon la taille du contexte et prévisualise le prochain point de compression limité avant son exécution
- **Capacités de l'agent** : ajoute des outils, des skills chargés à la demande, des ressources et des résultats contenant des fichiers ou aperçus d'images. Le Chat classique reste séparé : ses seuls outils sont la recherche web et la lecture de pages
- **Personnalisation de l'interface** : ajoute des onglets, panneaux, réglages, actions et thèmes dans les emplacements prévus ; les modules d'interface avancés demandent une approbation explicite
- **Plugins Beaver** : active Documents, PDF, Feuilles de calcul et Présentations indépendamment. Ces plugins Office restent distincts des Tools internes
- **Gestion et récupération** : active ou désactive les extensions, choisis les raccourcis du chat, consulte les diagnostics et récupère une application perturbée par une extension grâce au mode sûr

Les extensions exécutent du code de confiance, pas du code enfermé dans un bac à sable. Installe uniquement du code auquel tu fais confiance : une extension approuvée peut accéder aux ressources locales et aux identifiants pris en charge par l'API d'extensions. Les modules d'interface avancés comportent des risques supplémentaires.

Le guide **[EXTENSIONS.md](EXTENSIONS.md)** détaille l'utilisation et la création d'extensions : installation, exemples, compatibilité, limites, permissions et dépannage. Ce guide est actuellement en français.

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

Les mises à jour sont automatiques : une notification apparaît dans l'app quand une nouvelle version est disponible, avec les notes de version traduites lorsque la release les fournit. Un clic et l'app se met à jour toute seule.

### De CL-GO à Beaver

Beaver est le nouveau nom de CL-GO. Les utilisateurs existants passent par la version-pont CL-GO 1.0.2 et conservent leurs conversations, réglages, identifiants, connecteurs MCP, mémoire, modèles Forecast, données Ollama et sessions du navigateur. Les anciens identifiants internes et le dossier de données décrit plus bas sont volontairement conservés pour assurer la compatibilité.

## Fournisseurs compatibles

| Type | Fournisseur | Connexion |
|---|---|---|
| LLM | [Google Gemini](https://aistudio.google.com/app/apikey) | Clé API |
| LLM | [Mistral](https://console.mistral.ai/api-keys) | Clé API |
| LLM | [Cerebras](https://cloud.cerebras.ai/) | Clé API |
| LLM | [OpenRouter](https://openrouter.ai/settings/keys) | Clé API |
| LLM | [OpenAI](https://platform.openai.com/api-keys) | Clé API ou compte web OpenAI/Codex |
| LLM | [DeepSeek](https://platform.deepseek.com/api_keys) | Clé API |
| LLM | [xAI](https://console.x.ai) | Clé API ou compte web Grok |
| LLM | [Moonshot Kimi](https://platform.kimi.ai/console/api-keys) | Clé API ou compte web Kimi expérimental |
| LLM | [Z.ai GLM](https://z.ai/manage-apikey/apikey-list) | Clé API |
| LLM | [Anthropic Claude](https://platform.claude.com/settings/keys) | Clé API |
| LLM | [Alibaba Cloud Qwen](https://modelstudio.console.alibabacloud.com/) | Clé API et région Model Studio |
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
- **Runtime d'extensions** : Node.js et npm embarqués, avec Jiti pour les extensions JavaScript et TypeScript
- **Runtime LLM local** : Ollama géré et téléchargé par Beaver
- **Forecast runtime** : sidecar local Forecast plus API Nixtla optionnelle
- **Navigateur** : Chromium Embedded Framework isolé sur macOS et Windows
- **Recherche** : Brave, Exa et Firecrawl avec SearXNG comme solution locale de secours
- **Connector runtime** : bridge MCP, stockage OAuth et service Gateway pour les channels
- **Sécurité** : vault XChaCha20-Poly1305, master key dans keyring OS (macOS Keychain / Windows DPAPI / Linux Secret Service)
- **File watching** : crate `notify` (FSEvents macOS, inotify Linux, ReadDirectoryChangesW Windows)

## Runtimes externes (optionnels)

- macOS (Apple Silicon), Linux ou Windows
- Node.js 24 LTS — pour le développement et les outils externes nécessitant une installation système
- CPython 3.14 — uniquement pour la solution locale SearXNG

L'application distribuée embarque Node.js et npm pour son hôte d'extensions ; utiliser les extensions ne demande donc pas, à lui seul, d'installer Node.js séparément. Ce runtime embarqué n'installe pas Node.js globalement pour les autres programmes. CPython 3.14 reste un prérequis externe uniquement pour la solution locale SearXNG.

Utilise les instructions suivantes lorsque tu as besoin de ces runtimes externes : ce ne sont pas des étapes obligatoires pour toute installation de Beaver.

Les commandes ci-dessous ont été vérifiées le 31 août 2026 avec la [page officielle de téléchargement de Node.js](https://nodejs.org/en/download) (Node.js 24.20.0 LTS) et la [documentation d'Astral uv](https://docs.astral.sh/uv/getting-started/installation/). Elles évitent de dépendre d'un gestionnaire de paquets propre à une distribution Linux.

### macOS (Apple Silicon)

Installe Node.js et uv :

```bash
curl -fsSLO https://nodejs.org/dist/v24.20.0/node-v24.20.0.pkg
sudo installer -pkg node-v24.20.0.pkg -target /
rm node-v24.20.0.pkg

curl -LsSf https://astral.sh/uv/install.sh | sh
```

Ferme et rouvre le terminal. Installe ensuite CPython pour la solution locale SearXNG :

```bash
UV_PYTHON_BIN_DIR="$HOME/.local/bin" UV_PYTHON_INSTALL_BIN=1 uv python install 3.14
```

Ferme et rouvre le terminal et Beaver, puis vérifie :

```bash
node --version
python3.14 --version
```

### Linux (x64)

Installe Node.js et uv :

```bash
(
set -e
curl -fsSLO https://nodejs.org/dist/v24.20.0/node-v24.20.0-linux-x64.tar.xz
mkdir -p "$HOME/.local/opt" "$HOME/.local/bin"
tar -xJf node-v24.20.0-linux-x64.tar.xz -C "$HOME/.local/opt"
rm node-v24.20.0-linux-x64.tar.xz
nodeRoot="$HOME/.local/opt/node-v24.20.0-linux-x64"
binDir="$HOME/.local/bin"
for executable in node npm npx corepack; do
  target="$nodeRoot/bin/$executable"
  destination="$binDir/$executable"
  if [ ! -x "$target" ]; then
    printf 'Exécutable Node indisponible : %s\n' "$target" >&2
    exit 1
  elif [ ! -e "$destination" ] && [ ! -L "$destination" ]; then
    ln -s "$target" "$destination" || exit 1
  elif [ -L "$destination" ] && [ "$(readlink "$destination")" = "$target" ]; then
    : # Lien déjà géré : ne pas le modifier.
  else
    printf 'Remplacement de %s refusé ; déplace-le manuellement, puis relance cette commande.\n' "$destination" >&2
    exit 1
  fi
done

curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR="$binDir" sh
)
```

Ferme et rouvre le terminal. `UV_INSTALL_DIR` demande à l'installateur officiel d'uv de configurer ce dossier `~/.local/bin`, où se trouvent les liens Node.js protégés. Installe ensuite CPython pour la solution locale SearXNG dans le même dossier d'exécutables :

```bash
UV_PYTHON_BIN_DIR="$HOME/.local/bin" UV_PYTHON_INSTALL_BIN=1 uv python install 3.14
```

Ferme et rouvre le terminal et Beaver, puis vérifie :

```bash
node --version
python3.14 --version
```

### Windows (PowerShell)

Installe Node.js et uv :

```powershell
$nodeInstaller = Join-Path $env:TEMP "node-v24.20.0-x64.msi"
Invoke-WebRequest -Uri "https://nodejs.org/dist/v24.20.0/node-v24.20.0-x64.msi" -OutFile $nodeInstaller
Start-Process msiexec.exe -Wait -ArgumentList @("/i", $nodeInstaller, "/passive")
Remove-Item $nodeInstaller

powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

Ferme et rouvre PowerShell. Installe ensuite CPython pour la solution locale SearXNG :

```powershell
$env:UV_PYTHON_INSTALL_BIN = "1"
uv python install 3.14
uv python update-shell
```

Sous Windows, Beaver suit le dossier d'exécutables choisi par l'installateur officiel d'uv ; `uv python update-shell` publie ce dossier sans le remplacer par un chemin de type Unix.

Ferme et rouvre PowerShell et Beaver, puis vérifie :

```powershell
node --version
python3.14 --version
```

### Développement uniquement

Rust (via [`rustup`](https://rustup.rs/)) est nécessaire uniquement pour compiler ou développer Beaver ; il n'est pas requis par l'application installée.

---

## Développement

Installe d'abord Node.js depuis la section des runtimes externes ci-dessus. CPython est nécessaire uniquement pour SearXNG local. Installe ensuite les dépendances du projet :

```bash
# 1. Cloner le repo
git clone https://github.com/Kevin-hDev/Beaver.git
cd Beaver

# 2. Installer les dépendances
npm install

# 3. Télécharger le binaire Ollama pour votre OS
(cd src-tauri && bash scripts/download-ollama.sh)
```

## Commandes

```bash
npm run tauri dev       # Mode dev (hot reload)
npm run tauri build     # Build release (.dmg / -setup.exe / .deb)
npm run lint            # Vérifications du frontend et des limites React
npm test                # Tests du frontend, navigateur, hôte d'extensions et scripts
npx tsc --noEmit        # Check TypeScript
(cd src-tauri && cargo check)    # Check Rust
(cd src-tauri && cargo clippy --all-targets -- -D warnings)  # Lint strict
(cd src-tauri && cargo test)     # Tests unitaires
```

## Architecture

```text
src-tauri/                # Backend Rust + Tauri
├── src/
│   ├── commands/         # Commandes Tauri organisées par domaine
│   ├── services/
│   │   ├── agent_local/  # Sessions, outils, permissions, plans, mémoire, sous-agents
│   │   ├── agent_import/ # Import guidé depuis d'autres applications agentiques
│   │   ├── browser/      # Sessions Chromium isolées et vues natives
│   │   ├── compress/     # Profils de contexte, points de compression et résumés limités
│   │   ├── extensions/   # Registre, permissions, hôtes et récupération des extensions
│   │   ├── llm/          # Transports fournisseurs, catalogue, raisonnement, streaming
│   │   ├── codex_client/ et *_oauth/  # Connexions web OpenAI, Grok, Kimi et MCP
│   │   ├── provider_connections/  # Configuration des endpoints propres aux fournisseurs
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
└── resources/            # Icônes, ressources statiques et extension-host/ embarqué

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
├── features/extension-ui/ # Contributions d'interface des extensions et cycle de vie
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
| `agent-sessions/*.json` | Conversations de l'Agent |
| `agent-settings.json`, `session-tabs.json` | Permissions et onglets de conversation ouverts |
| `compression-profiles.json` | Profils réutilisables de compression du contexte et sélection globale |
| `projects.json`, `favorite-models.json`, `terminal-tabs.json` | Projets, modèles favoris et onglets du terminal |
| `AGENTS.md`, `external-agent-sources.json`, `agent-import-backups/` | Instructions importées, sources externes et sauvegardes |
| `plans/`, `skills/`, `tool-results/` | Plans de l'agent, skills locales et gros résultats d'outils |
| `subagent-changes/`, `subagent-worktrees/` | Changements et worktrees isolés des sous-agents |
| `memory/core/` | Fichiers Markdown de personnalité et de contexte |
| `memory/global/`, `memory/projects/`, `memory-settings.json` | Mémoire persistante globale et par projet |
| `browser/` | Sessions chiffrées et profil Chromium privé |
| `mcp-connectors.json`, `mcp-runtime/` | Configuration et données des connecteurs MCP |
| `extensions.json`, `extension-installs/` | Registre des extensions et installations gérées |
| `extension-discovery-preferences.json`, `extension-session-state/` | Préférences de découverte des extensions et état par conversation |
| `gateway-session-map.json`, `logs/gateway-audit.jsonl` | Liens des sessions Gateway et historique d'audit |
| `forecast-*` | Analyses, profils de données, modèles, réglages, brouillons, notes et exports Forecast |
| `ollama-*` | Runtime Ollama, métadonnées des modèles et instructions personnalisées |
| `searxng-sidecar/` | Runtime local de recherche SearXNG |
| `logs/` | Journaux limités des réveils, de Gateway, Ollama, SearXNG et des outils |

## Ollama — runtime géré

Pour les modèles locaux, Beaver gère **Ollama** afin d'éviter une installation manuelle séparée. Les requêtes aux modèles cloud passent par leurs fournisseurs, pas par Ollama :

- Au premier lancement, un écran de setup télécharge Ollama automatiquement dans `~/.local/share/cl-go-dash/ollama-bundle/`
- Beaver vérifie la disponibilité du runtime et gère le lancement ou la réutilisation d'un service Ollama
- Le runtime géré dispose d'un démarrage, d'un arrêt et d'une récupération supervisés ; un service lancé indépendamment n'est pas traité comme un processus enfant appartenant à Beaver
- Sur Linux, détection GPU automatique (AMD → archive ROCm, Nvidia → archive standard avec CUDA)
- Les paramètres, instructions système et modelfiles complets peuvent être personnalisés dans Beaver

**Les modèles sont partagés** avec Ollama.app si elle est installée (`~/.ollama/models/`).

## Sécurité

- **Vault chiffré** : clés API chiffrées XChaCha20-Poly1305, master key dans le keyring OS natif (Keychain / DPAPI / Secret Service)
- **Frontière des identifiants** : l'interface intégrée de gestion des clés ne propose pas de commande pour lire les clés API enregistrées. Les extensions approuvées peuvent demander les secrets pris en charge par l'API d'extensions ; cette frontière de confiance est détaillée dans [EXTENSIONS.md](EXTENSIONS.md)
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

> Les versions jusqu'à la v1.1.2 incluse avaient été publiées sous Apache
> License 2.0 ; elles ne sont plus distribuées.
