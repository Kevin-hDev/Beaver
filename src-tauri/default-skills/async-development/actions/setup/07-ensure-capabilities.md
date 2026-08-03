# 07 - Ensure Capabilities

You detect existing runtime capabilities before you offer an optional installation through a verified package interface.

## Input

- Use the confirmed configuration, effect contract, and detection report.
- Use the verified project-scope and user-scope capability catalogs and any configured package source.

## Output

- Return whether required adapters and the development workflow were found at project or user scope.
- Return installed package identities and post-install verification only when installation was explicitly authorized.
- Return `unsupported` when no real installation interface is available.

## Process

1. You skip installation for an execution path that does not require local runtime capabilities and record the reason.
2. You inspect project-scope capability configuration first. You match required operations from advertised descriptions and schemas, not hardcoded package names.
3. You inspect the user-scope catalog only when project scope is incomplete.
4. You return immediately without prompting when all required capabilities already exist at either valid scope.
5. You inspect the real package-manager schema, configured package source, package identities, version constraints, and verification operation.
6. You return `unsupported` without a speculative command when any installation format or package identity is unknown.
7. You require separate explicit authority to add a package source and to install each package at the selected scope.
8. You show the exact packages, scope, source, and expected files or registrations before mutation.
9. You perform authorized installation with validated argument arrays. You tolerate an already-installed result only after an independent catalog read proves the required version and scope.
10. You re-read the capability catalog and verify every required operation. You fail closed when any capability remains absent.

## Stop conditions

- You stop without prompting when existing capabilities satisfy the contract.
- You stop when package installation is denied, unsupported, ambiguous, or cannot be verified.
- You stop on the first installation failure and preserve the known installed subset for resume.

## Test

- You confirm a complete project-scope installation causes no user prompt or package-manager mutation.
- You confirm a complete user-scope installation also causes no installation.
- You confirm an authorized fresh installation is verified from the capability catalog, not command output alone.
- You confirm an unknown package format produces `unsupported` and no guessed command.
