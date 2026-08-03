# 03 - Verify

You prove the prepared release and obtain explicit approval of its exact contents.

## Input

- Accept prepared artifacts, required commands, expected tag and provider state, notes, affected files, endpoint, and repository rules.

## Output

- Return every check result, full notes, target, affected files, tag, endpoint, approval state, and `ready` or `blocked` verdict.

## Process

1. **Validate artifacts.** Run schema, consistency, changelog, localization, version, and packaging checks required by project rules.
2. **Run project gates.** Execute mandatory tests, lint, types, builds, or release validation without weakening them.
3. **Recheck uniqueness.** Confirm that version, tag, and provider release remain absent.
4. **Recheck diff.** Verify expected files, no unrelated staged state, and no secret-bearing content.
5. **Record evidence.** Capture command, result, and applicable platform for each gate without sensitive logs.
6. **Present exact release.** Show target version, complete notes, affected files, commit message, tag type and message, provider endpoint, and any explicit lease push.
7. **Wait for approval.** Require explicit user approval. Re-run affected checks and request approval again after any material change.
8. **Return verdict.** Use `ready` only when every mandatory check passes, every artifact is complete, and the exact release is approved.

## Stop conditions

- Stop on the first mandatory failing or unavailable check and never publish a blocked release.
- Stop when approval is absent, conditional, stale, or rejected.
- Do not repair unrelated code, reduce coverage, edit expectations, bypass hooks, or claim untested platforms.

## Test

- Confirm current evidence for every mandatory requirement.
- Confirm that `ready` implies a unique target, complete artifacts, clean diff, passing gates, and exact user approval.
