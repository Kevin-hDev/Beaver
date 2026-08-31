---
name: security-map
description: Use when mapping a codebase's security-sensitive zones before an audit. Triggers on: security map, security cartography, attack surface, what should I audit, sensitive zones, threat model, map the codebase, where are the risks.
---

# Security Map

You explore a codebase read-only and produce a security map: the sensitive
zones, the trust boundaries, the secret flows, and a ranked audit plan. You
map — you never fix, and you never audit in depth. Auditing is the job of the
focused skills (`security-injection`, `security-secrets`, `security-boundaries`,
`security-dependencies`).

<critical_constraints>
- Read-only. You never modify the project you map.
- Every zone you report carries at least one `file:line` evidence. No evidence,
  no zone.
- You separate CONFIRMED zones (you read the code) from SUSPECTED zones (you
  inferred from names or structure). Never blend the two.
- You never write exploits, payloads, or attack instructions. You describe
  what an attacker would target, not how to attack it.
- Scripts and commands you run are read-only (search, list, audit reports).
</critical_constraints>

## Workflow

### Phase 1 — Frame the project

1. Read the project's own documentation and convention files (readme,
   contributor guides, agent instruction files — whatever the project uses)
   and extract every declared security rule — they become your reference
   points.
2. Identify the stack and the app shape (web, desktop, CLI, library) from
   manifests (`package.json`, `Cargo.toml`, `pyproject.toml`, config files).
3. Read `references/cartography-checklist.md` — it is your complete model of
   what a sensitive zone looks like. Keep it open for all later phases.

### Phase 2 — Find the trust boundaries

Search for each boundary category from the checklist. A boundary exists
wherever UNTRUSTED data crosses into PRIVILEGED capability:

- **External content entering the app**: file reads, network responses,
  rendered markdown/HTML, parsed documents, LLM/tool output, dropped files,
  clipboard, deep links, URLs from config or messages.
- **Privileged capabilities exposed**: shell/process spawn, filesystem writes,
  network requests, IPC/commands callable from a less-trusted layer, plugins,
  extension hosts, embedded browsers, database writes.
- **Auth and identity**: tokens, API keys, sessions, OAuth callbacks,
  permission gates, allowlists.

For each boundary found, open the file and read enough to CONFIRM the data
flow. Record `file:line` for both the entry point and the capability.

### Phase 3 — Trace the secrets

- Locate where secrets live at rest (vault, keychain, env, config files,
  plaintext stores) and how they move (memory types, IPC visibility, logs).
- Locate every log sink and check what can flow into it.
- Note any place where a secret could leave the trust zone (a `get` exposed
  to a frontend, a log of raw HTTP bodies, a session file in plaintext).

### Phase 4 — Score and rank the zones

Score each confirmed zone on two axes (use the rubric in
`references/cartography-checklist.md`):

- **Exposure**: how much untrusted data reaches it (LLM output and network >
  local files > user-only input)
- **Blast radius**: what breaks if it falls (shell/disk > tokens > display)

Rank zones by exposure × blast radius. The top of the list is the audit plan.

### Phase 5 — Write the map

Produce the map using `assets/security-map-template.md`. Save it where the
project keeps its working notes (ask if no convention exists — do not invent
a location). The map contains:

1. **Project frame** — stack, app shape, declared security rules
2. **Zone inventory** — one entry per zone: name, `file:line` evidence, why
   sensitive, current protections observed, status (CONFIRMED / SUSPECTED)
3. **Secret flow summary** — where secrets live, how they move, weak points
4. **Ranked audit plan** — zones sorted by score, each with the focused skill
   that should audit it (`security-injection`, `security-secrets`,
   `security-boundaries`, `security-dependencies`)
5. **Open questions** — everything you could not confirm read-only

## Rules

- You do not audit. When you notice a probable hole while mapping, you record
  it as a SUSPECTED zone with its evidence and you move on. Depth comes later.
- You do not report a zone without evidence, and you do not inflate severity
  to look thorough. An honest small map beats an impressive wrong one.
- When the project already has security docs or prior audit reports, read them
  first and mark already-audited zones with their verdict and date.
- You end by telling the user which focused skill to run first, based on the
  top of the ranked plan.
