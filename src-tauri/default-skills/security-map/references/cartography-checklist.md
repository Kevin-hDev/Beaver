# Cartography Checklist — what a sensitive zone looks like

Use this as the complete model during Phase 2–4 of the map. A zone belongs on
the map when untrusted data can reach a privileged capability through it.

## A. Entry points for untrusted data

Look for these patterns and read each hit far enough to confirm the flow:

| Entry point | Search patterns |
|---|---|
| Rendered markup | `dangerouslySetInnerHTML`, `innerHTML`, `rehypeRaw`, `v-html`, `renderAsync` |
| Markdown / rich text | `ReactMarkdown`, `marked(`, `markdown-it`, sanitize configs |
| File parsing | preview/read of pdf, docx, xlsx, images, archives (zip/tar extraction) |
| Network responses | fetch clients, streaming parsers, SSE handlers, download + execute |
| LLM / agent output | tool dispatch, tool result rendering, prompt assembly with external content |
| User-supplied paths | drag & drop, file pickers, path params crossing into fs/process APIs |
| URLs from outside | link preview, open-in-browser, deep links, OAuth redirect/callback |
| Inbound messages | chat channels, bots, webhooks, IPC events from another layer |
| Deserialization | `JSON.parse` of external data, YAML/pickle-like loaders, binary decoders |

## B. Privileged capabilities

| Capability | Search patterns |
|---|---|
| Shell / process | `spawn`, `exec`, `Command`, PTY/terminal managers |
| Filesystem write | write/rename/delete APIs, path joins from external input |
| Network egress | HTTP clients, redirect handling, proxy config |
| IPC / command surface | command handlers callable from a renderer/frontend |
| Embedded runtimes | webviews, CEF/Electron, extension/plugin hosts, sidecar processes |
| Updates & installers | download-verify-install chains, checksums, signatures |

## C. Secrets and sensitive data

| Question | Where to look |
|---|---|
| Where do secrets live at rest? | vault/keystore modules, env files, config stores, plaintext JSON |
| Can a less-trusted layer read them? | IPC commands returning secrets, `get` accessors, global state |
| Do secrets reach logs? | log sinks, error formatters, HTTP body logging, audit trails |
| Are comparisons safe? | `==` on tokens vs constant-time compare |
| Are buffers cleared? | zeroization, cleanup after use |

## D. Log sinks

Every file/appender/stream that persists text. For each: what user-or-model
controlled content can flow in, and is there a redaction step before write.

## E. Scoring rubric (Phase 4)

Score each zone 1–3 on both axes, multiply:

**Exposure** (how much untrusted data reaches it)
- 3 — network/LLM/bot content flows in routinely
- 2 — local files or user-pasted content
- 1 — only the local user's own deliberate actions

**Blast radius** (what falls if it falls)
- 3 — code execution, shell, arbitrary file read/write
- 2 — secrets, tokens, accounts
- 1 — display glitches, limited data

**Score = exposure × blast radius.** 9–6: audit first. 4–3: soon. 2–1: backlog.

## F. Common false friends (do NOT map as zones)

- Test files exercising dangerous patterns with fake data
- Build/dev scripts that never ship
- Sanitized pipelines already covered by executed test batteries — map them
  as CONFIRMED with their protection noted, not as open risks
