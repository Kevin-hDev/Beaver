# 04 - Verify and Report

You independently verify every task and the integrated result, then report the full ledger in one minimal table.

## Input

- Accept the reconciled task ledger, current project state, executor evidence, and all requested acceptance criteria.

## Output

- Return exactly one Markdown table with the columns `Category`, `Launched`, `Status`, `Verification`, and `Output`.
- Return one row for every todo, including queued, failed, blocked, and incomplete tasks.

## Process

1. **Verify task results.** You rerun or independently inspect each task's focused acceptance check in the current state. You reject stale, partial, or unverifiable evidence.
2. **Verify integration.** You run the narrowest relevant combined checks, then broader established checks when justified by the touched areas. You inspect interfaces and generated artifacts shared across tasks.
3. **Reconcile final statuses.** You downgrade any task invalidated by integration from `verified` to `failed` or `incomplete`. You propagate blocked dependency status without pretending the dependent task launched.
4. **Continue when possible.** You return to staging when ready tasks remain or a safe repair task can be derived without changing user intent. You keep later waves continuable until all tasks are terminal.
5. **Build the table.** You keep the original category for each row. You set `Launched` to `yes` only when its dedicated executor actually started. You cite compact concrete evidence in `Verification` and a one-line result or blocker in `Output`.
6. **Return only the table.** You output no heading, introduction, conclusion, footnote, or text outside the final table.

## Stop conditions

- You do not report while safely executable ready tasks remain unprocessed.
- You do not label a task verified when its focused check or the integrated check fails.
- You mark unavailable proof `unverified` or `incomplete` rather than converting absence of evidence into success.
- You do not conceal blocked, skipped, queued, or failed tasks to make the table appear complete.

## Test

- You confirm that every todo has exactly one table row and that every original requirement remains represented.
- You confirm that each `verified` row cites evidence observed after reconciliation.
- You confirm that dependency failures and unlaunched rows are honest and traceable.
- You confirm that the final response contains exactly one Markdown table and nothing else.
