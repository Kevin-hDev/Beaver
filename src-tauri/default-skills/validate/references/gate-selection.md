# Gate Selection

Use the changed scope and project rules to classify gates before you run them.

## Required gates

Mark a gate required when any condition holds:

- The project instructions require it for the changed file type or domain.
- The approved plan names it as an acceptance or validation gate.
- The change directly affects the behavior that the gate verifies.
- The change crosses a compile, type, schema, architecture, security, or compatibility boundary.

## Optional gates

Mark a gate optional when it adds useful confidence but neither the project nor the changed behavior requires it. Run it only when its cost and side effects remain proportionate.

## Not-applicable gates

Mark a gate not applicable with one concrete reason when the changed scope cannot affect its surface. Do not mark a gate not applicable merely because it is slow, failing, or unavailable.

## Typical mapping

| Changed surface | Consider |
| --- | --- |
| Pure function or business rule | Focused unit tests, type or compile checks, lint |
| API or command boundary | Contract tests, validation failures, authorization, integration tests |
| Persistent data | Migration checks, compatibility, rollback or recovery behavior |
| User interface | Component tests, type checks, browser journey, accessibility |
| Build or packaging | Build, artifact checks, target-specific tests |
| Module boundaries | Architecture rules, dependency direction, cycle checks |

Use repository-defined names and commands. Do not assume a specific language, framework, package manager, or browser tool.
