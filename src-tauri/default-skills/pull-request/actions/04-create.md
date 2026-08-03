# 04 - Create

You refresh remote state and create exactly one approved pull or merge request through the available provider mechanism.

## Input

- Accept verified repository, published head, base, approved title and body, approved state, mapped label, and explicit metadata.

## Output

- Return URL, number, provider, head, base, state, mapped label, and applied explicit metadata.

## Process

1. **Refresh state.** Confirm head and base still exist, remote head still matches local `HEAD`, no duplicate appeared, and the approved draft remains current.
2. **Check gates.** Stop when repository rules require a passing pre-creation gate and its current result failed or is unavailable.
3. **Create once.** Call the selected provider mechanism with explicit repository, head, base, title, body, and approved state.
4. **Apply mapped label.** Apply the one documented triage label when it exists. Skip a missing mapped label without failing creation.
5. **Apply explicit metadata.** Add only reviewers, additional labels, milestone, assignees, or project values included in the approved draft.
6. **Verify request.** Fetch the created object and confirm URL, number, state, head, base, title, body, mapped label, and requested metadata.
7. **Return partial outcomes.** Distinguish created, duplicate, blocked, provider-failed, and created-with-metadata-failure.

## Stop conditions

- Never retry an ambiguous creation response before searching for the resulting request.
- Attempt creation at most twice and retry only a confirmed transient failure that created no request.
- Never push, commit, merge, retarget another request, or change approved draft state silently.
- Preserve a created request when later label or metadata application fails and report the partial result.

## Test

- Confirm exactly one request with the approved head, base, title, body, and state.
- Confirm that each applied label or metadata value was mapped by project convention or explicitly approved.
