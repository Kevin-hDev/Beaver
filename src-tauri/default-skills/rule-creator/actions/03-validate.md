# 03 - Validate

You independently check each intended rule target and the complete target set against the confirmed contract.

## Input

- Use the confirmed capture plan, canonical rule, intended target list, current written files, and resolved local conventions.

## Output

- Return one `passed`, `failed`, `blocked`, or `skipped` verdict per intended target, one cross-target consistency verdict, evidence for each check, and required corrections.

## Process

1. **Rebuild validation context.** You re-read the confirmed plan, current convention evidence, canonical rule, and target files instead of trusting the write summary.
2. **Validate existence and boundary.** You confirm each intended target exists when required, resolves inside the project, is a regular file, and occupies the confirmed location.
3. **Validate the authoring contract.** You read [rule-authoring.md](../references/rule-authoring.md) and check topic focus, imperative enforceability, scope, naming, duplicate and conflict absence, useful examples, language, and placeholder removal.
4. **Validate local format.** You check each target's extension, frontmatter or wrapper, metadata fields, globs, precedence, and links against the project's resolved convention rather than a generic path matrix.
5. **Validate semantic equivalence.** You read [consistency-validation.md](../references/consistency-validation.md) and compare each target with the canonical rule across obligations, prohibitions, strength, exceptions, scope, and examples.
6. **Validate preservation.** You compare changed targets with their prior content when available and flag unrelated deletion, reformatting, or restructuring outside the confirmed scope.
7. **Run available checks.** You run safe project-provided linting or validation for rule files when it exists. You capture failures without bypassing or suppressing them.
8. **Report every target.** You emit one evidence-backed verdict per intended target, explicitly name any required target that is missing or unverifiable, and issue one closed cross-target verdict.
9. **Correct narrowly.** You fix only mechanical defects that remain inside the confirmed meaning and boundary, then rerun all affected checks. You return to capture for any semantic, scope, destination, or overwrite change.

## Stop conditions

- You stop successful completion when any required target is missing, unreadable, outside the project, malformed, contradictory, weaker than the canonical rule, or unverifiable.
- You report `blocked` rather than guessing when project conventions or validation evidence remain ambiguous.
- You do not convert partial success, skipped validation, or absence of evidence into an all-target pass.

## Test

- You verify that every intended target has exactly one evidence-backed status against both the authoring and local-format contracts.
- You verify that the target set has a separate semantic-consistency verdict.
- You verify that the final summary distinguishes complete, partial, failed, and blocked outcomes honestly.
