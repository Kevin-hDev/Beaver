# Coding Assertions

## Fast gate

| Order | Command | Proves | Evidence |
| --- | --- | --- | --- |
| 1 | `{repository command}` | {format, lint, types, or focused tests} | `{manifest or instruction path}` |

## Full gate

| Order | Command | Proves | Evidence |
| --- | --- | --- | --- |
| 1 | `{repository command}` | {complete tests, build, or packaging} | `{manifest or workflow path}` |

## Required behavior

- {Fail-closed rule or ordering constraint supported by project instructions.}

<!-- Include only runnable repository-defined commands. Remove placeholders and this comment. -->
