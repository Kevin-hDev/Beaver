# Regression Test

Use these rules to select and prove a test for a known defect.

## Selection

- You choose the lowest stable boundary that reproduces the user-visible or caller-visible failure.
- You extend an existing test file and helper before you create an equivalent one.
- You test behavior and outputs rather than private implementation details.
- You keep the fixture minimal and free of real secrets, personal data, network dependencies, and nondeterministic time.

## Proof

- You run the test before the production fix.
- You confirm that the assertion fails for the reported defect rather than setup, syntax, timeout, or unrelated state.
- You record the failing assertion and the minimal trigger.
- You run the same test after the fix without weakening it.

## Scope

- You add one focused case for the defect and only the boundary cases required to prevent the same cause.
- You avoid broad snapshot updates and unrelated test cleanup.
- You keep externally supplied fixture collections bounded.
