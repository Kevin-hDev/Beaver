# Evidence Rules

Use these rules to keep the review complete, bounded, and actionable.

## Bounded input

- You process at most 50 changed files in one batch.
- You process at most 500 KiB of diff text in one batch.
- You split larger diffs into sequential batches and review every batch before synthesis.
- You mark the review incomplete when a tool truncates content and you cannot retrieve the remainder.
- You extract at most 100 acceptance conditions per functional batch and continue in further batches when needed.

## Finding contract

Publish a finding only when it includes:

1. A changed causal location with the narrowest useful line.
2. A reachable input, state, or execution scenario.
3. The incorrect or risky behavior.
4. Concrete user, data, security, reliability, or maintenance impact.
5. The smallest corrective direction without writing the patch.
6. Supported severity and confidence.

## Exclusions

- You exclude issues wholly outside the change unless the diff newly depends on or exposes them.
- You exclude pure style, formatting, naming taste, speculative future use, and unsupported performance guesses.
- You exclude test failures you did not run unless static evidence proves the test must fail.
- You exclude external facts you cannot verify from an authoritative source.

## Evidence handling

- You sanitize secrets, tokens, personal data, and raw sensitive bodies.
- You cite paths relative to the repository in the report.
- You keep raw internal tool failures out of user-visible error messages and summarize them generically.
