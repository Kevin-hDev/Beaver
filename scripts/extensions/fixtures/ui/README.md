# Extension UI acceptance fixtures

These fixtures exercise Beaver's public extension UI contract without network access or secrets.
They are test data only and are licensed under the repository's `AGPL-3.0-only` license.

| Fixture | Purpose |
|---|---|
| `standard-complete` | Declares a navigation tab, a settings tab, a toolbar action, and a composer action. |
| `standard-limits` | Names every collection/size overflow and both protected-occupant mutations. |
| `conflict-a`, `conflict-b` | Request equal-priority moves of the same Beaver occupant. |
| `theme-valid`, `theme-invalid` | Provide one accepted theme and one theme with a token outside the contract. |
| `advanced-valid` | Builds JavaScript and CSS and returns mount plus activation cleanup functions. |
| `advanced-throws` | Throws during `activate` without exposing internal data. |
| `advanced-tampered` | Provides approved and modified sources whose hashes must differ. |
| `unicode` | Carries long French, German, Chinese, and Japanese localized text. |

Every folder is self-contained, bounded, and deterministic. Tests must copy a fixture before
mutating it; the checked-in files are immutable authorities for their scenario.
