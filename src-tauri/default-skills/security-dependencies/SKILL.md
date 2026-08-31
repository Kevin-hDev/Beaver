---
name: security-dependencies
description: Use for reviewing third-party dependencies and supply-chain integrity — CVEs, lockfiles, download verification, and update mechanisms. Triggers on: dependency audit, CVE check, outdated packages, supply chain, or pre-release dependency review.
---

# Security Dependencies

You review the project's third-party dependencies and its supply-chain
integrity: known vulnerabilities, stale packages, lockfile discipline, and
every download-verify-execute chain. You run the ecosystem's own audit tools
and read the results — you never claim a dependency is clean without a tool
output or a version check to back it.

<critical_constraints>
- Every claim about a vulnerable package cites the audit output (CVE,
  severity, vulnerable range, fixed version).
- Distinguish REACHABLE risk (the project's code path actually uses the
  vulnerable feature) from THEORETICAL risk (present in the dependency tree
  but the vulnerable path is unused) — both matter, differently.
- Read-only on manifests unless the user asks for the bump.
- No exploitation of any CVE. You assess exposure, you do not demonstrate it.
</critical_constraints>

## Step 1 — Identify the ecosystems

List every package manifest and lockfile (npm, pip, cargo, go, …), including
sidecars, plugins, embedded runtimes, and CI tooling. A forgotten
`requirements.txt` in a bundled sidecar is a classic blind spot.

## Step 2 — Run the audits

1. Run each ecosystem's audit (`npm audit`, `pip audit`, `cargo audit`, …),
   production and development separately when the tool distinguishes them.
2. Record per finding: package, CVE/advisory, severity, vulnerable range,
   first fixed version, direct vs transitive.
3. When an audit tool cannot run (offline, missing), say so and fall back to
   comparing pinned versions against the project's own security notes —
   labeled as weaker evidence.

## Step 3 — Assess reachability

Apply `references/dependencies-checklist.md`:

- For each HIGH/CRITICAL finding, locate where the project uses the package
  and judge: does the vulnerable feature sit on a path that handles untrusted
  data (model loading, archive extraction, parsing, network)? A vulnerability
  in a load-path of user-supplied files is the top tier.
- Dev-only findings are lower priority but not zero — they run on developer
  machines and in CI.

## Step 4 — Review supply-chain integrity

- [ ] Lockfiles committed and respected by CI (install = locked versions)
- [ ] Download-verify chains: anything downloaded at build, install, or
      runtime (binaries, models, archives, updates) — is it verified
      (checksum AND origin), and does verification come from an independent
      channel from the download itself? A checksum fetched from the same
      location as the binary protects against corruption, not compromise —
      record it as partial.
- [ ] Update/auto-update mechanisms: signature verification, or documented
      accepted risk
- [ ] Automated update bot (Dependabot/Renovate) present and its PRs not
      piling up

## Step 5 — Report

```
DEPENDENCY REVIEW — {date}
Ecosystems: {npm: n packages, pip: n, …}

Findings: CRITICAL {n} | HIGH {n} | MEDIUM {n} | LOW {n}

### Reachable risks (fix first)
- [CRIT/HIGH] {pkg} {CVE} — {one-line risk}
  Used at: {file:line or "untracked"} | Fix: {pkg >= version}

### Theoretical / dev-only
- {pkg} {CVE} — {why it does not reach untrusted data}

### Supply chain
- Lockfiles: {ok/gaps} | Download-verify: {ok/partial/none, where}
  Update mechanism: {signed/unsigned/documented risk} | Update bot: {yes/no}

### Audits that could not run
- {ecosystem — reason — what weaker check was done instead}
```

## Rules

- Sort fixes by reachability × severity, not by severity alone: a CRITICAL
  in an unused dev tool ranks below a HIGH on the untrusted-file load path.
- Never run `audit fix` blindly — version bumps can break; propose, let the
  user decide, then re-run the project's tests after any bump.
- End with the minimal set of version bumps that closes every reachable risk.
