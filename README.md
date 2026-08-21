# Beaver

Beaver is an agentic desktop workspace for local models through Ollama and cloud models through API keys or web accounts. It combines conversations, tools, planning, subagents, persistent memory, an embedded browser, Git workflows, forecasting, MCP connectors, automated wakeups, file previews, and a terminal in one application.

## Features

- **Local Agent and tools**: use local or cloud models with files, shell commands, web search, Office documents, Git, MCP, Forecast, diagnostics, todos, and interactive choices
- **Planning and permissions**: explore safely in Plan mode, save Markdown plans, approve implementation, and choose automatic, manual, or per-chat tool permissions
- **Conversations and projects**: manage tabbed chats, attachments, favorites, queued messages, session branches, archived chats, hidden summaries, and project folders
- **Parent-controlled subagents**: coordinate isolated child sessions, follow their live status, correct or reuse them, review their changes, and clean up their worktrees safely
- **Persistent memory**: keep optional global and per-project memory with manual or automatic modes, bounded summaries, topic files, live activity, and read-only access for subagents
- **Embedded browser**: browse in up to ten tabs per conversation, keep signed-in sessions, detect local development sites, and share the side panel with previews and Forecast. Available on macOS and Windows
- **Complete Git workflow**: create, switch, merge, and delete branches or worktrees; commit and push; browse uncommitted changes; and inspect recent or historical diffs
- **Forecast V2**: audit time-series data, select models manually or automatically, run local or cloud forecasts, compare backtests, build ensembles, explore advanced analysis, and export results
- **Providers and usage**: connect OpenAI/Codex, Grok, and Kimi through web authentication, use API-key providers, and view available limits, credits, token usage, request counts, and estimated costs
- **MCP connectors and channels**: activate local or cloud MCP connectors per chat and optionally connect the background Gateway to Telegram, Slack, or Discord
- **Wakeups**: schedule one-time, daily, or weekly prompts with the internal scheduler and keep each result in a dedicated conversation
- **Managed Ollama runtime**: download Ollama on first launch, reuse an existing daemon when available, browse and install models, edit modelfiles, and configure model parameters or system prompts
- **Desktop workspace**: use the cross-platform tabbed terminal, file tree, rich text and Office previews, link previews, context usage breakdown, six visual themes, and the interactive Beaver companion
- **Guided onboarding and migration**: configure Beaver on first launch and import instructions, skills, or rules from Claude Code, Codex, Agents, Hermes, Qwen Code, ZCode, OpenClaw, OpenCode, and Kimi Code
- **Secure local storage**: keep credentials in an XChaCha20-Poly1305 encrypted vault whose master key stays in the OS keyring; raw secrets never reach the frontend

## Supported providers

| Type | Provider | Connection |
|---|---|---|
| LLM | [Groq](https://console.groq.com/keys) | API key |
| LLM | [Google Gemini](https://aistudio.google.com/app/apikey) | API key |
| LLM | [Mistral](https://console.mistral.ai/api-keys) | API key |
| LLM | [Cerebras](https://cloud.cerebras.ai/) | API key |
| LLM | [OpenRouter](https://openrouter.ai/settings/keys) | API key |
| LLM | [OpenAI](https://platform.openai.com/api-keys) | API key or OpenAI/Codex web account |
| LLM | [DeepSeek](https://platform.deepseek.com/api_keys) | API key |
| LLM | [xAI](https://console.x.ai) | API key or Grok web account |
| LLM | [Moonshot Kimi](https://platform.kimi.ai/console/api-keys) | API key or experimental Kimi web account |
| LLM | [Z.ai GLM](https://z.ai/manage-apikey/apikey-list) | API key |
| Search | [Brave Search](https://api-dashboard.search.brave.com/app/keys) | API key |
| Search | [Exa](https://dashboard.exa.ai/api-keys) | API key |
| Search / scraping | [Firecrawl](https://www.firecrawl.dev/app/api-keys) | API key |
| Search | SearXNG | Local fallback without an API key |
| Forecast | [Nixtla TimeGPT](https://dashboard.nixtla.io/) | API key |

Models, quotas, and prices can change at the provider. Beaver displays current account information when the provider makes it available.

## Forecast models

Beaver includes a dedicated Forecast workspace for time-series analysis:

- **Local families**: Amazon Chronos / Chronos-Bolt, Google TimesFM, Datadog Toto 2.0, Salesforce MOIRAI 2.0, IBM FlowState, PriorLabs TabPFN-TS, NX-AI TiRex, Kairos, and THUML Sundial
- **Cloud family**: Nixtla TimeGPT-2 / TimeGPT-2.1
- **Selection and data quality**: choose a model manually or let Beaver select one from the data profile, hardware, horizon, frequency, uncertainty needs, and model capabilities
- **Evaluation and analysis**: run rolling backtests, compare baselines and models, inspect MASE, sMAPE, MAE, coverage, anomalies, drift, decomposition, variable importance, and weighted ensembles
- **Workspace and exports**: explore Data, Forecast, Evaluation, Comparison, Scenarios, Notes, and Report views, then export to CSV, Excel, JSON, PNG, SVG, PDF, or the clipboard

## Technical stack

- **Backend**: Rust + Tauri 2
- **Frontend**: React 19 + TypeScript + Vite
- **Local LLM runtime**: Ollama managed and downloaded by Beaver
- **Forecast runtime**: local forecast sidecar plus optional Nixtla API
- **Browser runtime**: sandboxed Chromium Embedded Framework on macOS and Windows
- **Search runtime**: Brave, Exa, and Firecrawl with a local SearXNG fallback
- **Connector runtime**: MCP bridge, OAuth storage, and Gateway channel service
- **Security**: XChaCha20-Poly1305 vault, master key in the OS keyring (macOS Keychain / Windows DPAPI / Linux Secret Service)
- **File watching**: `notify` crate (FSEvents on macOS, inotify on Linux, ReadDirectoryChangesW on Windows)

## Prerequisites

### External runtimes

- macOS (Apple Silicon), Linux, or Windows
- Node.js 24 LTS — the general Beaver environment
- CPython 3.14 — only the local SearXNG fallback

Node.js and CPython are external prerequisites: Beaver does not embed either runtime. CPython 3.14 is needed only when using the local SearXNG fallback, not for Beaver features that do not use that fallback.

The commands below were checked on August 20, 2026 against the official [Node.js download page](https://nodejs.org/en/download) (Node.js 24.19.0 LTS) and [Astral uv documentation](https://docs.astral.sh/uv/getting-started/installation/). They avoid a Linux package manager tied to one distribution.

### macOS (Apple Silicon)

Install Node.js and uv:

```bash
curl -fsSLO https://nodejs.org/dist/v24.19.0/node-v24.19.0.pkg
sudo installer -pkg node-v24.19.0.pkg -target /
rm node-v24.19.0.pkg

curl -LsSf https://astral.sh/uv/install.sh | sh
```

Close and reopen the terminal. Then install CPython for the local SearXNG fallback:

```bash
UV_PYTHON_BIN_DIR="$HOME/.local/bin" UV_PYTHON_INSTALL_BIN=1 uv python install 3.14
```

Close and reopen the terminal and Beaver, then verify:

```bash
node --version
python3.14 --version
```

### Linux (x64)

Install Node.js and uv:

```bash
(
set -e
curl -fsSLO https://nodejs.org/dist/v24.19.0/node-v24.19.0-linux-x64.tar.xz
mkdir -p "$HOME/.local/opt" "$HOME/.local/bin"
tar -xJf node-v24.19.0-linux-x64.tar.xz -C "$HOME/.local/opt"
rm node-v24.19.0-linux-x64.tar.xz
nodeRoot="$HOME/.local/opt/node-v24.19.0-linux-x64"
binDir="$HOME/.local/bin"
for executable in node npm npx corepack; do
  target="$nodeRoot/bin/$executable"
  destination="$binDir/$executable"
  if [ ! -x "$target" ]; then
    printf 'Node executable is unavailable: %s\n' "$target" >&2
    exit 1
  elif [ ! -e "$destination" ] && [ ! -L "$destination" ]; then
    ln -s "$target" "$destination" || exit 1
  elif [ -L "$destination" ] && [ "$(readlink "$destination")" = "$target" ]; then
    : # Existing managed link: leave it unchanged.
  else
    printf 'Refusing to replace existing %s; move it manually, then rerun this command.\n' "$destination" >&2
    exit 1
  fi
done

curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR="$binDir" sh
)
```

Close and reopen the terminal. `UV_INSTALL_DIR` makes the official uv installer configure this exact `~/.local/bin` directory, where the guarded Node.js links live. Then install CPython for the local SearXNG fallback into the same executable directory:

```bash
UV_PYTHON_BIN_DIR="$HOME/.local/bin" UV_PYTHON_INSTALL_BIN=1 uv python install 3.14
```

Close and reopen the terminal and Beaver, then verify:

```bash
node --version
python3.14 --version
```

### Windows (PowerShell)

Install Node.js and uv:

```powershell
$nodeInstaller = Join-Path $env:TEMP "node-v24.19.0-x64.msi"
Invoke-WebRequest -Uri "https://nodejs.org/dist/v24.19.0/node-v24.19.0-x64.msi" -OutFile $nodeInstaller
Start-Process msiexec.exe -Wait -ArgumentList @("/i", $nodeInstaller, "/passive")
Remove-Item $nodeInstaller

powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

Close and reopen PowerShell. Then install CPython for the local SearXNG fallback:

```powershell
$env:UV_PYTHON_BIN_DIR = Join-Path $HOME ".local\bin"
$env:UV_PYTHON_INSTALL_BIN = "1"
uv python install 3.14
```

Close and reopen PowerShell and Beaver, then verify:

```powershell
node --version
py -3.14 --version
```

### Development only

Rust (via [`rustup`](https://rustup.rs/)) is required only to build or develop Beaver; it is not required by an installed application.

## Installation

### macOS / Linux (one command)

```bash
curl -fsSL https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.sh | bash
```

Downloads the latest release, installs the app, and launches it automatically.
- **macOS**: installs into `/Applications/`
- **Linux**: installs the Debian package through `apt-get` (Ubuntu/Debian only)

The Linux installer uses the `.deb` release asset so the app is visible in the system application menu.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.ps1 | iex
```

Downloads the latest release and launches the Windows NSIS `-setup.exe` installer automatically.

> **Windows Defender**: on first launch, "Controlled folder access" may block `ollama.exe`. Click "Allow" in the notification — it will not ask again.

### Updates

Updates are automatic: a notification appears in the app when a new version is available. One click and the app updates itself.

### From CL-GO to Beaver

Beaver is the new name of CL-GO. Existing users migrate through the CL-GO 1.0.2 bridge release and keep their conversations, settings, credentials, MCP connectors, memory, Forecast models, Ollama data, and browser sessions. Historical internal identifiers and the data directory described below are intentionally preserved for compatibility.

---

## Development

Install the external prerequisites above first, then install the development dependencies:

```bash
# 1. Clone the repo
git clone https://github.com/Kevin-hDev/Beaver.git
cd Beaver

# 2. Install dependencies
npm install

# 3. Download the Ollama binary for your OS
cd src-tauri && bash scripts/download-ollama.sh
```

## Commands

```bash
npm run tauri dev       # Dev mode (hot reload)
npm run tauri build     # Release build (.dmg / -setup.exe / .deb)
npm run lint            # Frontend lint and React boundary checks
npm test                # Frontend and embedded browser tests
npx tsc --noEmit        # TypeScript check
cd src-tauri && cargo check    # Rust check
cd src-tauri && cargo clippy --all-targets  # Strict lint
cd src-tauri && cargo test     # Unit tests
```

## Architecture

```
src-tauri/                # Rust + Tauri backend
├── src/
│   ├── commands/         # Tauri commands grouped by application domain
│   ├── services/
│   │   ├── agent_local/  # Sessions, tools, permissions, plans, memory, subagents
│   │   ├── agent_import/ # Guided import from other agent applications
│   │   ├── browser/      # Sandboxed Chromium sessions and native views
│   │   ├── llm/          # Unified cloud client, catalog, reasoning, streaming
│   │   ├── codex_client/ and *_oauth/  # OpenAI, Grok, Kimi, and MCP web auth
│   │   ├── provider_usage/  # Limits, usage history, and cost estimates
│   │   ├── search/ and searxng/  # Cloud search and local fallback
│   │   ├── forecast/     # Data audits, models, runs, evaluations, exports
│   │   ├── mcp_bridge/ and mcp_oauth/  # Local and cloud MCP connectors
│   │   ├── gateway/      # Telegram, Slack, and Discord background channels
│   │   ├── git/          # Branches, worktrees, commits, pushes, merges, diffs
│   │   ├── scheduler/ and terminal/  # Wakeups and cross-platform PTY
│   │   ├── paths.rs      # Centralized cross-platform data path
│   │   ├── vault.rs      # XChaCha20-Poly1305 encrypted vault
│   │   └── private_store/  # Private files and platform permissions
│   ├── tray.rs           # System tray integration
│   ├── storage_migration.rs  # Storage initialization and compatibility
│   └── ollama_polling.rs # Ollama status polling
└── resources/            # Icons and static resources

src/                      # React frontend
├── components/
│   ├── agent-local/ and agent-side-panel/  # Chat and shared side panel
│   ├── agent-import/     # External assistant migration wizard
│   ├── internal-browser/ # Embedded browser interface
│   ├── forecast/         # Workbench, charts, evaluations, notes, models
│   ├── providers/        # API/web connections and usage details
│   ├── connectors/ and channels/  # MCP and Gateway configuration
│   ├── ollama/           # Local model browser and customization
│   ├── heartbeat/        # Wakeup planning and history
│   ├── file-tree/ and file-preview/  # Navigation and rich previews
│   ├── mascot/           # Interactive Beaver companion
│   ├── onboarding/ and settings/  # Setup and application preferences
│   └── terminal/ and ui/ # Integrated PTY and shared interface components
├── hooks/                # Logic extracted by domain
├── lib/                  # Shared helpers and platform detection
├── types/                # TS types aligned with Rust
└── i18n/                 # 7 languages (FR, EN, DE, ES, IT, JA, ZH)
```

## Local storage

Data in `~/.local/share/cl-go-dash/` on all 3 OSes. The directory keeps its
historical identifier for compatibility with existing installations:

| Path | Contents |
|---|---|
| `secrets.enc` | Encrypted API and OAuth credentials |
| `configured-providers.json`, `provider-usage.json` | Connected providers and local usage history |
| `config.json`, `heartbeat-runtime.json` | Application settings and wakeup runtime state |
| `agent-sessions/*.json` | Local Agent conversations |
| `agent-settings.json`, `session-tabs.json` | Permissions and open conversation tabs |
| `projects.json`, `favorite-models.json`, `terminal-tabs.json` | Projects, model favorites, and terminal tabs |
| `AGENTS.md`, `external-agent-sources.json`, `agent-import-backups/` | Imported instructions, external sources, and safe backups |
| `plans/`, `skills/`, `tool-results/` | Agent plans, local skills, and large tool outputs |
| `subagent-changes/`, `subagent-worktrees/` | Isolated subagent changes and worktrees |
| `memory/core/` | Personality and context Markdown files |
| `memory/global/`, `memory/projects/`, `memory-settings.json` | Persistent global and per-project memory |
| `browser/` | Encrypted browser sessions and private Chromium profile |
| `mcp-connectors.json`, `mcp-runtime/` | MCP connector configuration and runtime data |
| `gateway-session-map.json`, `logs/gateway-audit.jsonl` | Gateway session links and audit history |
| `forecast-*` | Forecast analyses, data profiles, models, settings, drafts, notes, and exports |
| `ollama-*` | Managed Ollama runtime, model metadata, and system prompt overrides |
| `searxng-sidecar/` | Local SearXNG search runtime |
| `logs/` | Bounded wakeup, gateway, Ollama, SearXNG, and tool logs |

## Ollama — managed runtime

Beaver manages **Ollama** locally so a separate manual installation is not required:

- On first launch, a setup screen downloads Ollama automatically into `~/.local/share/cl-go-dash/ollama-bundle/`
- On startup, the app checks whether an Ollama daemon is already running on `localhost:11434`
- If yes (Ollama.app already installed), it uses it as is
- If not, it launches its own downloaded binary
- On close, the sidecar is stopped cleanly (Unix SIGTERM / Windows kill + 3s grace period)
- On Linux, automatic GPU detection (AMD → ROCm archive, Nvidia → standard archive with CUDA)
- Model parameters, system prompts, and complete modelfiles can be customized from Beaver

**Models are shared** with Ollama.app if it is installed (`~/.ollama/models/`).

## Security

- **Encrypted vault**: API keys encrypted with XChaCha20-Poly1305, master key in the native OS keyring (Keychain / DPAPI / Secret Service)
- **JS never sees a key**: no Tauri command exposes `get_api_key`; secrets stay in the Rust backend and are zeroized after use
- **Path traversal protection**: paths requested through the frontend are validated, canonicalized, and kept inside their allowed roots
- **Bounded collections**: ActiveStreams (32), PTY sessions (16), messages per session (2000), capped MCP JSON depth/size
- **Secure HTTP for credentials**: redirects blocked, HTTPS enforced, error messages sanitized
- **MCP hardening**: program allowlist, no shell, argument validation, environment isolation
- **Protected browser**: sandboxed helpers, restricted navigation, blocked sensitive permissions, private profile, and encrypted restored tabs
- **Verified updates**: strict release metadata, bounded downloads, SHA-256 manifests, health checks, and fail-closed installation
- **Filtered logs**: provider HTTP bodies truncated to 200 chars, known credential formats redacted

For the full threat model, vulnerability reporting policy, and safe usage guidance, see **[SECURITY.md](SECURITY.md)**.

For the complete release history, see **[CHANGELOG.md](CHANGELOG.md)**.

## License

Beaver is licensed under the **[GNU Affero General Public License v3.0](LICENSE)**.

Copyright © 2026 Kevin Huynh

You are free to use, study, modify and redistribute Beaver. In return, any
distributed or network-hosted version — modified or not — must be released
under the AGPL v3 and ship its complete source code.

Contributions are welcome and require signing the CLA described in
**[CONTRIBUTING.md](CONTRIBUTING.md)**.

For a commercial license exempting you from the AGPL obligations, contact
huynh.kevin7@outlook.fr.

Third-party components keep their own licenses — see
**[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)**.

> Releases up to and including v1.1.2 were published under the Apache License
> 2.0 and remain available under those terms.
