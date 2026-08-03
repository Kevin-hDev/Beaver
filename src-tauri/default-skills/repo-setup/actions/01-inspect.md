# 01 - Inspect

You determine which local and publication phases are safe and explicitly requested.

## Input

- Accept a target directory, optional branch, contribution-guide choice, bootstrap choice, remote URL or provider, visibility, and publication endpoint.

## Output

- Return the validated target, repository boundaries, file state, branch, requested phases, remote state, provider capability, and required checks.

## Process

1. **Validate target.** Canonicalize the directory, reject traversal, and confirm that it is inside the allowed workspace.
2. **Inspect boundaries.** Detect a current, parent, or nested Git repository and stop before overlapping metadata.
3. **Inspect files.** List project paths in bounded passes of at most 200. Preserve a continuation cursor when more paths remain. Flag sensitive, generated, binary, or unexpectedly large content without reproducing secrets.
4. **Resolve branch.** Validate an explicit branch, otherwise use the configured Git default, then `main` only as a fallback.
5. **Resolve phases.** Separate initialization, contribution guidance, bootstrap commit, remote attachment, remote creation, and first push. Treat an explicit end-to-end publication request as authorization for the minimum bootstrap needed to create a pushable HEAD.
6. **Resolve remote.** Inspect existing remotes. Resolve a provider mechanism only when attachment or publication is requested.
7. **Resolve visibility.** Use explicit visibility or private for a newly created remote.
8. **Inspect publication content.** When publication is requested, inspect tracked and candidate bootstrap content for secrets, generated artifacts, private data, and unrelated files.

## Stop conditions

- Return `already-initialized` without changing an existing work tree unless later bootstrap or publication phases remain explicitly requested.
- Stop on a parent or nested repository ambiguity, invalid branch, conflicting remote, unsafe publication content, or ambiguous requested endpoint.
- Stop before provider work when publication or remote attachment was not requested.

## Test

- Confirm whether the target is outside, inside, or equal to an existing repository boundary.
- Confirm that every planned mutation maps to one explicit phase and no file changed during inspection.
