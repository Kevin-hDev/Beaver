# 02 - Collect

You gather the complete committed change between merge base and published head without including working-tree state.

## Input

- Accept verified provider, repository, head, base, merge base, mapped label, and applicable rules.

## Output

- Return bounded commits and paths, behavior summary, breaking changes, risks, tests, unresolved checks, and verified mapped label.

## Process

1. **Compute range.** Identify the merge base and compare committed head against base.
2. **Read commits.** Collect at most 100 commits per pass. Preserve a continuation range when the history is larger.
3. **Read diff.** Inspect at most 300 changed paths per pass, file statistics, renames, and relevant sections without reproducing secrets. Continue in bounded passes when requested.
4. **Summarize behavior.** Describe user-visible, public-contract, data, configuration, and operational changes supported by evidence.
5. **Find risks.** Record actual migrations, compatibility changes, deployment needs, security boundaries, and rollback concerns.
6. **Gather checks.** Use verified local or remote results and label unknown or stale checks accurately.
7. **Verify label.** Recheck that the mapped triage label still exists. Keep only the one documented mapping.
8. **Exclude working tree.** Keep staged, unstaged, and untracked content out of the summary.

## Stop conditions

- Stop when merge base cannot be resolved or the range includes unexpected unrelated history.
- Continue in another bounded pass rather than silently omitting a range larger than 100 commits or 300 paths.
- Do not claim tests, behavior, compatibility, or migration facts absent from evidence.

## Test

- Confirm that collection equals committed merge-base-to-head history and excludes local changes.
- Confirm that every behavior, risk, check, and label claim maps to project, diff, commit, or provider evidence.
