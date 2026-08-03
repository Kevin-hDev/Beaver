# 04 - Investigate Cause

You confirm an unknown root cause by mapping the failing path and validating evidence-ranked hypotheses without applying a production fix.

## Input

- Accept the issue, symptom, error, prior reproduction batches, and previously invalidated hypotheses.

## Output

- Return a one-line confirmed root cause, an action-path Mermaid diagram, a three-to-five-level why chain, and every hypothesis with confidence, status, and evidence.

## Process

1. **Summarize.** Restate the expected and actual behavior without inventing missing facts.
2. **Map.** Read [mermaid-conventions.md](../references/mermaid-conventions.md) and draw the relevant action path across real files, calls, state, and boundaries.
3. **Ask why.** Trace the symptom through three to five documented causal levels.
4. **Inventory tools.** Identify the safe repository, log, trace, runtime, and inspection tools available for this investigation.
5. **Locate.** Inspect the smallest complete code and configuration paths that can explain the symptom.
6. **Create a batch.** Record three to five distinct candidate causes with analysis, evidence, confidence from 1 to 10, confirmation check, and `pending` status.
7. **Validate sequentially.** Test one candidate at a time with read-only evidence or bounded temporary instrumentation. Mark it `validated`, `invalidated`, or `blocked` and retain the evidence.
8. **Continue batches.** When every candidate is invalidated, preserve the complete journal and continue with `05-reflect-issue` for fresh sources. When a later turn resumes, start from the next pending candidate rather than rebuilding the list.
9. **Conclude.** State a one-line cause only when one hypothesis has direct consistent evidence. List the next steps without changing production behavior.
10. **Confirm.** Show the conclusion and wait for user validation before applying a production fix.

## Stop conditions

- Do not edit production behavior before a cause is confirmed and accepted.
- Do not expose secrets, personal data, raw sensitive bodies, or unnecessary internal failures in the journal.
- Stop with a named blocker only when required evidence, authorization, or a safe test surface is unavailable.

## Test

- Each hypothesis batch contains three to five candidates with confidence, status, confirmation check, and evidence.
- The action-path diagram uses real inspected relationships and the why chain contains three to five causal levels.
- A confirmed cause is consistent with a validated hypothesis, and an exhausted batch continues to reflection.
