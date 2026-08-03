# Documentation Evidence Contract

## Evidence priority

1. Use accepted specifications, declared public contracts, and supported schemas to identify intended current behavior.
2. Use observable behavior and passing focused tests to prove that the implementation satisfies that contract.
3. Use public interfaces, generated specifications, and validated configuration for exact contract details.
4. Use implementation and current defaults for internal mechanics only when they do not conflict with a higher accepted contract.
5. Use existing prose only when stronger current evidence does not contradict it.
6. Use current official primary sources for unstable third-party facts.

Stop on conflict between an accepted current contract and executable behavior. Report a product mismatch; do not silently redefine the contract from the regression or describe the unimplemented contract as working behavior.

## Claim ledger

Record each material claim with:

- the proposed reader-facing statement;
- its source path, interface, command, test, or official URL;
- whether it is current, planned, deprecated, platform-specific, or uncertain;
- the documentation owner and affected example;
- the verification method and result.

Treat security guarantees, destructive operations, migrations, authentication, compatibility, defaults, limits, and pricing as high-risk claims that require direct current evidence.

## Examples

- Use synthetic identifiers and values.
- Use environment-variable names without values for secrets.
- State the working directory and prerequisites when they affect the result.
- Show only stable output or describe variable fields explicitly.
- Include cleanup or rollback when an example mutates state.
- Mark an example unverified when safe execution is impossible; never fabricate output.
