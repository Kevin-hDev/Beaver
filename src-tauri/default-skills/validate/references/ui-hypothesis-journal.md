# UI Hypothesis Journal

Keep a complete, continuable record whenever UI validation enters a repair loop.

## Candidate batch

- Add exactly three evidence-based candidates per batch.
- Give each candidate a confidence from 1 to 10 and explain the evidence behind it.
- Record the path or boundary involved and the observation that can confirm or refute it.
- Mark each candidate `pending`, `validated`, `invalidated`, `blocked`, or `repaired`.

## Attempts

- Record the candidate, bounded change, exercised journey step, expected result, actual result, evidence reference, and status for every attempt.
- Keep at most three repair attempts for one candidate in one batch.
- Preserve failed attempts. Never overwrite the journal as though an earlier repair had not happened.

## Continuation

- When a candidate batch is exhausted, add a numbered batch of three fresh causes informed by the accumulated evidence.
- Continue batches until the selected UI journey passes or a real blocker prevents evidence or an authorized repair.
- At a turn boundary, return the complete journal state and the exact next pending candidate so another turn can resume without restarting.
