# Security Policy

Beaver is a desktop application (Tauri 2 + React 19) that runs local LLMs via Ollama and connects to cloud providers. It handles API keys, MCP connectors, external channels (Telegram, Slack, Discord), and agent tools that can read and write files on your machine. This document explains how secrets are protected, how to report a vulnerability, and how to use the app safely.

## Supported versions

Security fixes are only released for the latest version. Keep the app updated — updates are automatic and a notification appears when a new version is available.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security bugs.**

Please report vulnerabilities privately so they can be triaged and fixed before public disclosure:

1. Go to the **[Security tab](https://github.com/Kevin-hDev/Beaver/security)** of the repository.
2. Click **"Report a vulnerability"** to open a private advisory.
3. Include:
   - A clear description of the issue and its impact
   - Step-by-step reproduction (commands, inputs, file paths)
   - The version affected (visible in the app's About / Settings)
   - Your assessment of severity, if known

You should receive an initial response within a few days. Please avoid public disclosure until a fix has been released. Responsible disclosure is credited in the release notes unless you ask to remain anonymous.

## Threat model (summary)

Beaver is a **local desktop app**, not a public server. The most relevant attackers are:

- **A compromised frontend** (XSS via Markdown rendering, a malicious skill, or injected message content) trying to reach secrets or read arbitrary files through the Tauri IPC bridge.
- **A malicious or compromised LLM provider** returning crafted responses (redirects, error bodies) to leak credentials.
- **A malicious MCP connector or model** attempting command injection or environment poisoning.
- **Local abuse of agent tools** running in auto-permission mode (writing sensitive paths, running destructive shell commands).

The controls below are designed for these threats. Out of scope: physical access to an unlocked machine, malicious OS-level software with the user's privileges, and compromise of an OS keyring.

## Secret management

API keys (LLM, search, forecast, MCP, gateway) are the most sensitive data handled by the app.

- **Encrypted vault**: keys are stored in `secrets.enc`, encrypted with **XChaCha20-Poly1305** (authenticated encryption, random nonce per write via `OsRng`).
- **Master key in the OS keyring**: the encryption key lives in macOS Keychain, Windows DPAPI, or the Linux Secret Service — never on disk, never in the source code.
- **One keyring access at startup**: the master key is loaded once and kept in memory only.
- **Zeroization**: all secrets in memory use `Zeroizing<String>` and `ZeroizeOnDrop`. Intermediate buffers are zeroized after use; the garbage collector alone is not trusted to clear them.
- **Constant-time comparison**: tokens, hashes, and API keys are compared with XOR byte-by-byte or `subtle::ConstantTimeEq` — never with `==`.
- **Frontend never sees a key**: no Tauri command exposes `get_api_key`. The available commands are `set_api_key`, `delete_api_key`, `has_api_key`, `list_configured_providers`, and `test_api_key`. Rust loads the key only at the moment of the HTTPS call and zeroizes it afterward.

## Path traversal protection

Every file path coming from the frontend is validated before any read or write:

- `canonicalize()` resolves symlinks and `..` segments.
- `starts_with()` checks the resolved path is inside an allowed root (working directory or registered project roots).
- Paths containing `..` are rejected by validation.
- Attachment access uses an HMAC grant model with bounded size and count limits.

## Bounded collections and resource limits

All collections that can grow from external input are capped to prevent memory and disk exhaustion:

| Resource | Limit |
|---|---|
| Active LLM streams | 32 |
| PTY (terminal) sessions | 16 |
| Messages per session | 2,000 |
| Subagent history messages | 2,000 |
| Write-guard registry sessions | 32 |
| Gateway sessions per map | 1,000 |
| MCP JSON depth | 16 |
| MCP JSON nodes | 256 |
| MCP argument size | 64 KB |
| MCP line size | 1 MB |
| Attachments per message | 15 |
| Attachment size | 20 MB |
| Bash tool output lines | 2,000 |
| Scheduler log (rolling) | 500 lines |
| Gateway audit line size | 2 KB |

## Secure HTTP for credentials

Outbound calls that carry credentials go through a centralized `AuthenticatedClient` that:

- Blocks HTTP redirects (`Policy::none()`) — prevents credential leakage via malicious 302 redirects to attacker-controlled URLs.
- Enforces HTTPS for secret-bearing requests.
- Bounds response bodies to prevent memory DoS.
- Sanitizes error messages so no internal path, stack trace, or raw body reaches the UI.

## MCP connector hardening

MCP connectors can spawn local processes (`npx`, `uvx`, `deno`). To prevent command injection and environment poisoning:

- **Allowlist of programs**: only `npx`, `uvx`, `deno` are permitted.
- **No shell**: arguments are passed as a `Vec`, never concatenated into a shell string.
- **Argument validation**: a regex rejects `;`, `|`, `&`, backticks, `$()`, and other shell metacharacters.
- **Environment isolation**: `env_clear()` wipes the parent environment; only an explicit allowlist is passed. `NODE_OPTIONS`, `LD_PRELOAD`, `DYLD_INSERT_LIBRARIES`, and similar dangerous variables are blocked.
- **JSON-bomb defense**: deep nesting, large node counts, `$ref` cycles, oversized arguments, and oversized lines are all rejected (fail-closed).

## Gateway (external channels)

The optional Gateway lets external channels (Telegram, Slack, Discord) reach a local agent. Controls include:

- **Conversation isolation**: per-conversation locks prevent cross-talk; channel and message IDs are validated against a restricted charset (no `/` or `..`).
- **Rate limiting**: per-user token buckets bound request frequency.
- **Audit logging**: all inbound messages are hashed and logged to a rolling JSONL file. Log forging (newline injection) is rejected.
- **Credential isolation**: each channel's tokens are namespaced (`mcp_{id}_{key}`) and never mixed.

## Safe diagnostics and logs

- **Generic user errors**: the frontend only sees generic messages such as `operation_failed` or `attachment_access_denied`. Internal paths, table names, library versions, and stack traces never leave the backend.
- **Filtered logs**: provider HTTP bodies are truncated to 200 characters via `sanitize_log_body()` before any logging. Known credential formats (`sk-...`, `Bearer ...`, `AKIA...`, `xoxb-...`, `ghp_...`, JWT) are redacted. No secret is ever written to logs.
- **Bounded agent diagnostics**: when a stream or tool fails, the agent stores a short, redacted, bounded summary — never the raw error or the raw HTTP body.

## Safe usage recommendations

As a user, you can further reduce risk:

- **Prefer manual permission mode** for agent tools if you are unsure — it asks before every read, write, or shell command.
- **Review MCP connectors** before enabling them; only install connectors from sources you trust.
- **Keep auto-permission mode** for trusted, scoped working directories only.
- **Do not paste API keys** into chat messages or skills — always use the API Keys settings, which route them through the encrypted vault.
- **Review forecast datasets** before sending them to a cloud provider (Nixtla TimeGPT); local datasets may contain sensitive business data.
- **Pin the app to the latest version** to receive security fixes.

## Local data location

All app data lives under `~/.local/share/cl-go-dash/` on macOS, Linux, and Windows.
This directory keeps its historical identifier for compatibility with existing
installations. The most security-relevant files are:

- `secrets.enc` — encrypted vault (XChaCha20-Poly1305)
- `logs/wakeups.jsonl`, `logs/gateway-audit.jsonl` — rolling logs, no secrets
- `agent-sessions/*.json` — conversation history, no raw API keys
- `mcp-connectors.json` — connector config (tokens are in the vault, not here)

See the main README for the full file inventory.

## Limitations and known gaps

- **No code signing**: macOS builds are not signed. Gatekeeper may block the app when downloaded through a browser; use the provided `install.sh` script (which uses `curl`) or build from source.
- **Ollama is bundled, not signed by upstream**: on Windows, "Controlled folder access" may block `ollama.exe` on first launch — click "Allow".
- **The OS keyring is a single point of trust**: if the OS keyring is compromised, the vault master key is exposed. This is inherent to desktop secret storage.
- **Cloud providers see your prompts**: anything sent to Groq, OpenAI, Gemini, etc. transits their servers. Use local Ollama models for sensitive content.

## License

Beaver is licensed under the [Apache License 2.0](LICENSE). This security policy is part of the project documentation.
