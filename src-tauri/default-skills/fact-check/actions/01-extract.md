# 01 - Extract

You convert the target text into atomic claims that can be proved or disproved without losing context.

## Input

- Accept text, a prior answer, a document section, or an explicit claim list.

## Output

- Return ordered atomic-claim batches with wording, context, scope, time sensitivity, and source domain.
- Return a skip ledger containing each skipped factual statement and its concrete reason.

## Process

1. **Read context.** You identify the audience, time frame, project or external scope, and meaning of references such as this, current, or latest.
2. **Split claims.** You separate compound statements when their parts could receive different evidence.
3. **Preserve meaning.** You retain qualifiers, dates, quantities, jurisdiction, version, and applicability.
4. **Filter non-claims.** You exclude opinions, preferences, questions, predictions, hypotheticals, and stated plans that contain no factual assertion. You preserve any separable factual assertion inside mixed text.
5. **Select factual claims.** Unless exhaustive checking was requested, you skip a factual statement only when it is trivial, directly self-evident from the supplied text, or irrelevant to every conclusion. You record the exact statement and one of those concrete reasons in the skip ledger.
6. **Protect uncertainty.** You keep disputed, uncertain, cited, numeric, time-sensitive, technical, legal, historical, project-specific, and conclusion-bearing claims in verification. Difficulty, missing evidence, or expected disagreement is never a skip reason.
7. **Honor exhaustive scope.** When the user requests every or all factual statements, you select every factual claim, including trivial and background facts. You continue to exclude pure non-claims.
8. **Order claims.** You place consequential, disputed, cited, numeric, current, technical, legal, historical, and project-specific claims first without discarding any selected claim.
9. **Batch completely.** You inspect factual statements in source order, emit at most 30 selected claims per verification batch, keep a source cursor plus selected and skipped ledgers, and continue until every factual statement is classified exactly once.

## Stop conditions

- You stop when the target text is missing or a claim cannot be understood without unavailable context.
- You do not infer claims that the author did not make.
- You do not skip an uncertain claim to make the verification appear complete.

## Test

- Every extracted item is atomic, preserves qualifiers, and can receive independent evidence.
- Opinions and non-assertive text do not enter verification.
- Every skipped fact has an observable concrete reason and no uncertain or conclusion-bearing claim is skipped.
- The union of selected and skipped ledgers classifies every factual statement in the requested scope exactly once.
