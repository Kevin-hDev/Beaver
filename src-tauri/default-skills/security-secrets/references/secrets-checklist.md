# Secrets checklist

## A. Pattern search (hardcoded secrets)

Search these shapes across the project (excluding lockfiles and fixtures):

- API keys: `sk-…`, `sk-ant-…`, `xox[baprs]-…`, `ghp_…`, `gho_…`, `glpat-…`,
  `AKIA…`, `AIza…`, `hf_…`, `eyJ…` (JWT), generic `*_KEY`/`*_TOKEN`/`*_SECRET`
  assignments with real-looking values
- Private keys: `BEGIN … PRIVATE KEY`
- Passwords: `password|passwd|pwd` followed by a literal
- Connection strings with credentials: `scheme://user:pass@host`

Classify each hit: REAL (finding) / FAKE FIXTURE (test value, documented
example like `sk-test`, `your-key-here`) / NOT A SECRET (hash, public ID).

## B. Entropy review

In security-sensitive files, list long high-entropy strings. Random-looking
30+ character strings that are neither hashes nor documented IDs deserve a
human look. Never print them in full — last 4 chars max.

## C. Version control

- [ ] `.env`, key files, credential stores are git-ignored
- [ ] When the user allows: search history for deleted-but-committed secrets
      (a deleted secret is still a leaked secret → rotation advice)

## D. Storage at rest

- [ ] Keystore/vault/secure env only; encryption is authenticated (AEAD) and
      from a maintained library; master key in the OS keystore
- [ ] No plaintext fallback: test modes, e2e flags, migration paths, recovery
      code — each must not silently bypass the vault
- [ ] File permissions on any persisted sensitive file are user-only

## E. In motion

- [ ] No IPC/IPC-like accessor returns a secret to a less-trusted layer
      (`has`/`test`/`set`/`delete` only — never `get`)
- [ ] Secret buffers zeroized after use where the runtime permits
- [ ] Constant-time comparison for every token/hash check
- [ ] CSPRNG for token/ID/nonce generation

## F. Exit doors

- [ ] Logs: redaction before write; raw HTTP bodies of providers never
      logged; rotated/archived logs included in the check
- [ ] Errors: user-visible messages generic; provider error payloads sanitized
- [ ] Session/history/conversation files: plaintext is a design choice to
      record with its risk, pasted secrets included
- [ ] Crash/telemetry/support exports: no auth headers, no payloads
- [ ] Backups and sync folders: can the secret store be swept into a cloud
      backup unencrypted?

## G. When a live secret is found

1. Do not print it.
2. Report location + type + the door it leaks through.
3. Advise rotation — treat it as compromised.
