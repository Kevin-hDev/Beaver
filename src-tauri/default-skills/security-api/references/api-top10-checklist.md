# OWASP API Security Top 10 (2023) — review checklist

Use one section per endpoint group. A checklist item is not a finding until
current code makes it reachable.

## API1 — Broken Object Level Authorization (BOLA / IDOR)

- **Look for**: an object ID from the request (path, query, body) used in a
  data lookup without verifying the caller owns that object.
- **Patterns**: `getOrder(req.params.id)` without `where userId = currentUser`;
  sequential IDs enumerable; UUIDs treated as authorization.
- **Conform**: every object access is scoped by the authenticated identity.

## API2 — Broken Authentication

- **Look for**: token verification that skips signature or expiration;
  accepting `alg=none` or unexpected algorithms; password reset codes without
  attempt limits or expiration; sessions that survive logout.
- **Conform**: signature + expiration enforced, algorithm pinned, reset flows
  rate-limited and expiring.

## API3 — Broken Object Property Level Authorization

- **Excessive data exposure**: the handler returns a full database object and
  lets the client filter (internal fields, roles, other users' data inside).
- **Mass assignment**: request body spread/merged directly into the model
  (`Object.assign(user, req.body)`), letting the client write `role`,
  `isAdmin`, `balance`…
- **Conform**: explicit response shaping (DTO/serializer) and explicit
  allowlists of writable fields.

## API4 — Unrestricted Resource Consumption

- **Look for**: endpoints with no rate limiting (login, reset, search, export);
  unbounded `limit` parameters; file uploads without size caps; expensive
  aggregations callable by anyone.
- **Conform**: per-identity and per-IP limits, bounded pagination, capped
  expensive operations.

## API5 — Broken Function Level Authorization

- **Look for**: admin or internal endpoints whose only protection is that the
  UI hides the button; role checks missing or done client-side; HTTP method
  confusion (GET allowed where POST is checked).
- **Conform**: server-side role/permission check on every privileged handler.

## API6 — Unrestricted Access to Sensitive Business Flows

- **Look for**: automatable sensitive flows without anti-automation (ticket
  purchase, reservation, vote, referral bonus, comment posting).
- **Conform**: flow-level limits, per-identity quotas, abuse detection.

## API7 — Server-Side Request Forgery (SSRF)

- **Look for**: any user-influenced URL fetched server-side — webhook
  registration, URL preview, image-by-URL, import-from-URL, callback URLs.
- **Conform**: scheme allowlist (http/https), internal ranges blocked
  (loopback, RFC1918, link-local 169.254.0.0/16), redirects re-validated,
  DNS resolution pinned to the validated address.

## API8 — Security Misconfiguration

- **Look for**: debug/trace modes in production config; stack traces in error
  responses; CORS `*` with credentials; default credentials; directory
  listing; missing security headers where the API serves browsers.
- **Conform**: environment-driven config, generic errors, tight CORS.

## API9 — Improper Inventory Management

- **Look for**: `v1/` handlers kept alive next to `v2/`; `/debug`, `/test`,
  `/internal` routes in production builds; feature-flagged endpoints
  reachable when the flag is off; stale API docs exposing hidden routes.
- **Conform**: deprecated versions decommissioned, non-prod routes excluded
  from prod builds.

## API10 — Unsafe Consumption of APIs

- **Look for**: the project trusting third-party or internal API responses
  without validation — no schema check, no size limit, no timeout, following
  redirects blindly, storing/executing response content directly.
- **Conform**: validate, bound, and time out every upstream call; treat every
  response as untrusted input.

## Reachability test (apply to every candidate)

1. Name the untrusted input and its source.
2. Name the missing or weak check.
3. Name the sensitive effect it reaches.
4. All three named = CONFIRMED. Any one missing = UNVERIFIED.
