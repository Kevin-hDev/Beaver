# Changelog

## [Unreleased]

---

## v1.1.1

### Agent skills

- **Expanded built-in skill bundle** — added 40 new programming skills covering planning, implementation, debugging, testing, review, documentation, collaboration, releases, reusable agents, and automations.

### Agent behavior and reliability

- **Clearer system guidance** — restructured both agent tiers' system prompts, removed duplicated or conflicting instructions, consolidated uncertainty rules, and moved tool-specific requirements into the relevant tool definitions.
- **Reliable Plan and interactive decisions** — handled plan approval, adjustments, dismissal, and interactive answers as explicit backend decisions, removing unnecessary model round trips and preventing accidental keyboard approvals.
- **Stable workspace anchoring** — kept each session tied to its selected project or private session workspace, prevented shell directory changes from silently moving the agent's root, and preserved generated files when conversations are deleted.

### Agent chat performance

- **Accurate provider throughput** — fixed token-per-second reporting by using native generation timings from Ollama, Groq, and Cerebras when available, while keeping a measured streaming fallback for other providers.
- **Generation-only measurement** — excluded tool execution, compression, retries, and provider waiting time from the displayed generation speed, and combined multi-turn results using their actual generation durations.
- **Clear estimated rates** — marked stream-derived throughput with an approximation symbol and avoided overstating rates when the first token batch cannot be measured reliably.

### Context usage

- **Live context ring** — displays the context ring and its hover breakdown from the model's first generated thought, message, or tool call, then updates them continuously throughout streaming without appearing early after the initial user message.
- **Stable token breakdown** — calculates context categories from the request actually sent to the provider and preserves them across tool calls and stream completion, preventing messages, tools, or meta context from being redistributed when final usage arrives.
- **More accurate context accounting** — counts streamed output as estimated tokens instead of network fragments, applies provider-aware reasoning rules, and separates skill catalog descriptions from skill content loaded later in the conversation.

---

## v1.1.0

### Beaver identity

- **Complete visible rename** — renamed CL-GO to Beaver across the application, installers, documentation, support links, network identity, mascot, icons, and platform metadata.
- **Stable internal identity** — kept the historical executable name, application identifier, profile directory, vault service, browser profile, and internal protocols so the new name does not split or reset existing installations.

### Safe migration

- **Direct update from CL-GO** — added a verified migration path from the public CL-GO v1.0.2 bridge release to Beaver v1.1.0 while preserving conversations, settings, API and OAuth connections, MCP connectors, memory, Forecast models, Ollama data, and browser state.
- **Cross-platform package continuity** — added compatible replacement and cleanup rules for macOS bundles, Debian packages, Windows installers, shortcuts, and startup entries without touching user data.
- **Protected installation** — required bounded manifests, exact asset names, SHA-256 verification, private temporary files, health acknowledgements, and fail-closed installation behavior; macOS also restores the previous bundle when validation fails.

### Reliability

- **Stable embedded browser** — fixed CEF startup and navigation failures, eliminated the endless stop/refresh state, and preserved the established browser profile through the rename.
- **Clean macOS shutdown** — fixed the development application crash that appeared after a normal close.
- **Faster repeat development starts** — avoided unnecessary updater-helper rebuilds when its inputs have not changed and made frontend startup failures visible immediately.

### Release validation

- **Three-platform release gate** — added strict macOS, Linux, and Windows build inspection, package migration checks, independent manifest verification, and draft-only release creation before publication.

---

## v1.0.2

### Agent configuration migration

- **Guided assistant migration** — added migration from Claude Code, Codex, Agents, Hermes Agent, Qwen Code, ZCode, OpenClaw, OpenCode, and Kimi Code during onboarding or later from Settings. Global instruction files can be imported while external skills and rules remain individually selectable and readable from their original folders.

### Interactive companion and conversations

- **Interactive desktop companion** — added an animated companion that reflects agent activity, supports direct interactions, stays calm during long-running work, and preserves its settings across restarts.
- **More reliable conversations** — added Markdown rendering for user messages, clearer interactive choices, improved message editing, responsive layouts, readable skill names, keyboard navigation, and better stream recovery after provider outages.

### Persistent memory

- **Global and project MEMORY.md support** — added optional persistent Markdown memory with manual and automatic modes, bounded context summaries, on-demand topic reading, safe natural-language writes, and read-only memory access for subagents.
- **Integrated memory management** — added a dedicated Settings view, live MEMORY activity and file previews during streaming, accurate context usage accounting, secure atomic storage, and human-readable project folders with automatic migration from legacy identifiers.

### Forecast workspace

- **Forecast V2 workbench** — rebuilt Forecast around a dedicated workspace that remains linked to the active chat while keeping the side panel focused on the essential visual result.
- **Live session synchronization** — prediction changes made by the LLM or selected from the side panel now update the Forecast workspace in real time without reopening the window.
- **Complete result exploration** — added dedicated Data, Forecast, Evaluation, Comparison, Scenarios, Notes, and Report views with responsive navigation, scrollable reports, smoother charts, and layouts that remain usable across narrow, resized, and full-screen windows.
- **Focused side panel** — kept the main forecast visual and exports in the conversation panel while moving detailed reports, scenarios, notes, evaluation, and model comparison into the larger workspace.

### Prediction quality and model selection

- **Manual or automatic model selection** — users can keep full control of the forecast model or enable Auto so the agent selects a compatible model from the available hardware, data profile, requested horizon, frequency, uncertainty needs, and model capabilities.
- **Data quality contracts** — added reusable dataset profiles, mapping and quality audits, missing-period and anomaly checks, multi-series support, future covariates, bounded inputs, and consistent confidence-level handling before predictions run.
- **Stronger prediction contracts** — validated point forecasts, quantiles, horizons, series alignment, confidence intervals, and bounded model responses so incomplete or incoherent outputs fail clearly instead of producing misleading results.
- **Agent-ready Forecast tools** — expanded and clarified the LLM tool contracts for data audits, model discovery, prediction, reading, analysis, rolling backtests, and model comparison so agents can plan valid Forecast workflows from the first call.

### Evaluation and advanced analytics

- **Rolling backtests and baselines** — added chronological rolling validation with Drift, ETS, Naive, and Seasonal Naive baselines, plus MASE, sMAPE, MAE, interval coverage, duration, rankings, and comparable stored results.
- **Model comparison** — added a dedicated comparison view for forecast quality, speed, uncertainty coverage, resource constraints, and failed or unavailable models.
- **Advanced analysis** — added time-series decomposition, residual anomaly detection, variable importance, drift analysis, and confidence-aware reporting.
- **Forecast ensembles** — added validated weighted ensembles built from comparable backtested models, with explicit member weights, uncertainty ranges, and validation status.

### Models and local runtimes

- **Broader local model support** — completed the local adapters and runtime integration for the Forecast model catalog, including Chronos, TimesFM, Moirai, and other supported forecasting families.
- **Prepared model installs** — model downloads now prepare their required runtime and dependencies during installation, avoiding long first-prediction setup delays, and uninstalling a model cleans up its dedicated resources.
- **Verified model sources** — switched model runtimes and artifacts to official or trusted pinned sources with bounded downloads, revision and integrity checks, safer remote-code handling, and developer-only update discovery before changes can reach users.

### Reliability, exports, and security

- **Revisioned Forecast storage** — added indexed, versioned, bounded, and atomically written analyses with reliable rename, deletion, event propagation, session restoration, and display of LLM-created predictions.
- **Crash-safe notes** — protected Forecast notes with private permissions, canonical path checks, symlink rejection, bounded parsing, atomic writes, revision synchronization, and recovery after interrupted creation, updates, or deletion.
- **Reliable model lifecycle** — improved sidecar startup, shutdown, cancellation, error propagation, runtime preparation, and cleanup while keeping expected idle stops separate from actual prediction failures.
- **Safer exports** — prevented incomplete exports, neutralized spreadsheet formulas in CSV and clipboard output, verified XLSX values remain text, and kept Forecast reports available across CSV, Excel, JSON, PNG, SVG, PDF, and clipboard formats.
- **Responsive regression coverage** — added backend and interface tests for model selection, prediction contracts, data audits, backtests, storage revisions, notes, exports, responsive navigation, and cross-window synchronization.

### Update safety

- **Verified update downloads** — added strict release and asset validation, bounded downloads, SHA-256 manifest verification, and rejection of incomplete, unexpected, or older packages.
- **Health-checked installation** — moved installation into a dedicated platform-specific helper that verifies the replacement starts successfully on every platform; macOS also preserves and restores the previous application bundle if verification fails.

---

## v1.0.1

### Features

- **Native Grok and Kimi authentication** — added direct web authentication for xAI Grok and Moonshot Kimi subscriptions without installing Grok Build, Kimi Code, or another provider CLI. Kimi remains an experimental, unofficial integration.
- **Full CL-GO-DASH agent experience** — Grok and Kimi OAuth models now run through CL-GO-DASH's native agent loop in manual chats, including tools, skills, MCP connectors, permissions, plans, context compression, and subagents.
- **Integrated Git workflow** — added branch and worktree creation, selection, and deletion; local commits; authenticated pushes; explicit branch-to-branch merges; shared per-worktree state across sessions; uncommitted file browsing; recent commit history; and historical diff previews.
- **Four new visual themes** — added Emerald Night, Frosted Cobalt, Astral Mist, and Crimson Eclipse, each with a dedicated accessible palette and theme-aware added and deleted line colors across code and file diff previews.

### Interface

- **Unified visual system** — standardized buttons, icon buttons, dialogs, form controls, spacing, corner radii, layer ordering, glass menus, and popovers across the application.
- **Smoother and more accessible motion** — moved animations to GPU-friendly properties, added reduced-motion support, refined the chat composer and session icons, and introduced the animated CL-GO companion.

### Usage and costs

- **Provider usage details** — Settings now shows available limits, reset times, remaining credits, token usage, request counts, and exact or estimated costs for configured API and OAuth connections.
- **Local usage history** — added Today, 7 days, 30 days, and Total views with input, output, cache, and reasoning token breakdowns, without storing prompts or conversations.
- **Codex allowance labels** — separated the general Codex weekly allowance from the dedicated GPT-5.3-Codex-Spark weekly allowance, and added the model to the Codex selector.

### Local models

- **Safer Ollama customization** — added a model parameter catalog and per-model system prompt overrides while hardening native model creation and editing flows.

### Security and reliability

- **OAuth and telemetry hardening** — protected concurrent login, refresh, logout, account switching, usage refreshes, timestamps, and provider-specific price resolution while keeping API and OAuth credentials strictly separated.
- **Credential and session protection** — added pre-provider secret redaction, private atomic session storage, and one-time cleanup of legacy OAuth artifacts and historical session data without changing active vault credentials.
- **Clear subscription errors** — Grok and Kimi membership or credit errors now use stable, non-sensitive error codes instead of exposing provider response bodies.
- **Reliable Git state and operations** — added validated and bounded Git inputs, safer delete and merge confirmations, conflict-aware merges, credential-safe push errors, synchronized worktree refreshes, and clear loading or partial-change states.
- **Clear Git operation errors** — branch switching, commits, merges, branch and worktree deletion, and clone branch cleanup now show stable localized reasons instead of a generic failure.
- **Typed UI error contract** — Git and clone commands now return structured error codes decoded through one frontend mapping that never exposes internal paths or backend details.
- **Reliable context compression** — restored compression requests, progress feedback, and tool history while keeping clone summaries hidden from the conversation.

---

## v1.0.0

### Bug Fixes

- **Windows browser availability** — stopped inspecting Chromium's live cookie database after startup, which is exclusively locked on modern Windows builds and previously left the embedded browser unavailable.

### Reliability

- **Platform-aware browser validation** — Windows now validates cookie storage through supported CEF set, flush, and delete operations, while macOS keeps its existing at-rest protection check.
- **Regression coverage** — added release tests that prevent Windows production builds from reintroducing direct access to Chromium's locked cookie database.

### Security

- **Browser protections preserved** — kept the CEF sandbox, Chromium database locking, fail-closed callbacks, and existing macOS disk verification enabled.

---

## v0.9.9

### Bug Fixes

- **Windows production startup** — rebuilt the CEF-hosted Tauri DLL with the production custom protocol so packaged installs load the bundled interface instead of trying to reach the development server on localhost.
- **macOS browser availability** — allowed locally signed hardened-runtime builds to load the bundled CEF framework and helpers, restoring the secure Browser in release builds.

### Reliability

- **Release regression coverage** — added checks that require the Windows production protocol and the macOS library-validation entitlement before a release bundle can pass its test suite.

### Security

- **Targeted macOS compatibility** — kept the Hardened Runtime, CEF sandbox, verified runtime paths, and existing CEF integrity checks enabled while applying the required library-loading exception to locally signed executables.

---

## v0.9.8

### Bug Fixes

- **Windows release builds** — normalized the verified CEF archive layout before Rust compilation so Windows tests and NSIS installer builds can find Chromium runtime files and locales.
- **Windows startup permissions** — accepted valid private ACL entries whose trustee type is reported as unspecified by Windows while still verifying the exact user SID, access rights, inheritance, and entry count.
- **macOS release builds** — explicitly enabled the project's ad hoc signing policy for CEF release bundles while keeping production entitlements and fail-closed validation.

### Reliability

- **CEF cache migration** — invalidated older incompatible CEF layouts automatically instead of reusing a cache that would fail during compilation.
- **Release regression coverage** — added cross-platform checks for Windows CEF staging and macOS signing workflow configuration.
- **Windows storage coverage** — added an isolated native Windows test for private writes, ACL repair, and repeated permission checks without loading the browser runtime.

---

## v0.9.7

### Features

- **Embedded Chromium browser** — added a CEF-powered browser directly inside the shared side panel on macOS and Windows, without opening a separate native window.
- **Multi-tab browser sessions** — added up to ten tabs per conversation, internal popup interception, lazy page restoration, shared cookies and signed-in sessions, and bounded native-view eviction.
- **Local development discovery** — the browser home page now detects active local HTML sites without scanning every port and opens them with one click.
- **Smart address bar** — web addresses work without an explicit protocol, localhost uses HTTP automatically, and other text is sent to Google Search.

### Interface

- **Shared panel integration** — File Preview, Forecast, and Browser now share the same mode selector, resizing, conversation preferences, and internal full-screen behavior.
- **Responsive browser controls** — added compact tab and navigation bars, translated empty states and errors, light and dark themes, and layouts that adapt from narrow panels to full screen.
- **Stable native layout** — Chromium now follows panel resizing, window resizing, mode changes, sidebars, menus, dialogs, reloads, and full-screen transitions without stale views or visual overflow.

### Bug Fixes

- **Windows startup** — fixed an issue that could close the app immediately after launch while private storage permissions were being checked.

### Security and packaging

- **Hardened browsing boundary** — limited navigation to validated HTTP and HTTPS URLs, blocked invalid certificates and external protocols, refused sensitive permissions and downloads, and exposed no Tauri bridge to visited pages.
- **Protected browser data** — encrypted restored tab state with the existing OS-backed vault, kept the Chromium profile private, bounded browser collections, and verified cookie protection before enabling the feature.
- **Verified CEF runtime** — pinned CEF `150.0.0+150.0.10`, added SHA-256 integrity checks, sandboxed helper processes, and bundled the required Chromium frameworks, helpers, resources, and licenses for macOS and Windows.
- **Platform availability** — Linux continues to compile and run without CEF, and the Browser entry remains hidden until an embedded Linux engine is supported.

---

## v0.9.6

### Features

- **Archived chats** — deleting a chat now archives it first; Settings includes a new Archived chats tab to restore sessions or permanently delete archived chats grouped by project and standalone discussions.
- **Session branching** — clone a chat from any message, preserve cumulative hidden context, optionally isolate the clone in a Git branch or worktree, and safely clean linked branches during archive.
- **Parent-controlled subagents** — subagents now run as visible child sessions coordinated by the parent, with live status, corrections, archiving, explicit change review, and safer cancellation and cleanup.
- **Queued user messages** — messages sent during an active response remain attached to the live stream and are processed without losing conversation history or visual grouping.

### Models and reasoning

- **Updated cloud model catalogs** — added OpenAI GPT-5.6 Sol, Terra, and Luna for API and Codex OAuth, plus xAI Grok 4.5, Grok 4.3, Grok 4.20 reasoning variants, and Grok Build 0.1, with matching OpenRouter reasoning modes and migration for retired xAI model identifiers.
- **Separate reasoning selector** — model and reasoning choices now use independent controls, unsupported models hide the reasoning selector, and compatible models start at Medium while preserving the user's session choice.

### Security and reliability

- **Authenticated transports** — hardened redirects, credentials, error handling, and request validation across gateway channels, OAuth, MCP, vault storage, attachments, previews, and authenticated LLM streams.
- **Subagent lifecycle** — made completion, cancellation, worktree ownership, permission routing, cleanup, and concurrent Git operations safer and deterministic.
- **Git branch creation** — added validated branch names, duplicate-submit protection, typed errors, success feedback, and safer handling for empty repositories.

### Interface and documentation

- **Live agent activity** — improved concurrent stream indicators, subagent history, tool rows, cancellation states, file-preview links, and thinking alignment.
- **Settings and Forecast** — refreshed Settings organization, Forecast model configuration, wakeup details, model navigation, tooltips, table sizing, and splash styling.
- **Seven-language documentation** — translated Forecast documentation and rewrote Tools settings descriptions in French, English, Spanish, German, Italian, Chinese, and Japanese.
- **Maintenance** — removed unused frontend code and dependencies while keeping provider, session, and interface behavior covered by tests.

---

## v0.9.5

### Changes

- **Session summary bubble** — added a compact session summary bubble with todos, generated plans, subagents, git state, and recent file changes; generated plans can be opened in the side preview panel with a dedicated plan layout
- **Per-request file changes bubble** — added a compact chat bubble under assistant replies listing only the files created or modified during that request, with direct review access to each file diff
- **Chat and preview polish** — refreshed the chat input, Markdown bubble rendering, live tool display during streaming, collapsed work-phase summaries, and the visual layout of the side preview panel
- **Tools settings** — added a dedicated Settings > Tools tab to review always-on tools and enable or disable optional tool groups before they are shown to the LLM

---

## v0.9.4

### App release notes

- Context usage now shows a clear breakdown inside the chat input ring.
- Chat controls, sidebars, icons, and dropdowns have been visually refined.
- Font size is now configured directly in pixels with safer UI limits.
- Settings now apply correctly as soon as the app starts.
- Update notifications can now show short release notes.
- Office tools now support richer Excel formatting, Word rich text, and better document safety.

### Changes

- **Single chat session** — removed the multi-tab chat; replaced with a single-session header showing the session name in bold, with smooth animation when collapsing the sidebars
- **Context usage details** — the chat context ring now opens a compact breakdown by messages, tools, MCP/connectors, skills, meta context, and system prompt
- **Chat UI polish** — refreshed mode/model selectors, thinking indicators, header separator, input border, and sidebar hover alignment
- **Font size control** — replaced percentage scaling with a direct pixel control from 10px to 24px, including legacy setting migration
- **Settings startup fix** — font size and code theme settings now apply immediately when the app starts
- **Glass dropdowns** — dropdowns and update bubbles now use the same readable glass/opaque background pattern across the app

### Office tools improvements

- **Excel formatting** — added `set_format` (bold/italic/underline, font/background color, font size), `set_number_format` (number/date/currency), `set_border` (thin/medium/thick per side), `merge_cells`, and `set_row_height` operations; available on both create and edit backends
- **Word rich text** — paragraphs now support multiple runs with per-segment bold/italic/underline/color, plus paragraph and heading alignment
- **Word styles** — added a proper `styles.xml` (Heading1–6 and Normal) and real OOXML list numbering via `numbering.xml` (replaces the fake `"1."`/`"•"` text prefixes)
- **Office bug fixes** — `read_document` now preserves spaces between runs (no more collated words); empty optional fields (e.g. `bg_color: ""`) are ignored instead of erroring
- **Security** — `rm -rf /tmp/...` is no longer a false positive of the destructive command filter (regex now requires a terminator after the dangerous target)

---

## v0.9.3

### Features

- **Agent todo lists** — live task progress panel with hidden todo history, pause/resume, and delete support
- **Agent diagnostics** — structured safe diagnostics for stream errors, recent tools, and recovery context
- **Interactive choices** — `ask_user_choice` tool with keyboard/mouse selection and recommended options
- **Plan mode** — read-only planning workflow with local Markdown plans, approval gating, and implementation handoff

---

## v0.9.2

### Improvements

- **First-launch onboarding** — added welcome, theme/language, and LLM provider setup steps
- **API key setup** — added visibility toggles and clearer configured-provider states
- **Linux installation** — switched Linux installs and app updates to Debian packages
- **Vision support** — improved image handling and capability detection across local and cloud providers
- **Thinking support** — normalized provider reasoning output so it stays in the dedicated Thinking section
- **Streaming display** — batched live token and thinking updates per animation frame for smoother chat rendering

### Maintenance

- Removed obsolete Tauri command wrappers

---

## v0.9.1

### Fixes

- **Ollama setup flow** — added persistent Skip support and made the same setup available later from Settings > Ollama
- **Ollama install UX** — improved progress states, cancellation, cleanup of partial installs, and setup screen alignment
- **Ollama setup hardening** — strict checksum verification, safer archive extraction, and no Ollama polling when it is not installed

---

## v0.9.0

### Features

- **Provider reasoning modes** — per-session reasoning effort controls for Codex, OpenAI, OpenRouter, Ollama GPT-OSS, Groq, DeepSeek, Mistral, Moonshot, xAI, and Z.ai
- **Dynamic OpenRouter reasoning** — reasoning support and effort levels are detected from OpenRouter model metadata when available
- **Persistent reasoning settings** — each chat session keeps its own reasoning mode across app restarts
- **Tool display refactor** — compact, collapsible tool activity summaries with clearer detail labels

---

## v0.8.9

### Features

- **Communication channels** — Discord, Telegram, and Slack channel support
- **Forecast** — local and cloud LLM forecasting workflows
- **Keyboard navigation** — arrow-key navigation across the app

---

## v0.8.8

### Security

- **Full security audit** — 21 vulnerabilities fixed: URL whitelist for app updates, AllowSession disabled for bash/MCP, TOCTOU write protection with symlink rejection, SSRF DNS pinning, PTY token ownership, vault bounded to 500 entries, anti-ReDoS grep, CSPRNG for OAuth, WriteGuard re-enabled
- **Zeroize audit** — 16 fixes: all secrets (`Zeroizing<String>`), vault error paths guaranteed, OAuth PKCE/state/body zeroed after use, `Bearer` header via `.bearer_auth()`, env credentials migrated to vault/keyring
- **Sharp edges audit** — 7 fixes: Jina SSRF fallback removed, bash gate hardened (newline/redirect/background), circuit breaker without `DefaultHasher`, vault namespace isolation, config corruption sentinel
- **Semgrep static analysis** — full scan (Rust + TypeScript + JavaScript + Docker) with Trail of Bits, Decurity, and elttam rulesets: 0 true positives, 4 false positives (safe `dangerouslySetInnerHTML` on SVG/highlight)

---

## v0.8.7

### Features

- **File tree panel** — browsable project directory tree alongside the file preview panel
- **Git branch selector** — dropdown in chat toolbar to view, search, and switch branches with real-time updates via file watcher
- **Branch conflict dialog** — when switching with uncommitted files, shows dirty file list with real diff stats (+/-) and a "commit & switch" option that auto-commits a WIP save
- **Inline branch creation** — create and checkout a new branch directly from the selector dropdown
- **Worktree navigation** — click a worktree in the branch list to switch the active project to that directory
- **Git context for the agent** — branch name and dirty count injected into the LLM system prompt, plus `create_branch` and `checkout_branch` tools (gated, require user approval)
- **Branch bubble** — centered chat bubble when the agent creates or switches branches
- **Bundled skills** — 6 default skills (skill-create, cli-create, playwright-cli, video-analyzer, voxtral-cli, hk-telegram) ship with the app and auto-install on first launch or update

---

## v0.8.6

### Features

- **Subagent system** — the main agent can spawn autonomous explorer (read-only) and coder (isolated git worktree) subagents that run in the background. Results are auto-synthesized when all subagents complete.
- **Subagent accordion** — live panel above chat input showing active subagents with per-agent timers and stop buttons
- **Subagent bubble** — collapsible completion bubble in chat history with links to open subagent sessions in new tabs

### Improvements

- Structured English system prompts for subagents with XML tags and web research guidelines
- `delegate_task` tool with prompt structuring guidance and anti-duplication instructions
- Bounded spawn channel, prompt size limits, session ID validation, path traversal protection
- Worktree auto-cleanup after subagent execution via `git worktree remove`
- Guard cleanup pattern: registry + session + worktree guaranteed even on error
- `run_id` tracking across spawn/completion events for reliable multi-run isolation
- Web search/fetch tool bubbles collapsed by default
- i18n for all subagent UI in 7 languages

---

## v0.8.5

### Features

- **Working indicator** — persistent Lottie loader + "Working for Xs" with live token count shown during all streaming gaps (between segments, after tool results, waiting for first token). Timer never resets between gaps.
- **Thinking shimmer** — "Thinking" label shimmers while the model is actively thinking, stops when done
- **Tool spinner fix** — `@keyframes spin` was missing from CSS, tool bubble spinners now rotate correctly

### Improvements

- Lottie loader recolored to theme orange via CSS filter (dark/light aware)
- Streaming timer unified — both working indicator and thinking stats share the same continuous timer from stream start

---

## v0.8.4

### Features

- **Système de connecteurs MCP** — 18 connecteurs pour services externes (Notion, Slack, Linear, Reddit, HuggingFace, etc.) accessibles au LLM via un meta-tool unique `search_mcp_tools` (~80 tokens en contexte)
- **OAuth 2.1 complet** — PKCE S256, Dynamic Client Registration, discovery automatique, callback server local, refresh automatique avec mutex anti-race
- **Transport stdio pour MCP locaux** — Context7, HuggingFace, iMessage, ProductHunt, Reddit via process spawn (npx/uvx/deno) + stdin/stdout NDJSON
- **Trait McpTransport unifié** — interface commune HTTP/stdio, extensible pour futurs transports
- **ProcessManager** — pool borné max 8 process, TTL 10 min, lazy spawn, crash recovery, stderr drain
- **UI Settings → Connectors** — browse catalog 18 MCP, config tokens, OAuth auto, toggles chat
- **Menu chat connecteurs** — dropdown "+" avec sous-menu toggles par connecteur
- **Link previews in chat** — URLs in messages display rich preview cards (title, description, OG image, favicon, site name). Powered by a Rust backend that fetches and parses Open Graph metadata. YouTube videos get dedicated previews via the public oEmbed API (thumbnail + channel name). Previews are deduplicated, capped at 5 per message, and grouped at the bottom of the message bubble. Toggleable in Settings > General (7 languages supported).

- **Keyboard arrow navigation** — navigate between sidebar tabs (ArrowUp/Down) and list panel items (sub-tabs, sessions, wakeups, personality files) using arrow keys. ArrowLeft/Right switches focus between sidebar and list panel. Does not interfere with existing shortcuts (Cmd+arrows for history) or text inputs.

### Fixes

- **Codex OAuth persistence** — fixed premature `Done` event that caused tool data loss and frozen spinners on GPT sessions
- **Stream error recovery** — errors during multi-turn tool calls now persist completed segments instead of discarding them
- **Session reload race** — stale stream snapshot no longer overrides complete DB data on session load
- **Tool arguments round-trip** — `args` field now preserved through Rust serialization (was silently dropped)
- **Tool completion indicator** — saved tools show ✓/✗ correctly instead of frozen spinner after reload
- **Persist failure logging** — save errors are now logged and reported instead of silently swallowed
- **Multi-turn context** — chat history reconstruction preserves per-turn structure instead of flattening all tools
- **Retry back-off** — 5 retries with exponential back-off (2s→32s, ~62s total), SSE transport errors now retryable
- **Parallel tool order** — indexed slots preserve result order, fixes `tool_call_id` mapping for OpenAI-compat
- **`web_fetch` permission gate** — no longer classified as read-only, eager dispatch checks pre-hooks
- **`glob`** — returns absolute paths (consistent with `grep`)
- **`read_spreadsheet`** — formulas returned as text instead of `0.0`
- **`write_spreadsheet`** — operations target correct sheet, default `Sheet1` documented
- **`write_document`** — schema clarified per block type, empty tables skipped
- **`process_image`** — `operations` now optional for simple format conversion

### Security

- **Permission gate MCP** — `search_mcp_tools` mode "call" nécessite approbation utilisateur
- **Sérialisation request/response stdio** — `request_lock` empêche le mélange de réponses entre appels concurrents
- **Endpoint HTTP validé** — liste de domaines de confiance, pas d'URL arbitraire
- **Spawn sécurisé** — whitelist programmes (npx/uvx/deno), regex args, env_clear + env minimal, blocklist env_keys
- **Sanitisation tools MCP** — noms 64 chars, descriptions 250 chars, schemas profondeur 4 / 20 props
- **bounded_json OAuth** — réponses OAuth/discovery limitées à 512 KB
- **Mutex refresh token** — pas de race condition sur le refresh simultané
- **Tokens résolus au spawn** — pas stockés en mémoire dans la struct transport
- **Cache invalidé** — à la suppression de token OAuth ou env
- **Erreurs MCP sanitisées** — 200 chars max, control chars filtrés
- **notifications/initialized fail closed** — erreur bloque au lieu de laisser passer

## v0.8.3

### Features

- **3 new LLM providers** — xAI (Grok 4.x), Moonshot (Kimi K2.6) and Z.ai (GLM-5.1) added to the unified OpenAI-compatible backend. Static model catalogs with context length for providers without `/models` endpoint.
- **Grok 4.3** — latest xAI model added (1M context, native reasoning, vision)
- **Updated provider descriptions** — OpenAI updated to GPT-5.5, DeepSeek updated to V4-Flash/V4-Pro
- **Multi-turn reasoning** — thinking/reasoning content now persists across tool calls in chat sessions
- **Moonshot balance API** — quota display for Moonshot Kimi via `/v1/users/me/balance`
- **Provider capability detection** — per-provider modules for tools, thinking and vision detection (xAI, Moonshot, Z.ai)

### Security

- **Test-before-save for API keys** — keys are now tested before being saved to the vault. Invalid keys are never persisted. New `test_api_key_with_value` command tests without storing.
- **Vault base64 zeroization** — master key base64 strings from keyring read/write are now properly zeroized after use
- **IPC key zeroization** — API key strings from Tauri IPC are zeroized after being copied to the vault
- **Input validation** — provider ID and key format validation before any vault operation, unknown providers rejected
- **Bounded parsing** — model list parsing capped at 500 entries, model name length validated (max 128 bytes)
- **Generic error messages** — no filesystem paths or stack traces exposed to the frontend
- **Log redaction** — sensitive JSON fields redacted from HTTP body logs
- **Removed unused search providers** — SerpAPI and Google CSE removed from catalog (were listed but never implemented)

## v0.8.2

### Features

- **6 tools office natives** — Le LLM peut manipuler des fichiers Excel, Word, PDF et images sans dépendance externe (calamine, rust_xlsxwriter, umya-spreadsheet, pdf-extract, image). Cross-platform macOS/Linux/Windows.
  - `read_spreadsheet` / `write_spreadsheet` — xlsx, xls, ods, xlsm, csv, tsv
  - `read_document` / `write_document` — pdf, docx
  - `read_image` / `process_image` — jpeg, png, webp (resize, crop, conversion)
- **Previews office dans les bulles du chat** — chaque appel write_spreadsheet affiche un tableau avec les numéros de lignes et lettres de colonnes Excel correspondant aux cellules écrites. Les write_document affichent les blocs de contenu (titres, paragraphes, listes, tableaux).
- **Previews office dans le panel** — rendu fidèle des fichiers dans le panel latéral :
  - Spreadsheet : table custom avec en-têtes de colonnes, numéros de lignes, sélecteur de feuilles, scroll
  - DOCX : rendu Word via docx-preview (styles, polices, tableaux)
  - PDF : rendu PDFium via EmbedPDF (fidélité Chrome)
- **Historique des modifications** — chaque écriture office sauvegarde ses opérations pour afficher le contenu tel qu'il était au moment de l'écriture, pas l'état actuel du fichier
- **Icônes fichiers office** — xlsx, xls, xlsm, csv, ods, tsv, docx, pdf dans le panel
- **Détection d'éditeurs externe** — fonctionne nativement pour tous les formats office (macOS Launch Services, Linux xdg-mime, Windows assoc)
- **Tolérance JSON des LLMs** — coercion tolérante, réparation JSON malformé, normalisation des formules françaises (SOMME→SUM, etc.), détection auto du type de valeur (nombres en string, formules, booléens)

### Security

- Collections bornées : MAX_OPS, MAX_CELLS, MAX_ROW, MAX_COL (frontend), HARD_MAX_COLS (Rust)
- Limites de taille fichier : 50 MB pour les previews binaires et spreadsheet
- Validation is_file() + whitelist d'extensions pour read_binary_preview
- Path traversal bloqué par les pré-hooks sur les 3 tools write

### Fixes

- Fix toolsToRecords pour write_spreadsheet, write_document, process_image (summary était JSON.stringify au lieu du path)
- Fix historique panel : les previews write_file montrent le snapshot sauvegardé au lieu de relire le fichier sur disque
- Suppression des previews read_ dans les bulles et le panel (pas utiles pour la lecture seule)

## v0.8.1

### Features

- **i18n — 5 nouvelles langues** — Allemand, Espagnol, Italien, Chinois simplifié et Japonais (en plus de Français/Anglais)
- **Audit texte hardcodé** — tous les textes en dur dans l'UI remplacés par des clés i18n (12 fichiers corrigés, 21 nouvelles clés)
- **Dates localisées** — les mois et jours dans les réveils utilisent `Intl.DateTimeFormat` (support automatique de toutes les langues)
- **Langue de réponse du LLM** — nouveau setting dans General pour choisir dans quelle langue le modèle doit répondre (injecté dans le system prompt)
- **Settings réorganisés** — "Lancer au démarrage" et "Démarrage masqué" déplacés de Advanced vers General
- **`patch_advanced_settings`** — nouvelle commande Tauri pour la mise à jour partielle de la config

## v0.8.0

### Features

- **Context compression** — automatic and manual (`/compress`) conversation compression when token threshold is reached
- **Compression settings** — enable/disable toggle and threshold slider (0-100%, default 85%) in Settings > Advanced
- **Model eligibility** — compression available for models with native context >= 128k tokens
- **Dynamic architecture detection** — reads context length from any Ollama model architecture (Gemma, Qwen, LLaMA, Mistral, etc.)
- **All providers supported** — works with Ollama, Anthropic, OpenAI, Groq, Gemini and all OpenAI-compatible APIs
- **Post-response compression** — threshold check after each LLM response, not just before
- **Last response preserved** — the most recent LLM response is always kept visible after compression
- **Compression animation** — orange pulsing "Compression" indicator with Lottie loader at bottom of chat

### Fixes

- **Token counting** — context ring now uses real Ollama token count (`context_tokens` = last prompt + eval) instead of accumulating prompt tokens across requests
- **Per-message token display** — shows output tokens for that response only, not total context
- **Context window detection** — correctly reads `OLLAMA_CONTEXT_LENGTH` env var when no modelfile `num_ctx` is set

## v0.7.9

- **File Preview Panel** — side panel to view files created/edited by the agent (syntax highlighting, diffs, fullscreen, resize, open in external editor)
- **Syntax highlighting** in chat tool bubbles (37 languages)
- **Real line numbers** in edit diffs (shows actual file position)
- **Auto word-wrap** — text files wrap, code files scroll horizontally
- **File extension icons** (20+ types)
- Consistent diff colors and tool bubble width across chat and panel

## v0.7.6

### Features

- **Per-session permission mode**: each conversation now has its own mode (Chat/Manual/Auto) independent of others
- **Ollama model updates preserve customizations**: system prompt and parameters are saved before pull and restored after
- **Splash screen**: app icon displayed on themed background while the app loads
- **Single instance**: prevents opening duplicate windows when double-clicking the app icon (macOS/Linux/Windows)

### UI / Theming

- **Dark theme**: translucent background applied to model selector dropdown, permission mode dropdown, project directory dropdown, heartbeat cards/dialog/button, settings cards/selects, API connectors modal/cards, and Ollama modelfile raw block
- **Dark theme**: model selector provider and favorites headers now transparent (no opaque shell background)
- **Dark theme**: removed border on model selector search input and API connectors search input
- **Light theme**: user message bubbles use translucent gray (0.45 opacity)
- **Light theme**: chat input uses translucent gray background (0.80 opacity)
- **Settings subtabs**: added hover effect on mouse over
- **Sidebar**: settings icon and text now match the color of other nav items
- **Model selector dropdown**: opens to the right instead of left to avoid sidebar overlap
- **Permission mode dropdown**: removed "Mode" header line, Chat mode color changed to thinking blue (#4A9EE8)
- **API connectors modal**: fixed size (85vh) with top-aligned grid to prevent layout shift between tabs
- **Ollama Modelfile tab**: extended active tab indicator by 3px for visual balance
- **Ollama parameters editor**: `num_ctx` and `num_predict` rows shown by default

## v0.7.5

### UI / Theming

- **Dark theme**: lightened sidebar background (`--shell`) for better contrast
- **Dark theme**: chat input and user message bubbles now use translucent backgrounds (0.55 opacity)
- **Dark theme**: all sidebar hover and selection states switched to translucent white for a softer, more cohesive look
- **Light theme**: sidebar background shifted from warm beige to neutral light gray
- **Light theme**: chat background (`--void`) lightened to a clean off-white without being too bright
- **Light theme**: user message bubbles now use a translucent gray (0.45 opacity)
- **Light theme**: chat input uses a translucent gray background (0.80 opacity)
- **Light theme**: accent orange lightened across all buttons for a fresher appearance
- **Ollama tab**: extended Modelfile active tab indicator by 3px for visual balance

## v0.7.4

### Security

- Environment variable logging restricted to an explicit allowlist
- Level 3 security audit — 15 fixes covering secrets handling, input validation, error messages, and bounded collections

## v0.7.3

### Features

- Ollama sidecar: dynamic port allocation, environment variable passthrough, GPU status detection, retry logic

### Fixes

- 4 issues from GPT review + refactored files exceeding 200 lines

## v0.7.2

### Features

- Reliable hover actions, aligned icons, Ollama pull cancellation with cleanup
- Partial content and tool results preserved on stream stop

### Fixes

- Race condition: cancel now targets the correct stream token after stop+restart
- PID file to kill orphan Ollama sidecars on restart
- Toolbar alignment, model selector live refresh, CSP images

## v0.7.1

### Fixes

- Windows: 3px window border padding

## v0.7.0

### Fixes

- Windows: personality toggles fix

### Features

- Settings: CPU/GPU hardware acceleration toggle + Ollama restart

## v0.6.9

### Fixes

- Windows: update detection + NSIS installer

## v0.6.8

### Features

- Vulkan auto-enabled for AMD GPUs on Windows + sidecar logs

## v0.6.7

### Fixes

- Robust Ollama download + extraction validation (Windows fix)
