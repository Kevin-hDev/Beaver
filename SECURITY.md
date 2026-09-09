# Security Policy

Beaver is a desktop application (Tauri 2 + React 19) that runs local LLMs via Ollama and connects to cloud providers. It handles API keys, MCP connectors, external channels (Telegram, Slack, Discord), and agent tools that can read and write files on your machine. It also supports trusted custom extensions, including advanced interface modules. This document explains the security boundaries, how to report a vulnerability, and how to use the app safely.

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
- **A malicious or compromised extension or dependency** abusing the access granted when the user approves its execution.

The controls below protect Beaver's own interfaces and managed execution paths. They do not sandbox approved extension code. Out of scope for those protections: physical access to an unlocked machine, malicious OS-level software with the user's privileges, and compromise of an OS keyring. Report defects in Beaver's validation, approval, or isolation mechanisms even when an extension exposes them.

## Secret management

API keys (LLM, search, forecast, MCP, gateway) are the most sensitive data handled by the app.

- **Encrypted vault**: keys are stored in `secrets.enc`, encrypted with **XChaCha20-Poly1305** (authenticated encryption, random nonce per write via `OsRng`).
- **Master key in the OS keyring**: the encryption key lives in macOS Keychain, Windows DPAPI, or the Linux Secret Service — never on disk, never in the source code.
- **One keyring access at startup**: the master key is loaded once and kept in memory only.
- **Zeroization boundary**: Rust uses zeroizing containers for stored keys and sensitive transport buffers. This is not a guarantee that every copy is erased: once an approved extension receives a JavaScript string, Beaver cannot guarantee immediate erasure or prevent the extension from retaining it.
- **Secret comparisons**: authentication checks use constant-time comparison, including `subtle::ConstantTimeEq`. This requirement concerns secret values, not ordinary identifiers or public file-integrity checks.
- **Built-in credential interface**: the settings interface can set, delete, check, and test credentials, but does not expose a command to read stored API keys. This is distinct from the extension API: approved extensions can request supported provider keys, MCP credentials, and channel tokens.

## Plugins and custom extensions

Installing an extension from a local source, Git, or npm does not make it trustworthy. Review its source, dependencies, and origin before approving it. Beaver's official plugins remain distinct from built-in Tools.

- **Trusted Node.js code**: third-party extensions run in separate host processes, with controlled access to Beaver's core through an identity-bound bridge. Process separation helps contain failures; it does not restrict the extension's direct filesystem, network, or process access under the user's OS account.
- **Approval and integrity**: Beaver checks the files covered by its extension fingerprint and requires renewed approval when covered content changes. This is an integrity check, not a security audit of the extension or everything it may load later.
- **Tool permissions**: Beaver applies confirmation and Plan-mode rules according to the tool's declared effect. These rules govern calls dispatched by Beaver, not arbitrary code executed directly by the extension. An effect declaration is not proof that the code behaves as declared.
- **Secret access**: approved extensions can request supported secrets through the SDK. Beaver validates the requested resource and requires its sensitive-access audit step to succeed before releasing the secret. An extension can still retain or disclose a secret once received; disabling it cannot revoke copies it already holds.
- **Standard interface contributions**: Beaver validates and renders declarative tabs, panels, settings, actions, and themes. The extension's host code remains trusted code even when its interface is declarative.
- **Advanced interface modules**: these require additional explicit approval and execute in Beaver's own WebView. They share the page and its privileges; a restricted-looking SDK object is not a sandbox. Such a module can interfere with the interface and access capabilities available to that WebView.
- **Recovery**: diagnostics and safe mode help recover from loading failures or a broken interface. Safe mode is a recovery mechanism, not a way to safely execute untrusted code. Some advanced changes require restarting Beaver to clear.

See **[EXTENSIONS.md](EXTENSIONS.md)** for the supported API, approval lifecycle, fingerprint coverage, limits, and exact safe-mode recovery instructions. The guide is currently in French.

## Path traversal protection

Beaver's managed file-access paths use validation appropriate to the operation:

- `canonicalize()` resolves symlinks and `..` segments.
- Root checks keep access within the scope authorized for the operation, such as a project directory or an explicitly granted attachment.
- Paths containing `..` are rejected by validation.
- Attachment access uses an HMAC grant model with bounded size and count limits.

These checks do not constrain filesystem operations performed directly by approved extension code.

## Bounded collections and resource limits

Beaver caps managed resources to reduce memory and disk exhaustion risks. These application limits do not cap allocations or files created directly by approved extension code:

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

Extension protocol and interface limits are documented in [EXTENSIONS.md](EXTENSIONS.md), with the executable contracts linked there as their authority.

## Secure HTTP for credentials

Beaver's `AuthenticatedClient` protects the credential-bearing requests routed through it:

- Blocks HTTP redirects (`Policy::none()`) — prevents credential leakage via malicious 302 redirects to attacker-controlled URLs.
- Enforces HTTPS for secret-bearing requests.
- Bounds response bodies to prevent memory DoS.
- Sanitizes error messages so no internal path, stack trace, or raw body reaches the UI.

It does not mediate network requests made directly by an approved extension.

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
- **Credential isolation**: channel tokens are namespaced by channel, account, and token kind (`gateway.<channel>.<account>` with a kind suffix where applicable), separately from MCP credentials.

## Safe diagnostics and logs

- **User-facing errors**: managed error paths use safe error categories and translated messages instead of exposing raw internal errors. This does not make arbitrary extension output safe to publish.
- **Filtered logs**: Beaver's provider-log sanitization truncates bodies and redacts recognized credential formats. Redaction is not a guarantee that arbitrary sensitive text or a secret in an unrecognized format will be removed. Review diagnostics before sharing them.
- **Bounded agent diagnostics**: when a stream or tool fails, the agent stores a short, redacted, bounded summary — never the raw error or the raw HTTP body.

Extension authors must not put secrets in logs, errors, tool results, or diagnostics. Beaver cannot enforce this for files or external services the extension writes to directly.

## Safe usage recommendations

As a user, you can further reduce risk:

- **Prefer manual permission mode** for agent tools if you are unsure. It requests confirmation for operations covered by the permission policy; ordinary reads and some recognized safe commands do not require a prompt. Session approvals can also avoid repeated prompts. It is not an OS sandbox.
- **Review MCP connectors** before enabling them; only install connectors from sources you trust.
- **Review extensions and their dependencies** before approving them, especially advanced interface modules. If a secret may have leaked, disable the extension and revoke or rotate the credential at its provider; uninstalling alone is insufficient.
- **Keep auto-permission mode** for trusted, scoped working directories only.
- **Do not paste API keys** into chat messages or skills — always use the API Keys settings, which route them through the encrypted vault.
- **Review forecast datasets** before sending them to a cloud provider (Nixtla TimeGPT); local datasets may contain sensitive business data.
- **Pin the app to the latest version** to receive security fixes.

## Local data location

All app data lives under `~/.local/share/cl-go-dash/` on macOS, Linux, and Windows.
This directory keeps its historical identifier for compatibility with existing
installations. The most security-relevant files are:

- `secrets.enc` — encrypted vault (XChaCha20-Poly1305)
- `logs/wakeups.jsonl`, `logs/gateway-audit.jsonl` — rolling operational logs; review before sharing
- `agent-sessions/*.json` — conversation history, which may include sensitive text pasted by the user or returned by tools; it is not protected by the credentials vault
- `mcp-connectors.json` — connector config (tokens are in the vault, not here)
- `extensions.json`, `extension-installs/` — extension approvals, metadata, and managed extension code; do not modify these to bypass validation

See [README.md](README.md) for the broader file inventory. The credentials vault does not encrypt all application data.

## Limitations and known gaps

- **No code signing**: macOS builds are not signed. Gatekeeper may block the app when downloaded through a browser; use the provided `install.sh` script (which uses `curl`) or build from source.
- **Ollama is downloaded separately**: Beaver installs its managed runtime when needed rather than including it in the application installer. Windows security controls may block `ollama.exe`; verify the origin of the executable before granting access.
- **The OS keyring is a single point of trust**: if the OS keyring is compromised, the vault master key is exposed. This is inherent to desktop secret storage.
- **Cloud providers see your prompts**: anything sent to OpenAI, Gemini, Mistral, etc. transits their servers. Use local Ollama models for sensitive content.

## License

Beaver is licensed under the [GNU Affero General Public License v3.0](LICENSE). This security policy is part of the project documentation.
