# 05 - Write Configuration

You persist the confirmed secret-free configuration without corrupting an existing valid file.

## Input

- Use the confirmed configuration and effect contract from action 02.
- Use the validated project configuration path and configuration template.

## Output

- Return the configuration path, schema version, content digest, and atomic-write verification.
- Return the preserved prior-file status when no write is authorized.

## Process

1. You require explicit authority for the exact configuration path.
2. You copy the generic template, replace every placeholder, and include the normalized effect contract without secret values.
3. You add a schema version and creation or update timestamp. You preserve a stable installation identifier when updating an existing configuration.
4. You validate paths, fields, types, bounds, unique states, adapter identities, and credential references against the configuration contract.
5. You reject unexpected credential-shaped values, private-key blocks, access tokens, passwords, or raw environment contents.
6. You diff an existing file and require confirmed replacement authority for material changes.
7. You write a temporary sibling, parse and validate it, synchronize it when supported, and atomically rename it over the target.
8. You compute and return a digest from the final file. You leave the file unstaged.

## Stop conditions

- You stop without replacing the prior file when validation, temporary write, synchronization, or rename fails.
- You stop when the configuration contains a secret value or an unsupported adapter.
- You stop when an existing file differs and replacement was not authorized.

## Test

- You parse the final file and confirm every required section and positive bound.
- You confirm its digest matches the returned digest.
- You confirm the file contains only opaque credential references and no secret-looking value.
- You simulate a validation failure and confirm the prior valid file remains byte-for-byte unchanged.
