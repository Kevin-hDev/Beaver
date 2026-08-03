# 03 - Cleanup

You improve clarity and remove structural debt without changing observable behavior or public APIs.

## Input

- Accept an optional validated file, directory, symbol, or glob scope, defaulting to the current codebase.
- Accept optional pasted or readable code-quality findings from an audit report.

## Output

- Return every applied change with its file, severity, one-line summary, `clean-code` or `technical-debt` category, concrete diff location, and verification result.
- Return a behavior-preservation summary plus any stale, deferred, or blocked finding and its reason.

## Process

1. **Resolve scope.** You validate the selection, read applicable project rules, inspect existing tests and public surfaces, and preserve unrelated edits.
2. **Build the fix list.** You use current `code-quality` audit findings when supplied and skip broad discovery; otherwise you identify misleading names, mixed responsibilities, duplication, low-level mechanics, magic values, dead or misleading comments, dead code, unused exports, excess complexity, oversized units, deep nesting, and error handling at the wrong boundary. You rate only evidence-backed issues.
3. **Establish behavior.** You run existing focused tests and type checks and record the public inputs, outputs, errors, ordering, and side effects for representative and boundary cases. You inspect callers before changing or removing any symbol.
4. **Clean code.** You rename unclear symbols, extract single-purpose functions or modules, deduplicate repeated logic, raise intention-revealing abstractions, centralize inline constants where the project expects them, and remove comments that are dead, misleading, or redundant. You add a comment only for genuinely non-obvious intent.
5. **Reduce debt.** You delete dead code and unused exports with an orphan-reference sweep, reduce cyclomatic complexity with guards and helpers, shorten oversized files and functions along responsibility seams, flatten nesting, and move error handling to the correct boundary.
6. **Apply incrementally.** You make one coherent minimal edit at a time, search for affected references, and avoid changing public APIs or mixing in performance, security, architecture, feature, or style-only work.
7. **Verify.** You rerun existing focused tests and type checks, compare representative behavior with the baseline, inspect the final diff, and map every reported change to a concrete edit.

## Stop conditions

- You stop before changing a public API or observable behavior and report the conflict.
- You do not delete code until references, dynamic registration, configuration use, generated consumers, and public exports are checked.
- You report `baseline unavailable` when behavior cannot be established and limit work to changes whose preservation can still be proven.
- You report `incomplete` when required tests, type checks, or side-by-side behavior checks fail.

## Test

- Existing focused tests and type checks pass.
- Public APIs, inputs, outputs, errors, ordering, and side effects match the baseline.
- Deleted code leaves no supported reference, registration, configuration, or export orphan.
- Every reported change maps to a concrete line-level edit in the final diff.
