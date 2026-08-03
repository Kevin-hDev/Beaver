# Branch Convention Fallback

Read this reference only when the repository uses branch prefixes but does not provide a complete mapping.

| Prefix | Typical base policy | Existing triage label |
| --- | --- | --- |
| `feat/` | Use the confirmed default integration branch | `feature` when present |
| `fix/` | Use the confirmed default integration branch | `bug` when present |
| `docs/` | Use the confirmed default integration branch | `documentation` when present |
| `refactor/` | Use the confirmed default integration branch | `refactor` when present |
| `chore/` | Use the confirmed default integration branch | `maintenance` when present |
| `test/` | Use the confirmed default integration branch | `test` when present |
| `hotfix/` | Use only a project-documented production base; otherwise ask | `bug` when present |

- Prefer project-documented mappings over this fallback.
- Never derive a non-default base from a prefix unless project evidence maps it.
- Verify every base remotely and every label through the provider before use.
- Skip a fallback label that does not exist.
