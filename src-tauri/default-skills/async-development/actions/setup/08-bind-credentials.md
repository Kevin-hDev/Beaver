# 08 - Bind Credentials

You verify secure credential references without receiving, reading, displaying, or persisting secret values.

## Input

- Use the confirmed configuration, effect contract, detection report, and secure-store reference names.
- Use the adapter's documented metadata-only credential inspection capability.

## Output

- Return each required reference as `available`, `missing`, `unverifiable`, or `not-required`.
- Return provider-specific secure-store guidance that never contains or requests a credential value.

## Process

1. You skip remote credential checks when the selected execution paths require none and record the reason.
2. You derive required references from the integration, tracker, version-control, change-request, and package-source operations actually authorized.
3. You deduplicate reference names in a bounded set and validate each name against the provider's documented format.
4. You use metadata-only listing or existence checks. You never request secret contents, inspect process environments, read credential files, or print a stored value.
5. You verify the fallback reference and every configured actor or assignee routing reference independently.
6. You mark a missing reference as `missing` and direct the user to configure it through the provider's secure interface outside the conversation.
7. You never offer a paste prompt. You ask the user only to confirm when the secure-store operation has been completed, then repeat the metadata-only existence check.
8. You mark the setup blocked until every required reference is observed. You do not substitute a broader credential automatically.

## Stop conditions

- You stop immediately if a secret value appears in user input; you do not repeat or store it.
- You stop when the adapter can expose values but cannot perform a metadata-only existence check.
- You stop when a required reference is missing or unverifiable.

## Test

- You confirm an existing reference is detected from its name and metadata only.
- You confirm a missing reference produces out-of-band secure-store guidance without a paste request.
- You confirm actor routing falls back only to the explicitly configured fallback reference.
- You scan outputs and audit data and confirm no secret value or raw environment content appears.
