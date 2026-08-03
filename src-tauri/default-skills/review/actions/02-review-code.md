# 02 - Review Code

You inspect the changed code for defects and risks that a maintainer should address before shipping.

## Input

- Use the bounded diff batches, surrounding code, applicable rules, and impact map from `01-collect`.

## Output

- Return evidence-backed candidate findings for correctness, security, reliability, error handling, concurrency, performance, and maintainability.

## Process

1. **Read complete units.** You inspect every changed line with its enclosing function, type, configuration block, tests, and relevant callers or callees.
2. **Trace inputs.** You follow external input through validation, authorization, storage, command execution, output, and error paths.
3. **Check failure behavior.** You test the reasoning for empty, malformed, boundary, timeout, cancellation, partial-failure, and retry cases that the change can encounter.
4. **Check state.** You inspect ownership, cleanup, concurrency, ordering, caching, collection bounds, migrations, and backward compatibility where applicable.
5. **Check secrets and errors.** You flag unsafe secret handling, non-constant-time secret comparison, sensitive logs, unzeroized secret buffers, or internal details exposed to users.
6. **Check tests.** You verify that changed behavior has meaningful coverage and that assertions can fail for the intended regression.
7. **Reject noise.** You discard style-only preferences, pre-existing defects untouched by the diff, speculative issues without a reachable scenario, and duplicates.
8. **Rate candidates.** You apply [review-rubric.md](../references/review-rubric.md) and keep the narrowest useful location.

## Stop conditions

- You stop before editing code or running a mutating command.
- You label a candidate uncertain when missing runtime or external evidence prevents confirmation.
- You do not convert a broad architectural concern into a diff finding without a changed causal location.

## Test

- Every candidate names a changed location, reachable scenario, impact, and corrective direction.
- Every security candidate identifies the controlled input or trust boundary.
- No candidate exists only because the reviewer prefers a different style.
