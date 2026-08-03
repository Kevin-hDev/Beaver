# Gap Rubric

| Category | Question |
| --- | --- |
| Actors | Who acts, benefits, owns, approves, or is denied? |
| States | Which meaningful starting, intermediate, empty, success, failure, and recovery states exist? |
| Failures | What can fail, conflict, time out, retry, or require rollback? |
| Boundaries | What is in or out, limited, supported, accessible, localized, private, or compatible? |
| Data | Where does information come from, how is it validated, owned, retained, migrated, and deleted? |
| Dependencies | Which decisions, systems, teams, assumptions, or ordering must hold? |
| Verification | How can a consumer prove completion, correctness, failure, and success? |
| Assumptions | Which condition is treated as agreed or obvious without being stated or supported? |
| Ambiguities | Which term has multiple reasonable interpretations that change a decision or result? |

- Blocker: the next phase cannot start or cannot produce a pass/fail verdict.
- Major: work can start, but the omission is likely to cause wrong behavior or a rework cycle.
- Minor: resolving the omission improves clarity without changing implementation or verification.
