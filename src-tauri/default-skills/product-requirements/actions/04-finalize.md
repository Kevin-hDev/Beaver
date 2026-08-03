# 04 - Finalize

You deliver the validated PRD without creating downstream work implicitly.

## Input

- Accept the drafted PRD, document conventions, and optional file destination.

## Output

- Return the final PRD in the conversation or at one validated destination.

## Process

1. **Check completeness.** You verify problem, users, goals, non-goals, requirements, user stories, acceptance criteria, success, constraints, dependencies, assumptions, and questions across every completed batch.
2. **Check utility.** You remove explanations that cannot guide a product decision or later specification.
3. **Check language.** You keep terminology consistent and understandable to non-technical stakeholders.
4. **Confirm.** You show the complete PRD, ask the user to approve or correct it, and wait. You show the revised complete PRD after each correction.
5. **Keep default local to conversation.** After approval, you return the PRD directly when no destination was requested.
6. **Write conditionally.** After approval, you validate a requested path, preserve existing user content, and write only the PRD when file output is explicit.
7. **Report gaps.** You identify open questions that must be resolved before stories, specification, or planning.

## Stop conditions

- You never overwrite an existing document without a clearly scoped update request.
- You do not create tracker items, implementation tasks, technical plans, or Git changes beyond an explicit PRD file.
- You do not write back to supplied stories or their tracker.

## Test

- The output contains one coherent PRD and no implicit downstream artifact.
- A file write occurs only at an explicitly requested validated path.
- The user approves the complete PRD before any file write.
