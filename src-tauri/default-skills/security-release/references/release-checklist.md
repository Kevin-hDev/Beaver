# Release Checklist — modèle complet de revue

Reference for `security-release`. Each link of the chain: what to search,
what weak looks like, what conform looks like. No verdict without evidence —
and "lives outside the repo" is UNVERIFIED, never guessed.

## 1. Update integrity

- [ ] Updates signed and signature **verified before install**
  - Weak: unsigned update feed; signature generated but never checked;
    verification that logs the failure and installs anyway (fail open)
  - Conform: the updater refuses any artifact whose signature does not
    verify, with the check in code you read
- [ ] Update feed over TLS, from a controlled origin; the version-check
  response cannot redirect the download to an arbitrary host
- [ ] Downgrade protection: an older (possibly vulnerable) version cannot be
  served as "newest" — version comparison is monotone
- [ ] Update metadata (manifest, JSON feed) is itself signed or served from
  the same authenticated origin as the artifacts

## 2. Installer and package integrity

- [ ] Artifacts signed per platform: code signing on desktop, store signing
  on mobile, package signatures for registry distributions
- [ ] No unsigned build path reachable by the release workflow (debug or
  "temporary" unsigned jobs included)
- [ ] Direct downloads: checksums published on a channel independent from the
  download host (a checksum next to the file it protects proves nothing if
  both can be replaced together)
- [ ] Install scripts (curl | sh patterns): served over HTTPS, versioned, and
  they verify what they install (checksum or signature pinned in the script)

## 3. Download channel

- [ ] Every public download link uses HTTPS — search the site, docs, and
  readme for `http://` download links
- [ ] No third-party mirrors without published checksums
- [ ] The download page itself cannot be trivially redirected (links
  hardcoded to the controlled origin, not built from user input)

## 4. Build pipeline

- [ ] Release workflow triggers explicitly: version tag or manual approval —
  never a plain push to a shared branch
- [ ] Secrets scoped: signing keys and publish tokens available only to the
  release job, not to every CI run (a PR from a fork must not see them)
- [ ] Build inputs pinned: dependency lockfiles honored in the release build,
  build tools versioned
- [ ] Permissions reviewed: who can push the tag that ships a release, who
  can edit the workflow file itself (workflow edits = release power)
- [ ] Provenance when available: build attestation, reproducible builds,
  public build logs — note their presence or absence, absence is a note,
  not a finding

## 5. First-run and bundled extras

- [ ] Anything the app downloads at first launch (bundled runtimes, models,
  dictionaries, sidecars) follows the same standard as the app: TLS +
  signature or pinned hash verified before use
  - Weak: a first-run download verified by nothing, or by a hash fetched
    from the same unauthenticated origin
- [ ] Bundled third-party binaries inventoried with their version and origin

## After the review

- [ ] Each link has a verdict: CONFIRMED WEAKNESS / CONFORM / UNVERIFIED /
  N/A — with evidence or reason
- [ ] Unverified items phrased as questions the user can check (store
  console, CI secret scopes), never as findings
- [ ] Hardening list ordered: update/installer verification first, channel
  second, pipeline hygiene third
