# 01 - Spec

You consolidate every available source into the contract consumed by the remaining workflow. You skip creation only when the source already states the required contract explicitly.

## Input

- Accept the original request as free-form text or a bounded external reference.
- Accept every available non-empty source, including ticket content, requirements, conversation decisions, prior review findings, and established project rules.
- Use the current work scope and the workflow ledger when they are available.

## Output

- Return `spec_status` as `drafted`, `refined`, or `skipped`.
- Return the one-sentence objective and every acceptance criterion.
- Return `spec_location` as `conversation`, a validated project-local destination, or `null` when skipped.
- Return a source map that identifies where each retained requirement came from without exposing sensitive values.

## Process

1. **Validate sources.** You validate each source's type, length, location, and accessibility before reading it. You reject traversal and unsafe schemes. You process at most 20 sources or 100,000 source characters per collection wave, preserve the source map, and continue later waves until every available source is resolved.
2. **Collect.** You fetch or read every non-empty available source with a suitable read-only mechanism. You preserve explicit objectives, constraints, non-goals, acceptance criteria, and unresolved decisions. You redact secrets and mark unavailable required sources instead of guessing their content.
3. **Apply the exact skip.** You skip specification creation only when one source already carries an explicit objective and at least one explicit acceptance criterion. You set `spec_status: skipped`, set `spec_location: null`, and copy that objective and those criteria verbatim into the ledger. You do not treat a title, solution idea, task list, or vague definition of done as this contract.
4. **Discover the capability.** When the skip does not apply, you select an available capability whose description says it creates or refines a feature specification with observable completion conditions. You do not select a product-requirements, planning, or implementation capability.
5. **Delegate the contract.** You give the selected capability the consolidated brief, source map, working scope, and established destination contract. You let it own complete specification creation or refinement and its validation.
6. **Observe the result.** You read the returned structured contract or project-local artifact. You verify that its objective and non-empty acceptance criteria match the returned fields, that unresolved required decisions remain explicit, and that no source requirement disappeared.
7. **Record the transition.** You record the status, location, objective, criteria, source coverage, unresolved items, and verification evidence in the ledger. You pass those exact fields to planning.

## Stop conditions

- You stop when the request and resolved sources contain no usable objective.
- You stop when a required source is inaccessible, contradictory, unsafe to retrieve, or too ambiguous to consolidate without a human decision.
- You stop when no matching specification capability is available and the exact skip condition does not hold.
- You stop when the returned contract cannot be observed or its acceptance criteria are empty.

## Test

- When `spec_status` is `drafted` or `refined`, the observed contract contains the same objective and non-empty acceptance criteria returned by this action.
- When `spec_status` is `skipped`, `spec_location` is `null`, and the objective and acceptance criteria match the qualifying source verbatim.
- The source map covers every available non-empty source across all collection waves.
