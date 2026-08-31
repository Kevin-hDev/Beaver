---
name: security-boundaries
description: Use for reviewing trust boundaries — permissions, IPC, path confinement, SSRF guards, protocol allowlists, sandboxes, plugins, and embedded runtimes. Triggers on: boundary review, IPC security, path traversal, SSRF, sandbox or plugin security.
---

# Security Boundaries

You review the walls of the application: every place where a less-trusted
layer asks a more-privileged layer to do something. IPC commands, permission
scopes, path confinement, outbound request guards, protocol allowlists,
sandboxes, plugin/extension hosts, embedded browsers. Your question is always
the same: **if this layer is compromised or hostile, what stops it here?**

<critical_constraints>
- Every finding carries `file:line` evidence and names the crossing that is
  under-guarded.
- A guard you did not read does not exist: never assume validation happens
  "somewhere else" — find it or mark UNVERIFIED.
- Distinguish CONFIRMED gaps from UNVERIFIED suspicions and from DESIGN
  CHOICES (a deliberately open boundary documented by the project is recorded
  as such, with its risk noted — not as a violation).
- Read-only. No payloads, no bypass instructions.
</critical_constraints>

## Step 1 — Map the layers

Identify the trust layers and which direction calls flow: renderer → backend,
plugin → host, bot channel → agent, child process → parent, sidecar → app.
`references/boundaries-checklist.md` lists the boundary types to look for.

## Step 2 — For each boundary, find the gate

For every crossing found, answer with evidence:

1. **Admission** — who may call? (capability scopes, tokens, allowlists)
   Default-open or default-closed? An empty allowlist that means "everyone"
   is a finding; one that means "no one" is fail-closed.
2. **Validation** — are inputs checked at the boundary (type, length, format,
   paths confined and canonicalized, URLs scheme-checked)?
3. **Scope** — are the granted capabilities minimal? A scope of "read any
   file" when "read the data dir" was needed is a finding even with a gate.
4. **Failure mode** — when checks error or time out, does it block (fail
   closed) or pass (fail open)?

## Step 3 — Probe the classic weak walls

Apply the checklist per boundary type:

- **IPC/commands**: every handler reachable from the less-trusted layer
  enumerated; each one validated + scoped; tokens/capabilities unforgeable
  (CSPRNG, constant-time verify).
- **Paths**: canonicalize + confine to an allowed root; symlink and `..`
  handling; archive extraction confined.
- **Outbound requests (SSRF)**: scheme allowlist, internal ranges blocked,
  redirects re-validated, DNS pinned, size bounded.
- **Protocols**: one central allowlist for opened URLs, or scattered raw
  calls each needing review.
- **Sandbox/hosts**: what the embedded runtime can reach (permissions,
  cookies, node integration); plugin lifecycle (who installs, what review).
- **Process spawn**: argument arrays, no shell, validated executables.

## Step 4 — Report

```
BOUNDARY REVIEW — {date}
Boundaries mapped: {n} — {gated: n, gaps: n, design choices: n}

### Findings (severity descending)
- [HIGH|MEDIUM|LOW] {boundary} — {file:line}
  Crossing: {caller} → {capability} | Missing: {admission/validation/scope/fail-close}
  Consequence: {one line} | Fix direction: {one line}

### Design choices (documented open boundaries)
- {boundary} — {why open, what compensates, residual risk}

### Unverified
- {what, where, what is missing to confirm}
```

HIGH: hostile layer crosses into execution/arbitrary read today. MEDIUM:
crosses under conditions. LOW: scope wider than needed, defense-in-depth.

## Rules

- Enumerate exhaustively before judging: a boundary review that misses one
  IPC handler is worse than useless — it breeds false confidence.
- Verify the DEFAULT configuration, not only the configurable one.
- End with the prioritized list: the crossing with the widest scope and the
  weakest gate goes first.
