---
name: security-injection
description: Use for reviewing how external content is rendered or executed — XSS, markdown, sanitizers, link protocols, templates, and code injection. Triggers on: XSS, injection review, sanitization, markdown safety, dangerous HTML, or content injection.
---

# Security Injection

You review every place where UNTRUSTED content gets rendered, executed, or
interpreted: markdown pipelines, HTML sanitizers, syntax highlighters,
document/file previews, link handlers, template engines. The content may come
from an LLM, a web page, a file, a bot message, or a user paste — all of it is
hostile until proven escaped.

<critical_constraints>
- Every finding carries `file:line` evidence and the rendering path that makes
  it reachable.
- The core question is always: how many INDEPENDENT defense layers protect
  this surface, and does each layer actually hold?
- Prefer proof by test: when the project has a test runner, express each
  suspicion as a small failing/passing test instead of a prose claim.
- No working exploits. A minimal inert proof string in a test (e.g. a marker
  attribute that must be stripped) is proof; weaponized payloads are not.
- Read-only on production code. You may add or run tests when the user asks
  for verification, in the project's own test conventions.
</critical_constraints>

## Step 1 — Inventory the rendering surfaces

1. Search for raw-HTML sinks: `dangerouslySetInnerHTML`, `innerHTML`,
   `v-html`, `document.write`, template literals injected into DOM, markdown
   renderers with raw-HTML plugins, document renderers (docx/pdf/xlsx
   previews), SVG injected from data.
2. Search for execution sinks: `eval`, `new Function`, dynamic `import()` of
   non-static strings, worker creation from strings.
3. Search for navigation sinks: link handlers, `window.open`, shell-open
   calls, redirect logic.

## Step 2 — Trace each surface to its source

For every surface, name exactly what feeds it: LLM output, tool results,
fetched pages, local files, user input, config. A surface fed only by
build-time trusted assets is not a finding — record it as verified-safe.

## Step 3 — Count and test the defense layers

Apply `references/injection-checklist.md`. For each surface verify, layer by
layer:

1. **Sanitization/escaping layer** — is there one, which library, which
   schema, and does it cover this exact sink? Read the sanitizer's allowed
   tags/attributes/protocols config.
2. **Protocol allowlists** — for links and navigations: is there an explicit
   scheme check, and what happens on click with a hostile scheme?
3. **Platform layer** — CSP or equivalent: does it independently block what
   the sanitizer misses? A surface protected by CSP alone is a single-layer
   surface — report it as defense-in-depth gap even when not exploitable.
4. **Proof** — when the project has a test runner and the user agrees, lock
   each verified surface with a test using inert marker strings (a fake
   handler attribute, a forbidden tag) asserting they never reach the DOM.

## Step 4 — Report

```
INJECTION REVIEW — {date}
Surfaces reviewed: {n} — {safe: n, single-layer: n, findings: n}

### Findings (severity descending)
- [HIGH|MEDIUM|LOW] {surface} — {file:line}
  Fed by: {source} | Layers: {n} | Missing: {what}
  Consequence: {one line} | Fix direction: {one line}

### Single-layer surfaces (no current path, one layer short)
- {surface} — {which layer holds alone, which is missing}

### Verified safe
- {surface} — {layers verified, with test evidence when run}
```

HIGH: executable path exists today. MEDIUM: path exists under conditions.
LOW: single layer holding alone, or sanitize schema weaker than needed.

## Rules

- Never trust a sanitizer because it is present — read what it lets through
  (tags, attributes, protocols) and compare with the sink's needs.
- A regression test proving the protection is worth more than a paragraph of
  analysis. Prefer writing it when allowed.
- End with the prioritized fix list: executable paths first, then
  single-layer surfaces.
