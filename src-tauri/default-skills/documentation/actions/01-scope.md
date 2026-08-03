# 01 - Scope

You turn the request into a precise documentation contract without inventing a new information architecture.

## Input

- Accept a documentation goal, audience, feature or subject, optional source scope, and optional destination.
- Accept existing documentation conventions and observable acceptance criteria when supplied.

## Output

- Return the resolved document type, audiences, covered and excluded subjects, source scope, destination, delivery mode, and validation criteria.

## Process

1. **Classify the work.** You read [document-types.md](../references/document-types.md) and select the narrowest matching document type or a justified combination.
2. **Inspect conventions.** You find existing documentation indexes, neighboring pages, navigation, terminology, frontmatter, link style, generated-doc boundaries, and validation commands. You inspect at most 100 paths per numbered batch and continue until the relevant convention is resolved.
3. **Resolve ownership.** You update an existing owning page when it can contain the subject cleanly. You create a new document only when the project convention or explicit request defines its place.
4. **Define audiences.** You state what each reader should understand or accomplish and what prior knowledge the document may assume.
5. **Define boundaries.** You list included behavior, excluded adjacent topics, planned behavior that must be labeled, and documentation that must remain generated or untouched.
6. **Set validation.** You identify available documentation build, lint, link, schema, example, command, rendering, and source-consistency checks.
7. **Confirm only when needed.** You proceed directly when the request and existing convention determine the contract. You ask one focused set of questions when audience, ownership, destination, or expected outcome would materially change the result.

## Stop conditions

- You stop when no safe project-local destination or owning document can be resolved.
- You stop when the request is actually a product specification, ADR, release note, memory update, or code-comment-only task.
- You do not write documentation, alter navigation, or change product code in this action.

## Test

- Confirm that document type, audience, scope, destination, source boundary, and validation criteria are explicit.
- Confirm that every new-file decision follows an existing convention or an explicit user destination.
- Confirm that no project state changed.
