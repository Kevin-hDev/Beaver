# Auth Checklist — modèle complet de revue

Reference for `security-auth`. Each control lists: what to search, what weak
looks like, what conform looks like. A checklist item is not a finding until
the code makes it reachable — and it is never CONFORM without evidence.

## 1. Token validation

- [ ] Signature verified against an explicit expected algorithm
  - Weak: algorithm read from the token header (enables `alg=none` and
    RS256→HS256 confusion); signature check skipped in some code path
  - Conform: single verify function, algorithm pinned, used by all paths
- [ ] Expiration (`exp`) checked; `aud` and `iss` checked when the project
  issues tokens for multiple audiences
- [ ] Refresh tokens: revocable, rotated on use, bound to a session
- [ ] Revocation actually consulted on each request (or tokens short-lived
  enough that revocation lag is acceptable — and the project says which)

## 2. Session lifecycle

- [ ] Session created only after successful authentication
- [ ] Session identifier rotated after login and after privilege change
  (fixation)
- [ ] Logout destroys the session at the trusted layer, not only in the UI
- [ ] Idle expiry and absolute expiry exist and are enforced where the
  session is checked
- [ ] Session storage fits the platform: OS keychain/keystore on desktop and
  mobile, `HttpOnly`+`Secure`+`SameSite` cookies or memory on web — never
  plaintext files, `localStorage` for long-lived tokens is flagged

## 3. OAuth and redirects

- [ ] `state` parameter generated with CSPRNG and verified on callback
- [ ] PKCE used for public clients (mobile, desktop, SPA)
- [ ] Redirect URIs pinned exactly — no wildcards, no open redirect via
  `redirect_uri` or `return_to` parameters
- [ ] Tokens never appear in URLs, page history, analytics, or logs
- [ ] Deep-link / custom-scheme callbacks (mobile, desktop) validate the
  response origin — any app can claim a URL scheme

## 4. Password handling

- [ ] Hashing = established memory-hard function (argon2id, bcrypt, scrypt)
  with sane parameters; never fast hashes (SHA-*, MD5), never reversible
  encryption, never plaintext
- [ ] Reset tokens: CSPRNG-generated, single-use, short-lived, invalidated
  after use and after email/password change
- [ ] Reset and signup responses do not reveal whether an account exists
- [ ] Password policy enforced at the trusted layer; breach-list check when
  the project's context justifies it

## 5. Authorization context

- [ ] Identity re-derived from the token/session at the trusted layer on
  every operation — never from a client-supplied user id, role, or flag
- [ ] Tenant / ownership context preserved across asynchronous work, queued
  jobs, and sub-agent or plugin calls
- [ ] Privileged state transitions (role change, email change, payout,
  delete) re-check authorization and, where sensitive, re-authenticate

## 6. Failure behavior

- [ ] Every auth or parse failure denies — no path where an exception skips
  the check (fail closed)
- [ ] Repeated failures trigger rate limiting or lockout at the trusted
  layer (per account AND per source)
- [ ] Error messages generic: no account enumeration, no internal detail
  (stack, query, path) in user-visible errors
- [ ] MFA: enrollment verified, bypass/recovery codes single-use and
  rate-limited, no MFA check that a client flag can skip

## After the review

- [ ] Each control has a verdict: CONFIRMED WEAKNESS (`file:line` + path +
  impact) / CONFORM (`file:line`) / UNVERIFIED (reason) / N/A (reason)
- [ ] The report lists weaknesses first, ranked by impact
- [ ] The fix list is minimal and ordered — identity controls before polish
