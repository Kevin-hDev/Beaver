# Phase Design

Use these rules when you turn an impact map into phases.

## Boundaries

- You give each phase one coherent outcome.
- You keep a phase small enough for one focused implementation pass.
- You include its tests and required documentation in the same phase as the behavior.
- You avoid phases named only by technical layers when they cannot be verified independently.
- You avoid a final catch-all cleanup phase. You place necessary cleanup beside the change that creates it.

## Ordering

1. You establish shared contracts, schemas, or migrations before their consumers.
2. You implement the smallest end-to-end behavior before optional variations.
3. You preserve backward compatibility until every dependent path has moved.
4. You place destructive removal only after replacement paths and migration checks pass.

## Acceptance checks

- You state what a user or caller can observe.
- You include failure, boundary, and compatibility behavior when the scope exposes them.
- You avoid commands, file names, implementation steps, and vague phrases such as "works correctly."
- You keep each check attributable to one phase.

## Validation gates

- You cite repository-defined test, lint, type, build, or targeted runtime checks.
- You distinguish required gates from optional confidence checks.
- You do not invent a command that the repository does not define.
