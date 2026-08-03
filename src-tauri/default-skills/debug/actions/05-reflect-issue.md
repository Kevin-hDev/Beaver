# 05 - Reflect Issue

You reopen a resistant investigation with fresh sources and bounded temporary instrumentation before any new production fix.

## Input

- Accept the issue, complete diagnostic journal, invalidated hypotheses, disproved fixes, and available runtime evidence.

## Output

- Return each fresh-source batch, the one or two most likely sources with confidence, and every temporary diagnostic with file, location, sanitized message, purpose, and result.

## Process

1. **Broaden.** List five to seven fresh possible sources that do not repeat already invalidated causes.
2. **Distill.** Select the one or two most likely sources using symptom consistency, recent relevant changes, and available evidence. Give each a confidence from 1 to 10.
3. **Design instrumentation.** Add only bounded, sanitized diagnostics that can confirm or refute one selected source. Exclude secrets, payloads, personal data, and unbounded logs.
4. **Observe.** Exercise the safe trigger, record each diagnostic result, and mark the source validated, invalidated, or blocked.
5. **Clean up.** Remove every temporary diagnostic after it has produced evidence and before completing the action. Confirm the final diff contains none.
6. **Continue batches.** When both selected sources are invalidated, preserve the evidence and create another batch of five to seven fresh sources. Continue until a cause is supported or a real blocker remains.
7. **Return to confirmation.** Feed supported evidence into `04-investigate-cause`, state the candidate root cause, and wait for user validation before any production fix.

## Stop conditions

- Do not apply a production fix during reflection.
- Do not leave temporary instrumentation, sensitive values, or unrelated edits behind.
- Do not stop solely because one five-to-seven-source batch was exhausted.

## Test

- Each batch contains five to seven fresh sources and one or two evidence-ranked selections.
- Every diagnostic names a real location and the exact observation it confirms or refutes.
- All temporary diagnostics are removed, and exhausted batches remain continuable.
