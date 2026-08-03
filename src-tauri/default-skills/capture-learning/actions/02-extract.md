# 02 - Extract

You extract reusable project learning while discarding transient noise.

## Input

- Use the complete ordered source specifications and their readable slices from `01-select-source`.

## Output

- Return continuable candidate batches with source, evidence, learning, durability reason, scope, and duplicate key.
- Return a no-candidate result with the discarded-signal reasons when nothing is durable.

## Process

1. **Read the extraction contract.** You read [extraction.md](../references/extraction.md) before classifying any signal.
2. **Trace evidence.** You preserve the source label and a short sanitized quote or faithful paraphrase for every candidate.
3. **Keep durable signals.** You retain decisions and consequences, recurring conventions, costly pitfalls, reusable workflows, and missing project context whose absence would cause future contradiction or rework.
4. **Drop noise.** You discard personal preferences, temporary state, routine implementation detail, one-off facts without reuse, unsupported inference, and content already useful only inside its source artifact.
5. **Normalize and deduplicate.** You derive a stable duplicate key from the project scope plus normalized lesson. You merge repeated evidence without merging materially different decisions.
6. **Batch completely.** You return at most 30 candidates per numbered batch, keep a candidate and source ledger, and continue until every selected source slice is exhausted.

## Stop conditions

- You stop when evidence cannot support a faithful candidate or exposes sensitive content that cannot be sanitized safely.
- You do not invent rationale, destination, authority, or project-wide applicability.
- You do not persist a candidate merely because the user asked to learn from a source.

## Test

- Every candidate has selected-source evidence, a reusable lesson, scope, and a persistence reason.
- Every discarded signal matches an explicit noise rule.
- The union of candidate batches covers every durable signal exactly once.
