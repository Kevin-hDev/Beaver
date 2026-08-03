# 04 - Publish

You record and publish only the approved release in the required sequence.

## Input

- Accept a `ready` approved release, endpoint, final artifacts, commit message, tag, remote, provider mechanism, and optional safe lease parameters.

## Output

- Return verified commit, tag, branch SHA, remote tag, provider URL, assets, and precise partial failures.

## Process

1. **Final preflight.** Confirm that artifacts, notes, approval, checks, `HEAD`, remote expected SHA, target uniqueness, and endpoint did not change.
2. **Record release.** Create only the approved project-required release commit and include only verified artifacts.
3. **Create tag.** Create the documented annotated or signed tag and approved message. Never replace or move a tag.
4. **Push branch normally.** Push the release branch normally when possible.
5. **Use a lease only when approved.** If the approved release explicitly requires a non-fast-forward branch update, use an exact `--force-with-lease=<remote-ref>:<observed-sha>` after refreshing the expected SHA. Never force a tag.
6. **Push tag.** Push the unique release tag normally and verify its remote commit.
7. **Create provider release.** Publish approved notes through the selected provider mechanism only when the endpoint includes it.
8. **Verify remote object.** Confirm version, tag, URL, notes, state, and expected assets. When the approved endpoint stops at a remote tag or the provider has no release object, return the provider's tag URL or compose it only from the validated remote identity and tag pattern.
9. **Report partial states.** Preserve successful steps and identify the exact remaining approved step after later failure.

## Stop conditions

- Never use `--force`, bare `--force-with-lease`, bypass hooks, replace tags, delete releases, publish twice, or deploy implicitly.
- Attempt each external creation at most twice and search remote state before retrying an ambiguous response.
- Stop when local commit, tag, approval, remote lease, or notes differ from verified state.

## Test

- Confirm every reported commit, tag, remote SHA, provider URL, release state, and asset directly.
- Confirm that no unrelated commit, branch, tag, remote, release, or deployment changed.
