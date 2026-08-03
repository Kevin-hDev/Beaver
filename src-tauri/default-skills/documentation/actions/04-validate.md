# 04 - Validate

You prove that the completed documentation is accurate, navigable, reproducible, and compatible with the project's documentation system.

## Input

- Use the completed documentation, evidence ledger, confirmed validation criteria, and pre-write file identities.
- Use the checks in [validation-rubric.md](../references/validation-rubric.md).

## Output

- Return executed checks, exact scope, evidence, remaining limitations, changed files, and a `complete`, `partial`, or `blocked` result.

## Process

1. **Re-read changes.** You inspect every changed document and its surrounding navigation after writing. You compare claims with the current evidence ledger rather than memory.
2. **Run deterministic checks.** You run the project's documentation formatter, linter, build, schema, anchor, and link checks that apply. You pass system arguments separately and never build a shell command from untrusted prose.
3. **Validate references.** You resolve every changed local link and anchor. For changed external URLs, you allow only expected `https` or explicitly justified `http`, reject embedded credentials and non-web schemes, resolve and reject loopback, link-local, private, and project-internal destinations, bound redirects to approved public hosts, and limit response size and duration.
4. **Validate commands and examples.** You execute only a project-established or explicitly approved executable with strictly validated arguments passed separately. You use a disposable project-local fixture, a clean allowlisted environment without secrets, bounded time and output, no network by default, and explicit approved public destinations when network access is required. You compare observable output shape and exit behavior without modifying live services.
5. **Validate rendering.** You inspect the rendered output for tables, diagrams, callouts, navigation, code fences, wrapping, and accessibility when a renderer is available.
6. **Check source consistency.** You recheck every material changed claim and every material claim in a no-change decision. You fully reconcile accepted contracts, current behavior, and documentation, with extra scrutiny for API, security, migration, configuration, and destructive-operation claims.
7. **Repair documentation defects.** You correct only documentation defects within the confirmed contract and rerun affected checks. You keep each repair batch bounded and continue while a concrete fix remains.
8. **Close honestly.** You return `complete` only when all required checks pass. You return `partial` with named unchecked areas when optional tooling is unavailable, or `blocked` when a required check, source, example, or product behavior fails.

## Stop conditions

- You stop on a required documentation build, link, example, schema, or source-consistency failure that cannot be corrected within documentation.
- You stop and report the mismatch instead of changing product code, public APIs, tests, infrastructure, or credentials.
- You stop when a command or link cannot be checked inside the executable, argument, environment, network, time, and output boundaries; you report it as unchecked instead of weakening the boundary.
- You never claim an unavailable check passed and never execute destructive documentation examples against real state.

## Test

- Confirm that every required validation criterion has a recorded pass or a blocking failure.
- Confirm that local links resolve, examples are safe and verified or explicitly unchecked, and rendered structures remain readable.
- Confirm that the final result, changed-file list, and limitations match the evidence actually obtained.
