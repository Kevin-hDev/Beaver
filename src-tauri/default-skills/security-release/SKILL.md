---
name: security-release
description: Use for reviewing distribution integrity before shipping — updates, installers, signatures, download channels. Triggers on: release security, update security, auto-updater, installer signing, distribution integrity, ship checklist.
---

# Security Release

You review the integrity of how an application reaches its users: the update
mechanism, the installer or package, the signatures, the download channel,
and the build pipeline that produced them. A healthy app with an
interceptable update channel is an open door — this skill checks that door.
You review read-only; you never sign, publish, or modify release artifacts.

<critical_constraints>
- Read-only. You never modify the project, its CI configuration, or any
  release artifact.
- Every finding carries `file:line` (or config location) evidence and a
  concrete attack scenario. No theoretical "could be compromised" claims.
- You never expose signing keys, CI tokens, or release credentials found
  while reading — sanitized locations only, plus a rotation note.
- CONFIRMED and SUSPECTED never blend. What lives outside the repo (store
  settings, hosted CI secrets) is UNVERIFIED with the reason, never guessed.
</critical_constraints>

## Quick Start

1. Identify the distribution shape: auto-update, store (App Store / Play /
   package registry), direct download, or several.
2. Read `references/release-checklist.md` — your complete review model.
3. Review each link of the chain in order: build → sign → publish → download
   → install → update.
4. Record one verdict per link: CONFIRMED WEAKNESS / CONFORM (evidence) /
   UNVERIFIED (reason — it often lives outside the repo).
5. Report: weaknesses first with attack scenario, then unverified, then the
   minimal hardening list.

## Workflow

### Phase 1 — Map the chain

1. Find the update mechanism: auto-updater configuration, update manifest or
   feed, version check endpoint, rollback behavior.
2. Find the packaging: installer builders, bundle configuration, scripts that
   assemble the release.
3. Find the pipeline: CI/release workflows, who can trigger them, what
   secrets they hold, where artifacts are uploaded.
4. Find the public surface: download page links, store listings, checksum
   publication.

### Phase 2 — Review each link

Follow `references/release-checklist.md`. The core questions:

1. **Update integrity** — every update is signed and the signature is
   verified *before* install; the update feed itself comes over TLS; version
   downgrade is refused.
2. **Installer/package integrity** — artifacts signed (code signing, package
   signature); unsigned builds never shipped; checksums published on an
   independent channel when direct download is offered.
3. **Channel** — downloads served over HTTPS from a controlled origin; no
   plain-HTTP links, no third-party mirrors without checksums.
4. **Pipeline** — release workflows need explicit triggers (tag, manual
   approval), not push-to-main; secrets scoped to the release job only;
   dependencies of the build itself pinned; artifact provenance (build logs,
   attestation when available).
5. **First-run and bundled extras** — anything downloaded at first launch is
   fetched over TLS and verified (signature or pinned hash), same standard as
   the app itself.

### Phase 3 — Report

Compact, in the chat:

1. **Confirmed weaknesses** — config/code evidence, the attack scenario
   ("an attacker on the network can…"), ranked by severity.
2. **Unverified** — what lives outside the repo (store console settings, CI
   secret scopes) listed as questions for the user to check, not findings.
3. **Conform links** — counted, one line each with evidence.

End with the minimal hardening list, most severe first.

## Rules

- An unsigned or unverified update path is always the top finding — it turns
  every other protection into decoration.
- You distinguish "not signed" from "signature not verified": both exist,
  the second is subtler and just as open.
- You never mark a link CONFORM from documentation or intention — only from
  configuration and code you read.
- When the project has no release pipeline yet (development stage), say so,
  list what the pipeline must guarantee when it arrives, and stop.
