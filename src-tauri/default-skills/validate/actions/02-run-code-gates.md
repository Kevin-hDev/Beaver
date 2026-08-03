# 02 - Run Code Gates

You run the selected automated code gates and repair only failures introduced by the current work.

## Input

- Use the code gates and baseline from `01-discover`.

## Output

- Return each command, its final result, bounded repairs applied, and sanitized failure evidence.

## Process

1. **Order.** You run fast focused tests first, then type or compile checks, lint or static checks, broader tests, and builds when selected.
2. **Execute safely.** You use the repository-defined executable and pass validated arguments separately. You apply a reasonable timeout and bound captured output.
3. **Classify failure.** You determine whether each failure comes from the current work, predates it, or remains uncertain. You do not guess.
4. **Repair current failures.** You apply the smallest in-scope fix, add or update a focused regression test when behavior changes, and rerun that focused check.
5. **Batch repairs.** You attempt at most three repairs for the same failing gate in one batch. When a batch fails, you preserve sanitized evidence, revise the next batch's hypotheses, and continue until the gate passes or an actual authorization, safety, or evidence blocker remains.
6. **Sweep.** You rerun every required selected code gate in one clean pass after all individual gates have passed.

## Stop conditions

- You stop on a timeout, tool crash, unsafe command, unavailable required executable, or real blocker. A completed three-attempt batch alone is not a blocker.
- You stop before changing dependencies, generated outputs, public contracts, or unrelated code without approved scope.
- You do not delete tests, lower thresholds, add blanket ignores, or convert errors into warnings.

## Test

- A code-gate pass is backed by a final zero exit result from every required command.
- Every repair cites an actual changed file and a rerun result.
- A missing or failed required gate prevents a pass verdict.
