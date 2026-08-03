# 03 - Bootstrap

You create only the explicitly requested contribution guidance and pushable initial commit.

## Input

- Accept an initialized repository, explicit contribution-guide or bootstrap intent, a verified project name, and the requested publication endpoint.

## Output

- Return whether `CONTRIBUTING.md` was created, the bootstrap commit SHA, its message, and the remaining content state.

## Process

1. **Recheck scope.** Confirm that contribution guidance, a bootstrap commit, or end-to-end publication requiring `HEAD` was explicitly requested.
2. **Prepare guidance conditionally.** When contribution guidance is requested, use the project's supplied content or copy [CONTRIBUTING.md](../assets/CONTRIBUTING.md), replace `{{PROJECT_NAME}}`, and leave no placeholder. Do not overwrite an existing file.
3. **Inspect the index.** Preserve existing staged content. Stop when it contains unrelated user work unless the user explicitly included it in the bootstrap.
4. **Stage narrowly.** Stage only the newly created contribution guide when requested. Do not stage existing project content implicitly.
5. **Create one bootstrap commit.** Create `chore: initialize repository`. Use `--allow-empty` only when a pushable `HEAD` was explicitly requested and no scoped file belongs in the commit.
6. **Verify.** Read the new SHA, changed paths, message, branch, and remaining untracked state.

## Stop conditions

- Never overwrite contribution guidance, stage unrelated content, bypass hooks, configure identity, or create more than one bootstrap commit.
- Stop when `HEAD` already exists unless the contribution-guide-only phase remains requested.
- Stop when a hook rejects the commit or modifies unrelated files.

## Test

- Confirm that `HEAD` resolves when a bootstrap commit was requested.
- Confirm that the commit contains only the requested contribution guide or is empty by explicit bootstrap authorization.
- Confirm that every unrelated project file remains unchanged and untracked or unstaged.
