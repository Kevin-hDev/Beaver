# Commit Message Fallback

Read this reference only when the repository defines no message convention and recent history is inconsistent or empty.

## Format

```text
type(scope): imperative summary

optional reason or important consequence

optional verified footer
```

## Types

| Type | Use it for |
| --- | --- |
| `feat` | New user-visible or public capability |
| `fix` | Correction of faulty behavior |
| `docs` | Documentation-only change |
| `refactor` | Internal restructuring without intended behavior change |
| `perf` | Measured performance improvement |
| `test` | Test-only change |
| `build` | Build system or dependency change |
| `ci` | Continuous-integration change |
| `style` | Formatting without logic change |
| `chore` | Narrow maintenance not covered above |
| `revert` | Reversal of an identified commit |

## Rules

- Use a lowercase type and optional concise project term for scope.
- Write an imperative summary, normally no longer than 72 characters and without a final period.
- Explain why in the body only when it adds durable context.
- Wrap body lines near 72 characters when practical.
- Use `BREAKING CHANGE:` only when a verified public contract becomes incompatible.
- Add issue footers only when the repository or user supplied the identifier and relationship.
