# Provider validation

## Usage checks without credentials

Run `npm run test:provider-usage` from the repository root. This runs the
existing Rust usage tests and bounded fixture transport tests. It needs no API
key, does not read the user's vault, and makes no billable provider request.
Each Cargo filter must execute tests and exit successfully; zero tests is not
validation. The fixture transport tests use controlled loopback servers.

This command is an **offline check**, not proof that a provider account works.
The former `scripts/check-provider-usage.mjs` live client was removed on
2026-09-07 with Kevin's approval: it required a separately supplied environment
key and bypassed Beaver's request budgets and secure credential path.

## Live usage checks through Beaver

Use the existing debug-only `run_reasoning_fixture_agent_local` command in an
already open Beaver instance. Configure the provider through Beaver's normal
settings; Rust reads the credential from the encrypted vault when needed.
Never copy a vault credential into JavaScript, a shell variable, or a report.

Before sending, verify the exact provider/model/mode, approved remaining spend,
and the bounded fixture limits (`BEAVER_FIXTURE_OUTPUT_TOKENS`,
`BEAVER_FIXTURE_ATTEMPTS`, `BEAVER_FIXTURE_INPUT_BYTES`). A short prompt alone is
not a cost bound. Unsupported routes, missing counters and missing credentials
are explicit blocked cases, not successful checks or reasons to use another
unbounded client.

Inspect the completed session diagnostics and matching request metrics for
input/output/cache tokens and costs. Keep absent counters distinct from zero.
Export sanitized continuation evidence with `export_reasoning_fixture_report`;
this export alone does **not** prove usage accounting. Record usage observations
separately, tied to the actual session/request and with no secret or opaque
reasoning payload. Follow `docs/providers/plan-de-tests.md` for the full matrix.
