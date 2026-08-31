# Dependencies checklist

## A. Ecosystem inventory

- [ ] Every manifest found: app, sidecars, plugins, scripts, CI, docs tooling
- [ ] Every lockfile present and committed; CI installs from lockfiles
- [ ] No "latest"/floating versions on security-sensitive paths

## B. Audit runs

- [ ] `npm audit` (prod and dev separately), `pip audit`, `cargo audit`,
      `govulncheck` — whichever applies
- [ ] Per finding recorded: package, advisory, severity, range, fixed
      version, direct/transitive
- [ ] When a tool cannot run: stated, plus the weaker fallback used

## C. Reachability grading

For each HIGH/CRITICAL, answer:

1. Where does the project use this package? (`file:line` or "untracked")
2. Does that path touch untrusted data? (files from users, network payloads,
   model/binary loading, archive extraction, parsers)
3. Is the vulnerable feature the one being used?

Grades:
- **REACHABLE** — vulnerable feature on an untrusted-data path → top priority
- **DEV-ONLY** — build/test tooling; developer-machine and CI risk
- **THEORETICAL** — in the tree but the vulnerable path is unused

## D. Supply-chain integrity

- [ ] Downloads at build/install/runtime are verified; verification material
      comes from an independent channel (a same-origin checksum only proves
      integrity, not authenticity — label it partial)
- [ ] Installers/updates: signature verification present, or explicitly
      documented accepted risk
- [ ] Extract steps: path confinement, symlink handling, size limits
- [ ] Update bot active; its open PRs reviewed regularly
- [ ] New dependencies policy: who approves adding one

## E. After any bump

- [ ] Project tests re-run and green
- [ ] The audit re-run: the alert is gone (not assumed — verified)
- [ ] Changelog/release notes mention security bumps when the project has
      that convention
